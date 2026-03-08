use crate::message::ServerMessage;
use serde::{Deserialize, Serialize};
use socket2::SockRef;
use sova_core::protocol::DeviceInfo;
use sova_core::scene::{ExecutionMode, Frame, Line, Scene};
use sova_core::schedule::ActionTiming;
use sova_core::schedule::SchedulerMessage;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

pub const PROTOCOL_VERSION: u8 = 0x02;
pub const MAX_MESSAGE_SIZE: u32 = 10 * 1024 * 1024;

pub fn serialize_to_wire_frame(msg: &ServerMessage) -> io::Result<Vec<u8>> {
    let payload = rmp_serde::to_vec_named(msg).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize ServerMessage: {}", e),
        )
    })?;
    Ok(build_v2_frame_raw(&payload))
}

fn build_v2_frame_raw(payload: &[u8]) -> Vec<u8> {
    let crc = crc32fast::hash(payload);
    let len_bytes = (payload.len() as u32).to_be_bytes();
    let mut frame = Vec::with_capacity(8 + payload.len());
    frame.push(PROTOCOL_VERSION);
    frame.extend_from_slice(&len_bytes[1..4]);
    frame.extend_from_slice(&crc.to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn validate_length(length: u32) -> io::Result<()> {
    if length == 0 || length > MAX_MESSAGE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid message length: {} bytes", length),
        ));
    }
    Ok(())
}

/// Reads one wire frame, auto-detecting v1/v2 format.
/// Returns raw payload bytes (already CRC-verified for v2).
/// Returns `Ok(None)` for v2 CRC mismatch (corrupted frame, skip it).
pub async fn read_wire_frame<R: AsyncReadExt + Unpin>(
    reader: &mut R,
) -> io::Result<Option<Vec<u8>>> {
    let mut first = [0u8; 1];
    reader.read_exact(&mut first).await?;

    match first[0] {
        0x00 => {
            let mut rest = [0u8; 3];
            reader.read_exact(&mut rest).await?;
            let length = u32::from_be_bytes([0x00, rest[0], rest[1], rest[2]]);
            validate_length(length)?;
            let mut buf = vec![0u8; length as usize];
            reader.read_exact(&mut buf).await?;
            Ok(Some(buf))
        }
        PROTOCOL_VERSION => {
            let mut header = [0u8; 7];
            reader.read_exact(&mut header).await?;
            let length = u32::from_be_bytes([0x00, header[0], header[1], header[2]]);
            let expected_crc =
                u32::from_be_bytes([header[3], header[4], header[5], header[6]]);
            validate_length(length)?;
            let mut buf = vec![0u8; length as usize];
            reader.read_exact(&mut buf).await?;
            let actual_crc = crc32fast::hash(&buf);
            if actual_crc != expected_crc {
                return Ok(None);
            }
            Ok(Some(buf))
        }
        v => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported protocol version: 0x{v:02x}"),
        )),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    SchedulerControl(SchedulerMessage),
    SetTempo(f64, ActionTiming),
    SetName(String),
    GetScene,
    SetScene(Scene, ActionTiming),
    GetLine(usize),
    SetLines(Vec<(usize, Line)>, ActionTiming),
    ConfigureLines(Vec<(usize, Line)>, ActionTiming),
    AddLine(usize, Line, ActionTiming),
    RemoveLine(usize, ActionTiming),
    GetFrame(usize, usize),
    SetFrames(Vec<(usize, usize, Frame)>, ActionTiming),
    AddFrame(usize, usize, Frame, ActionTiming),
    RemoveFrame(usize, usize, ActionTiming),
    GetClock,
    GetPeers,
    Chat(String),
    GetSnapshot,
    StartedEditingFrame(usize, usize),
    StoppedEditingFrame(usize, usize),
    CursorPosition(usize, usize),
    TransportStart(ActionTiming),
    TransportStop(ActionTiming),
    SetSceneMode(ExecutionMode, ActionTiming),
    RequestDeviceList,
    ConnectMidiDeviceByName(String),
    DisconnectMidiDeviceByName(String),
    CreateVirtualMidiOutput(String),
    AssignDeviceToSlot(usize, String),
    UnassignDeviceFromSlot(usize),
    CreateOscDevice(String, String, u16),
    RemoveOscDevice(String),
    RestoreDevices(Vec<DeviceInfo>),
    GetAudioEngineState,
    RestartAudioEngine {
        device: Option<String>,
        input_device: Option<String>,
        channels: u16,
        buffer_size: Option<u32>,
        sample_paths: Vec<String>,
    },
    PreviewSample {
        folder: String,
        index: usize,
        begin: f64,
    },
    EnableFeedback,
}

impl ClientMessage {
    pub fn deserialize(bytes: &[u8]) -> io::Result<Option<Self>> {
        match rmp_serde::from_slice::<ClientMessage>(bytes) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("MessagePack deserialization error: {}", e),
            )),
        }
    }
}

pub struct SovaClient {
    pub ip: String,
    pub port: u16,
    reader: Option<BufReader<OwnedReadHalf>>,
    writer: Option<BufWriter<OwnedWriteHalf>>,
    pub connected: bool,
}

impl SovaClient {
    pub fn new(ip: String, port: u16) -> Self {
        SovaClient {
            ip,
            port,
            reader: None,
            writer: None,
            connected: false,
        }
    }

    pub async fn connect(&mut self) -> io::Result<()> {
        let addr = format!("{}:{}", self.ip, self.port);
        let stream = TcpStream::connect(&addr).await?;
        stream.set_nodelay(true)?;
        let keepalive = socket2::TcpKeepalive::new()
            .with_time(std::time::Duration::from_secs(60))
            .with_interval(std::time::Duration::from_secs(10));
        let _ = SockRef::from(&stream).set_tcp_keepalive(&keepalive);
        let (read_half, write_half) = stream.into_split();
        self.reader = Some(BufReader::with_capacity(32 * 1024, read_half));
        self.writer = Some(BufWriter::with_capacity(32 * 1024, write_half));
        self.connected = true;
        Ok(())
    }

    pub async fn send(&mut self, message: ClientMessage) -> io::Result<()> {
        let payload = rmp_serde::to_vec_named(&message).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize ClientMessage: {}", e),
            )
        })?;

        let writer = self.writer.as_mut().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "Client not connected")
        })?;

        let frame = build_v2_frame_raw(&payload);

        if let Err(e) = writer.write_all(&frame).await {
            self.connected = false;
            return Err(e);
        }

        if let Err(e) = writer.flush().await {
            self.connected = false;
            return Err(e);
        }

        Ok(())
    }

    pub async fn disconnect(&mut self) -> io::Result<()> {
        self.connected = false;
        if let Some(mut writer) = self.writer.take() {
            let _ = writer.shutdown().await;
        }
        self.reader.take();
        Ok(())
    }

    pub async fn read(&mut self) -> io::Result<Option<ServerMessage>> {
        if !self.connected {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Client not connected",
            ));
        }
        let reader = self.reader.as_mut().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotConnected, "Client not connected")
        })?;

        let payload = match read_wire_frame(reader).await {
            Ok(Some(buf)) => buf,
            Ok(None) => return Ok(None),
            Err(e) => {
                self.connected = false;
                return Err(e);
            }
        };

        match rmp_serde::from_slice::<ServerMessage>(&payload) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Deserialization failed: {}", e),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::ServerMessage;
    use std::collections::HashMap;
    use sova_core::{
        clock::TimeSpan,
        protocol::DeviceInfo,
        scene::{Frame, Line, Scene, script::Script},
        schedule::ActionTiming,
        vm::{
            language::{
                LanguageDefinition, LanguageDocumentation, LanguageElement, LanguageSyntax,
                ReferenceEntry, SyntaxRule, TokenCategory,
            },
            variable::{VariableStore, VariableValue},
        },
    };

    fn roundtrip(msg: &ClientMessage) {
        let bytes = rmp_serde::to_vec_named(msg)
            .unwrap_or_else(|e| panic!("serialize failed for {:?}: {e}", std::mem::discriminant(msg)));
        rmp_serde::from_slice::<ClientMessage>(&bytes)
            .unwrap_or_else(|e| {
                panic!(
                    "deserialize failed for {:?} (len={}, first 32 bytes: {:02x?}): {e}",
                    std::mem::discriminant(msg),
                    bytes.len(),
                    &bytes[..bytes.len().min(32)],
                )
            });
    }

    fn frame_with_vars() -> Frame {
        let mut f = Frame::default();
        f.vars = VariableStore::from(HashMap::from([
            ("i".into(), VariableValue::Integer(10)),
            ("f".into(), VariableValue::Float(0.5)),
            ("d".into(), VariableValue::Dur(TimeSpan::Beats(2.0))),
            ("m".into(), VariableValue::Map(HashMap::from([
                ("k".into(), VariableValue::Str("v".into())),
            ]))),
            ("v".into(), VariableValue::Vec(vec![
                VariableValue::Bool(true),
                VariableValue::Integer(99),
            ])),
        ]));
        f.set_script(Script::new("note 60".into(), "bob".into()));
        f
    }

    #[test]
    fn client_message_roundtrip_with_variable_values() {
        let frame = frame_with_vars();
        let mut scene = Scene::default();
        scene.lines.push(Line::default());
        scene.lines[0].frames.push(frame.clone());

        let variants: Vec<ClientMessage> = vec![
            ClientMessage::SetScene(scene, ActionTiming::Immediate),
            ClientMessage::SetFrames(
                vec![(0, 0, frame.clone())],
                ActionTiming::AtNextBeat,
            ),
            ClientMessage::AddFrame(0, 1, frame, ActionTiming::Immediate),
        ];

        for msg in &variants {
            roundtrip(msg);
        }
    }

    // -- v2 frame structure --

    #[test]
    fn v2_frame_byte_layout() {
        let msg = ServerMessage::ClockState(120.0, 1.5, 1000, 4.0);
        let frame = serialize_to_wire_frame(&msg).unwrap();

        assert_eq!(frame[0], PROTOCOL_VERSION);
        let length = u32::from_be_bytes([0x00, frame[1], frame[2], frame[3]]);
        let expected_crc = u32::from_be_bytes([frame[4], frame[5], frame[6], frame[7]]);
        let payload = &frame[8..];
        assert_eq!(payload.len(), length as usize);
        assert_eq!(crc32fast::hash(payload), expected_crc);

        let decoded: ServerMessage = rmp_serde::from_slice(payload).unwrap();
        assert!(matches!(decoded, ServerMessage::ClockState(..)));
    }

    #[test]
    fn v2_frame_crc_is_over_payload() {
        let payload = b"test payload data";
        let frame = build_v2_frame_raw(payload);
        let crc_in_frame = u32::from_be_bytes([frame[4], frame[5], frame[6], frame[7]]);
        assert_eq!(crc_in_frame, crc32fast::hash(payload));
    }

    // -- helpers --

    fn build_v1_frame(payload: &[u8]) -> Vec<u8> {
        let len = (payload.len() as u32).to_be_bytes();
        [&len[..], payload].concat()
    }

    fn build_v1_server_msg(msg: &ServerMessage) -> Vec<u8> {
        let payload = rmp_serde::to_vec_named(msg).unwrap();
        build_v1_frame(&payload)
    }

    // -- stream roundtrip --

    #[tokio::test]
    async fn read_v2_frame() {
        let msg = ServerMessage::Success;
        let bytes = serialize_to_wire_frame(&msg).unwrap();
        let payload = read_wire_frame(&mut &bytes[..]).await.unwrap().unwrap();
        let decoded: ServerMessage = rmp_serde::from_slice(&payload).unwrap();
        assert!(matches!(decoded, ServerMessage::Success));
    }

    #[tokio::test]
    async fn read_v1_frame() {
        let msg = ServerMessage::Success;
        let bytes = build_v1_server_msg(&msg);
        let payload = read_wire_frame(&mut &bytes[..]).await.unwrap().unwrap();
        let decoded: ServerMessage = rmp_serde::from_slice(&payload).unwrap();
        assert!(matches!(decoded, ServerMessage::Success));
    }

    #[tokio::test]
    async fn read_multiple_v2_frames() {
        let msgs = [
            ServerMessage::Success,
            ServerMessage::ClockState(120.0, 1.5, 1000, 4.0),
            ServerMessage::Success,
        ];
        let mut buf = Vec::new();
        for m in &msgs {
            buf.extend_from_slice(&serialize_to_wire_frame(m).unwrap());
        }
        let mut cursor = &buf[..];
        for _ in &msgs {
            let payload = read_wire_frame(&mut cursor).await.unwrap().unwrap();
            let _: ServerMessage = rmp_serde::from_slice(&payload).unwrap();
        }
    }

    #[tokio::test]
    async fn read_mixed_v1_v2() {
        let msg = ServerMessage::Success;
        let mut buf = Vec::new();
        buf.extend_from_slice(&build_v1_server_msg(&msg));
        buf.extend_from_slice(&serialize_to_wire_frame(&msg).unwrap());
        buf.extend_from_slice(&build_v1_server_msg(&msg));

        let mut cursor = &buf[..];
        for _ in 0..3 {
            let payload = read_wire_frame(&mut cursor).await.unwrap().unwrap();
            let decoded: ServerMessage = rmp_serde::from_slice(&payload).unwrap();
            assert!(matches!(decoded, ServerMessage::Success));
        }
    }

    // -- corruption detection --

    #[tokio::test]
    async fn crc_catches_flipped_payload_byte() {
        let msg = ServerMessage::Success;
        let mut bytes = serialize_to_wire_frame(&msg).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        let result = read_wire_frame(&mut &bytes[..]).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn crc_catches_flipped_crc_byte() {
        let msg = ServerMessage::Success;
        let mut bytes = serialize_to_wire_frame(&msg).unwrap();
        bytes[5] ^= 0xFF; // flip a CRC byte
        let result = read_wire_frame(&mut &bytes[..]).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn frame_after_corruption_reads_ok() {
        let msg = ServerMessage::Success;
        let mut corrupted = serialize_to_wire_frame(&msg).unwrap();
        let last = corrupted.len() - 1;
        corrupted[last] ^= 0xFF;
        let valid = serialize_to_wire_frame(&msg).unwrap();

        let mut buf = Vec::new();
        buf.extend_from_slice(&corrupted);
        buf.extend_from_slice(&valid);

        let mut cursor = &buf[..];
        // First frame: corrupted → None
        assert!(read_wire_frame(&mut cursor).await.unwrap().is_none());
        // Second frame: valid → Some
        assert!(read_wire_frame(&mut cursor).await.unwrap().is_some());
    }

    // -- error conditions --

    #[tokio::test]
    async fn unsupported_version_byte() {
        let bytes = [0xFFu8, 0, 0, 1, 0, 0, 0, 0, 0x42];
        let err = read_wire_frame(&mut &bytes[..]).await.unwrap_err();
        assert!(err.to_string().contains("Unsupported protocol version: 0xff"));
    }

    #[tokio::test]
    async fn zero_length_payload() {
        let frame = build_v2_frame_raw(b"");
        // Manually patch length to 0 (it already is since payload is empty)
        let err = read_wire_frame(&mut &frame[..]).await.unwrap_err();
        assert!(err.to_string().contains("Invalid message length: 0"));
    }

    #[tokio::test]
    async fn oversized_length() {
        // v2 header with length > MAX_MESSAGE_SIZE
        let mut frame = vec![PROTOCOL_VERSION];
        let big_len = (MAX_MESSAGE_SIZE + 1).to_be_bytes();
        frame.extend_from_slice(&big_len[1..4]);
        frame.extend_from_slice(&[0u8; 4]); // CRC placeholder
        let err = read_wire_frame(&mut &frame[..]).await.unwrap_err();
        assert!(err.to_string().contains("Invalid message length"));
    }

    #[tokio::test]
    async fn eof_mid_frame() {
        // v2 header but truncated before payload
        let mut frame = vec![PROTOCOL_VERSION];
        let len_bytes = 100u32.to_be_bytes();
        frame.extend_from_slice(&len_bytes[1..4]);
        frame.extend_from_slice(&[0u8; 4]); // CRC
        // No payload bytes
        let err = read_wire_frame(&mut &frame[..]).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    // -- serialization compat --

    #[test]
    fn named_encoding_reads_both_formats() {
        let msg = ClientMessage::GetScene;
        let named = rmp_serde::to_vec_named(&msg).unwrap();
        rmp_serde::from_slice::<ClientMessage>(&named).unwrap();
        let compact = rmp_serde::to_vec(&msg).unwrap();
        rmp_serde::from_slice::<ClientMessage>(&compact).unwrap();
    }

    // -- realistic message builders --

    fn make_device(slot: usize, name: &str) -> DeviceInfo {
        DeviceInfo {
            slot_id: Some(slot),
            name: name.into(),
            kind: sova_core::protocol::DeviceKind::Midi,
            direction: sova_core::protocol::DeviceDirection::Output,
            is_connected: true,
            address: None,
            latency: 2.0,
        }
    }

    fn make_scene_with_code() -> Scene {
        let bob_code = r#"
            note 60 0.5
            note 64 0.5
            cc 1 64
            note 67 1.0
            let x = 42
            repeat 4 { note (60 + x) 0.25 }
        "#;
        let bali_code = "n 60 . n 64 . n 67 . n 72";
        let forth_code = ": melody 60 note 64 note 67 note ;";

        let mut scene = Scene::default();
        for (code, lang) in [(bob_code, "bob"), (bali_code, "bali"), (forth_code, "forth")] {
            let mut line = Line::default();
            line.frames.clear();
            line.looping = true;
            line.speed_factor = 1.0;

            for i in 0..4 {
                let mut frame = Frame::default();
                frame.duration = 4.0;
                frame.repetitions = if i == 0 { 2 } else { 1 };
                frame.enabled = i != 3;
                frame.set_script(Script::new(code.into(), lang.into()));
                frame.vars = VariableStore::from(HashMap::from([
                    ("device".into(), VariableValue::Integer(1)),
                    ("chan".into(), VariableValue::Integer(1)),
                    ("vel".into(), VariableValue::Integer(100)),
                    ("scale".into(), VariableValue::Str("minor".into())),
                ]));
                line.frames.push(frame);
            }
            scene.lines.push(line);
        }
        scene
    }

    fn make_hello() -> ServerMessage {
        use std::collections::BTreeMap;
        use std::path::PathBuf;

        let scene = make_scene_with_code();
        let devices: Vec<DeviceInfo> = (1..=5)
            .map(|i| make_device(i, &format!("MIDI Device {i}")))
            .collect();
        let peers = vec!["alice".into(), "bob".into(), "charlie".into()];

        let lang_def = |name: &str| LanguageDefinition {
            name: name.into(),
            documentation: LanguageDocumentation {
                articles: vec![
                    ("Getting Started".into(), format!("How to use {name} for live coding.")),
                    ("Reference".into(), format!("{name} built-in functions and operators.")),
                ],
                reference: BTreeMap::from([
                    (LanguageElement::Word("note".into()), ReferenceEntry {
                        description: "Play a MIDI note".into(),
                        example: Some("note 60 0.5".into()),
                        category: Some("MIDI".into()),
                        aliases: vec!["n".into()],
                    }),
                    (LanguageElement::Brackets("{".into(), "}".into()), ReferenceEntry {
                        description: "Block delimiter".into(),
                        example: None,
                        category: Some("Syntax".into()),
                        aliases: vec![],
                    }),
                ]),
                escape: vec![("\\n".into(), "newline".into())],
            },
            syntax: Some(LanguageSyntax {
                rules: vec![
                    SyntaxRule::new(TokenCategory::Keyword, r"\b(let|if|else|repeat|fn)\b"),
                    SyntaxRule::new(TokenCategory::Builtin, r"\b(note|cc|sleep|random)\b"),
                    SyntaxRule::new(TokenCategory::Number, r"\b\d+(\.\d+)?\b"),
                    SyntaxRule::new(TokenCategory::Comment, r"//.*$"),
                ],
            }),
        };

        let audio = crate::audio::AudioEngineState {
            running: true,
            device: Some("BlackHole 16ch".into()),
            sample_rate: 44100.0,
            channels: 2,
            buffer_size: Some(512),
            active_voices: 8,
            sample_paths: vec![PathBuf::from("/samples/drums"), PathBuf::from("/samples/synth")],
            error: None,
            cpu_load: 12.5,
            peak_voices: 16,
            max_voices: 64,
            schedule_depth: 32,
            sample_pool_mb: 128.5,
        };

        ServerMessage::Hello {
            username: "test_user".into(),
            scene,
            devices,
            peers,
            link_state: (120.0, 3.75, 4.0, 4, true),
            is_playing: true,
            languages: vec![lang_def("bob"), lang_def("bali"), lang_def("forth"), lang_def("boinx")],
            audio_engine_state: audio,
        }
    }

    fn make_scope_data(samples: usize) -> ServerMessage {
        let data: Vec<f32> = (0..samples)
            .map(|i| (i as f32 * 0.01).sin() * 0.8)
            .collect();
        ServerMessage::ScopeData(data)
    }

    fn make_frame_position(lines: usize) -> ServerMessage {
        let positions: Vec<Vec<(usize, usize)>> = (0..lines)
            .map(|l| vec![(l % 8, l * 2), (l % 4, l)])
            .collect();
        ServerMessage::FramePosition(positions)
    }

    /// Serialize a ServerMessage to v2 wire frame, read it back, deserialize, return.
    async fn server_msg_wire_roundtrip(msg: &ServerMessage) -> ServerMessage {
        let frame = serialize_to_wire_frame(msg).unwrap();
        let payload = read_wire_frame(&mut &frame[..]).await.unwrap().unwrap();
        rmp_serde::from_slice(&payload).unwrap()
    }

    /// Serialize a ClientMessage to v2 wire frame, read it back, deserialize, return.
    async fn client_msg_wire_roundtrip(msg: &ClientMessage) -> ClientMessage {
        let payload = rmp_serde::to_vec_named(msg).unwrap();
        let frame = build_v2_frame_raw(&payload);
        let read_back = read_wire_frame(&mut &frame[..]).await.unwrap().unwrap();
        rmp_serde::from_slice(&read_back).unwrap()
    }

    // -- realistic message roundtrips --

    #[tokio::test]
    async fn v2_roundtrip_hello() {
        let msg = make_hello();
        let frame = serialize_to_wire_frame(&msg).unwrap();
        assert!(frame.len() > 500, "Hello should be a large payload, got {} bytes", frame.len());

        let decoded = server_msg_wire_roundtrip(&msg).await;
        assert!(matches!(decoded, ServerMessage::Hello { .. }));
    }

    #[tokio::test]
    async fn v2_roundtrip_scope_data() {
        let msg = make_scope_data(1024);
        let decoded = server_msg_wire_roundtrip(&msg).await;
        match decoded {
            ServerMessage::ScopeData(ref samples) => assert_eq!(samples.len(), 1024),
            other => panic!("expected ScopeData, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[tokio::test]
    async fn v2_roundtrip_frame_position() {
        let msg = make_frame_position(16);
        let decoded = server_msg_wire_roundtrip(&msg).await;
        match decoded {
            ServerMessage::FramePosition(ref positions) => assert_eq!(positions.len(), 16),
            other => panic!("expected FramePosition, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[tokio::test]
    async fn v2_roundtrip_scene_with_code() {
        let msg = ServerMessage::SceneValue(make_scene_with_code());
        let decoded = server_msg_wire_roundtrip(&msg).await;
        match decoded {
            ServerMessage::SceneValue(ref scene) => {
                assert_eq!(scene.lines.len(), 3);
                assert_eq!(scene.lines[0].frames.len(), 4);
            }
            other => panic!("expected SceneValue, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[tokio::test]
    async fn v1_roundtrip_hello() {
        let msg = make_hello();
        let bytes = build_v1_server_msg(&msg);
        assert!(bytes.len() > 500);

        let payload = read_wire_frame(&mut &bytes[..]).await.unwrap().unwrap();
        let decoded: ServerMessage = rmp_serde::from_slice(&payload).unwrap();
        assert!(matches!(decoded, ServerMessage::Hello { .. }));
    }

    // -- sustained mixed traffic --

    #[tokio::test]
    async fn sustained_mixed_traffic() {
        use sova_core::compiler::{CompilationError, CompilationState};

        let mut buf = Vec::new();
        let mut expected_count = 0;

        // 30 FramePosition (one per ~33ms)
        for _ in 0..30 {
            buf.extend_from_slice(&serialize_to_wire_frame(&make_frame_position(16)).unwrap());
            expected_count += 1;
        }
        // 60 ScopeData (one per ~16ms)
        for i in 0..60 {
            buf.extend_from_slice(&serialize_to_wire_frame(&make_scope_data(512 + i * 8)).unwrap());
            expected_count += 1;
        }
        // 2 ClockState
        for _ in 0..2 {
            buf.extend_from_slice(
                &serialize_to_wire_frame(&ServerMessage::ClockState(120.0, 42.5, 123456789, 4.0))
                    .unwrap(),
            );
            expected_count += 1;
        }
        // 1 CompilationUpdate
        buf.extend_from_slice(
            &serialize_to_wire_frame(&ServerMessage::CompilationUpdate(
                0,
                0,
                7,
                CompilationState::Error(CompilationError {
                    lang: "bob".into(),
                    info: "unexpected token '}'".into(),
                    from: 12,
                    to: 13,
                }),
            ))
            .unwrap(),
        );
        expected_count += 1;

        let mut cursor = &buf[..];
        let mut read_count = 0;
        while let Ok(Some(_payload)) = read_wire_frame(&mut cursor).await {
            read_count += 1;
        }
        assert_eq!(read_count, expected_count);
    }

    // -- all ServerMessage variants through wire --

    #[tokio::test]
    async fn all_server_message_variants_through_wire() {
        use sova_core::{
            compiler::{CompilationError, CompilationState},
            error::SovaError,
            protocol::log::{LogMessage, Severity},
            schedule::{SchedulerMessage, playback::PlaybackState},
        };

        let scene = make_scene_with_code();
        let device = make_device(1, "Test MIDI");
        let audio = crate::audio::AudioEngineState::default();
        let snapshot = crate::server::Snapshot {
            scene: scene.clone(),
            tempo: 120.0,
            beat: 3.5,
            micros: 999_999,
            quantum: 4.0,
            devices: vec![device.clone()],
        };

        let variants: Vec<ServerMessage> = vec![
            make_hello(),
            ServerMessage::PeersUpdated(vec!["alice".into(), "bob".into()]),
            ServerMessage::PeerStartedEditing("user".into(), 0, 1),
            ServerMessage::PeerStoppedEditing("user".into(), 0, 1),
            ServerMessage::PeerCursorMoved("user".into(), 2, 3),
            ServerMessage::PlaybackStateChanged(PlaybackState::Stopped),
            ServerMessage::PlaybackStateChanged(PlaybackState::Starting(1.0)),
            ServerMessage::PlaybackStateChanged(PlaybackState::Playing),
            ServerMessage::Log(LogMessage {
                level: Severity::Info,
                event: None,
                msg: "test log".into(),
            }),
            ServerMessage::Chat("alice".into(), "hello everyone".into()),
            ServerMessage::Success,
            ServerMessage::InternalError("something went wrong".into()),
            ServerMessage::ConnectionRefused("server full".into()),
            ServerMessage::Snapshot(snapshot),
            ServerMessage::DeviceList(vec![device.clone()]),
            ServerMessage::ClockState(140.0, 7.25, 5_000_000, 4.0),
            ServerMessage::SceneValue(scene.clone()),
            ServerMessage::SceneMode(ExecutionMode::Free),
            ServerMessage::SceneMode(ExecutionMode::AtQuantum),
            ServerMessage::SceneMode(ExecutionMode::LongestLine),
            ServerMessage::LineValues(vec![(0, Line::default()), (1, Line::default())]),
            ServerMessage::LineConfigurations(vec![(0, Line::default())]),
            ServerMessage::AddLine(2, Line::default()),
            ServerMessage::RemoveLine(1),
            ServerMessage::FrameValues(vec![(0, 0, frame_with_vars())]),
            ServerMessage::AddFrame(1, 0, frame_with_vars()),
            ServerMessage::RemoveFrame(0, 3),
            make_frame_position(16),
            ServerMessage::GlobalVariablesUpdate(HashMap::from([
                ("bpm".into(), VariableValue::Float(120.0)),
                ("scale".into(), VariableValue::Str("dorian".into())),
            ])),
            ServerMessage::CompilationUpdate(0, 0, 1, CompilationState::NotCompiled),
            ServerMessage::CompilationUpdate(0, 0, 2, CompilationState::Compiling),
            ServerMessage::CompilationUpdate(0, 0, 3, CompilationState::Compiled(Default::default())),
            ServerMessage::CompilationUpdate(0, 0, 4, CompilationState::Parsed(None)),
            ServerMessage::CompilationUpdate(
                0,
                0,
                5,
                CompilationState::Error(CompilationError {
                    lang: "bob".into(),
                    info: "syntax error at line 3".into(),
                    from: 20,
                    to: 25,
                }),
            ),
            ServerMessage::DevicesRestored {
                missing_devices: vec!["IAC Driver".into()],
            },
            ServerMessage::AudioEngineState(audio),
            make_scope_data(1024),
            ServerMessage::Error(SovaError {
                line: 0,
                frame: 1,
                position: None,
                text: "division by zero".into(),
            }),
            ServerMessage::FeedbackEnabled {
                scene: scene.clone(),
                tempo: 120.0,
                quantum: 4.0,
                is_playing: true,
            },
            ServerMessage::Feedback(SchedulerMessage::SetTempo(140.0, ActionTiming::Immediate)),
            ServerMessage::Feedback(SchedulerMessage::TransportStart(ActionTiming::AtNextBeat)),
            ServerMessage::Feedback(SchedulerMessage::SetScene(scene, ActionTiming::Immediate)),
        ];

        for msg in &variants {
            let frame = serialize_to_wire_frame(msg).unwrap();
            let payload = read_wire_frame(&mut &frame[..]).await.unwrap().unwrap();
            rmp_serde::from_slice::<ServerMessage>(&payload).unwrap_or_else(|e| {
                panic!(
                    "wire roundtrip failed for {:?} (frame={} bytes): {e}",
                    std::mem::discriminant(msg),
                    frame.len()
                )
            });
        }
    }

    // -- all ClientMessage variants through wire --

    #[tokio::test]
    async fn all_client_message_variants_through_wire() {
        let scene = make_scene_with_code();
        let frame = frame_with_vars();
        let device = make_device(1, "Test MIDI");

        let variants: Vec<ClientMessage> = vec![
            ClientMessage::SchedulerControl(sova_core::schedule::SchedulerMessage::SetTempo(
                120.0,
                ActionTiming::Immediate,
            )),
            ClientMessage::SetTempo(140.0, ActionTiming::AtNextBeat),
            ClientMessage::SetName("alice".into()),
            ClientMessage::GetScene,
            ClientMessage::SetScene(scene.clone(), ActionTiming::Immediate),
            ClientMessage::GetLine(0),
            ClientMessage::SetLines(
                vec![(0, Line::default()), (1, Line::default())],
                ActionTiming::AtNextBeat,
            ),
            ClientMessage::ConfigureLines(vec![(0, Line::default())], ActionTiming::Immediate),
            ClientMessage::AddLine(2, Line::default(), ActionTiming::Immediate),
            ClientMessage::RemoveLine(1, ActionTiming::AtNextBeat),
            ClientMessage::GetFrame(0, 0),
            ClientMessage::SetFrames(
                vec![(0, 0, frame.clone()), (1, 0, frame.clone())],
                ActionTiming::Immediate,
            ),
            ClientMessage::AddFrame(0, 1, frame, ActionTiming::Immediate),
            ClientMessage::RemoveFrame(0, 2, ActionTiming::AtNextBeat),
            ClientMessage::GetClock,
            ClientMessage::GetPeers,
            ClientMessage::Chat("hello world".into()),
            ClientMessage::GetSnapshot,
            ClientMessage::StartedEditingFrame(0, 1),
            ClientMessage::StoppedEditingFrame(0, 1),
            ClientMessage::CursorPosition(3, 15),
            ClientMessage::TransportStart(ActionTiming::AtNextBeat),
            ClientMessage::TransportStop(ActionTiming::Immediate),
            ClientMessage::SetSceneMode(ExecutionMode::AtQuantum, ActionTiming::Immediate),
            ClientMessage::RequestDeviceList,
            ClientMessage::ConnectMidiDeviceByName("IAC Driver Bus 1".into()),
            ClientMessage::DisconnectMidiDeviceByName("IAC Driver Bus 1".into()),
            ClientMessage::CreateVirtualMidiOutput("Sova Out".into()),
            ClientMessage::AssignDeviceToSlot(2, "MIDI Device 1".into()),
            ClientMessage::UnassignDeviceFromSlot(2),
            ClientMessage::CreateOscDevice("sc".into(), "127.0.0.1".into(), 57120),
            ClientMessage::RemoveOscDevice("sc".into()),
            ClientMessage::RestoreDevices(vec![device]),
            ClientMessage::GetAudioEngineState,
            ClientMessage::RestartAudioEngine {
                device: Some("BlackHole 16ch".into()),
                input_device: None,
                channels: 2,
                buffer_size: Some(512),
                sample_paths: vec!["/samples/drums".into(), "/samples/synth".into()],
            },
            ClientMessage::PreviewSample {
                folder: "/samples/drums".into(),
                index: 3,
                begin: 0.0,
            },
            ClientMessage::EnableFeedback,
        ];

        for msg in &variants {
            let decoded = client_msg_wire_roundtrip(msg).await;
            assert_eq!(
                std::mem::discriminant(msg),
                std::mem::discriminant(&decoded),
                "variant mismatch for {:?}",
                std::mem::discriminant(msg)
            );
        }
    }

    // -- corruption in realistic payloads --

    #[tokio::test]
    async fn corruption_in_large_payload() {
        let msg = make_hello();
        let mut bytes = serialize_to_wire_frame(&msg).unwrap();
        assert!(bytes.len() > 100);
        // Corrupt a byte in the middle of the payload
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        let result = read_wire_frame(&mut &bytes[..]).await.unwrap();
        assert!(result.is_none(), "CRC should catch corruption in large payload");
    }

    #[tokio::test]
    async fn corruption_recovery_with_realistic_messages() {
        let scope = make_scope_data(512);
        let position = make_frame_position(16);

        let mut corrupted = serialize_to_wire_frame(&scope).unwrap();
        let corrupt_idx = corrupted.len() / 2;
        corrupted[corrupt_idx] ^= 0xAA;
        let valid = serialize_to_wire_frame(&position).unwrap();

        let mut buf = Vec::new();
        buf.extend_from_slice(&corrupted);
        buf.extend_from_slice(&valid);

        let mut cursor = &buf[..];
        // Corrupted ScopeData → None
        assert!(read_wire_frame(&mut cursor).await.unwrap().is_none());
        // Valid FramePosition → Some
        let payload = read_wire_frame(&mut cursor).await.unwrap().unwrap();
        let decoded: ServerMessage = rmp_serde::from_slice(&payload).unwrap();
        assert!(matches!(decoded, ServerMessage::FramePosition(..)));
    }

    // -- corner cases --

    #[tokio::test]
    async fn minimal_one_byte_payload_v2() {
        let payload = &[0x42u8];
        let frame = build_v2_frame_raw(payload);
        let read_back = read_wire_frame(&mut &frame[..]).await.unwrap().unwrap();
        assert_eq!(read_back, payload);
    }

    #[tokio::test]
    async fn minimal_one_byte_payload_v1() {
        let payload = &[0x42u8];
        let frame = build_v1_frame(payload);
        let read_back = read_wire_frame(&mut &frame[..]).await.unwrap().unwrap();
        assert_eq!(read_back, payload);
    }

    #[tokio::test]
    async fn max_message_size_boundary() {
        // Exactly at MAX_MESSAGE_SIZE should be accepted
        let payload = vec![0xABu8; MAX_MESSAGE_SIZE as usize];
        let frame = build_v2_frame_raw(&payload);
        let read_back = read_wire_frame(&mut &frame[..]).await.unwrap().unwrap();
        assert_eq!(read_back.len(), MAX_MESSAGE_SIZE as usize);
    }

    #[tokio::test]
    async fn one_over_max_message_size_rejected() {
        let len = MAX_MESSAGE_SIZE + 1;
        let len_bytes = len.to_be_bytes();
        let mut frame = vec![PROTOCOL_VERSION];
        frame.extend_from_slice(&len_bytes[1..4]);
        frame.extend_from_slice(&[0u8; 4]); // CRC
        frame.extend_from_slice(&vec![0u8; len as usize]);
        let err = read_wire_frame(&mut &frame[..]).await.unwrap_err();
        assert!(err.to_string().contains("Invalid message length"));
    }

    #[tokio::test]
    async fn version_byte_0x01_rejected() {
        let mut frame = vec![0x01u8];
        frame.extend_from_slice(&[0, 0, 5]); // length
        frame.extend_from_slice(&[0u8; 5]); // payload
        let err = read_wire_frame(&mut &frame[..]).await.unwrap_err();
        assert!(err.to_string().contains("Unsupported protocol version: 0x01"));
    }

    #[tokio::test]
    async fn multiple_consecutive_corrupted_then_recovery() {
        let valid_msg = make_frame_position(8);
        let valid_frame = serialize_to_wire_frame(&valid_msg).unwrap();

        let mut buf = Vec::new();
        // 3 corrupted frames
        for i in 0..3u8 {
            let mut bad = serialize_to_wire_frame(&make_scope_data(256)).unwrap();
            let last = bad.len() - 1;
            bad[last] ^= 0x10 + i;
            buf.extend_from_slice(&bad);
        }
        // 1 valid frame
        buf.extend_from_slice(&valid_frame);

        let mut cursor = &buf[..];
        for _ in 0..3 {
            assert!(read_wire_frame(&mut cursor).await.unwrap().is_none());
        }
        let payload = read_wire_frame(&mut cursor).await.unwrap().unwrap();
        let decoded: ServerMessage = rmp_serde::from_slice(&payload).unwrap();
        assert!(matches!(decoded, ServerMessage::FramePosition(..)));
    }

    #[tokio::test]
    async fn length_corruption_shrinks_causes_desync() {
        // Corrupt the length field to be smaller than actual payload.
        // Reader consumes fewer bytes, leaving leftover bytes that
        // start the next "frame" mid-payload — should error or EOF.
        let msg = make_hello();
        let mut frame = serialize_to_wire_frame(&msg).unwrap();
        let real_len = u32::from_be_bytes([0x00, frame[1], frame[2], frame[3]]);
        assert!(real_len > 100);

        // Shrink length to 10 — reader reads 10 bytes as "payload",
        // CRC won't match → None, then leftover bytes are garbage.
        let small_len = 10u32.to_be_bytes();
        frame[1] = small_len[1];
        frame[2] = small_len[2];
        frame[3] = small_len[3];

        let mut cursor = &frame[..];
        // First read: CRC mismatch on the 10-byte "payload"
        assert!(read_wire_frame(&mut cursor).await.unwrap().is_none());
        // Second read: starts mid-payload, version byte is garbage
        let result = read_wire_frame(&mut cursor).await;
        assert!(result.is_err() || matches!(result, Ok(None)));
    }

    #[tokio::test]
    async fn length_corruption_grows_causes_eof() {
        // Corrupt the length field to be larger than actual payload.
        // Reader tries to read more bytes than exist → UnexpectedEof.
        let msg = ServerMessage::Success;
        let mut frame = serialize_to_wire_frame(&msg).unwrap();

        // Inflate length to 5000
        let big_len = 5000u32.to_be_bytes();
        frame[1] = big_len[1];
        frame[2] = big_len[2];
        frame[3] = big_len[3];

        let err = read_wire_frame(&mut &frame[..]).await.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }
}

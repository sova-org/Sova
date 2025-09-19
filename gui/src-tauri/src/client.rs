use anyhow::Result;
use sova_core::server::client::{ClientMessage, CompressionStrategy};
use sova_core::server::ServerMessage;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

struct BufferPool {
    small_buffers: Vec<Vec<u8>>,
    large_buffers: Vec<Vec<u8>>,
}

impl BufferPool {
    fn new() -> Self {
        BufferPool {
            small_buffers: Vec::new(),
            large_buffers: Vec::new(),
        }
    }

    fn get_buffer(&mut self, size: usize) -> Vec<u8> {
        if size < 1024 {
            self.small_buffers
                .pop()
                .map(|mut buf| {
                    buf.clear();
                    buf.reserve(size);
                    buf
                })
                .unwrap_or_else(|| Vec::with_capacity(size.max(512)))
        } else {
            self.large_buffers
                .pop()
                .map(|mut buf| {
                    buf.clear();
                    buf.reserve(size);
                    buf
                })
                .unwrap_or_else(|| Vec::with_capacity(size.max(2048)))
        }
    }

    #[allow(dead_code)]
    fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        if buffer.capacity() < 1024 && self.small_buffers.len() < 8 {
            buffer.clear();
            self.small_buffers.push(buffer);
        } else if buffer.capacity() >= 1024 && self.large_buffers.len() < 4 {
            buffer.clear();
            self.large_buffers.push(buffer);
        }
    }
}

pub struct BuboCoreClient {
    pub ip: String,
    pub port: u16,
    pub stream: Option<TcpStream>,
    pub connected: bool,
    buffer_pool: BufferPool,
}

impl BuboCoreClient {
    pub fn new(ip: String, port: u16) -> Self {
        BuboCoreClient {
            ip,
            port,
            stream: None,
            connected: false,
            buffer_pool: BufferPool::new(),
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let addr: SocketAddr = format!("{}:{}", self.ip, self.port).parse()?;
        self.stream = Some(TcpStream::connect(addr).await?);
        self.connected = true;
        Ok(())
    }

    pub async fn send(&mut self, message: ClientMessage) -> Result<()> {
        let msgpack_bytes = rmp_serde::to_vec_named(&message)?;
        let (final_bytes, is_compressed) = self.compress_intelligently(&message, &msgpack_bytes)?;

        let mut length = final_bytes.len() as u32;
        if is_compressed {
            length |= 0x80000000;
        }

        let socket = self.mut_socket()?;
        socket.write_all(&length.to_be_bytes()).await?;
        socket.write_all(&final_bytes).await?;

        Ok(())
    }

    fn compress_intelligently(
        &mut self,
        message: &ClientMessage,
        msgpack_bytes: &[u8],
    ) -> Result<(Vec<u8>, bool)> {
        match message.compression_strategy() {
            CompressionStrategy::Never => {
                let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                buffer.extend_from_slice(msgpack_bytes);
                Ok((buffer, false))
            }
            CompressionStrategy::Always => {
                if msgpack_bytes.len() > 64 {
                    let compression_level = if msgpack_bytes.len() < 1024 { 1 } else { 3 };
                    let compressed = zstd::encode_all(msgpack_bytes, compression_level)?;
                    if compressed.len() < msgpack_bytes.len() {
                        Ok((compressed, true))
                    } else {
                        let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                        buffer.extend_from_slice(msgpack_bytes);
                        Ok((buffer, false))
                    }
                } else {
                    let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                    buffer.extend_from_slice(msgpack_bytes);
                    Ok((buffer, false))
                }
            }
            CompressionStrategy::Adaptive => {
                if msgpack_bytes.len() < 256 {
                    let mut buffer = self.buffer_pool.get_buffer(msgpack_bytes.len());
                    buffer.extend_from_slice(msgpack_bytes);
                    Ok((buffer, false))
                } else {
                    let compression_level = if msgpack_bytes.len() < 1024 { 1 } else { 3 };
                    let compressed = zstd::encode_all(msgpack_bytes, compression_level)?;
                    Ok((compressed, true))
                }
            }
        }
    }

    fn mut_socket(&mut self) -> Result<&mut TcpStream> {
        match &mut self.stream {
            Some(x) => Ok(x),
            None => Err(anyhow::anyhow!("Client not connected")),
        }
    }

    pub async fn ready(&mut self) -> bool {
        let mut buf = [0];
        let Ok(socket) = self.socket() else {
            return false;
        };
        match socket.peek(&mut buf).await {
            Ok(0) => {
                self.connected = false;
                false
            }
            Ok(_) => true,
            Err(_) => {
                self.connected = false;
                false
            }
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        if let Some(mut stream) = self.stream.take() {
            let _ = stream.shutdown().await;
        }
        Ok(())
    }

    fn socket(&self) -> Result<&TcpStream> {
        match &self.stream {
            Some(x) => Ok(x),
            None => Err(anyhow::anyhow!("Client not connected")),
        }
    }

    pub async fn read(&mut self) -> Result<ServerMessage> {
        if !self.connected {
            return Err(anyhow::anyhow!("Client not connected"));
        }

        let socket = self.mut_socket()?;

        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf).await?;

        let len_with_flag = u32::from_be_bytes(len_buf);
        let is_compressed = (len_with_flag & 0x80000000) != 0;
        let length = len_with_flag & 0x7FFFFFFF;

        if length == 0 {
            return Err(anyhow::anyhow!("Received zero-length message"));
        }

        let mut message_buf = vec![0u8; length as usize];
        socket.read_exact(&mut message_buf).await?;

        let final_bytes = if is_compressed {
            zstd::decode_all(message_buf.as_slice())?
        } else {
            message_buf
        };

        let message = rmp_serde::from_slice::<ServerMessage>(&final_bytes)?;
        Ok(message)
    }
}

pub struct ClientManager {
    client: Option<BuboCoreClient>,
    message_sender: Option<mpsc::UnboundedSender<ClientMessage>>,
    message_receiver: Option<mpsc::UnboundedReceiver<ServerMessage>>,
    disconnect_sender: Option<mpsc::UnboundedSender<()>>,
}

impl Default for ClientManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientManager {
    pub fn new() -> Self {
        ClientManager {
            client: None,
            message_sender: None,
            message_receiver: None,
            disconnect_sender: None,
        }
    }

    pub async fn connect(&mut self, ip: String, port: u16) -> Result<()> {
        let mut client = BuboCoreClient::new(ip, port);
        client.connect().await?;

        let (msg_tx, msg_rx) = mpsc::unbounded_channel();
        let (server_tx, server_rx) = mpsc::unbounded_channel();
        let (disconnect_tx, disconnect_rx) = mpsc::unbounded_channel();

        self.spawn_client_task(client, msg_rx, server_tx, disconnect_rx).await;

        self.message_sender = Some(msg_tx);
        self.message_receiver = Some(server_rx);
        self.disconnect_sender = Some(disconnect_tx);

        Ok(())
    }

    async fn spawn_client_task(
        &self,
        mut client: BuboCoreClient,
        mut message_receiver: mpsc::UnboundedReceiver<ClientMessage>,
        server_sender: mpsc::UnboundedSender<ServerMessage>,
        mut disconnect_receiver: mpsc::UnboundedReceiver<()>,
    ) {
        tauri::async_runtime::spawn(async move {
            let mut consecutive_failures = 0;
            loop {
                tokio::select! {
                    Some(message) = message_receiver.recv() => {
                        if let Err(e) = client.send(message).await {
                            eprintln!("Failed to send message: {}", e);
                            return;
                        }
                    }
                    Some(_) = disconnect_receiver.recv() => {
                        eprintln!("Disconnect signal received, closing connection");
                        if let Err(e) = client.disconnect().await {
                            eprintln!("Failed to disconnect client: {}", e);
                        }
                        return;
                    }
                    read_result = async {
                        if client.ready().await {
                            client.read().await
                        } else {
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                            Err(anyhow::anyhow!("Not ready"))
                        }
                    } => {
                        match read_result {
                            Ok(message) => {
                                consecutive_failures = 0;
                                if server_sender.send(message).is_err() {
                                    return;
                                }
                            }
                            Err(_) => {
                                consecutive_failures += 1;
                                if consecutive_failures > 500 { // ~5 seconds of failures
                                    eprintln!("Connection appears to be dead, task exiting");
                                    if let Err(e) = client.disconnect().await {
                                        eprintln!("Failed to disconnect client: {}", e);
                                    }
                                    return;
                                }
                                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn send_message(&self, message: ClientMessage) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            sender.send(message)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Not connected"))
        }
    }

    pub fn try_receive_message(&mut self) -> Option<ServerMessage> {
        if let Some(receiver) = &mut self.message_receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }

    pub fn is_connected(&self) -> bool {
        if let Some(sender) = &self.message_sender {
            // Check if the channel is still open (task is still running)
            !sender.is_closed()
        } else {
            false
        }
    }

    pub fn disconnect(&mut self) {
        // Send disconnect signal to the task
        if let Some(disconnect_sender) = &self.disconnect_sender {
            let _ = disconnect_sender.send(());
        }
        
        // Clear all channels
        self.message_sender = None;
        self.message_receiver = None;
        self.disconnect_sender = None;
        self.client = None;
    }
}
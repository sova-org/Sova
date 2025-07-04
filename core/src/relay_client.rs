use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
    time::{timeout, Duration},
};
use uuid::Uuid;

use crate::server::client::ClientMessage;

/// Version of the BuboCore protocol (must match relay server)
pub const BUBOCORE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Messages sent between relay server and BuboCore instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayMessage {
    /// Instance registration request
    RegisterInstance {
        instance_name: String,
        version: String,
        session_token: Option<String>,
    },
    
    /// Registration response
    RegistrationResponse {
        success: bool,
        message: String,
        assigned_id: Option<Uuid>,
        current_instances: Vec<InstanceInfo>,
    },

    /// State update from an instance to be relayed
    StateUpdate {
        source_instance_id: Uuid,
        timestamp: u64,
        update_data: Vec<u8>, // Serialized ClientMessage from core
    },

    /// Broadcast of state update to other instances
    StateBroadcast {
        source_instance_name: String,
        timestamp: u64,
        update_data: Vec<u8>,
    },

    /// Instance disconnection notification
    InstanceDisconnected {
        instance_id: Uuid,
        instance_name: String,
    },

    /// Ping for connection health check
    Ping { timestamp: u64 },
    
    /// Pong response
    Pong { timestamp: u64 },

    /// Error message
    Error { message: String },
}

/// Information about a connected instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub connected_at: SystemTime,
    pub last_activity: SystemTime,
}

/// Configuration for relay client
#[derive(Debug, Clone)]
pub struct RelayConfig {
    pub relay_address: String,
    pub instance_name: String,
    pub session_token: Option<String>,
}

/// Client for connecting to a relay server
pub struct RelayClient {
    config: RelayConfig,
    instance_id: Option<Uuid>,
    incoming_rx: mpsc::UnboundedReceiver<RelayMessage>,
    outgoing_tx: mpsc::UnboundedSender<RelayMessage>,
}

impl RelayClient {
    /// Create a new relay client
    pub fn new(config: RelayConfig) -> Self {
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();
        
        // These will be used by the connection handler
        let _ = (incoming_tx, outgoing_rx);
        
        Self {
            config,
            instance_id: None,
            incoming_rx,
            outgoing_tx,
        }
    }
    
    /// Connect to the relay server
    pub async fn connect(&mut self) -> Result<()> {
        println!("[RELAY] Connecting to {}...", self.config.relay_address);
        
        // Connect with timeout
        let mut socket = timeout(Duration::from_secs(10), TcpStream::connect(&self.config.relay_address))
            .await
            .map_err(|_| anyhow::anyhow!("Connection timeout"))?
            .map_err(|e| anyhow::anyhow!("Connection failed: {}", e))?;
        
        // Send registration message
        let register_msg = RelayMessage::RegisterInstance {
            instance_name: self.config.instance_name.clone(),
            version: BUBOCORE_VERSION.to_string(),
            session_token: self.config.session_token.clone(),
        };
        
        Self::send_message(&mut socket, &register_msg).await?;
        
        // Wait for registration response
        let response = Self::read_message(&mut socket).await?;
        
        match response {
            RelayMessage::RegistrationResponse { success, message, assigned_id, current_instances } => {
                if success {
                    self.instance_id = assigned_id;
                    println!("[RELAY] Connected successfully! Instance ID: {:?}", assigned_id);
                    println!("[RELAY] Other instances: {:?}", current_instances.iter().map(|i| &i.name).collect::<Vec<_>>());
                    
                    // Start message handlers
                    let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
                    let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel();
                    
                    // Replace our channels
                    self.incoming_rx = incoming_rx;
                    self.outgoing_tx = outgoing_tx.clone();
                    
                    // Split the socket into read and write halves
                    let (reader, writer) = socket.into_split();
                    
                    // Start reader task
                    let reader_tx = incoming_tx.clone();
                    tokio::spawn(async move {
                        let mut reader = reader;
                        loop {
                            match Self::read_message(&mut reader).await {
                                Ok(msg) => {
                                    if reader_tx.send(msg).is_err() {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[RELAY] Read error: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                    
                    // Start writer task
                    let mut writer = writer;
                    tokio::spawn(async move {
                        while let Some(msg) = outgoing_rx.recv().await {
                            if let Err(e) = Self::send_message(&mut writer, &msg).await {
                                eprintln!("[RELAY] Write error: {}", e);
                                break;
                            }
                        }
                    });
                    
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Registration failed: {}", message))
                }
            }
            RelayMessage::Error { message } => {
                Err(anyhow::anyhow!("Relay error: {}", message))
            }
            _ => {
                Err(anyhow::anyhow!("Unexpected response from relay"))
            }
        }
    }
    
    /// Send a client message to the relay
    pub async fn send_update(&self, client_msg: &ClientMessage) -> Result<()> {
        if let Some(instance_id) = self.instance_id {
            // Serialize the client message
            let update_data = rmp_serde::to_vec_named(client_msg)?;
            
            let relay_msg = RelayMessage::StateUpdate {
                source_instance_id: instance_id,
                timestamp: current_timestamp(),
                update_data,
            };
            
            self.outgoing_tx.send(relay_msg)?;
        }
        Ok(())
    }
    
    /// Receive incoming messages from relay
    pub async fn recv(&mut self) -> Option<RelayMessage> {
        self.incoming_rx.recv().await
    }
    
    /// Check if a client message should be relayed
    pub fn should_relay(msg: &ClientMessage) -> bool {
        use ClientMessage::*;
        match msg {
            // State-changing messages that should be relayed
            SetScript(..) | 
            EnableFrames(..) |
            DisableFrames(..) |
            UpdateLineFrames(..) |
            InsertFrame(..) |
            RemoveFrame(..) |
            SetScene(..) |
            SetFrameName(..) |
            SetScriptLanguage(..) |
            SetFrameRepetitions(..) |
            DuplicateFrameRange { .. } |
            RemoveFramesMultiLine { .. } |
            SetSceneLength(..) |
            SetLineLength(..) |
            SetLineSpeedFactor(..) |
            SetLineStartFrame(..) |
            SetLineEndFrame(..) |
            PasteDataBlock { .. } => true,
            
            // Local-only messages
            SetTempo(..) |
            GetClock |
            GetPeers |
            TransportStart(..) |
            TransportStop(..) |
            ConnectMidiDeviceByName(..) |
            DisconnectMidiDeviceByName(..) |
            CreateVirtualMidiOutput(..) |
            CreateOscDevice(..) |
            RemoveOscDevice(..) |
            AssignDeviceToSlot(..) |
            UnassignDeviceFromSlot(..) |
            RequestDeviceList |
            GetScene |
            GetSnapshot |
            GetSceneLength => false,
            
            // These need special handling
            UpdateGridSelection(..) => true, // Show remote cursors
            Chat(..) => true, // Relay chat messages
            StartedEditingFrame(..) => true,
            StoppedEditingFrame(..) => true,
            
            // Don't relay these
            SetName(..) => false, // Names are instance-specific
            GetScript(..) => false,
            SchedulerControl(..) => false, // Local scheduling
            
            // Deprecated - don't relay
            ConnectMidiDeviceById(..) |
            DisconnectMidiDeviceById(..) => false,
            
            // Default to not relaying unknown messages
            _ => false,
        }
    }
    
    /// Read a message from the socket
    async fn read_message(socket: &mut (impl AsyncReadExt + Unpin)) -> Result<RelayMessage> {
        // Read message length (4 bytes, big-endian)
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf).await?;
        let message_len = u32::from_be_bytes(len_buf) as usize;

        // Read message data
        let mut message_buf = vec![0u8; message_len];
        socket.read_exact(&mut message_buf).await?;

        // Deserialize
        let msg = rmp_serde::from_slice(&message_buf)?;
        Ok(msg)
    }

    /// Send a message to the socket
    async fn send_message(socket: &mut (impl AsyncWriteExt + Unpin), message: &RelayMessage) -> Result<()> {
        let message_bytes = rmp_serde::to_vec_named(message)?;
        let len = message_bytes.len() as u32;

        // Send length prefix and message
        socket.write_all(&len.to_be_bytes()).await?;
        socket.write_all(&message_bytes).await?;
        socket.flush().await?;

        Ok(())
    }
}

/// Get current timestamp in milliseconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
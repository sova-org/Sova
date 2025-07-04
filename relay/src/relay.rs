use crate::types::{
    InstanceInfo, RateLimit, RateLimitConfig, RelayError, RelayMessage, BUBOCORE_VERSION,
};
use anyhow::Result;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}},
    sync::{Mutex, RwLock, mpsc},
    time::{interval, Duration},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use hyper::{Request, Response, service::service_fn, Method, StatusCode};
use hyper_util::rt::TokioIo;
use http_body_util::Full;

/// Connection state for an instance
struct ConnectionState {
    instance_info: InstanceInfo,
    writer: Arc<Mutex<OwnedWriteHalf>>,
    _reader_handle: tokio::task::JoinHandle<()>,
}

/// Messages sent through the internal broadcast channel
#[derive(Debug, Clone)]
enum BroadcastMessage {
    StateUpdate {
        source_id: Uuid,
        source_name: String,
        timestamp: u64,
        data: Vec<u8>,
    },
    InstanceDisconnected {
        instance_id: Uuid,
        instance_name: String,
    },
}

/// Main relay server that manages BuboCore instance connections
pub struct RelayServer {
    /// Connected instances with their socket writers
    connections: Arc<RwLock<HashMap<Uuid, ConnectionState>>>,
    /// Instance info (for quick read access without locking connections)
    instances: Arc<RwLock<HashMap<Uuid, InstanceInfo>>>,
    /// Rate limiters per instance
    rate_limits: Arc<Mutex<HashMap<Uuid, RateLimit>>>,
    /// Broadcast channel for message distribution
    broadcast_tx: mpsc::UnboundedSender<BroadcastMessage>,
    broadcast_rx: Arc<Mutex<mpsc::UnboundedReceiver<BroadcastMessage>>>,
    /// Configuration
    max_instances: usize,
    rate_limit_config: RateLimitConfig,
}

impl RelayServer {
    pub fn new(max_instances: usize, rate_limit: u32) -> Self {
        let rate_limit_config = RateLimitConfig {
            messages_per_minute: rate_limit,
            ..Default::default()
        };

        let (broadcast_tx, broadcast_rx) = mpsc::unbounded_channel();

        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
            broadcast_tx,
            broadcast_rx: Arc::new(Mutex::new(broadcast_rx)),
            max_instances,
            rate_limit_config,
        }
    }

    /// Start the relay server
    pub async fn run(&self, relay_addr: SocketAddr, http_addr: SocketAddr) -> Result<()> {
        let relay_listener = TcpListener::bind(relay_addr).await?;
        let http_listener = TcpListener::bind(http_addr).await?;
        info!("Relay server listening on {}", relay_addr);
        info!("HTTP server listening on {}", http_addr);

        // Start broadcast distribution task
        let connections = self.connections.clone();
        let broadcast_rx = self.broadcast_rx.clone();
        tokio::spawn(async move {
            let mut rx = broadcast_rx.lock().await;
            while let Some(msg) = rx.recv().await {
                Self::distribute_broadcast(msg, &connections).await;
            }
        });

        // Start cleanup task for rate limiters
        let rate_limits = self.rate_limits.clone();
        let cleanup_interval = self.rate_limit_config.cleanup_interval_secs;
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(cleanup_interval));
            loop {
                interval.tick().await;
                Self::cleanup_rate_limits(&rate_limits).await;
            }
        });

        // Start HTTP server
        let instances_for_http = self.instances.clone();
        tokio::spawn(async move {
            loop {
                match http_listener.accept().await {
                    Ok((socket, addr)) => {
                        debug!("HTTP connection from {}", addr);
                        let instances = instances_for_http.clone();
                        tokio::spawn(async move {
                            let io = TokioIo::new(socket);
                            if let Err(e) = hyper::server::conn::http1::Builder::new()
                                .serve_connection(
                                    io,
                                    service_fn(move |req| Self::handle_http_request(req, instances.clone())),
                                )
                                .await
                            {
                                debug!("HTTP connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept HTTP connection: {}", e);
                    }
                }
            }
        });

        // Accept relay connections
        loop {
            match relay_listener.accept().await {
                Ok((socket, addr)) => {
                    info!("New relay connection from {}", addr);
                    let connections = self.connections.clone();
                    let instances = self.instances.clone();
                    let rate_limits = self.rate_limits.clone();
                    let max_instances = self.max_instances;
                    let rate_config = self.rate_limit_config.clone();
                    let broadcast_tx = self.broadcast_tx.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_instance(
                            socket,
                            addr,
                            connections,
                            instances,
                            rate_limits,
                            max_instances,
                            rate_config,
                            broadcast_tx,
                        )
                        .await
                        {
                            warn!("Error handling instance from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept relay connection: {}", e);
                }
            }
        }
        
        #[allow(unreachable_code)]
        Ok(())
    }

    /// Handle a single instance connection
    async fn handle_instance(
        mut socket: TcpStream,
        _addr: SocketAddr,
        connections: Arc<RwLock<HashMap<Uuid, ConnectionState>>>,
        instances: Arc<RwLock<HashMap<Uuid, InstanceInfo>>>,
        rate_limits: Arc<Mutex<HashMap<Uuid, RateLimit>>>,
        max_instances: usize,
        rate_config: RateLimitConfig,
        broadcast_tx: mpsc::UnboundedSender<BroadcastMessage>,
    ) -> Result<()> {
        // Read handshake message
        let handshake = Self::read_message(&mut socket).await?;
        
        let (instance_id, instance_info) = match handshake {
            RelayMessage::RegisterInstance {
                instance_name,
                version,
                session_token: _,
            } => {
                Self::handle_registration(
                    &mut socket,
                    instance_name,
                    version,
                    &instances,
                    max_instances,
                )
                .await?
            }
            _ => {
                Self::send_message(&mut socket, &RelayMessage::Error {
                    message: "Expected RegisterInstance message".to_string()
                }).await?;
                return Err(anyhow::anyhow!("Invalid handshake"));
            }
        };

        // Now split the socket for concurrent read/write
        let (reader, writer) = socket.into_split();

        info!(
            "Instance '{}' registered with ID {}",
            instance_info.name, instance_id
        );

        // Add rate limiter
        {
            let mut limits = rate_limits.lock().await;
            limits.insert(instance_id, RateLimit::new(instance_id));
        }

        // Create reader task
        let reader_connections = connections.clone();
        let reader_instances = instances.clone();
        let reader_broadcast_tx = broadcast_tx.clone();
        let reader_instance_id = instance_id;
        let reader_instance_name = instance_info.name.clone();
        let reader_rate_limits = rate_limits.clone();
        let reader_rate_config = rate_config.clone();
        
        let reader_handle = tokio::spawn(async move {
            if let Err(e) = Self::reader_loop(
                reader,
                reader_instance_id,
                reader_instance_name.clone(),
                reader_rate_limits.clone(),
                reader_rate_config,
                reader_broadcast_tx.clone(),
            )
            .await
            {
                debug!("Reader loop ended for {}: {}", reader_instance_name, e);
            }
            
            // Notify disconnection
            let _ = reader_broadcast_tx.send(BroadcastMessage::InstanceDisconnected {
                instance_id: reader_instance_id,
                instance_name: reader_instance_name.clone(),
            });
            
            // Cleanup
            Self::cleanup_instance(
                reader_instance_id,
                &reader_connections,
                &reader_instances,
                &reader_rate_limits,
            )
            .await;
        });

        // Store connection state
        let connection_state = ConnectionState {
            instance_info: instance_info.clone(),
            writer: Arc::new(Mutex::new(writer)),
            _reader_handle: reader_handle,
        };

        {
            let mut conns = connections.write().await;
            conns.insert(instance_id, connection_state);
        }

        // TODO: Notify other instances about new connection
        // This would require a new message type like InstanceConnected

        Ok(())
    }

    /// Handle instance registration
    async fn handle_registration(
        socket: &mut TcpStream,
        instance_name: String,
        version: String,
        instances: &Arc<RwLock<HashMap<Uuid, InstanceInfo>>>,
        max_instances: usize,
    ) -> Result<(Uuid, InstanceInfo)> {
        // Check version compatibility
        if version != BUBOCORE_VERSION {
            let error = RelayError::VersionMismatch {
                expected: BUBOCORE_VERSION.to_string(),
                actual: version,
            };
            Self::send_message(socket, &RelayMessage::Error {
                message: error.to_string()
            }).await?;
            return Err(error.into());
        }

        let instances_read = instances.read().await;

        // Check if we're at capacity
        if instances_read.len() >= max_instances {
            let error = RelayError::MaxInstancesReached { max: max_instances };
            Self::send_message(socket, &RelayMessage::Error {
                message: error.to_string()
            }).await?;
            return Err(error.into());
        }

        // Check if name is already taken
        if instances_read.values().any(|i| i.name == instance_name) {
            let error = RelayError::InstanceNameTaken {
                name: instance_name,
            };
            Self::send_message(socket, &RelayMessage::Error {
                message: error.to_string()
            }).await?;
            return Err(error.into());
        }

        let current_instances: Vec<InstanceInfo> = instances_read.values().cloned().collect();
        drop(instances_read);

        // Create new instance
        let instance_id = Uuid::new_v4();
        let now = SystemTime::now();
        let instance_info = InstanceInfo {
            id: instance_id,
            name: instance_name,
            version,
            connected_at: now,
            last_activity: now,
        };

        // Add to instances
        {
            let mut instances_write = instances.write().await;
            instances_write.insert(instance_id, instance_info.clone());
        }

        // Send registration response
        let response = RelayMessage::RegistrationResponse {
            success: true,
            message: format!("Registered as {}", instance_info.name),
            assigned_id: Some(instance_id),
            current_instances,
        };

        Self::send_message(socket, &response).await?;

        Ok((instance_id, instance_info))
    }

    /// Reader loop for handling incoming messages
    async fn reader_loop(
        mut reader: OwnedReadHalf,
        instance_id: Uuid,
        instance_name: String,
        rate_limits: Arc<Mutex<HashMap<Uuid, RateLimit>>>,
        rate_config: RateLimitConfig,
        broadcast_tx: mpsc::UnboundedSender<BroadcastMessage>,
    ) -> Result<()> {
        loop {
            let message = match Self::read_message_from_reader(&mut reader).await {
                Ok(msg) => msg,
                Err(e) => {
                    debug!("Failed to read message from {}: {}", instance_name, e);
                    break;
                }
            };

            // Check rate limit
            {
                let mut limits = rate_limits.lock().await;
                if let Some(limit) = limits.get_mut(&instance_id) {
                    let message_size = message.serialized_size().unwrap_or(0);
                    if !limit.check_and_update(&rate_config, message_size) {
                        warn!("Rate limit exceeded for instance {}", instance_name);
                        // Can't send error back without writer, just log
                        continue;
                    }
                }
            }

            match message {
                RelayMessage::StateUpdate {
                    source_instance_id,
                    timestamp,
                    update_data,
                } => {
                    if source_instance_id != instance_id {
                        warn!("Instance ID mismatch in StateUpdate");
                        continue;
                    }

                    // Broadcast to other instances
                    let _ = broadcast_tx.send(BroadcastMessage::StateUpdate {
                        source_id: instance_id,
                        source_name: instance_name.clone(),
                        timestamp,
                        data: update_data,
                    });
                }
                RelayMessage::Ping { timestamp } => {
                    // Need to handle ping separately since we don't have writer access here
                    debug!("Received ping from {} with timestamp {}", instance_name, timestamp);
                }
                _ => {
                    warn!("Unexpected message type from {}", instance_name);
                }
            }
        }

        Ok(())
    }

    /// Distribute broadcast message to all connected instances
    async fn distribute_broadcast(
        msg: BroadcastMessage,
        connections: &Arc<RwLock<HashMap<Uuid, ConnectionState>>>,
    ) {
        let (relay_msg, exclude_id) = match msg {
            BroadcastMessage::StateUpdate {
                source_id,
                source_name,
                timestamp,
                data,
            } => {
                let msg = RelayMessage::StateBroadcast {
                    source_instance_name: source_name,
                    timestamp,
                    update_data: data,
                };
                (msg, Some(source_id))
            }
            BroadcastMessage::InstanceDisconnected {
                instance_id,
                instance_name,
            } => {
                let msg = RelayMessage::InstanceDisconnected {
                    instance_id,
                    instance_name,
                };
                (msg, None)
            }
        };

        let conns = connections.read().await;
        let failed_instances = Vec::new();
        
        for (&id, conn) in conns.iter() {
            // Don't send to the source instance
            if Some(id) == exclude_id {
                continue;
            }
            
            let writer = conn.writer.clone();
            let msg_clone = relay_msg.clone();
            
            // Send asynchronously
            tokio::spawn(async move {
                let mut writer_guard = writer.lock().await;
                if let Err(e) = Self::send_message_to_writer(&mut *writer_guard, &msg_clone).await {
                    warn!("Failed to send message to instance {}: {}", id, e);
                    return Some(id);
                }
                None
            });
        }
        
        drop(conns);

        // Clean up failed connections
        if !failed_instances.is_empty() {
            let mut conns = connections.write().await;
            for id in failed_instances {
                conns.remove(&id);
            }
        }
    }

    /// Clean up instance on disconnection
    async fn cleanup_instance(
        instance_id: Uuid,
        connections: &Arc<RwLock<HashMap<Uuid, ConnectionState>>>,
        instances: &Arc<RwLock<HashMap<Uuid, InstanceInfo>>>,
        rate_limits: &Arc<Mutex<HashMap<Uuid, RateLimit>>>,
    ) {
        // Remove from connections
        {
            let mut conns = connections.write().await;
            if let Some(conn) = conns.remove(&instance_id) {
                info!("Removed connection for instance '{}'", conn.instance_info.name);
            }
        }

        // Remove from instances
        {
            let mut insts = instances.write().await;
            insts.remove(&instance_id);
        }

        // Remove rate limiter
        {
            let mut limits = rate_limits.lock().await;
            limits.remove(&instance_id);
        }
    }

    /// Read a message from the socket
    async fn read_message(socket: &mut TcpStream) -> Result<RelayMessage> {
        // Read message length (4 bytes, big-endian)
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf).await?;
        let message_len = u32::from_be_bytes(len_buf) as usize;

        // Read message data
        let mut message_buf = vec![0u8; message_len];
        socket.read_exact(&mut message_buf).await?;

        // Deserialize
        RelayMessage::from_bytes(&message_buf).map_err(|e| e.into())
    }

    /// Read a message from an OwnedReadHalf
    async fn read_message_from_reader(reader: &mut OwnedReadHalf) -> Result<RelayMessage> {
        // Read message length (4 bytes, big-endian)
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let message_len = u32::from_be_bytes(len_buf) as usize;

        // Read message data
        let mut message_buf = vec![0u8; message_len];
        reader.read_exact(&mut message_buf).await?;

        // Deserialize
        RelayMessage::from_bytes(&message_buf).map_err(|e| e.into())
    }

    /// Send a message to the socket
    async fn send_message(socket: &mut TcpStream, message: &RelayMessage) -> Result<()> {
        let message_bytes = message.to_bytes()?;
        let len = message_bytes.len() as u32;

        // Send length prefix and message
        socket.write_all(&len.to_be_bytes()).await?;
        socket.write_all(&message_bytes).await?;
        socket.flush().await?;

        Ok(())
    }

    /// Send a message to an OwnedWriteHalf
    async fn send_message_to_writer(writer: &mut OwnedWriteHalf, message: &RelayMessage) -> Result<()> {
        let message_bytes = message.to_bytes()?;
        let len = message_bytes.len() as u32;

        // Send length prefix and message
        writer.write_all(&len.to_be_bytes()).await?;
        writer.write_all(&message_bytes).await?;
        writer.flush().await?;

        Ok(())
    }

    /// Clean up stale rate limiters
    async fn cleanup_rate_limits(rate_limits: &Arc<Mutex<HashMap<Uuid, RateLimit>>>) {
        let mut limits = rate_limits.lock().await;
        let now = SystemTime::now();

        limits.retain(|_, limit| {
            // Keep rate limiters that have been active in the last 5 minutes
            now.duration_since(limit.window_start)
                .unwrap_or_default()
                .as_secs()
                < 300
        });

        if !limits.is_empty() {
            debug!("Cleaned up rate limiters, {} remaining", limits.len());
        }
    }

    /// Handle HTTP requests for the web interface
    async fn handle_http_request(
        req: Request<hyper::body::Incoming>,
        instances: Arc<RwLock<HashMap<Uuid, InstanceInfo>>>,
    ) -> Result<Response<Full<bytes::Bytes>>, std::convert::Infallible> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                let instances_read = instances.read().await;
                let instances_list: Vec<&InstanceInfo> = instances_read.values().collect();
                let instance_count = instances_list.len();
                
                let html = format!(
                    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BuboCore Relay Server</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: #333;
        }}
        .container {{
            background: white;
            padding: 2rem;
            border-radius: 10px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
        }}
        h1 {{
            color: #4a5568;
            text-align: center;
            margin-bottom: 1rem;
        }}
        .status {{
            background: #f7fafc;
            padding: 1rem;
            border-radius: 5px;
            margin: 1rem 0;
            border-left: 4px solid #4299e1;
        }}
        .instances {{
            margin-top: 2rem;
        }}
        .instance {{
            background: #edf2f7;
            padding: 1rem;
            margin: 0.5rem 0;
            border-radius: 5px;
            border-left: 4px solid #48bb78;
        }}
        .empty {{
            text-align: center;
            color: #718096;
            font-style: italic;
            margin: 2rem 0;
        }}
        .version {{
            text-align: center;
            color: #718096;
            font-size: 0.9rem;
            margin-top: 2rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔗 BuboCore Relay Server</h1>
        
        <div class="status">
            <strong>Status:</strong> Online<br>
            <strong>Connected Instances:</strong> {}<br>
            <strong>Version:</strong> {}
        </div>
        
        <div class="instances">
            <h2>Connected Instances</h2>
            {}"#,
                    instance_count,
                    BUBOCORE_VERSION,
                    if instance_count == 0 {
                        "<div class=\"empty\">No instances currently connected</div>".to_string()
                    } else {
                        instances_list
                            .iter()
                            .map(|instance| {
                                let connected_duration = instance.connected_at
                                    .elapsed()
                                    .unwrap_or_default();
                                format!(
                                    "<div class=\"instance\">
                                        <strong>{}</strong><br>
                                        <small>ID: {}</small><br>
                                        <small>Connected: {}s ago</small>
                                    </div>",
                                    instance.name,
                                    instance.id,
                                    connected_duration.as_secs()
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    }
                );
                
                let html = format!("{}{}", html, 
                    r#"
        </div>
        
        <div class="version">
            This is a BuboCore Relay Server for remote collaboration.
        </div>
    </div>
</body>
</html>"#);

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/html")
                    .body(Full::new(bytes::Bytes::from(html)))
                    .unwrap())
            }
            (&Method::GET, "/status") => {
                let instances_read = instances.read().await;
                let status = serde_json::json!({
                    "status": "online",
                    "version": BUBOCORE_VERSION,
                    "connected_instances": instances_read.len(),
                    "instances": instances_read.values().map(|i| {
                        serde_json::json!({
                            "id": i.id,
                            "name": i.name,
                            "version": i.version,
                            "connected_at": i.connected_at.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
                        })
                    }).collect::<Vec<_>>()
                });

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Full::new(bytes::Bytes::from(status.to_string())))
                    .unwrap())
            }
            _ => {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(bytes::Bytes::from("Not Found")))
                    .unwrap())
            }
        }
    }
}

/// Helper function to get current timestamp
#[allow(dead_code)]
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
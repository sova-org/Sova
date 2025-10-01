//! Network manager for client-server communication.
//!
//! This module handles all network communication between the client and the server
//! using asynchronous channels for bidirectional communication.

use crate::event::Event;
use corelib::server::{
    ServerMessage,
    client::{SovaClient, ClientMessage},
};
use std::io;
use tokio::sync::mpsc;

/// Main structure for managing network communication.
///
/// This structure holds the communication channels and connection information
/// necessary for interacting with the server.
pub struct NetworkManager {
    /// Channel for sending commands to the network task.
    client_sender: mpsc::UnboundedSender<NetworkCommand>,
    /// Server IP address.
    ip: String,
    /// Server port.
    port: u16,
    /// Username for connection.
    username: String,
}

/// Possible commands for the network manager task.
///
/// This enum defines all the commands that can be sent to the network task
/// to control the communication flow.
#[derive(Debug)]
pub enum NetworkCommand {
    /// Send a message to the server.
    SendMessage(ClientMessage),
    /// Update connection information (IP, port, username).
    UpdateConnection(String, u16, String),
}

impl NetworkManager {
    /// Creates a new network manager with the given connection parameters.
    ///
    /// Spawns a background task (`run_network_task`) to handle the actual
    /// network I/O and command processing.
    ///
    /// # Arguments
    ///
    /// * `ip` - The server's IP address.
    /// * `port` - The server's port.
    /// * `username` - The username for the connection.
    /// * `sender` - The channel used to send events (like received messages)
    ///    main application or UI.
    ///
    /// # Returns
    ///
    /// A new instance of `NetworkManager`.
    pub fn new(
        ip: String,
        port: u16,
        username: String,
        sender: mpsc::UnboundedSender<Event>,
    ) -> Self {
        // Create communication channels for the network task
        let (client_tx, client_rx) = mpsc::unbounded_channel::<NetworkCommand>();
        // The receiving end of server_tx is currently unused in run_network_task
        let (server_tx, _) = mpsc::unbounded_channel::<ServerMessage>();

        // Spawn the network task in the background
        tokio::spawn(run_network_task(
            ip.clone(),
            port,
            username.clone(),
            client_rx,
            server_tx,
            sender,
        ));

        NetworkManager {
            client_sender: client_tx,
            ip,
            port,
            username,
        }
    }

    /// Retrieves the current connection information.
    ///
    /// # Returns
    ///
    /// A tuple containing the current IP address and port.
    pub fn get_connection_info(&self) -> (String, u16) {
        (self.ip.clone(), self.port)
    }

    /// Updates the connection information and signals the network task to reconnect.
    ///
    /// Sends an `UpdateConnection` command to the background network task.
    ///
    /// # Arguments
    ///
    /// * `ip` - The new IP address.
    /// * `port` - The new port.
    /// * `username` - The new username.
    ///
    /// # Returns
    ///
    /// An `io::Result` indicating whether the command was successfully sent
    /// to the network task. Returns an error if the channel is closed.
    pub fn update_connection_info(
        &mut self,
        ip: String,
        port: u16,
        username: String,
    ) -> io::Result<()> {
        self.ip = ip.clone();
        self.port = port;
        self.username = username.clone();

        self.client_sender
            .send(NetworkCommand::UpdateConnection(ip, port, username))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }

    /// Sends a message to the server via the network task.
    ///
    /// Sends a `SendMessage` command to the background network task.
    ///
    /// # Arguments
    ///
    /// * `message` - The `ClientMessage` to send.
    ///
    /// # Returns
    ///
    /// An `io::Result` indicating whether the command was successfully sent
    /// to the network task. Returns an error if the channel is closed.
    pub fn send(&self, message: ClientMessage) -> io::Result<()> {
        self.client_sender
            .send(NetworkCommand::SendMessage(message))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }
}

/// Main background task that handles network communication.
///
/// This function manages the primary loop for communicating with the server,
/// processing received commands, and reading messages from the server.
/// It attempts to maintain a connection and handles reconnection logic.
///
/// # Arguments
///
/// * `ip` - The initial server IP address.
/// * `port` - The initial server port.
/// * `initial_username` - The initial username for the connection.
/// * `command_rx` - The channel for receiving `NetworkCommand`s from the `NetworkManager`.
/// * `_server_tx` - The channel for sending received `ServerMessage`s (currently unused receiver).
/// * `sender` - The channel for sending `Event`s (like `Event::Network`) back to the main application/UI.
async fn run_network_task(
    ip: String,
    port: u16,
    initial_username: String,
    mut command_rx: mpsc::UnboundedReceiver<NetworkCommand>,
    _server_tx: mpsc::UnboundedSender<ServerMessage>,
    sender: mpsc::UnboundedSender<Event>,
) {
    let mut current_ip = ip;
    let mut current_port = port;
    let mut current_username = initial_username;
    let mut client = SovaClient::new(current_ip.clone(), current_port);
    let mut should_run = true;

    // Main loop for command processing and message handling
    while should_run {
        tokio::select! {
            // Handle received commands
            maybe_cmd = command_rx.recv() => {
                match maybe_cmd {
                    Some(cmd) => {
                        match cmd {
                            NetworkCommand::SendMessage(msg) => {
                                if client.connected {
                                    if let Err(_e) = client.send(msg).await {
                                        client.connected = false;
                                    }
                                }
                            },
                            NetworkCommand::UpdateConnection(new_ip, new_port, new_username) => {
                                current_ip = new_ip;
                                current_port = new_port;
                                current_username = new_username;
                                client = SovaClient::new(current_ip.clone(), current_port);
                                client.connected = false;
                            },
                        }
                    },
                    None => {
                        should_run = false;
                    }
                }
            },
            // Read messages from the server, only poll if connected.
            result = client.read(), if client.connected => {
                match result {
                    Ok(msg) => {
                        if sender.send(Event::Network(msg)).is_err() {
                            should_run = false;
                        }
                    },
                    Err(_e) => {
                        client.connected = false;
                    }
                }
            },
            else => {
                should_run = false;
            }
        }

        // Attempt connection/reconnection if needed
        if should_run && !client.connected {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if client.connect().await.is_ok() {
                if let Err(_e) = client
                    .send(ClientMessage::SetName(current_username.clone()))
                    .await
                {
                    client.connected = false;
                }
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

use crate::event::Event;
use bubocorelib::server::{
    ServerMessage,
    client::{BuboCoreClient, ClientMessage},
};
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct NetworkManager {
    client_sender: mpsc::UnboundedSender<NetworkCommand>,
    server_receiver: mpsc::UnboundedReceiver<ServerMessage>,
    ip: String,
    port: u16,
    username: String,
}

pub enum NetworkCommand {
    SendMessage(ClientMessage),
    Reconnect,
    UpdateConnection(String, u16, String),
    Shutdown,
}

impl NetworkManager {
    pub fn new(ip: String, port: u16, username: String, sender: mpsc::UnboundedSender<Event>) -> Self {
        let (client_tx, client_rx) = mpsc::unbounded_channel::<NetworkCommand>();
        let (server_tx, server_rx) = mpsc::unbounded_channel::<ServerMessage>();

        // Spawn a background task to manage the actual client
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
            server_receiver: server_rx,
            ip,
            port,
            username,
        }
    }

    pub fn get_connection_info(&self) -> (String, u16) {
        (self.ip.clone(), self.port)
    }

    pub fn update_connection_info(&mut self, ip: String, port: u16, username: String) -> io::Result<()> {
        self.ip = ip.clone();
        self.port = port;
        self.username = username.clone();

        // Tell the network task to reconnect with new parameters
        self.client_sender
            .send(NetworkCommand::UpdateConnection(ip, port, username))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }

    pub fn send(&self, message: ClientMessage) -> io::Result<()> {
        self.client_sender
            .send(NetworkCommand::SendMessage(message))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }

    pub fn try_receive(&mut self) -> Option<ServerMessage> {
        self.server_receiver.try_recv().ok()
    }

    pub fn reconnect(&self) -> io::Result<()> {
        self.client_sender
            .send(NetworkCommand::Reconnect)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }

    pub fn shutdown(&self) -> io::Result<()> {
        self.client_sender
            .send(NetworkCommand::Shutdown)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }

    pub async fn receive(&mut self) -> Option<ServerMessage> {
        self.server_receiver.recv().await
    }
}

async fn run_network_task(
    ip: String,
    port: u16,
    initial_username: String,
    mut command_rx: mpsc::UnboundedReceiver<NetworkCommand>,
    server_tx: mpsc::UnboundedSender<ServerMessage>,
    sender: mpsc::UnboundedSender<Event>,
) {
    let mut current_username = initial_username.clone();
    let mut client = BuboCoreClient::new(ip.clone(), port);
    let mut should_run = true;

    while should_run {
        tokio::select! {
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    NetworkCommand::SendMessage(msg) => {
                        if !client.connected {
                            // Try to reconnect if disconnected
                            if client.connect().await.is_ok() {
                                // Send SetName again after reconnecting
                                let _ = client.send(ClientMessage::SetName(current_username.clone())).await;
                            }
                        }

                        if client.connected {
                            let _ = client.send(msg).await;
                        }
                    },
                    NetworkCommand::UpdateConnection(new_ip, new_port, new_username) => {
                        current_username = new_username;
                        client = BuboCoreClient::new(new_ip.clone(), new_port);
                        if client.connect().await.is_ok() {
                            // Send SetName after successful connection with new details
                            let _ = client.send(ClientMessage::SetName(current_username.clone())).await;
                        }
                    },
                    NetworkCommand::Reconnect => {
                        // Reconnect uses the existing client details
                        if client.connect().await.is_ok() {
                            // Send SetName again after reconnecting
                            let _ = client.send(ClientMessage::SetName(current_username.clone())).await;
                        }
                    },
                    NetworkCommand::Shutdown => {
                        should_run = false;
                    }
                }
            },
            _ = async {
                if client.connected && client.ready().await {
                    if let Ok(msg) = client.read().await {
                        let _ = sender.send(Event::Network(msg));
                    }
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            } => {}
        }
    }
}

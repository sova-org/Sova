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
}

pub enum NetworkCommand {
    SendMessage(ClientMessage),
    Reconnect,
    UpdateConnection(String, u16),
    Shutdown,
}

impl NetworkManager {
    pub fn new(ip: String, port: u16) -> Self {
        let (client_tx, client_rx) = mpsc::unbounded_channel::<NetworkCommand>();
        let (server_tx, server_rx) = mpsc::unbounded_channel::<ServerMessage>();

        // Spawn a background task to manage the actual client
        tokio::spawn(run_network_task(ip.clone(), port, client_rx, server_tx));

        NetworkManager {
            client_sender: client_tx,
            server_receiver: server_rx,
            ip,
            port,
        }
    }

    pub fn get_connection_info(&self) -> (String, u16) {
        (self.ip.clone(), self.port)
    }

    pub fn update_connection_info(&mut self, ip: String, port: u16) -> io::Result<()> {
        self.ip = ip.clone();
        self.port = port;

        // Tell the network task to reconnect with new parameters
        self.client_sender
            .send(NetworkCommand::UpdateConnection(ip, port))
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
    mut command_rx: mpsc::UnboundedReceiver<NetworkCommand>,
    server_tx: mpsc::UnboundedSender<ServerMessage>,
) {
    let mut client = BuboCoreClient::new(ip, port);
    let mut should_run = true;

    // Try initial connection
    let _ = client.connect().await;

    while should_run {
        tokio::select! {
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    NetworkCommand::SendMessage(msg) => {
                        if !client.connected {
                            // Try to reconnect if disconnected
                            let _ = client.connect().await;
                        }

                        if client.connected {
                            let _ = client.send(msg).await;
                        }
                    },
                    NetworkCommand::UpdateConnection(new_ip, new_port) => {
                        let ip = new_ip.clone();
                        let port = new_port;
                        client = BuboCoreClient::new(ip.clone(), port);
                        let _ = client.connect().await;
                    },
                    NetworkCommand::Reconnect => {
                        let _ = client.connect().await;
                    },
                    NetworkCommand::Shutdown => {
                        should_run = false;
                    }
                }
            },
            _ = async {
                if client.connected && client.ready().await {
                    if let Ok(msg) = client.read().await {
                        let _ = server_tx.send(msg);
                    }
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            } => {}
        }
    }
}

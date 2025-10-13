use anyhow::Result;
use sova_core::server::client::{ClientMessage, SovaClient};
use sova_core::server::ServerMessage;
use tokio::sync::mpsc;

pub struct ClientManager {
    client: Option<SovaClient>,
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
        let mut client = SovaClient::new(ip, port);
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
        mut client: SovaClient,
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
                            Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Client cannot start"))
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
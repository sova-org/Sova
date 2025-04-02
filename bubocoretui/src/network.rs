//! Gestionnaire de réseau pour la communication client-serveur.
//! 
//! Ce module gère toute la communication réseau entre le client et le serveur,
//! en utilisant des canaux asynchrones pour la communication bidirectionnelle.

use crate::event::Event;
use bubocorelib::server::{
    ServerMessage,
    client::{BuboCoreClient, ClientMessage},
};
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

/// Structure principale de gestion de la communication réseau.
/// 
/// Cette structure maintient les canaux de communication et les
/// informations de connexion nécessaires pour la communication avec le serveur.
pub struct NetworkManager {
    /// Canal pour l'envoi des commandes au client
    client_sender: mpsc::UnboundedSender<NetworkCommand>,
    /// Canal pour la réception des messages serveur
    server_receiver: mpsc::UnboundedReceiver<ServerMessage>,
    /// Adresse IP du serveur
    ip: String,
    /// Port du serveur
    port: u16,
    /// Nom d'utilisateur
    username: String,
}

/// Commandes possibles pour le gestionnaire réseau.
/// 
/// Cette énumération définit toutes les commandes qui peuvent être envoyées
/// au gestionnaire réseau pour contrôler la communication.
#[derive(Debug)]
pub enum NetworkCommand {
    /// Envoyer un message au serveur
    SendMessage(ClientMessage),
    /// Reconnecter au serveur
    Reconnect,
    /// Mettre à jour les informations de connexion
    UpdateConnection(String, u16, String),
    /// Arrêter le gestionnaire réseau
    Shutdown,
}

impl NetworkManager {
    /// Crée un nouveau gestionnaire réseau avec les paramètres de connexion.
    /// 
    /// # Arguments
    /// 
    /// * `ip` - L'adresse IP du serveur
    /// * `port` - Le port du serveur
    /// * `username` - Le nom d'utilisateur
    /// * `sender` - Le canal pour l'envoi des événements à l'UI
    /// 
    /// # Returns
    /// 
    /// Une nouvelle instance de `NetworkManager`
    pub fn new(ip: String, port: u16, username: String, sender: mpsc::UnboundedSender<Event>) -> Self {
        // Création des canaux de communication
        let (client_tx, client_rx) = mpsc::unbounded_channel::<NetworkCommand>();
        let (server_tx, server_rx) = mpsc::unbounded_channel::<ServerMessage>();

        // Lancement de la tâche réseau en arrière-plan
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

    /// Récupère les informations de connexion actuelles.
    /// 
    /// # Returns
    /// 
    /// Un tuple contenant l'IP et le port actuels
    pub fn get_connection_info(&self) -> (String, u16) {
        (self.ip.clone(), self.port)
    }

    /// Met à jour les informations de connexion et force une reconnexion.
    /// 
    /// # Arguments
    /// 
    /// * `ip` - La nouvelle adresse IP
    /// * `port` - Le nouveau port
    /// * `username` - Le nouveau nom d'utilisateur
    /// 
    /// # Returns
    /// 
    /// Un `Result` indiquant si la mise à jour a réussi
    pub fn update_connection_info(&mut self, ip: String, port: u16, username: String) -> io::Result<()> {
        self.ip = ip.clone();
        self.port = port;
        self.username = username.clone();

        self.client_sender
            .send(NetworkCommand::UpdateConnection(ip, port, username))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }

    /// Envoie un message au serveur.
    /// 
    /// # Arguments
    /// 
    /// * `message` - Le message à envoyer
    /// 
    /// # Returns
    /// 
    /// Un `Result` indiquant si l'envoi a réussi
    pub fn send(&self, message: ClientMessage) -> io::Result<()> {
        self.client_sender
            .send(NetworkCommand::SendMessage(message))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }

    /// Tente de recevoir un message du serveur de manière non-bloquante.
    /// 
    /// # Returns
    /// 
    /// Un `Option` contenant le message reçu, s'il y en a un
    pub fn try_receive(&mut self) -> Option<ServerMessage> {
        self.server_receiver.try_recv().ok()
    }

    /// Force une reconnexion au serveur.
    /// 
    /// # Returns
    /// 
    /// Un `Result` indiquant si la commande de reconnexion a été envoyée avec succès
    pub fn reconnect(&self) -> io::Result<()> {
        self.client_sender
            .send(NetworkCommand::Reconnect)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }

    /// Arrête le gestionnaire réseau.
    /// 
    /// # Returns
    /// 
    /// Un `Result` indiquant si la commande d'arrêt a été envoyée avec succès
    pub fn shutdown(&self) -> io::Result<()> {
        self.client_sender
            .send(NetworkCommand::Shutdown)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Channel closed"))
    }

    /// Attend et reçoit un message du serveur de manière asynchrone.
    /// 
    /// # Returns
    /// 
    /// Un `Option` contenant le message reçu, s'il y en a un
    pub async fn receive(&mut self) -> Option<ServerMessage> {
        self.server_receiver.recv().await
    }
}

/// Fonction principale qui gère la communication réseau en arrière-plan.
/// 
/// Cette fonction gère la boucle principale de communication avec le serveur,
/// en traitant les commandes reçues et en lisant les messages du serveur.
/// 
/// # Arguments
/// 
/// * `ip` - L'adresse IP du serveur
/// * `port` - Le port du serveur
/// * `initial_username` - Le nom d'utilisateur initial
/// * `command_rx` - Le canal pour recevoir les commandes
/// * `server_tx` - Le canal pour envoyer les messages du serveur
/// * `sender` - Le canal pour envoyer des événements à l'interface utilisateur
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

    // Boucle principale de gestion des commandes et des messages
    while should_run {
        tokio::select! {
            // Gestion des commandes reçues
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    NetworkCommand::SendMessage(msg) => {
                        // Si le client est déconnecté, tente une reconnexion
                        if !client.connected {
                            if client.connect().await.is_ok() {
                                // Réenvoie le nom d'utilisateur après reconnexion
                                let _ = client.send(ClientMessage::SetName(current_username.clone())).await;
                            }
                        }

                        // Envoie le message si connecté
                        if client.connected {
                            let _ = client.send(msg).await;
                        }
                    },
                    NetworkCommand::UpdateConnection(new_ip, new_port, new_username) => {
                        // Met à jour les informations de connexion
                        current_username = new_username;
                        client = BuboCoreClient::new(new_ip.clone(), new_port);
                        if client.connect().await.is_ok() {
                            // Envoie le nom d'utilisateur après la nouvelle connexion
                            let _ = client.send(ClientMessage::SetName(current_username.clone())).await;
                        }
                    },
                    NetworkCommand::Reconnect => {
                        // Tente une reconnexion avec les paramètres actuels
                        if client.connect().await.is_ok() {
                            let _ = client.send(ClientMessage::SetName(current_username.clone())).await;
                        }
                    },
                    NetworkCommand::Shutdown => {
                        should_run = false;
                    }
                }
            },
            // Lecture des messages du serveur
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

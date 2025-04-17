//!
//! Gestion des événements pour l'application TUI.
//! 
//! Ce module définit les différents types d'événements utilisés dans l'application
//! (Tick, Crossterm, Application, Réseau) et fournit un `EventHandler` pour
//! gérer leur production et consommation de manière asynchrone.
//!

use bubocorelib::server::ServerMessage;
use color_eyre::eyre::OptionExt;
use crossterm::event::{Event as CrosstermEvent, EventStream};
use futures::{FutureExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;
use chrono::{DateTime, Utc};
use bubocorelib::server::Snapshot;
use bubocorelib::schedule::ActionTiming;

/// Fréquence des événements de type `Tick` par seconde.
const TICK_FPS: f64 = 60.0;

/// Énumération représentant tous les types d'événements gérés par l'application.
#[derive(Clone, Debug)]
pub enum Event {
    /// Un événement émis à intervalle régulier (tick).
    ///
    /// Utile pour exécuter du code périodiquement, indépendamment des actions utilisateur,
    /// comme la mise à jour de l'horloge ou l'état de la synchronisation.
    Tick,
    /// Événements provenant de la bibliothèque `crossterm` (entrées utilisateur, redimensionnement).
    Crossterm(CrosstermEvent),
    /// Événements spécifiques à la logique de l'application.
    App(AppEvent),
    /// Événements reçus du réseau (messages du serveur).
    Network(ServerMessage),
}

/// Événements spécifiques à la logique de l'application.
#[derive(Clone, Debug)]
pub enum AppEvent {
    // --- Navigation --- 
    /// Passer à la vue Éditeur.
    SwitchToEditor,
    /// Passer à la vue Grille.
    SwitchToGrid,
    /// Passer à la vue Options.
    SwitchToOptions,
    /// Passer à la vue Aide.
    SwitchToHelp,
    /// Passer à la vue Devices.
    SwitchToDevices,
    /// Passer à la vue Logs.
    SwitchToLogs,
    /// Passer à la vue Save/Load.
    SwitchToSaveLoad,
    /// Move the navigation cursor by (dy, dx).
    MoveNavigationCursor((i32, i32)),
    /// Exit navigation mode
    ExitNavigation,

    // --- Mode Commande --- 
    // ExecuteCommand(String),

    // --- Synchronisation (Link) --- 
    /// Mettre à jour le tempo.
    UpdateTempo(f64),
    /// Mettre à jour le quantum.
    UpdateQuantum(f64),

    // --- Gestion des fichiers --- 
    /// Indique que la liste des projets a été chargée.
    ProjectListLoaded(Result<Vec<(String, Option<DateTime<Utc>>, Option<DateTime<Utc>>)>, String>),
    /// Indique qu'une erreur s'est produite lors du chargement d'un projet.
    ProjectLoadError(String),
    /// Confirmation que le projet a été supprimé.
    ProjectDeleted(String),
    /// Erreur lors de la suppression d'un projet.
    ProjectDeleteError(String),
    /// Reçu après lecture disque réussie, contient les données à envoyer au serveur.
    LoadProject(Snapshot, ActionTiming),
    /// Requête pour charger un projet par nom, avec timing.
    LoadProjectRequest(String, ActionTiming),
    /// Requête pour sauvegarder l'état actuel, avec un nom optionnel.
    SaveProjectRequest(Option<String>),

    // --- Contrôle de l'application --- 
    /// Quitter l'application.
    Quit,
}

/// Gestionnaire d'événements pour le terminal.
///
/// Cette structure encapsule les canaux d'envoi et de réception des événements
/// et lance une tâche asynchrone pour produire les événements `Tick` et `Crossterm`.
#[derive(Debug)]
pub struct EventHandler {
    /// Canal pour envoyer des événements.
    pub sender: mpsc::UnboundedSender<Event>,
    /// Canal pour recevoir des événements.
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Construit une nouvelle instance de [`EventHandler`] et lance une tâche
    /// dédiée à la gestion des événements en arrière-plan.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        // Lance la tâche asynchrone qui va générer les événements Tick et lire les événements Crossterm.
        tokio::spawn(async { actor.run().await });
        Self { sender, receiver }
    }

    /// Reçoit le prochain événement disponible.
    ///
    /// Cette fonction est bloquante jusqu'à ce qu'un événement soit reçu.
    /// 
    /// # Returns
    /// 
    /// Un `Result` contenant l'événement reçu ou une erreur si le canal est fermé.
    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Impossible de recevoir l'événement : canal fermé.")
    }

    /// Met en file d'attente un événement d'application (`AppEvent`) pour qu'il soit traité.
    ///
    /// # Arguments
    /// 
    /// * `app_event` - L'événement d'application à envoyer.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore le résultat car le récepteur ne peut pas être fermé tant que cette structure existe.
        let _ = self.sender.send(Event::App(app_event));
    }
}

/// Tâche asynchrone gérant la lecture des événements `crossterm`
/// et l'émission régulière d'événements `Tick`.
struct EventTask {
    /// Canal pour envoyer les événements générés.
    sender: mpsc::UnboundedSender<Event>,
}

impl EventTask {
    /// Construit une nouvelle instance de [`EventTask`].
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }

    /// Exécute la boucle principale de la tâche d'événements.
    ///
    /// Cette fonction émet des événements `Tick` à une fréquence fixe (`TICK_FPS`)
    /// et interroge les événements `crossterm` entre les ticks.
    async fn run(self) -> color_eyre::Result<()> {
        let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut reader = EventStream::new(); // Lecteur pour les événements crossterm
        let mut tick = tokio::time::interval(tick_rate); // Intervalle pour les Ticks
        
        // Boucle principale : attend soit un tick, soit un événement crossterm,
        // soit la fermeture du canal de l'expéditeur.
        loop {
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse(); // Prépare la lecture du prochain événement crossterm

            tokio::select! {
              // Si le canal est fermé (l'application se termine), arrête la boucle.
              _ = self.sender.closed() => {
                break;
              }
              // Si l'intervalle de tick est écoulé, envoie un événement Tick.
              _ = tick_delay => {
                self.send(Event::Tick);
              }
              // Si un événement crossterm est reçu, envoie un événement Crossterm.
              Some(Ok(evt)) = crossterm_event => {
                self.send(Event::Crossterm(evt));
              }
            };
        }
        Ok(())
    }

    /// Envoie un événement via le canal de l'expéditeur.
    /// 
    /// Ignore les erreurs d'envoi, car elles se produisent normalement lorsque
    /// l'application s'arrête et que le récepteur est fermé.
    fn send(&self, event: Event) {
        let _ = self.sender.send(event);
    }
}

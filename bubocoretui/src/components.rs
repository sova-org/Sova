//! Module définissant les composants réutilisables de l'interface utilisateur (UI).
//! 
//! Ce module contient le trait `Component` que tous les éléments d'UI doivent implémenter,
//! ainsi que des fonctions utilitaires communes pour la gestion des composants.

use crate::app::App;
use color_eyre::Result as EyreResult;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;

// Déclare les sous-modules pour chaque composant spécifique.
pub mod editor;
pub mod grid;
pub mod help;
pub mod markdownparser;
pub mod options;
pub mod splash;
pub mod navigation;
pub mod devices;
pub mod logs;
pub mod saveload;
pub mod command_palette;

/// Trait définissant le comportement attendu de chaque composant de l'UI.
/// 
/// Chaque composant doit pouvoir gérer les événements clavier et se dessiner
/// dans une zone donnée de l'écran.
pub trait Component {
    /// Gère un événement clavier reçu par le composant.
    /// 
    /// # Arguments
    /// 
    /// * `app` - Une référence mutable à l'état global de l'application.
    /// * `key_event` - L'événement clavier à traiter.
    /// 
    /// # Returns
    /// 
    /// Un `Result` contenant :
    /// * `Ok(true)` si l'événement a été géré par ce composant.
    /// * `Ok(false)` si l'événement n'a pas été géré par ce composant (il peut être propagé).
    /// * `Err` si une erreur s'est produite lors du traitement.
    fn handle_key_event(
        &mut self,
        app: &mut App,
        key_event: KeyEvent,
    ) -> EyreResult<bool>;

    /// Dessine le composant dans la zone spécifiée du frame.
    /// 
    /// # Arguments
    /// 
    /// * `app` - Une référence à l'état global de l'application (lecture seule).
    /// * `frame` - Le frame `ratatui` dans lequel dessiner.
    /// * `area` - Le rectangle (`Rect`) délimitant la zone où le composant doit se dessiner.
    fn draw(&self, app: &App, frame: &mut ratatui::Frame, area: Rect);
}
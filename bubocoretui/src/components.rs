//!
//! Module définissant les composants réutilisables de l'interface utilisateur (UI).
//! 
//! Ce module contient le trait `Component` que tous les éléments d'UI doivent implémenter,
//! ainsi que des fonctions utilitaires communes pour la gestion des composants.
//!

use crate::{App, event::AppEvent};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::Rect;
use std::error::Error;

// Déclare les sous-modules pour chaque composant spécifique.
pub mod editor;
pub mod grid;
pub mod help;
pub mod options;
pub mod splash;

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
    ) -> Result<bool, Box<dyn Error>>;

    /// Dessine le composant dans la zone spécifiée du frame.
    /// 
    /// # Arguments
    /// 
    /// * `app` - Une référence à l'état global de l'application (lecture seule).
    /// * `frame` - Le frame `ratatui` dans lequel dessiner.
    /// * `area` - Le rectangle (`Rect`) délimitant la zone où le composant doit se dessiner.
    fn draw(&self, app: &App, frame: &mut ratatui::Frame, area: Rect);
}

/// Gère les raccourcis clavier communs à plusieurs composants.
/// 
/// Cette fonction vérifie si l'événement clavier correspond à un raccourci global
/// (comme Ctrl+C pour quitter, ou les touches F pour changer de vue).
/// 
/// # Arguments
/// 
/// * `app` - Une référence mutable à l'état global de l'application.
/// * `key_event` - L'événement clavier à traiter.
/// 
/// # Returns
/// 
/// Un `Result` contenant :
/// * `Ok(true)` si l'événement correspondait à un raccourci commun et a été géré.
/// * `Ok(false)` si l'événement ne correspondait pas à un raccourci commun.
/// * `Err` si une erreur s'est produite (bien que peu probable ici).
pub fn handle_common_keys(
    app: &mut App,
    key_event: KeyEvent,
) -> Result<bool, Box<dyn Error + 'static>> {
    match key_event.code {
        // Ctrl+C pour quitter l'application
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.events.send(AppEvent::Quit);
            Ok(true)
        }
        // Touches F1, F2, F3 pour naviguer entre les vues
        KeyCode::F(1) => {
            app.events.send(AppEvent::SwitchToEditor);
            Ok(true)
        }
        KeyCode::F(2) => {
            app.events.send(AppEvent::SwitchToGrid);
            Ok(true)
        }
        KeyCode::F(3) => {
            app.events.send(AppEvent::SwitchToOptions);
            Ok(true)
        }
        KeyCode::F(4) => {
            app.events.send(AppEvent::SwitchToHelp);
            Ok(true)
        }
        // Si aucune touche commune n'est détectée
        _ => Ok(false),
    }
}

/// Calcule une zone intérieure en réduisant les bordures d'un `Rect`.
/// 
/// Utile pour dessiner un contenu à l'intérieur d'une bordure ou d'un cadre.
/// 
/// # Arguments
/// 
/// * `area` - Le rectangle extérieur.
/// 
/// # Returns
/// 
/// Un nouveau `Rect` représentant la zone intérieure, réduit de 1 unité sur chaque côté.
pub fn inner_area(area: Rect) -> Rect {
    let inner = area;
    Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),  // Utilise saturating_sub pour éviter les paniques si width < 2
        height: inner.height.saturating_sub(2), // Utilise saturating_sub pour éviter les paniques si height < 2
    }
}

use std::fmt::Display;

pub mod scene_view;
pub mod connection_view;
pub mod connecting_view;
pub mod script_editor;
pub mod prelude_view;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Connection,
    Connecting,
    Scene,
    Prelude,
    Edit,
    Logs,
    Help,
    Chat,
    Network,
    Devices,
    Audio,
    Samples,
}

impl AppPage {
    pub fn is_in_session(&self) -> bool {
        match self {
            AppPage::Connection | AppPage::Connecting => false,
            _ => true
        }
    }
}

impl Display for AppPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppPage::Connection => write!(f, "Connection"),
            AppPage::Connecting => write!(f, "Connecting"),
            AppPage::Scene => write!(f, "Scene"),
            AppPage::Prelude => write!(f, "Prelude"),
            AppPage::Edit => write!(f, "Edit"),
            AppPage::Logs => write!(f, "Logs"),
            AppPage::Chat => write!(f, "Chat"),
            AppPage::Network => write!(f, "Network"),
            AppPage::Devices => write!(f, "Devices"),
            AppPage::Audio => write!(f, "Audio"),
            AppPage::Samples => write!(f, "Samples"),
            AppPage::Help => write!(f, "Help"),
        }
    }
}

impl From<&str> for AppPage {
    fn from(value: &str) -> Self {
        match value {
            "Connection" => AppPage::Connection, 
            "Connecting" => AppPage::Connecting,
            "Scene" => AppPage::Scene,
            "Prelude" => AppPage::Prelude,
            "Edit" => AppPage::Edit,
            "Logs" => AppPage::Logs,
            "Chat" => AppPage::Chat,
            "Network" => AppPage::Network,
            "Devices" => AppPage::Devices,
            "Audio" => AppPage::Audio,
            "Samples" => AppPage::Samples,
            "Help" => AppPage::Help,
            _ => AppPage::Scene
        }
    }
}
pub mod scene_view;
pub mod connection_view;
pub mod connecting_view;
pub mod editor_view;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Connection,
    Connecting,
    Scene,
    Prelude,
    Edit,
    Logs,
    Chat,
    Network,
    Devices,
    Audio,
    Samples,
}
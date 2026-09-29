pub mod scene_view;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Scene,
    Edit,
    Logs,
    Chat,
    Network,
    Devices,
    Audio
}
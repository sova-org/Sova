mod hud;
mod keyboard;
mod lifecycle;
mod panel;
mod prelude;
mod sequencer_grid;
mod sequencer_popup;
mod stack_view;
mod state;

use std::collections::{BTreeSet, HashMap};

use eframe::egui;
use sova_core::scene::Frame;
use sova_core::scene::script::Script;
use sova_core::vm::language::LanguageDefinition;
use crate::client_bridge::ClientBridge;
use crate::widgets::EditorSettings;
use crate::widgets::inline_scene_view::{InlineFrameState, InlineScriptState};
use crate::widgets::syntax_highlight::SyntaxTheme;
pub(crate) use state::{
    ContextTarget, KeyboardContext, Overlay, PendingDestructive, SceneState,
};

pub fn resolve_default_language(preferred: &str, available: &[LanguageDefinition]) -> String {
    if available.is_empty() || available.iter().any(|l| l.name == preferred) {
        preferred.to_string()
    } else {
        available[0].name.clone()
    }
}

pub fn new_frame(lang: &str) -> Frame {
    Frame::from(Script::new(String::new(), lang.to_string()))
}

const DEFAULT_COL_WIDTH: f32 = 450.0;
pub const CELL_HEIGHT: f32 = 180.0;
pub(crate) const MIN_FRAME_HEIGHT: f32 = 60.0;
pub(crate) const MAX_FRAME_HEIGHT: f32 = 600.0;
pub(crate) const DRAG_HANDLE_HEIGHT: f32 = 6.0;
pub(crate) const HEADER_HEIGHT: f32 = 26.0;
pub(crate) const LINE_HEADER_HEIGHT: f32 = 26.0;
pub(crate) const GAP: f32 = 1.0;
pub(crate) const KB_PREVIEW_LIFETIME: std::time::Duration = std::time::Duration::from_secs(2);

pub use crate::widgets::SceneOpacity;

pub(super) struct SceneRenderCtx<'a> {
    pub bridge: &'a ClientBridge,
    pub accent: egui::Color32,
    pub opacity: &'a SceneOpacity,
    pub theme: &'a SyntaxTheme,
    pub editor_settings: &'a EditorSettings,
    pub default_lang: &'a str,
    pub sample_names: &'a [String],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Sequencer,
    Stack,
}

pub struct PanelVisibility {
    pub sidebar: bool,
    pub devices: bool,
    pub scope: bool,
    pub spectrum: bool,
    pub vu_meter: bool,
    pub scope_bar: bool,
    pub logs: bool,
    pub debug: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SequencerInlineField {
    Duration,
    Repetitions,
}

#[derive(Clone, Debug)]
pub(crate) struct SequencerInlineEdit {
    pub target: (usize, usize),
    pub field: SequencerInlineField,
    pub buffer: String,
    pub request_focus: bool,
}

pub(crate) fn parse_inline_duration(input: &str) -> Option<f64> {
    let value = input.trim().parse::<f64>().ok()?;
    value.is_finite().then_some(value.max(0.001))
}

pub(crate) fn parse_inline_repetitions(input: &str) -> Option<usize> {
    let value = input.trim().parse::<usize>().ok()?;
    Some(value.max(1))
}

// ---------------------------------------------------------------------------

pub struct ScenePanel {
    // State machine
    pub(crate) state: SceneState,
    pub(crate) overlay: Overlay,

    // Selection (only valid during NavigatingFrame, cleared by transitions)
    pub(crate) anchor: Option<(usize, usize)>,
    pub(crate) selection: BTreeSet<(usize, usize)>,
    pub(crate) clipboard: Vec<Frame>,
    pub(crate) frame_states: HashMap<(usize, usize), InlineFrameState>,
    pub(crate) column_widths: Vec<f32>,
    pub(crate) prev_editing: Option<(usize, usize)>,
    pub(crate) scroll_to_cursor: bool,
    pub prelude_states: Vec<InlineScriptState>,
    pub prelude_collapsed: bool,
    pub prelude_col_width: f32,
    pub(crate) open_picker_on_cursor: bool,
    pub view_mode: ViewMode,
    pub(crate) pending_mutation_flashes: Vec<(usize, usize)>,
    pub(crate) pending_compilation_flashes: Vec<((usize, usize), bool)>,
    pub(crate) sequencer_inline_edit: Option<SequencerInlineEdit>,
    pub(crate) sequencer_line_speed_focus: Option<usize>,
    pub(crate) confirm_dialog: crate::widgets::ConfirmDialog,
    pub(crate) last_observed_cursor: Option<(usize, usize)>,
    pub(crate) cursor_preview_deadline: Option<std::time::Instant>,
}

impl Default for ScenePanel {
    fn default() -> Self {
        Self {
            state: SceneState::Empty,
            overlay: Overlay::None,
            anchor: None,
            selection: BTreeSet::new(),
            clipboard: Vec::new(),
            frame_states: HashMap::new(),
            column_widths: Vec::new(),
            prev_editing: None,
            scroll_to_cursor: false,
            prelude_states: Vec::new(),
            prelude_collapsed: true,
            prelude_col_width: 300.0,
            open_picker_on_cursor: false,
            view_mode: ViewMode::Sequencer,
            pending_mutation_flashes: Vec::new(),
            pending_compilation_flashes: Vec::new(),
            sequencer_inline_edit: None,
            sequencer_line_speed_focus: None,
            confirm_dialog: crate::widgets::ConfirmDialog::new("scene_confirm"),
            last_observed_cursor: None,
            cursor_preview_deadline: None,
        }
    }
}

impl ScenePanel {
    pub fn new() -> Self {
        Default::default()
    }
}

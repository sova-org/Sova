use eframe::egui;
use sova_core::scene::{Frame, Scene};
use sova_server::ClientMessage;

use crate::client_bridge::ClientBridge;
use crate::widgets::inline_scene_view::FocusRequest;

use super::{ScenePanel, SequencerInlineField, SequencerInlineEdit};

#[derive(Clone, Copy)]
pub(crate) enum ContextTarget {
    Cell(usize, usize),
    Header(usize),
}

/// Primary navigation/editing state of the scene panel.
/// Each variant carries exactly the data needed for that state.
#[derive(Clone, Debug)]
pub(crate) enum SceneState {
    /// No frame or prelude selected. Only arrow keys to place cursor.
    Empty,
    /// Frame selected, sequencer grid visible, navigation keys active.
    NavigatingFrame { cursor: (usize, usize) },
    /// Prelude selected, sequencer grid visible, prelude nav keys active.
    NavigatingPrelude { index: usize },
    /// Frame editor has focus, sequencer grid hidden.
    EditingFrame { cursor: (usize, usize) },
    /// Prelude editor has focus, sequencer grid hidden.
    EditingPrelude { index: usize },
    /// Stack mode only: single frame fills the panel.
    FocusedFrame { frame: (usize, usize) },
}

impl SceneState {
    pub fn is_editing(&self) -> bool {
        matches!(
            self,
            Self::EditingFrame { .. } | Self::EditingPrelude { .. }
        )
    }

    pub fn shows_sequencer_grid(&self) -> bool {
        matches!(
            self,
            Self::Empty | Self::NavigatingFrame { .. } | Self::NavigatingPrelude { .. }
        )
    }

    pub fn cursor(&self) -> Option<(usize, usize)> {
        match self {
            Self::NavigatingFrame { cursor, .. } | Self::EditingFrame { cursor } => Some(*cursor),
            _ => None,
        }
    }

    pub fn selected_prelude(&self) -> Option<usize> {
        match self {
            Self::NavigatingPrelude { index } | Self::EditingPrelude { index } => Some(*index),
            _ => None,
        }
    }

    pub fn focused_frame(&self) -> Option<(usize, usize)> {
        match self {
            Self::FocusedFrame { frame } => Some(*frame),
            _ => None,
        }
    }
}

/// Modal overlay layered on top of any SceneState.
#[derive(Clone)]
pub(crate) enum Overlay {
    None,
    ContextMenu {
        target: ContextTarget,
        pos: egui::Pos2,
        just_opened: bool,
    },
    ConfirmDialog {
        action: PendingDestructive,
    },
}

/// Which part of the scene panel currently owns keyboard input.
/// Derived each frame from `SceneState` + `Overlay` + runtime flags.
/// Only the matching handler may read keys; all others skip.
pub(crate) enum KeyboardContext {
    /// Modal overlay (confirm dialog, context menu) is open.
    Overlay,
    /// Language picker is open in a frame or prelude editor.
    LangPicker,
    /// An egui widget (DragValue, TextEdit) has focus.
    WidgetFocused,
    /// Frame or prelude editor has focus.
    Editing,
    /// Navigation keyboard is active.
    Navigating,
}

/// A destructive action awaiting confirmation.
#[derive(Clone)]
pub(crate) enum PendingDestructive {
    RemoveLine(usize),
    RemoveFrames(Vec<(usize, usize)>),
}

impl SequencerInlineField {
    pub(super) fn prefill_value(self, frame: &Frame) -> String {
        match self {
            Self::Duration => format!("{}", frame.duration),
            Self::Repetitions => frame.repetitions.to_string(),
        }
    }
}

impl ScenePanel {
    pub(super) fn close_inline_lang_pickers(&mut self) {
        self.open_picker_on_cursor = false;
        for state in self.frame_states.values_mut() {
            state.lang_picker_open = false;
            state.lang_picker_filter.clear();
        }
        for state in &mut self.prelude_states {
            state.lang_picker_open = false;
            state.lang_picker_filter.clear();
        }
    }

    pub(super) fn should_auto_open_picker_after_insert(&self) -> bool {
        self.view_mode == super::ViewMode::Stack
    }

    pub(crate) fn clear_sequencer_inline_edit(&mut self) {
        self.sequencer_inline_edit = None;
    }

    pub(crate) fn clear_sequencer_line_speed_focus(&mut self) {
        self.sequencer_line_speed_focus = None;
    }

    pub(crate) fn begin_sequencer_inline_edit(
        &mut self,
        pos: (usize, usize),
        field: SequencerInlineField,
        frame: &Frame,
    ) {
        self.sequencer_inline_edit = Some(SequencerInlineEdit {
            target: pos,
            field,
            buffer: field.prefill_value(frame),
            request_focus: true,
        });
        self.scroll_to_cursor = true;
    }

    pub(crate) fn begin_sequencer_line_speed_focus(&mut self, li: usize) {
        self.sequencer_line_speed_focus = Some(li);
        self.scroll_to_cursor = true;
    }

    pub(super) fn sync_sequencer_inline_edit(&mut self, scene: &Scene) {
        let Some(edit) = &self.sequencer_inline_edit else {
            return;
        };
        let keep = self.view_mode == super::ViewMode::Sequencer
            && matches!(
                self.state,
                SceneState::NavigatingFrame { cursor } if cursor == edit.target
            )
            && scene
                .lines
                .get(edit.target.0)
                .is_some_and(|line| edit.target.1 < line.frames.len());
        if !keep {
            self.clear_sequencer_inline_edit();
        }
    }

    pub(super) fn sync_sequencer_line_speed_focus(&mut self, scene: &Scene) {
        let Some(li) = self.sequencer_line_speed_focus else {
            return;
        };
        let keep = self.view_mode == super::ViewMode::Sequencer && li < scene.lines.len();
        if !keep {
            self.clear_sequencer_line_speed_focus();
        }
    }

    pub(crate) fn open_context_menu(&mut self, target: ContextTarget, pos: egui::Pos2) {
        self.overlay = Overlay::ContextMenu {
            target,
            pos,
            just_opened: true,
        };
    }

    /// Place cursor on a frame, entering navigation mode.
    pub(crate) fn navigate_to_frame(&mut self, pos: (usize, usize), bridge: &ClientBridge) {
        if self
            .sequencer_inline_edit
            .as_ref()
            .is_some_and(|edit| edit.target != pos)
        {
            self.clear_sequencer_inline_edit();
        }
        if self
            .sequencer_line_speed_focus
            .is_some_and(|li| li != pos.0)
        {
            self.clear_sequencer_line_speed_focus();
        }
        self.state = SceneState::NavigatingFrame { cursor: pos };
        self.selection.clear();
        self.selection.insert(pos);
        self.anchor = Some(pos);
        self.scroll_to_cursor = true;
        if bridge.is_connected() {
            bridge.send(ClientMessage::CursorPosition(pos.0, pos.1, None));
        }
    }

    /// Move cursor within navigation mode. Does NOT reset selection (caller decides).
    pub(crate) fn move_cursor(&mut self, pos: (usize, usize), bridge: &ClientBridge) {
        if self
            .sequencer_inline_edit
            .as_ref()
            .is_some_and(|edit| edit.target != pos)
        {
            self.clear_sequencer_inline_edit();
        }
        if let SceneState::NavigatingFrame { ref mut cursor, .. }
        | SceneState::EditingFrame { ref mut cursor } = self.state
        {
            *cursor = pos;
        } else {
            self.state = SceneState::NavigatingFrame { cursor: pos };
        }
        self.scroll_to_cursor = true;
        if bridge.is_connected() {
            bridge.send(ClientMessage::CursorPosition(pos.0, pos.1, None));
        }
    }

    /// Select a prelude script for navigation.
    pub(crate) fn navigate_to_prelude(&mut self, index: usize) {
        self.clear_sequencer_inline_edit();
        self.clear_sequencer_line_speed_focus();
        self.state = SceneState::NavigatingPrelude { index };
        self.selection.clear();
        self.anchor = None;
    }

    /// Enter edit mode for the current frame.
    pub(crate) fn enter_frame_edit(&mut self, pos: (usize, usize)) {
        self.clear_sequencer_inline_edit();
        self.clear_sequencer_line_speed_focus();
        self.state = SceneState::EditingFrame { cursor: pos };
        self.selection.clear();
        self.selection.insert(pos);
        self.anchor = Some(pos);
        if let Some(state) = self.frame_states.get_mut(&pos) {
            state.focus_request = FocusRequest::Editor;
        }
    }

    /// Enter edit mode for a prelude script.
    pub(crate) fn enter_prelude_edit(&mut self, index: usize) {
        self.clear_sequencer_inline_edit();
        self.clear_sequencer_line_speed_focus();
        self.state = SceneState::EditingPrelude { index };
        if let Some(state) = self.prelude_states.get_mut(index) {
            state.request_focus = true;
        }
    }

    /// Exit edit mode back to navigation.
    pub(crate) fn exit_edit_mode(&mut self) {
        self.clear_sequencer_inline_edit();
        self.clear_sequencer_line_speed_focus();
        self.close_inline_lang_pickers();
        match self.state {
            SceneState::EditingFrame { cursor } => {
                self.state = SceneState::NavigatingFrame { cursor };
                self.selection.clear();
                self.selection.insert(cursor);
                self.anchor = Some(cursor);
                self.scroll_to_cursor = true;
            }
            SceneState::EditingPrelude { index } => {
                self.state = SceneState::NavigatingPrelude { index };
            }
            _ => {}
        }
    }

    /// Enter focus mode (stack only).
    pub(crate) fn enter_focus_mode(&mut self, pos: (usize, usize)) {
        self.clear_sequencer_inline_edit();
        self.clear_sequencer_line_speed_focus();
        self.state = SceneState::FocusedFrame { frame: pos };
    }

    /// Exit focus mode back to navigation.
    pub(crate) fn exit_focus_mode(&mut self, bridge: &ClientBridge) {
        self.clear_sequencer_inline_edit();
        self.clear_sequencer_line_speed_focus();
        if let SceneState::FocusedFrame { frame } = self.state {
            self.navigate_to_frame(frame, bridge);
        }
    }

    /// Return to empty state (escape from navigation).
    pub(crate) fn deselect_all(&mut self) {
        self.clear_sequencer_inline_edit();
        self.clear_sequencer_line_speed_focus();
        self.state = SceneState::Empty;
        self.selection.clear();
        self.anchor = None;
    }
}

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::{Buffer, Rect},
    widgets::StatefulWidget,
};

use sova_core::{
    scene::script::Script,
    schedule::{ActionTiming, SchedulerMessage},
};

use crate::{
    app::{AppPage, AppState}, widgets::editor::Editor,
};

pub struct ScriptEditor {
    pub editor: Editor,
}

impl ScriptEditor {
    pub fn new() -> Self {
        Self {
            editor: Editor::new(),
        }
    }

    pub fn update_content(&mut self, state: &AppState) {
        let Some(frame) = state.selected_frame() else {
            self.editor.set_text("");
            return;
        };
        self.editor.set_text(frame.script().content());
        self.editor.lang = frame.script().lang().to_string();
    }

    pub fn upload(&mut self, state: &mut AppState) {
        self.editor.flash.trigger();
        state.events.send(SchedulerMessage::SetScript(
            state.selected.0,
            state.selected.1,
            Script::new(self.editor.content(), self.editor.lang.clone()),
            ActionTiming::Immediate,
        ));
        self.editor.mark_unedited();
    }

    pub fn on_key_event(&mut self, event: KeyEvent, state: &mut AppState) {
        match event.code {
            KeyCode::Enter
                if event.modifiers.intersects(
                    KeyModifiers::CONTROL | KeyModifiers::SHIFT | KeyModifiers::ALT,
                ) =>
            {
                self.upload(state);
            }
            KeyCode::Char('s') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.upload(state);
            }
            KeyCode::PageUp => {
                state.go_to_previous_frame();
            }
            KeyCode::PageDown => {
                state.go_to_next_frame();
            }
            KeyCode::Esc => {
                state.go_to(AppPage::Scene);
            }
            _ => {
                self.editor.on_key_event(event, state);
            }
        }
    }
}

impl StatefulWidget for &ScriptEditor {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.editor.render(area, buf, state);
    }
}

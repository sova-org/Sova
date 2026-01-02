use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, style::{Style, Stylize}, widgets::{StatefulWidget, Widget}};
use sova_core::{scene::script::Script, schedule::{ActionTiming, SchedulerMessage}};
use tui_textarea::{CursorMove, TextArea};

use crate::{app::AppState, event::AppEvent, popup::PopupValue};

#[derive(Default)]
pub struct EditWidget {
    text_area: TextArea<'static>
}

fn upload_script(state: &mut AppState, script: Script) {
    let (line_id, frame_id) = state.selected;
    state.events.send(
        SchedulerMessage::SetScript(
            line_id, 
            frame_id, 
            script,
            ActionTiming::Immediate
        ).into()
    );
    state.events.send(
        AppEvent::Positive("Sent script".to_owned())
    );
}

fn upload_content(state: &mut AppState, content: String) {
    let Some(frame) = state.selected_frame() else {
        return;
    };
    let mut script = frame.script().clone();
    script.set_content(content);
    upload_script(state, script);
}

fn upload_lang(state: &mut AppState, lang: String) {
    let Some(frame) = state.selected_frame() else {
        return;
    };
    let mut script = frame.script().clone();
    script.set_lang(lang);
    upload_script(state, script);
}

impl EditWidget {

    pub fn open(&mut self, state: &AppState) {
        let Some(frame) = state.selected_frame() else {
            return;
        };
        let content = frame.script().content();
        self.text_area = content.lines().into();
        self.text_area.set_line_number_style(Style::default().dark_gray());
    }

    pub fn get_help() -> &'static str {
        "\
        C-S: Upload \n\
        C-L: Change language \n\
        C-A: Select all \n\
        "
    }

    pub fn process_event(&mut self, state: &mut AppState, mut event: KeyEvent) { 
        match event.code {
            KeyCode::Char('s') if event.modifiers == KeyModifiers::CONTROL => {
                upload_content(state, self.get_content());
            } 
            KeyCode::Char('a') if event.modifiers == KeyModifiers::CONTROL => {
                self.text_area.select_all();
            }
            KeyCode::Char('l') if event.modifiers == KeyModifiers::CONTROL => {
                let Some(frame) = state.selected_frame() else {
                    return;
                };
                let langs : Vec<String> = state.languages.languages().map(str::to_owned).collect();
                let i = langs.iter().position(|l| l == frame.script().lang()).unwrap_or_default();
                state.events.send(AppEvent::Popup(
                    "Script language".to_owned(), 
                    "Which language to use for this script ?".to_owned(), 
                    PopupValue::Choice(i, langs),
                    Box::new(|state, x| {
                        upload_lang(state, x.into());
                    })));
            }
            KeyCode::Char('w') if event.modifiers == KeyModifiers::CONTROL => {
                self.text_area.start_selection();
                self.text_area.move_cursor(CursorMove::WordForward);
            }
            KeyCode::Char('c') if event.modifiers == KeyModifiers::CONTROL => {
                self.text_area.copy();
                if let Some(clipboard) = &mut state.clipboard {
                    let _ = clipboard.set_text(self.text_area.yank_text());
                }
                state.events.send(
                    AppEvent::Positive("Text yanked !".to_owned())
                );
            }
            KeyCode::Char('x') if event.modifiers == KeyModifiers::CONTROL => {
                self.text_area.cut();
                if let Some(clipboard) = &mut state.clipboard {
                    let _ = clipboard.set_text(self.text_area.yank_text());
                }
                state.events.send(
                    AppEvent::Positive("Text yanked !".to_owned())
                );
            }
            KeyCode::Char('v') if event.modifiers == KeyModifiers::CONTROL => {
                if let Some(clipboard) = &mut state.clipboard {
                    if let Ok(txt) = clipboard.get_text() {
                        self.text_area.set_yank_text(txt);
                    }
                }
                self.text_area.paste();
            }
            _ => {
                if cfg!(windows) {
                    if event.modifiers == (KeyModifiers::CONTROL | KeyModifiers::ALT) {
                        event.modifiers = KeyModifiers::empty();
                    }
                }
                self.text_area.input(event);
            }
        }
    }

    pub fn get_content(&self) -> String {
        self.text_area.lines().join("\n")
    }

}

impl StatefulWidget for &EditWidget {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, _state: &mut Self::State) {
        use Constraint::*;
        let layout = Layout::vertical([Min(0), Length(2)]);
        let [main_area, _tools_area] = layout.areas(area);
        self.text_area.render(main_area, buf);
    }
}

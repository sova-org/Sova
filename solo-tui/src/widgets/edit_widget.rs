use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, style::{Style, Stylize}, widgets::{StatefulWidget, Widget}};
use sova_core::schedule::{ActionTiming, SchedulerMessage};
use tui_textarea::TextArea;

use crate::{app::AppState, event::AppEvent, popup::PopupValue};

#[derive(Default)]
pub struct EditWidget {
    text_area: TextArea<'static>
}

fn upload_content(state: &mut AppState, content: String) {
    let Some(frame) = state.selected_frame() else {
        return;
    };
    let (line_id, frame_id) = state.selected;
    state.events.send(
        SchedulerMessage::SetScript(
            line_id, 
            frame_id, 
            frame.script().lang().to_owned(), 
            content,
            ActionTiming::Immediate
        ).into()
    );
    state.events.send(
        AppEvent::Positive("Sent script".to_owned())
    );
}

fn upload_lang(state: &mut AppState, lang: String) {
    let Some(frame) = state.selected_frame() else {
        return;
    };
    let (line_id, frame_id) = state.selected;
    state.events.send(
        SchedulerMessage::SetScript(
            line_id, 
            frame_id, 
            lang,
            frame.script().content().to_owned(),
            ActionTiming::Immediate
        ).into()
    );
    state.events.send(
        AppEvent::Positive("Changed language".to_owned())
    );
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
        C-S: Upload
        "
    }

    pub fn process_event(&mut self, state: &mut AppState, event: KeyEvent) { 
        match event.code {
            KeyCode::Char('s') if event.modifiers == KeyModifiers::CONTROL => {
                upload_content(state, self.get_content());
            } 
            KeyCode::Char('a') if event.modifiers == KeyModifiers::CONTROL => {
                self.text_area.select_all();
            }
            KeyCode::Char('l') if event.modifiers == KeyModifiers::CONTROL => {
                state.events.send(AppEvent::Popup(
                    "Script language".to_owned(), 
                    "Which language to use for this script ?".to_owned(), 
                    PopupValue::Text(state
                        .selected_frame()
                        .map(|f| f.script().lang().to_owned())
                        .unwrap_or(String::new())),
                    Box::new(|state, x| {
                        upload_lang(state, x.text());
                    })));
            }
            _ => { 
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

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use Constraint::*;
        let layout = Layout::vertical([Min(0), Length(2)]);
        let [main_area, tools_area] = layout.areas(area);
        self.text_area.render(main_area, buf);
    }
}
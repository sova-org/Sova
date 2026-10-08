use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{buffer::Buffer, layout::{Alignment, Constraint::*, Flex, Layout, Margin, Rect}, style::Stylize, widgets::{Block, BorderType, Paragraph, StatefulWidget, Widget}};
use ratatui_textarea::TextArea;
use ratatui_themes::Style;
use tui_big_text::{BigText, PixelSize};
use tui_checkbox::Checkbox;

use crate::{app::AppState, event::AppEvent, theme::Palette};

pub struct ConnectionView {
    pub username_field: TextArea<'static>,
    pub ip_field: TextArea<'static>,
    pub port_field: TextArea<'static>,
    pub password_field: TextArea<'static>,
    pub feedback: bool,
    pub current_input: usize,
}

impl ConnectionView {

    pub fn new() -> Self {
        let mut view = ConnectionView { 
            ip_field: TextArea::default(), 
            port_field: TextArea::default(), 
            username_field: TextArea::default(), 
            password_field: TextArea::default(),
            feedback: false,
            current_input: 0
        };
        view.password_field.set_mask_char('\u{2022}');
        view.ip_field.set_cursor_line_style(Style::new());
        view.port_field.set_cursor_line_style(Style::new());
        view.username_field.set_cursor_line_style(Style::new());
        view.password_field.set_cursor_line_style(Style::new());
        view
    }

    pub fn apply_palette(&mut self, palette: &Palette) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(palette.accent)
            .bg(palette.selection);
        let muted_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(palette.foreground)
            .bg(palette.surface);
        let select_block_for = |i: usize, t: String, input: &mut TextArea| {
            let selected = self.current_input == i;
            input.set_block(if selected { block.clone() } else { muted_block.clone() }.title(t));
            if selected {
                input.set_cursor_style(palette.text().reversed());
            } else {
                input.set_cursor_style(input.cursor_line_style());
            }
        };
        select_block_for(0, "IP".to_owned(), &mut self.ip_field);
        select_block_for(1, "Port".to_owned(), &mut self.port_field);
        select_block_for(2, "Username".to_owned(), &mut self.username_field);
        select_block_for(3, "Password".to_owned(), &mut self.password_field);

        
    }

    pub fn on_key_event(&mut self, event: KeyEvent, state: &mut AppState) {
        match event.code {
            KeyCode::Up => {
                self.current_input = self.current_input.saturating_sub(1)
                    .min(4);
            }
            KeyCode::BackTab => {
                self.current_input = self.current_input.saturating_sub(1);
            }
            KeyCode::Down => {
                if self.current_input < 5 {
                    self.current_input = (self.current_input + 1).min(5);
                }
            }
            KeyCode::Tab => {
                if self.current_input == 6 {
                    self.current_input = 5;
                    return;
                }
                self.current_input = (self.current_input + 1).min(6);
            }
            KeyCode::Left if self.current_input == 6 => {
                self.current_input = 5; 
            }
            KeyCode::Right if self.current_input == 5 => {
                self.current_input = 6;
            }
            KeyCode::Enter => {
                if self.current_input == 4 {
                    self.feedback = !self.feedback;
                } else if self.current_input == 5 || self.current_input == 6 {
                    let ip = self.ip_field.lines().join("");
                    let Ok(port) = self.port_field.lines().join("").parse() else {
                        state.events.send(AppEvent::Negative("Invalid port format !".to_string()));
                        return;
                    };
                    let username = self.username_field.lines().join("");
                    if username.is_empty() {
                        state.events.send(AppEvent::Negative("Username must not be empty !".to_string()));
                        return;
                    }
                    let password = self.password_field.lines().join("");
                    if self.current_input == 5 {
                        if ip.is_empty() {
                            state.events.send(AppEvent::Negative("IP must not be empty !".to_string()));
                            return;
                        }
                        state.events.send(AppEvent::Connect(ip, port, username, password));
                    } else {
                        state.events.send(AppEvent::Server(port, username, password));
                    }
                } else {
                    self.current_input = self.current_input + 1;
                } 
            }
            _ => {
                match self.current_input {
                    0 => { self.ip_field.input(event); },
                    1 => { self.port_field.input(event); },
                    2 => { self.username_field.input(event); },
                    3 => { self.password_field.input(event); },
                    _ => ()
                };
            }
        }
    }

}

impl StatefulWidget for &mut ConnectionView {
    type State = AppState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.apply_palette(&state.palette);
        let big_text = BigText::builder()
            .pixel_size(PixelSize::Full)
            .style(state.palette.foreground)
            .centered()
            .lines(vec!["Sova".into()])
            .build();
        let layout = Layout::vertical([Length(8), Length(18)])
            .flex(Flex::Center)
            .spacing(2)
            .split(area);
        big_text.render(layout[0], buf);

        let block_layout = Layout::horizontal([Length(40)])
            .flex(Flex::Center)
            .split(layout[1]);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(state.palette.foreground)
            .bg(state.palette.surface)
            .fg(state.palette.foreground)
            .title("Connection")
            .title_alignment(Alignment::Center);
        
        let fields_area = block.inner(block_layout[0]);
        block.render(block_layout[0], buf);
        let fields_layout = Layout::vertical([
            Length(3), 
            Length(3), 
            Length(3), 
            Length(3), 
            Length(1), 
            Length(3)
        ]).flex(Flex::Center).split(fields_area);

        self.ip_field.render(fields_layout[0], buf);
        self.port_field.render(fields_layout[1], buf);
        self.username_field.render(fields_layout[2], buf);
        self.password_field.render(fields_layout[3], buf);

        let checkbox = Checkbox::new("Audio feedback", self.feedback)
            .checked_symbol(nerd_font_symbols::md::MD_CHECKBOX_OUTLINE)
            .unchecked_symbol(nerd_font_symbols::md::MD_CHECKBOX_BLANK_OUTLINE)
            .style(
                if self.current_input == 4 {
                    Style::default().fg(state.palette.accent).bg(state.palette.selection)
                } else {
                    Style::default().bg(state.palette.surface).fg(if self.feedback {
                        state.palette.foreground
                    } else {
                        state.palette.muted
                    })
                }
            );
        checkbox.render(fields_layout[4].inner(Margin::new(1, 0)), buf);

        let buttons_layout = Layout::horizontal([Percentage(50), Percentage(50)])
            .split(fields_layout[5]);

        let button_for = |text: String, i: usize| {
            let selected = self.current_input == i;
            let block = Block::bordered().border_type(BorderType::Rounded)
                .border_style(if selected { state.palette.accent } else { state.palette.foreground })
                .bg(if selected { state.palette.selection } else { state.palette.surface });
            Paragraph::new(text).style(
                if selected { state.palette.accent } else { state.palette.foreground }
            ).centered().block(block)
        };
        
        let connect_button = button_for("Connect".to_owned(), 5);
        let create_button = button_for("Create".to_owned(), 6);
        
        connect_button.render(buttons_layout[0], buf);
        create_button.render(buttons_layout[1], buf);
    }
}
use crate::app::{App, EditableSetting};
use crate::components::Component;
use crate::disk::{self, Theme}; // Import disk module and Theme
use crate::utils::layout::centered_rect;
use crate::utils::styles::CommonStyles;
use color_eyre::Result as EyreResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Color,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use tui_textarea::TextArea;

/// Component responsible for displaying and handling application settings.
pub struct OptionsComponent;

impl Default for OptionsComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionsComponent {
    /// Creates a new `OptionsComponent`.
    pub fn new() -> Self {
        Self {}
    }
}

// Constante pour le pas de modification par flèches
const DURATION_STEP: u64 = 1; // Modifier en secondes
const MIN_DURATION: u64 = 1; // Minimum 1 seconde

impl Component for OptionsComponent {
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        // --- Handle Setting Input Mode ---
        if app.interface.components.is_editing_setting {
            let target_setting = app.interface.components.setting_input_target;
            match key_event.code {
                KeyCode::Enter => {
                    let input_str = app.interface.components.setting_input_area.lines()[0].trim();
                    let status_msg;
                    let mut clear_editing_state = false;

                    match input_str.parse::<u64>() {
                        Ok(value) if value >= MIN_DURATION => {
                            if let Some(target) = target_setting {
                                match target {
                                    EditableSetting::SketchDuration => {
                                        app.client_config.sketch_duration_secs = value;
                                        status_msg =
                                            format!("Sketch duration set to {} seconds.", value);
                                    }
                                    EditableSetting::ScreensaverTimeout => {
                                        app.client_config.screensaver_timeout_secs = value;
                                        status_msg = format!(
                                            "Screensaver timeout set to {} seconds.",
                                            value
                                        );
                                    }
                                }
                                clear_editing_state = true;
                            } else {
                                status_msg = "Error: No target setting for input.".to_string();
                            }
                        }
                        Ok(_) => {
                            status_msg = format!(
                                "Invalid value: Must be at least {} second(s).",
                                MIN_DURATION
                            );
                        }
                        Err(_) => {
                            status_msg = "Invalid input: Please enter a number.".to_string();
                        }
                    }

                    if !status_msg.is_empty() {
                        app.set_status_message(status_msg);
                    }
                    if clear_editing_state {
                        app.interface.components.is_editing_setting = false;
                        app.interface.components.setting_input_target = None;
                    }
                    return Ok(true);
                }
                KeyCode::Esc => {
                    app.interface.components.is_editing_setting = false;
                    app.interface.components.setting_input_target = None;
                    app.set_status_message("Setting edit cancelled.".to_string());
                    return Ok(true);
                }
                _ => {
                    // Emprunter mutuellement setting_input_area ici
                    let handled = app.interface.components.setting_input_area.input(key_event);
                    return Ok(handled);
                }
            }
        }

        // --- Handle Normal Navigation/Action Mode ---
        let selected_index = app.interface.components.options_selected_index;
        let num_options: usize = 5; // Updated to include theme option
        let mut new_selected_index = selected_index;

        match key_event.code {
            KeyCode::Up => {
                new_selected_index = selected_index.saturating_sub(1);
                app.interface.components.options_selected_index = new_selected_index;
                Ok(true)
            }
            KeyCode::Down => {
                if num_options > 0 {
                    new_selected_index = (selected_index + 1).min(num_options.saturating_sub(1));
                }
                app.interface.components.options_selected_index = new_selected_index;
                Ok(true)
            }
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('e') => {
                let setting_to_edit = match selected_index {
                    0 => None, // Keymap
                    1 => None, // Theme
                    2 => None, // Screensaver Enabled (toggle only)
                    3 => Some(EditableSetting::SketchDuration),
                    4 => Some(EditableSetting::ScreensaverTimeout),
                    _ => None,
                };

                if let Some(setting) = setting_to_edit {
                    let current_value = match setting {
                        EditableSetting::SketchDuration => app.client_config.sketch_duration_secs,
                        EditableSetting::ScreensaverTimeout => {
                            app.client_config.screensaver_timeout_secs
                        }
                    };
                    let components = &mut app.interface.components;
                    components.is_editing_setting = true;
                    components.setting_input_target = Some(setting);
                    components.setting_input_area = TextArea::new(vec![current_value.to_string()]);
                    components.setting_input_area.set_block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Enter Value (Esc: Cancel, Enter: Confirm) ")
                            .border_style(CommonStyles::warning_themed(&app.client_config.theme)),
                    );
                    components
                        .setting_input_area
                        .set_style(CommonStyles::default_text_themed(&app.client_config.theme));
                    app.set_status_message("Editing setting value...".to_string());
                } else if selected_index == 0 {
                    // Toggle Keymap
                    let current_mode = app.client_config.editing_mode.clone();
                    app.client_config.editing_mode = match current_mode {
                        disk::EditingMode::Normal => disk::EditingMode::Vim,
                        disk::EditingMode::Vim => disk::EditingMode::Normal,
                    };
                    let new_mode_str = app.client_config.editing_mode.to_string();
                    app.set_status_message(format!("Editor keymap set to {}", new_mode_str));
                } else if selected_index == 1 {
                    // Cycle Theme
                    app.client_config.theme = match app.client_config.theme {
                        Theme::Classic => Theme::Ocean,
                        Theme::Ocean => Theme::Forest,
                        Theme::Forest => Theme::Classic,
                    };
                    app.set_status_message(format!("Theme set to {}", app.client_config.theme));
                } else if selected_index == 2 {
                    // Toggle Screensaver Enabled
                    app.client_config.screensaver_enabled = !app.client_config.screensaver_enabled;
                    let status = if app.client_config.screensaver_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    };
                    app.set_status_message(format!("Screensaver {}", status));
                }
                Ok(true)
            }
            KeyCode::Left => {
                let mut status_msg = "".to_string();
                match selected_index {
                    0 => {
                        if app.client_config.editing_mode == disk::EditingMode::Vim {
                            app.client_config.editing_mode = disk::EditingMode::Normal;
                            status_msg =
                                format!("Editor keymap set to {}", app.client_config.editing_mode);
                        }
                    }
                    1 => {
                        // Cycle theme backwards
                        app.client_config.theme = match app.client_config.theme {
                            Theme::Classic => Theme::Forest,
                            Theme::Ocean => Theme::Classic,
                            Theme::Forest => Theme::Ocean,
                        };
                        status_msg = format!("Theme set to {}", app.client_config.theme);
                    }
                    2 => {
                        if app.client_config.screensaver_enabled {
                            app.client_config.screensaver_enabled = false;
                            status_msg = "Screensaver disabled".to_string();
                        }
                    }
                    3 => {
                        let current_val = app.client_config.sketch_duration_secs;
                        let new_val = current_val.saturating_sub(DURATION_STEP).max(MIN_DURATION);
                        if new_val != current_val {
                            app.client_config.sketch_duration_secs = new_val;
                            status_msg = format!("Sketch duration: {}s", new_val);
                        }
                    }
                    4 => {
                        let current_val = app.client_config.screensaver_timeout_secs;
                        let new_val = current_val.saturating_sub(DURATION_STEP).max(MIN_DURATION);
                        if new_val != current_val {
                            app.client_config.screensaver_timeout_secs = new_val;
                            status_msg = format!("Screensaver timeout: {}s", new_val);
                        }
                    }
                    _ => {}
                }
                if !status_msg.is_empty() {
                    app.set_status_message(status_msg);
                }
                Ok(true)
            }
            KeyCode::Right => {
                let mut status_msg = "".to_string();
                match selected_index {
                    0 => {
                        if app.client_config.editing_mode == disk::EditingMode::Normal {
                            app.client_config.editing_mode = disk::EditingMode::Vim;
                            status_msg =
                                format!("Editor keymap set to {}", app.client_config.editing_mode);
                        }
                    }
                    1 => {
                        // Cycle theme forwards
                        app.client_config.theme = match app.client_config.theme {
                            Theme::Classic => Theme::Ocean,
                            Theme::Ocean => Theme::Forest,
                            Theme::Forest => Theme::Classic,
                        };
                        status_msg = format!("Theme set to {}", app.client_config.theme);
                    }
                    2 => {
                        if !app.client_config.screensaver_enabled {
                            app.client_config.screensaver_enabled = true;
                            status_msg = "Screensaver enabled".to_string();
                        }
                    }
                    3 => {
                        let current_val = app.client_config.sketch_duration_secs;
                        let new_val = current_val.saturating_add(DURATION_STEP);
                        if new_val != current_val {
                            app.client_config.sketch_duration_secs = new_val;
                            status_msg = format!("Sketch duration: {}s", new_val);
                        }
                    }
                    4 => {
                        let current_val = app.client_config.screensaver_timeout_secs;
                        let new_val = current_val.saturating_add(DURATION_STEP);
                        if new_val != current_val {
                            app.client_config.screensaver_timeout_secs = new_val;
                            status_msg = format!("Screensaver timeout: {}s", new_val);
                        }
                    }
                    _ => {}
                }
                if !status_msg.is_empty() {
                    app.set_status_message(status_msg);
                }
                Ok(true)
            }
            // Passer les autres touches non gérées
            _ => Ok(false),
        }
    }

    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let components = &app.interface.components;
        let config = &app.client_config;
        let selected_index = components.options_selected_index;

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .title(" Options ")
            .title_alignment(Alignment::Center)
            .style(CommonStyles::default_text_themed(&config.theme));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // List of options
                Constraint::Length(1), // Help text at the bottom
            ])
            .split(inner_area);

        let options_area = chunks[0];
        let help_area = chunks[1];

        // Définir les styles with current theme
        let theme = &config.theme;
        let normal_style = CommonStyles::default_text_themed(theme);
        let value_style = CommonStyles::value_text_themed(theme);
        let selected_style = CommonStyles::selected_item_themed(theme);
        let name_width = 25;

        // Define the list items for available options
        let options = vec![
            // 0: Editor Keymap
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:<width$}", "Editor Keymap:", width = name_width)),
                Span::styled(config.editing_mode.to_string(), value_style),
            ]))
            .style(if selected_index == 0 {
                selected_style
            } else {
                normal_style
            }),
            // 1: Theme
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:<width$}", "Theme:", width = name_width)),
                Span::styled(config.theme.to_string(), value_style),
            ]))
            .style(if selected_index == 1 {
                selected_style
            } else {
                normal_style
            }),
            // 2: Screensaver Enabled
            ListItem::new(Line::from(vec![
                Span::raw(format!(
                    "{:<width$}",
                    "Screensaver Enabled:",
                    width = name_width
                )),
                Span::styled(
                    if config.screensaver_enabled {
                        "[X]"
                    } else {
                        "[ ]"
                    },
                    if config.screensaver_enabled {
                        value_style.fg(Color::Green)
                    } else {
                        value_style.fg(Color::Red)
                    },
                ),
            ]))
            .style(if selected_index == 2 {
                selected_style
            } else {
                normal_style
            }),
            // 3: Sketch Duration
            ListItem::new(Line::from(vec![
                Span::raw(format!(
                    "{:<width$}",
                    "Sketch Duration (s):",
                    width = name_width
                )),
                Span::styled(config.sketch_duration_secs.to_string(), value_style),
            ]))
            .style(if selected_index == 3 {
                selected_style
            } else {
                normal_style
            }),
            // 4: Screensaver Timeout
            ListItem::new(Line::from(vec![
                Span::raw(format!(
                    "{:<width$}",
                    "Screensaver Timeout (s):",
                    width = name_width
                )),
                Span::styled(config.screensaver_timeout_secs.to_string(), value_style),
            ]))
            .style(if selected_index == 4 {
                selected_style
            } else {
                normal_style
            }),
        ];

        let options_list = List::new(options).highlight_symbol("> ");

        let mut list_state = ListState::default();
        list_state.select(Some(selected_index));

        frame.render_stateful_widget(options_list, options_area, &mut list_state);

        let help_style = CommonStyles::description_themed(&config.theme);
        let key_style = CommonStyles::key_binding_themed(&config.theme);
        let help_text = Line::from(vec![
            Span::styled("↑↓", key_style),
            Span::styled(": Navigate, ", help_style),
            Span::styled("Enter/Space/e", key_style),
            Span::styled(": Edit/Toggle, ", help_style),
            Span::styled("←→", key_style),
            Span::styled(": Set, ", help_style),
            Span::styled("Esc", key_style),
            Span::styled(": Back", help_style),
        ])
        .alignment(Alignment::Center);

        frame.render_widget(Paragraph::new(help_text), help_area);

        if components.is_editing_setting {
            let popup_width_percentage = 50;
            let desired_height = 3;

            let centered_area = centered_rect(popup_width_percentage, 20, area);

            let popup_area = Rect {
                x: centered_area.x,
                y: area.y + (area.height.saturating_sub(desired_height)) / 2,
                width: centered_area.width,
                height: desired_height,
            };
            let mut textarea_to_render = components.setting_input_area.clone();
            textarea_to_render.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Enter Value (Esc: Cancel, Enter: Confirm) ")
                    .style(CommonStyles::warning_themed(&config.theme)),
            );
            textarea_to_render.set_style(CommonStyles::default_text_themed(&config.theme));

            frame.render_widget(Clear, popup_area);
            frame.render_widget(&textarea_to_render, popup_area);
        }
    }
}

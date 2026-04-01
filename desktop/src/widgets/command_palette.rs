use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandId {
    // Panels
    Server,
    Audio,
    Devices,
    Scope,
    Spectrum,
    VuMeter,
    ScopeBar,
    Chat,
    Logs,
    Options,
    Debug,
    Keybindings,
    About,
    SampleBrowser,
    Documentation,
    Visuals,
    // Engine
    RestartCore,
    // Transport
    PlayPause,
    // File
    SaveScene,
    LoadScene,
    ResetScene,
    // View
    ZoomIn,
    ZoomOut,
    ZoomReset,
}

pub enum PaletteAction {
    None,
    Execute(CommandId),
}

struct Command {
    id: CommandId,
    label: String,
    category: String,
    desc: String,
    shortcut: Option<String>,
    active: bool,
}

fn modifier_prefix() -> &'static str {
    if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    }
}

fn commands() -> Vec<Command> {
    let m = modifier_prefix();

    let panel = |id, label_key: &str, desc_key: &str, shortcut: String| Command {
        id,
        label: t!(label_key).into(),
        category: t!("cmd.category.panel").into(),
        desc: t!(desc_key).into(),
        shortcut: Some(shortcut),
        active: false,
    };

    vec![
        // Panels
        panel(
            CommandId::Options,
            "cmd.options",
            "cmd.options.desc",
            format!("{m}+,"),
        ),
        panel(
            CommandId::Server,
            "cmd.server",
            "cmd.server.desc",
            format!("{m}+Shift+S"),
        ),
        panel(
            CommandId::Audio,
            "cmd.audio",
            "cmd.audio.desc",
            format!("{m}+Shift+A"),
        ),
        panel(
            CommandId::Devices,
            "cmd.devices",
            "cmd.devices.desc",
            format!("{m}+Shift+I"),
        ),
        panel(
            CommandId::Scope,
            "cmd.scope",
            "cmd.scope.desc",
            format!("{m}+Shift+O"),
        ),
        panel(
            CommandId::Spectrum,
            "cmd.spectrum",
            "cmd.spectrum.desc",
            format!("{m}+Shift+P"),
        ),
        panel(
            CommandId::VuMeter,
            "cmd.vu_meter",
            "cmd.vu_meter.desc",
            format!("{m}+Shift+U"),
        ),
        panel(
            CommandId::ScopeBar,
            "cmd.scope_bar",
            "cmd.scope_bar.desc",
            format!("{m}+Shift+W"),
        ),
        panel(
            CommandId::Chat,
            "cmd.chat",
            "cmd.chat.desc",
            format!("{m}+Shift+C"),
        ),
        panel(
            CommandId::Logs,
            "cmd.logs",
            "cmd.logs.desc",
            format!("{m}+Shift+L"),
        ),
        panel(
            CommandId::SampleBrowser,
            "cmd.sample_browser",
            "cmd.sample_browser.desc",
            format!("{m}+Shift+E"),
        ),
        panel(
            CommandId::Documentation,
            "cmd.documentation",
            "cmd.documentation.desc",
            format!("{m}+Shift+H"),
        ),
        panel(
            CommandId::Visuals,
            "cmd.visuals",
            "cmd.visuals.desc",
            format!("{m}+Shift+V"),
        ),
        panel(
            CommandId::Debug,
            "cmd.debug",
            "cmd.debug.desc",
            format!("{m}+Shift+B"),
        ),
        panel(
            CommandId::Keybindings,
            "cmd.keybindings",
            "cmd.keybindings.desc",
            "F1".into(),
        ),
        panel(
            CommandId::About,
            "cmd.about",
            "cmd.about.desc",
            String::new(),
        ),
        // Engine
        Command {
            id: CommandId::RestartCore,
            label: t!("cmd.restart_core").into(),
            category: t!("cmd.category.engine").into(),
            desc: t!("cmd.restart_core.desc").into(),
            shortcut: None,
            active: false,
        },
        // Transport
        Command {
            id: CommandId::PlayPause,
            label: t!("cmd.play_pause").into(),
            category: t!("cmd.category.transport").into(),
            desc: t!("cmd.play_pause.desc").into(),
            shortcut: Some(format!("{m}+Shift+Space")),
            active: false,
        },
        // File
        Command {
            id: CommandId::SaveScene,
            label: t!("cmd.save_scene").into(),
            category: t!("cmd.category.file").into(),
            desc: t!("cmd.save_scene.desc").into(),
            shortcut: Some(format!("{m}+S")),
            active: false,
        },
        Command {
            id: CommandId::LoadScene,
            label: t!("cmd.load_scene").into(),
            category: t!("cmd.category.file").into(),
            desc: t!("cmd.load_scene.desc").into(),
            shortcut: Some(format!("{m}+O")),
            active: false,
        },
        Command {
            id: CommandId::ResetScene,
            label: t!("cmd.reset_scene").into(),
            category: t!("cmd.category.file").into(),
            desc: t!("cmd.reset_scene.desc").into(),
            shortcut: None,
            active: false,
        },
        // View
        Command {
            id: CommandId::ZoomIn,
            label: t!("cmd.zoom_in").into(),
            category: t!("cmd.category.view").into(),
            desc: t!("cmd.zoom_in.desc").into(),
            shortcut: Some(format!("{m}+=")),
            active: false,
        },
        Command {
            id: CommandId::ZoomOut,
            label: t!("cmd.zoom_out").into(),
            category: t!("cmd.category.view").into(),
            desc: t!("cmd.zoom_out.desc").into(),
            shortcut: Some(format!("{m}+-")),
            active: false,
        },
        Command {
            id: CommandId::ZoomReset,
            label: t!("cmd.zoom_reset").into(),
            category: t!("cmd.category.view").into(),
            desc: t!("cmd.zoom_reset.desc").into(),
            shortcut: Some(format!("{m}+0")),
            active: false,
        },
    ]
}

/// Panel open/closed states, passed each frame before showing the palette.
pub struct PanelStates {
    pub sidebar: bool,
    pub devices: bool,
    pub scope: bool,
    pub spectrum: bool,
    pub vu_meter: bool,
    pub scope_bar: bool,
    pub chat: bool,
    pub logs: bool,
    pub debug: bool,
    pub keybindings: bool,
    pub about: bool,
    pub sample_browser: bool,
    pub documentation: bool,
    pub visuals: bool,
}

struct FilteredEntry {
    cmd_idx: usize,
    /// Matched character indices within the label (for highlighting)
    label_matches: Vec<usize>,
}

pub struct CommandPalette {
    open: bool,
    query: String,
    selected: usize,
    filtered: Vec<FilteredEntry>,
    commands: Vec<Command>,
}

impl CommandPalette {
    pub fn new() -> Self {
        let cmds = commands();
        let count = cmds.len();
        Self {
            open: false,
            query: String::new(),
            selected: 0,
            filtered: (0..count)
                .map(|i| FilteredEntry {
                    cmd_idx: i,
                    label_matches: Vec::new(),
                })
                .collect(),
            commands: cmds,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.filtered = (0..self.commands.len())
            .map(|i| FilteredEntry {
                cmd_idx: i,
                label_matches: Vec::new(),
            })
            .collect();
    }

    pub fn update_states(&mut self, states: &PanelStates) {
        for cmd in &mut self.commands {
            cmd.active = match cmd.id {
                CommandId::Server | CommandId::Audio | CommandId::Options => states.sidebar,
                CommandId::Devices => states.devices,
                CommandId::Scope => states.scope,
                CommandId::Spectrum => states.spectrum,
                CommandId::VuMeter => states.vu_meter,
                CommandId::ScopeBar => states.scope_bar,
                CommandId::Chat => states.chat,
                CommandId::Logs => states.logs,
                CommandId::Debug => states.debug,
                CommandId::Keybindings => states.keybindings,
                CommandId::About => states.about,
                CommandId::SampleBrowser => states.sample_browser,
                CommandId::Documentation => states.documentation,
                CommandId::Visuals => states.visuals,
                _ => false,
            };
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> PaletteAction {
        if !self.open {
            return PaletteAction::None;
        }

        let mut action = PaletteAction::None;

        // Backdrop
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::new("cmd_palette_backdrop"))
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let resp = ui.allocate_response(screen.size(), egui::Sense::click());
                if resp.clicked() {
                    self.open = false;
                }
            });

        egui::Window::new("Command Palette")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 80.0])
            .default_width(600.0)
            .min_width(600.0)
            .show(ctx, |ui| {
                let input_id = ui.id().with("query");
                let input = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .id(input_id)
                        .hint_text(t!("cmd.type_command"))
                        .desired_width(f32::INFINITY),
                );
                input.request_focus();

                if input.changed() {
                    self.refilter();
                    self.selected = 0;
                }

                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
                let arrow_up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
                let arrow_down = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));

                if escape {
                    self.open = false;
                    return;
                }

                let prev_selected = self.selected;
                if arrow_up && self.selected > 0 {
                    self.selected -= 1;
                }
                if arrow_down && self.selected + 1 < self.filtered.len() {
                    self.selected += 1;
                }
                let selection_changed = self.selected != prev_selected;

                if enter && !self.filtered.is_empty() {
                    let idx = self.filtered[self.selected].cmd_idx;
                    action = PaletteAction::Execute(self.commands[idx].id);
                    self.open = false;
                    return;
                }

                ui.separator();
                let available_width = ui.available_width();
                let max_scroll = (screen.height() * 0.6).min(500.0);
                egui::ScrollArea::vertical()
                    .max_height(max_scroll)
                    .show(ui, |ui| {
                        ui.set_min_width(available_width);
                        let row_height = 40.0;
                        let accent = ui.visuals().selection.bg_fill;
                        let text_color = ui.visuals().text_color();
                        let weak_color = ui.visuals().weak_text_color();

                        for (i, entry) in self.filtered.iter().enumerate() {
                            let cmd = &self.commands[entry.cmd_idx];
                            let selected = i == self.selected;

                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), row_height),
                                egui::Sense::click(),
                            );

                            if selected {
                                ui.painter()
                                    .rect_filled(rect, 0.0, ui.visuals().selection.bg_fill);
                            } else if resp.hovered() {
                                ui.painter().rect_filled(
                                    rect,
                                    0.0,
                                    ui.visuals().widgets.hovered.bg_fill,
                                );
                            }

                            let label_x = 6.0;

                            // Active dot for open panels
                            if cmd.active {
                                let dot_center = rect.min + egui::vec2(label_x + 3.0, 10.0);
                                ui.painter().circle_filled(dot_center, 3.0, accent);
                            }

                            let text_offset = if cmd.active { 14.0 } else { 0.0 };

                            // Label with fuzzy match highlighting
                            let label_pos = rect.min + egui::vec2(label_x + text_offset, 2.0);
                            let (label_normal, label_highlight) = if selected {
                                let sel = ui.visuals().selection.stroke.color;
                                (sel.gamma_multiply(0.7), sel)
                            } else {
                                (text_color, accent)
                            };
                            if entry.label_matches.is_empty() {
                                ui.painter().text(
                                    label_pos,
                                    egui::Align2::LEFT_TOP,
                                    &cmd.label,
                                    egui::FontId::proportional(13.0),
                                    label_normal,
                                );
                            } else {
                                super::paint_highlighted_text(
                                    ui,
                                    label_pos,
                                    &cmd.label,
                                    &entry.label_matches,
                                    egui::FontId::proportional(13.0),
                                    label_normal,
                                    label_highlight,
                                );
                            }

                            // Shortcut (right-aligned)
                            if let Some(shortcut) = &cmd.shortcut
                                && !shortcut.is_empty()
                            {
                                ui.painter().text(
                                    rect.max - egui::vec2(6.0, row_height - 10.0),
                                    egui::Align2::RIGHT_TOP,
                                    shortcut,
                                    egui::FontId::monospace(11.0),
                                    weak_color,
                                );
                            }

                            // Description
                            let desc_color = if selected {
                                ui.visuals().selection.stroke.color.gamma_multiply(0.6)
                            } else {
                                weak_color
                            };
                            ui.painter().text(
                                rect.min + egui::vec2(label_x + text_offset, 20.0),
                                egui::Align2::LEFT_TOP,
                                &cmd.desc,
                                egui::FontId::proportional(11.0),
                                desc_color,
                            );

                            if selected && selection_changed {
                                resp.scroll_to_me(None);
                            }
                            if resp.clicked() {
                                action = PaletteAction::Execute(cmd.id);
                                self.open = false;
                                return;
                            }
                        }
                        if self.filtered.is_empty() {
                            ui.weak(t!("cmd.no_match"));
                        }
                    });
            });

        action
    }

    fn refilter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.commands.len())
                .map(|i| FilteredEntry {
                    cmd_idx: i,
                    label_matches: Vec::new(),
                })
                .collect();
            return;
        }

        let mut scored: Vec<(usize, i32, Vec<usize>)> = self
            .commands
            .iter()
            .enumerate()
            .filter_map(|(i, cmd)| {
                let haystack = format!("{} {} {}", cmd.category, cmd.label, cmd.desc);
                super::fuzzy_score(&self.query, &haystack).map(|(score, _full_indices)| {
                    // Get match indices within the label specifically for highlighting
                    let label_matches = super::fuzzy_score(&self.query, &cmd.label)
                        .map(|(_, indices)| indices)
                        .unwrap_or_default();
                    (i, score, label_matches)
                })
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        self.filtered = scored
            .into_iter()
            .map(|(i, _, label_matches)| FilteredEntry {
                cmd_idx: i,
                label_matches,
            })
            .collect();
    }
}

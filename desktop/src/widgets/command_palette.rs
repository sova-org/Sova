use eframe::egui;

use crate::icons;
use crate::widgets::shortcut::{self, Key, Shortcut};

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
    ToggleViewMode,
}

pub enum PaletteAction {
    None,
    Execute(CommandId),
}

struct Command {
    id: CommandId,
    icon: Option<&'static str>,
    label: String,
    category: String,
    desc: String,
    shortcut: Option<Shortcut>,
    active: bool,
}

fn commands() -> Vec<Command> {
    let panel = |id, icon: &'static str, label_key: &str, desc_key: &str, sc: Shortcut| Command {
        id,
        icon: Some(icon),
        label: t!(label_key).into(),
        category: t!("cmd.category.panel").into(),
        desc: t!(desc_key).into(),
        shortcut: Some(sc),
        active: false,
    };

    vec![
        // Panels
        panel(
            CommandId::Options,
            icons::GEAR,
            "cmd.options",
            "cmd.options.desc",
            Shortcut::cmd(Key::Char(',')),
        ),
        panel(
            CommandId::Server,
            icons::CONNECT,
            "cmd.server",
            "cmd.server.desc",
            Shortcut::cmd_shift(Key::Char('S')),
        ),
        panel(
            CommandId::Audio,
            icons::UNMUTE,
            "cmd.audio",
            "cmd.audio.desc",
            Shortcut::cmd_shift(Key::Char('A')),
        ),
        panel(
            CommandId::Devices,
            icons::PLUGS_CONNECTED,
            "cmd.devices",
            "cmd.devices.desc",
            Shortcut::cmd_shift(Key::Char('I')),
        ),
        panel(
            CommandId::Scope,
            icons::WAVE_SINE,
            "cmd.scope",
            "cmd.scope.desc",
            Shortcut::cmd_shift(Key::Char('O')),
        ),
        panel(
            CommandId::Spectrum,
            icons::WAVE_SINE,
            "cmd.spectrum",
            "cmd.spectrum.desc",
            Shortcut::cmd_shift(Key::Char('P')),
        ),
        panel(
            CommandId::VuMeter,
            icons::WAVE_SINE,
            "cmd.vu_meter",
            "cmd.vu_meter.desc",
            Shortcut::cmd_shift(Key::Char('U')),
        ),
        panel(
            CommandId::ScopeBar,
            icons::WAVE_SINE,
            "cmd.scope_bar",
            "cmd.scope_bar.desc",
            Shortcut::cmd_shift(Key::Char('W')),
        ),
        panel(
            CommandId::Chat,
            icons::SEND,
            "cmd.chat",
            "cmd.chat.desc",
            Shortcut::cmd_shift(Key::Char('C')),
        ),
        panel(
            CommandId::Logs,
            icons::FILE_TEXT,
            "cmd.logs",
            "cmd.logs.desc",
            Shortcut::cmd_shift(Key::Char('L')),
        ),
        panel(
            CommandId::SampleBrowser,
            icons::MUSIC_NOTE,
            "cmd.sample_browser",
            "cmd.sample_browser.desc",
            Shortcut::cmd_shift(Key::Char('E')),
        ),
        panel(
            CommandId::Documentation,
            icons::BOOK,
            "cmd.documentation",
            "cmd.documentation.desc",
            Shortcut::cmd_shift(Key::Char('H')),
        ),
        panel(
            CommandId::Debug,
            icons::CPU,
            "cmd.debug",
            "cmd.debug.desc",
            Shortcut::cmd_shift(Key::Char('B')),
        ),
        Command {
            id: CommandId::Keybindings,
            icon: Some(icons::KEYBOARD),
            label: t!("cmd.keybindings").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.keybindings.desc").into(),
            shortcut: None,
            active: false,
        },
        Command {
            id: CommandId::About,
            icon: Some(icons::CIRCLE_FILLED),
            label: t!("cmd.about").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.about.desc").into(),
            shortcut: None,
            active: false,
        },
        // Engine
        Command {
            id: CommandId::RestartCore,
            icon: Some(icons::REFRESH),
            label: t!("cmd.restart_core").into(),
            category: t!("cmd.category.engine").into(),
            desc: t!("cmd.restart_core.desc").into(),
            shortcut: None,
            active: false,
        },
        // Transport
        Command {
            id: CommandId::PlayPause,
            icon: Some(icons::PLAY),
            label: t!("cmd.play_pause").into(),
            category: t!("cmd.category.transport").into(),
            desc: t!("cmd.play_pause.desc").into(),
            shortcut: Some(Shortcut::cmd_shift(Key::Space)),
            active: false,
        },
        // File
        Command {
            id: CommandId::SaveScene,
            icon: Some(icons::FILE_TEXT),
            label: t!("cmd.save_scene").into(),
            category: t!("cmd.category.file").into(),
            desc: t!("cmd.save_scene.desc").into(),
            shortcut: Some(Shortcut::cmd(Key::Char('S'))),
            active: false,
        },
        Command {
            id: CommandId::LoadScene,
            icon: Some(icons::FILE_TEXT),
            label: t!("cmd.load_scene").into(),
            category: t!("cmd.category.file").into(),
            desc: t!("cmd.load_scene.desc").into(),
            shortcut: Some(Shortcut::cmd(Key::Char('O'))),
            active: false,
        },
        Command {
            id: CommandId::ResetScene,
            icon: Some(icons::TRASH),
            label: t!("cmd.reset_scene").into(),
            category: t!("cmd.category.file").into(),
            desc: t!("cmd.reset_scene.desc").into(),
            shortcut: None,
            active: false,
        },
        // View
        Command {
            id: CommandId::ZoomIn,
            icon: Some(icons::FOCUS),
            label: t!("cmd.zoom_in").into(),
            category: t!("cmd.category.view").into(),
            desc: t!("cmd.zoom_in.desc").into(),
            shortcut: Some(Shortcut::cmd(Key::Char('='))),
            active: false,
        },
        Command {
            id: CommandId::ZoomOut,
            icon: Some(icons::UNFOCUS),
            label: t!("cmd.zoom_out").into(),
            category: t!("cmd.category.view").into(),
            desc: t!("cmd.zoom_out.desc").into(),
            shortcut: Some(Shortcut::cmd(Key::Char('-'))),
            active: false,
        },
        Command {
            id: CommandId::ZoomReset,
            icon: Some(icons::FOCUS),
            label: t!("cmd.zoom_reset").into(),
            category: t!("cmd.category.view").into(),
            desc: t!("cmd.zoom_reset.desc").into(),
            shortcut: Some(Shortcut::cmd(Key::Char('0'))),
            active: false,
        },
        Command {
            id: CommandId::ToggleViewMode,
            icon: Some(icons::SWAP),
            label: t!("cmd.toggle_view_mode").into(),
            category: t!("cmd.category.view").into(),
            desc: t!("cmd.toggle_view_mode.desc").into(),
            shortcut: Some(Shortcut::plain(Key::Char('V'))),
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

    /// Iterator over `(CommandId, Shortcut)` pairs for every command that has
    /// a key binding. Used by the global shortcut handler so the keyboard map
    /// and the palette stay in sync from a single source.
    pub fn shortcut_table(&self) -> impl Iterator<Item = (CommandId, Shortcut)> + '_ {
        self.commands
            .iter()
            .filter_map(|c| c.shortcut.map(|s| (c.id, s)))
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

                            // Icon
                            let icon_offset = if let Some(icon) = cmd.icon {
                                let icon_pos = rect.min + egui::vec2(label_x + text_offset, 2.0);
                                let icon_color = if selected {
                                    ui.visuals().selection.stroke.color
                                } else {
                                    text_color
                                };
                                ui.painter().text(
                                    icon_pos,
                                    egui::Align2::LEFT_TOP,
                                    icon,
                                    egui::FontId::new(13.0, icons::family()),
                                    icon_color,
                                );
                                18.0
                            } else {
                                0.0
                            };

                            // Label with fuzzy match highlighting
                            let label_pos =
                                rect.min + egui::vec2(label_x + text_offset + icon_offset, 2.0);
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
                            if let Some(sc) = &cmd.shortcut {
                                ui.painter().text(
                                    rect.max - egui::vec2(6.0, row_height - 10.0),
                                    egui::Align2::RIGHT_TOP,
                                    shortcut::format(ui.ctx(), sc),
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
                                rect.min + egui::vec2(label_x + text_offset + icon_offset, 20.0),
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

        let needle = self.query.to_lowercase();
        let mut scored: Vec<(usize, i32, Vec<usize>)> = self
            .commands
            .iter()
            .enumerate()
            .filter_map(|(i, cmd)| {
                let haystack = format!("{} {} {}", cmd.category, cmd.label, cmd.desc);
                let hay_lower = haystack.to_lowercase();
                let pos = hay_lower.find(&needle)?;
                // Prefer matches in label over category/desc
                let label_lower = cmd.label.to_lowercase();
                let label_pos = label_lower.find(&needle);
                let score = if label_pos == Some(0) {
                    30 // prefix match in label
                } else if label_pos.is_some() {
                    20 // substring match in label
                } else if pos == 0 {
                    15 // prefix match in category+label+desc
                } else {
                    10 // substring match somewhere
                };
                let label_matches = label_pos
                    .map(|start| (start..start + needle.len()).collect())
                    .unwrap_or_default();
                Some((i, score, label_matches))
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

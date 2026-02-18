use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandId {
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
}

fn commands() -> Vec<Command> {
    vec![
        Command {
            id: CommandId::Server,
            label: t!("cmd.server").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.server.desc").into(),
        },
        Command {
            id: CommandId::Audio,
            label: t!("cmd.audio").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.audio.desc").into(),
        },
        Command {
            id: CommandId::Devices,
            label: t!("cmd.devices").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.devices.desc").into(),
        },
        Command {
            id: CommandId::Scope,
            label: t!("cmd.scope").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.scope.desc").into(),
        },
        Command {
            id: CommandId::Spectrum,
            label: t!("cmd.spectrum").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.spectrum.desc").into(),
        },
        Command {
            id: CommandId::VuMeter,
            label: t!("cmd.vu_meter").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.vu_meter.desc").into(),
        },
        Command {
            id: CommandId::ScopeBar,
            label: t!("cmd.scope_bar").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.scope_bar.desc").into(),
        },
        Command {
            id: CommandId::Chat,
            label: t!("cmd.chat").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.chat.desc").into(),
        },
        Command {
            id: CommandId::Logs,
            label: t!("cmd.logs").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.logs.desc").into(),
        },
        Command {
            id: CommandId::Options,
            label: t!("cmd.options").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.options.desc").into(),
        },
        Command {
            id: CommandId::Debug,
            label: t!("cmd.debug").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.debug.desc").into(),
        },
        Command {
            id: CommandId::Keybindings,
            label: t!("cmd.keybindings").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.keybindings.desc").into(),
        },
        Command {
            id: CommandId::About,
            label: t!("cmd.about").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.about.desc").into(),
        },
        Command {
            id: CommandId::SampleBrowser,
            label: t!("cmd.sample_browser").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.sample_browser.desc").into(),
        },
        Command {
            id: CommandId::Documentation,
            label: t!("cmd.documentation").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.documentation.desc").into(),
        },
        Command {
            id: CommandId::Visuals,
            label: t!("cmd.visuals").into(),
            category: t!("cmd.category.panel").into(),
            desc: t!("cmd.visuals.desc").into(),
        },
    ]
}

pub struct CommandPalette {
    open: bool,
    query: String,
    selected: usize,
    filtered: Vec<usize>,
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
            filtered: (0..count).collect(),
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
        self.filtered = (0..self.commands.len()).collect();
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
                ui.painter()
                    .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(120));
                if resp.clicked() {
                    self.open = false;
                }
            });

        egui::Window::new("Command Palette")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 80.0])
            .fixed_size([500.0, 0.0])
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

                if arrow_up && self.selected > 0 {
                    self.selected -= 1;
                }
                if arrow_down && self.selected + 1 < self.filtered.len() {
                    self.selected += 1;
                }

                if enter && !self.filtered.is_empty() {
                    let idx = self.filtered[self.selected];
                    action = PaletteAction::Execute(self.commands[idx].id);
                    self.open = false;
                    return;
                }

                ui.separator();
                let available_width = ui.available_width();
                egui::ScrollArea::vertical()
                    .max_height(350.0)
                    .show(ui, |ui| {
                        ui.set_min_width(available_width);
                        let row_height = 36.0;
                        for (i, &cmd_idx) in self.filtered.iter().enumerate() {
                            let cmd = &self.commands[cmd_idx];
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

                            let label = format!("{}: {}", cmd.category, cmd.label);
                            ui.painter().text(
                                rect.min + egui::vec2(6.0, 2.0),
                                egui::Align2::LEFT_TOP,
                                &label,
                                egui::FontId::proportional(13.0),
                                ui.visuals().text_color(),
                            );
                            ui.painter().text(
                                rect.min + egui::vec2(6.0, 18.0),
                                egui::Align2::LEFT_TOP,
                                &cmd.desc,
                                egui::FontId::proportional(11.0),
                                ui.visuals().weak_text_color(),
                            );

                            if selected {
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
            self.filtered = (0..self.commands.len()).collect();
            return;
        }

        let mut scored: Vec<(usize, i32)> = self
            .commands
            .iter()
            .enumerate()
            .filter_map(|(i, cmd)| {
                let haystack = format!("{} {} {}", cmd.category, cmd.label, cmd.desc);
                super::fuzzy_score(&self.query, &haystack).map(|score| (i, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        self.filtered = scored.into_iter().map(|(i, _)| i).collect();
    }
}

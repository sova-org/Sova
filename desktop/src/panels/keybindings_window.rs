use eframe::egui;

use crate::{scene_panel, widgets};

pub(crate) fn show_keybindings_window(
    ctx: &egui::Context,
    open: &mut bool,
    view_mode: scene_panel::ViewMode,
) {
    use widgets::shortcut::{self, Key, Shortcut};

    let screen = ctx.content_rect();
    let max_h = screen.height() * 0.8;
    let wide = screen.width() > 700.0;

    egui::Window::new(t!("kb.title"))
        .open(open)
        .resizable(true)
        .collapsible(false)
        .default_width(if wide { 640.0 } else { 340.0 })
        .max_height(max_h)
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(screen.center())
        .vscroll(true)
        .show(ctx, |ui| {
            let is_sequencer = view_mode == scene_panel::ViewMode::Sequencer;

            let left = |ui: &mut egui::Ui| {
                ui.heading(t!("kb.general"));
                egui::Grid::new("kb_general")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        shortcut::grid_row(
                            ui,
                            t!("kb.command_palette"),
                            &Shortcut::cmd(Key::Char('K')),
                        );
                    });

                ui.add_space(8.0);
                ui.heading(t!("kb.file"));
                egui::Grid::new("kb_file")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        shortcut::grid_row(ui, t!("kb.save_scene"), &Shortcut::cmd(Key::Char('S')));
                        shortcut::grid_row(ui, t!("kb.load_scene"), &Shortcut::cmd(Key::Char('O')));
                    });

                ui.add_space(8.0);
                ui.heading(t!("kb.transport"));
                egui::Grid::new("kb_transport")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        shortcut::grid_row(
                            ui,
                            t!("kb.play_pause"),
                            &Shortcut::cmd_shift(Key::Space),
                        );
                    });

                ui.add_space(8.0);
                ui.heading(t!("kb.panels"));
                egui::Grid::new("kb_panels")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        shortcut::grid_row(ui, t!("options.title"), &Shortcut::cmd(Key::Char(',')));
                        shortcut::grid_row(
                            ui,
                            t!("server.title"),
                            &Shortcut::cmd_shift(Key::Char('S')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("audio.title"),
                            &Shortcut::cmd_shift(Key::Char('A')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("devices.title"),
                            &Shortcut::cmd_shift(Key::Char('I')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("scope.title"),
                            &Shortcut::cmd_shift(Key::Char('O')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("spectrum.title"),
                            &Shortcut::cmd_shift(Key::Char('P')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("cmd.vu_meter"),
                            &Shortcut::cmd_shift(Key::Char('U')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("cmd.scope_bar"),
                            &Shortcut::cmd_shift(Key::Char('W')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("chat.title"),
                            &Shortcut::cmd_shift(Key::Char('C')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("cmd.logs"),
                            &Shortcut::cmd_shift(Key::Char('L')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("sample_browser.title"),
                            &Shortcut::cmd_shift(Key::Char('E')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("doc.title"),
                            &Shortcut::cmd_shift(Key::Char('H')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("visuals.title"),
                            &Shortcut::cmd_shift(Key::Char('V')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("debug.title"),
                            &Shortcut::cmd_shift(Key::Char('B')),
                        );
                        shortcut::grid_row(ui, t!("kb.title"), &Shortcut::plain(Key::F(1)));
                    });
            };

            let right = |ui: &mut egui::Ui| {
                ui.heading(t!("kb.scene_nav"));

                // Mode indicator
                let mode_label = if is_sequencer {
                    t!("kb.mode_sequencer")
                } else {
                    t!("kb.mode_classic")
                };
                ui.weak(mode_label);
                ui.add_space(2.0);

                egui::Grid::new("kb_scene_nav")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        shortcut::grid_row(ui, t!("kb.navigate"), &Shortcut::literal("Arrow keys"));
                        shortcut::grid_row(
                            ui,
                            t!("kb.navigate_vim"),
                            &Shortcut::literal("h / j / k / l"),
                        );
                        if is_sequencer {
                            shortcut::grid_row(
                                ui,
                                t!("kb.nav_lines"),
                                &Shortcut::literal("↑ / ↓ / k / j"),
                            );
                            shortcut::grid_row(
                                ui,
                                t!("kb.nav_frames"),
                                &Shortcut::literal("← / → / h / l"),
                            );
                        } else {
                            shortcut::grid_row(
                                ui,
                                t!("kb.nav_frames"),
                                &Shortcut::literal("↑ / ↓ / k / j"),
                            );
                            shortcut::grid_row(
                                ui,
                                t!("kb.nav_lines"),
                                &Shortcut::literal("← / → / h / l"),
                            );
                        }
                        let enter_edit_sc = if is_sequencer {
                            Shortcut::literal("Enter")
                        } else {
                            Shortcut::literal("Enter / i")
                        };
                        shortcut::grid_row(ui, t!("kb.enter_edit"), &enter_edit_sc);
                        shortcut::grid_row(ui, t!("kb.exit_edit"), &Shortcut::plain(Key::Esc));
                        shortcut::grid_row(
                            ui,
                            t!("kb.focus_mode"),
                            &Shortcut::plain(Key::Char('F')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.extend_selection"),
                            &Shortcut::literal("Shift+Arrow"),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.duplicate_after"),
                            &Shortcut::cmd(Key::Char('D')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.duplicate_line"),
                            &Shortcut::cmd_shift(Key::Char('D')),
                        );
                        let insert_after_sc = if is_sequencer {
                            Shortcut::plain(Key::Char('I'))
                        } else {
                            Shortcut::shift(Key::Char('I'))
                        };
                        shortcut::grid_row(ui, t!("kb.insert_after"), &insert_after_sc);
                        shortcut::grid_row(
                            ui,
                            t!("kb.insert_before"),
                            &Shortcut::cmd_shift(Key::Char('I')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.delete_frame"),
                            &Shortcut::plain(Key::Delete),
                        );
                        shortcut::grid_row(ui, t!("kb.delete_line"), &Shortcut::cmd(Key::Delete));
                        shortcut::grid_row(
                            ui,
                            t!("kb.move_frame_down"),
                            &Shortcut::shift(Key::Char('J')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.move_frame_up"),
                            &Shortcut::shift(Key::Char('K')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.move_line_left"),
                            &Shortcut::alt(Key::Char('H')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.move_line_right"),
                            &Shortcut::alt(Key::Char('L')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.add_line_below"),
                            &Shortcut::plain(Key::Char('O')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.add_line_above"),
                            &Shortcut::shift(Key::Char('O')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.toggle_enabled"),
                            &Shortcut::plain(Key::Char('E')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.toggle_line_enabled"),
                            &Shortcut::shift(Key::Char('E')),
                        );
                        if is_sequencer {
                            shortcut::grid_row(
                                ui,
                                t!("kb.edit_line_speed"),
                                &Shortcut::shift(Key::Char('S')),
                            );
                        }
                        if is_sequencer {
                            shortcut::grid_row(
                                ui,
                                t!("kb.edit_duration_inline"),
                                &Shortcut::plain(Key::Char('T')),
                            );
                            shortcut::grid_row(
                                ui,
                                t!("kb.edit_repetitions_inline"),
                                &Shortcut::plain(Key::Char('Y')),
                            );
                            shortcut::grid_row(
                                ui,
                                t!("kb.confirm_inline_value"),
                                &Shortcut::plain(Key::Enter),
                            );
                        }
                        shortcut::grid_row(ui, t!("kb.toggle_looping"), &Shortcut::literal("r"));
                        shortcut::grid_row(
                            ui,
                            t!("kb.toggle_trailing"),
                            &Shortcut::plain(Key::Char(',')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.toggle_manual"),
                            &Shortcut::plain(Key::Char('M')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.preview_frame"),
                            &Shortcut::plain(Key::Char('P')),
                        );
                        shortcut::grid_row(
                            ui,
                            t!("kb.clear_frame"),
                            &Shortcut::plain(Key::Char('X')),
                        );
                        shortcut::grid_row(ui, t!("kb.select_all"), &Shortcut::cmd(Key::Char('A')));
                        shortcut::grid_row(ui, t!("kb.copy"), &Shortcut::cmd(Key::Char('C')));
                        shortcut::grid_row(ui, t!("kb.cut"), &Shortcut::cmd(Key::Char('X')));
                        shortcut::grid_row(ui, t!("kb.paste"), &Shortcut::cmd(Key::Char('V')));
                    });

                ui.add_space(8.0);
                ui.heading(t!("kb.scene_edit"));
                egui::Grid::new("kb_scene_edit")
                    .num_columns(2)
                    .min_col_width(150.0)
                    .striped(true)
                    .show(ui, |ui| {
                        shortcut::grid_row(ui, t!("kb.exit_edit"), &Shortcut::plain(Key::Esc));
                        shortcut::grid_row(ui, t!("kb.evaluate"), &Shortcut::cmd(Key::Enter));
                        shortcut::grid_row(
                            ui,
                            t!("kb.lang_selector"),
                            &Shortcut::cmd(Key::Char('L')),
                        );
                        shortcut::grid_row(ui, t!("kb.search"), &Shortcut::cmd(Key::Char('F')));
                    });
            };

            if wide {
                ui.columns(2, |cols| {
                    left(&mut cols[0]);
                    right(&mut cols[1]);
                });
            } else {
                left(ui);
                right(ui);
            }
        });
}

pub(crate) fn show_debug_window(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new(t!("debug.title"))
        .open(open)
        .resizable(true)
        .collapsible(true)
        .default_width(320.0)
        .vscroll(true)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new(t!("debug.settings")).show(ui, |ui| ctx.settings_ui(ui));
            egui::CollapsingHeader::new(t!("debug.inspection"))
                .show(ui, |ui| ctx.inspection_ui(ui));
            egui::CollapsingHeader::new(t!("debug.textures")).show(ui, |ui| ctx.texture_ui(ui));
            egui::CollapsingHeader::new(t!("debug.memory")).show(ui, |ui| ctx.memory_ui(ui));
        });
}

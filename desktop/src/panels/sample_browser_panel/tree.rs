use eframe::egui;

use crate::sample_browser::{SampleBrowserState, TreeLineKind};

pub(super) struct TreeInteraction {
    pub new_cursor: Option<usize>,
    pub should_toggle: bool,
    pub clicked_file: Option<(String, usize)>,
}

pub(super) fn render_tree(
    ui: &mut egui::Ui,
    state: &SampleBrowserState,
    row_height: f32,
    visible_rows: usize,
    cursor_changed: bool,
) -> TreeInteraction {
    let mut interaction = TreeInteraction {
        new_cursor: None,
        should_toggle: false,
        clicked_file: None,
    };

    egui::ScrollArea::vertical()
        .max_height(visible_rows as f32 * row_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let entries = state.entries();
            if entries.is_empty() {
                ui.colored_label(egui::Color32::GRAY, t!("sample_browser.no_entries"));
                return;
            }

            for (i, entry) in entries.iter().enumerate() {
                let selected = i == state.cursor;
                let indent = entry.depth as f32 * 16.0;
                let is_file = matches!(entry.kind, TreeLineKind::File);

                let (rect, resp) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), row_height),
                    egui::Sense::click(),
                );

                if selected {
                    ui.painter()
                        .rect_filled(rect, 0.0, ui.visuals().selection.bg_fill);
                } else if resp.hovered() {
                    ui.painter()
                        .rect_filled(rect, 0.0, ui.visuals().widgets.hovered.bg_fill);
                }

                let icon_x = rect.left() + 4.0 + indent;
                let text_x = icon_x + 16.0;

                let icon_font = egui::FontId::new(12.0, crate::icons::family());
                match &entry.kind {
                    TreeLineKind::Root { expanded } | TreeLineKind::Folder { expanded } => {
                        let icon = if *expanded {
                            crate::icons::CHEVRON_DOWN
                        } else {
                            crate::icons::CHEVRON_RIGHT
                        };
                        ui.painter().text(
                            egui::pos2(icon_x, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            icon,
                            icon_font,
                            ui.visuals().strong_text_color(),
                        );
                    }
                    TreeLineKind::File => {
                        let color = if selected || resp.hovered() {
                            ui.visuals().selection.bg_fill
                        } else {
                            ui.visuals().weak_text_color()
                        };
                        ui.painter().text(
                            egui::pos2(icon_x, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            crate::icons::PLAY,
                            icon_font,
                            color,
                        );
                    }
                }

                let color = if selected {
                    ui.visuals().selection.stroke.color
                } else if entry.is_default && matches!(entry.kind, TreeLineKind::Root { .. }) {
                    ui.visuals().weak_text_color()
                } else if is_file {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().strong_text_color()
                };

                ui.painter().text(
                    egui::pos2(text_x, rect.min.y + 1.0),
                    egui::Align2::LEFT_TOP,
                    &entry.label,
                    egui::FontId::monospace(12.0),
                    color,
                );

                if resp.clicked() {
                    interaction.new_cursor = Some(i);
                    if !is_file {
                        interaction.should_toggle = true;
                    } else {
                        interaction.clicked_file = Some((entry.folder.clone(), entry.index));
                    }
                }

                if selected && cursor_changed {
                    resp.scroll_to_me(None);
                }
            }
        });

    interaction
}

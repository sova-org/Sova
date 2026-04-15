use eframe::egui;

use crate::{
    theme::COLOR_MUTED,
    widgets::shortcut::{self, Key, Shortcut},
};

const BADGE_FONT_SIZE: f32 = 10.5;
const BADGE_PADDING: egui::Vec2 = egui::vec2(4.0, 1.5);
const TITLE_COLUMN_WIDTH: f32 = 86.0;
const TITLE_COLUMN_GAP: f32 = 14.0;
const SECTION_GAP: f32 = 6.0;
const ENTRY_COLUMN_GAP: f32 = 12.0;
const ENTRY_ROW_GAP: f32 = 4.0;
const ENTRY_INNER_GAP: f32 = 6.0;
const ENTRY_MIN_HEIGHT: f32 = 18.0;
const MAX_ENTRY_COLUMNS: usize = 4;

#[derive(Clone, Copy)]
struct HudItem {
    label_key: &'static str,
    shortcut: Shortcut,
}

#[derive(Clone, Copy)]
struct HudSection {
    title_key: &'static str,
    items: &'static [HudItem],
}

const FRAME_ITEMS: &[HudItem] = &[
    HudItem {
        label_key: "scene.hud.insert_before",
        shortcut: Shortcut::literal("b"),
    },
    HudItem {
        label_key: "scene.hud.insert_after",
        shortcut: Shortcut::literal("i"),
    },
    HudItem {
        label_key: "scene.hud.duplicate",
        shortcut: Shortcut::literal("d"),
    },
    HudItem {
        label_key: "scene.hud.toggle_enabled",
        shortcut: Shortcut::literal("e"),
    },
    HudItem {
        label_key: "scene.hud.edit_duration",
        shortcut: Shortcut::literal("t"),
    },
    HudItem {
        label_key: "scene.hud.edit_repetitions",
        shortcut: Shortcut::literal("y"),
    },
    HudItem {
        label_key: "scene.hud.preview",
        shortcut: Shortcut::literal("p"),
    },
    HudItem {
        label_key: "scene.hud.clear",
        shortcut: Shortcut::literal("c"),
    },
    HudItem {
        label_key: "scene.hud.delete",
        shortcut: Shortcut::plain(Key::Delete),
    },
];

const LINE_ITEMS: &[HudItem] = &[
    HudItem {
        label_key: "scene.hud.add_above",
        shortcut: Shortcut::literal("a"),
    },
    HudItem {
        label_key: "scene.hud.add_below",
        shortcut: Shortcut::literal("o"),
    },
    HudItem {
        label_key: "scene.hud.duplicate_line",
        shortcut: Shortcut::literal("s"),
    },
    HudItem {
        label_key: "scene.hud.toggle_line_enabled",
        shortcut: Shortcut::shift(Key::Char('E')),
    },
    HudItem {
        label_key: "scene.hud.edit_speed",
        shortcut: Shortcut::shift(Key::Char('S')),
    },
    HudItem {
        label_key: "scene.hud.toggle_looping",
        shortcut: Shortcut::literal("r"),
    },
    HudItem {
        label_key: "scene.hud.toggle_trailing",
        shortcut: Shortcut::literal(","),
    },
    HudItem {
        label_key: "scene.hud.toggle_manual",
        shortcut: Shortcut::literal("m"),
    },
    HudItem {
        label_key: "scene.hud.delete_line",
        shortcut: Shortcut::literal("z"),
    },
];

const MOVE_ITEMS: &[HudItem] = &[
    HudItem {
        label_key: "scene.hud.move_left",
        shortcut: Shortcut::literal("h"),
    },
    HudItem {
        label_key: "scene.hud.move_forward",
        shortcut: Shortcut::literal("j"),
    },
    HudItem {
        label_key: "scene.hud.move_backward",
        shortcut: Shortcut::literal("k"),
    },
    HudItem {
        label_key: "scene.hud.move_right",
        shortcut: Shortcut::literal("l"),
    },
];

const HUD_SECTIONS: &[HudSection] = &[
    HudSection {
        title_key: "scene.hud.frame",
        items: FRAME_ITEMS,
    },
    HudSection {
        title_key: "scene.hud.line",
        items: LINE_ITEMS,
    },
    HudSection {
        title_key: "scene.hud.move",
        items: MOVE_ITEMS,
    },
];

fn shortcut_text(ctx: &egui::Context, shortcut: &Shortcut) -> String {
    if ctx.os() == egui::os::OperatingSystem::Mac {
        shortcut::format_plain_text(shortcut)
    } else {
        shortcut::format(ctx, shortcut)
    }
}

fn badge_font(ui: &egui::Ui) -> egui::FontId {
    let mut font = egui::TextStyle::Small.resolve(ui.style());
    font.size = BADGE_FONT_SIZE;
    font
}

fn badge_size(ui: &egui::Ui, text: &str, fg: egui::Color32) -> egui::Vec2 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), badge_font(ui), fg)
        .size()
        + BADGE_PADDING * 2.0
}

fn min_entry_width(ui: &egui::Ui) -> f32 {
    let fg = ui.visuals().text_color();
    let label_font = egui::TextStyle::Small.resolve(ui.style());
    HUD_SECTIONS
        .iter()
        .flat_map(|section| section.items.iter().copied())
        .map(|item| {
            let shortcut_text = shortcut_text(ui.ctx(), &item.shortcut);
            let label_text = t!(item.label_key).to_string();
            let label_width = ui
                .painter()
                .layout_no_wrap(label_text, label_font.clone(), fg)
                .size()
                .x;
            badge_size(ui, &shortcut_text, fg).x + ENTRY_INNER_GAP + label_width
        })
        .fold(0.0, f32::max)
}

fn entry_columns(content_width: f32, min_entry_width: f32) -> usize {
    let max_columns = MAX_ENTRY_COLUMNS.min(
        HUD_SECTIONS
            .iter()
            .map(|section| section.items.len())
            .max()
            .unwrap_or(1),
    );
    (1..=max_columns)
        .rev()
        .find(|&columns| {
            let gaps = ENTRY_COLUMN_GAP * columns.saturating_sub(1) as f32;
            columns as f32 * min_entry_width + gaps <= content_width
        })
        .unwrap_or(1)
}

fn column_width(content_width: f32, entry_columns: usize) -> f32 {
    let gaps = ENTRY_COLUMN_GAP * entry_columns.saturating_sub(1) as f32;
    ((content_width - gaps) / entry_columns as f32).max(0.0)
}

fn badge(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let bg = ui.visuals().faint_bg_color;
    let fg = ui.visuals().text_color();
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), badge_font(ui), fg);
    let (rect, resp) =
        ui.allocate_exact_size(galley.size() + BADGE_PADDING * 2.0, egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, bg);
    ui.painter().galley(rect.min + BADGE_PADDING, galley, fg);
    resp
}

fn entry(ui: &mut egui::Ui, entry_width: f32, item: HudItem) {
    let weak = ui.visuals().weak_text_color();
    ui.allocate_ui_with_layout(
        egui::vec2(entry_width, ENTRY_MIN_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = ENTRY_INNER_GAP;
            ui.set_min_height(ENTRY_MIN_HEIGHT);
            let shortcut_text = shortcut_text(ui.ctx(), &item.shortcut);
            badge(ui, &shortcut_text);
            ui.add(
                egui::Label::new(egui::RichText::new(t!(item.label_key)).small().color(weak))
                    .truncate(),
            );
        },
    );
}

fn section(
    ui: &mut egui::Ui,
    content_width: f32,
    column_width: f32,
    entry_columns: usize,
    section: HudSection,
) {
    ui.horizontal_top(|ui| {
        ui.add_sized(
            egui::vec2(TITLE_COLUMN_WIDTH, ENTRY_MIN_HEIGHT),
            egui::Label::new(
                egui::RichText::new(t!(section.title_key))
                    .small()
                    .strong()
                    .monospace()
                    .color(COLOR_MUTED),
            ),
        );
        ui.add_space(TITLE_COLUMN_GAP);

        ui.allocate_ui(egui::vec2(content_width, 0.0), |ui| {
            ui.set_min_width(content_width);
            egui::Grid::new(ui.id().with(section.title_key))
                .num_columns(entry_columns)
                .min_col_width(column_width)
                .min_row_height(ENTRY_MIN_HEIGHT)
                .spacing(egui::vec2(ENTRY_COLUMN_GAP, ENTRY_ROW_GAP))
                .show(ui, |ui| {
                    for chunk in section.items.chunks(entry_columns) {
                        for index in 0..entry_columns {
                            if let Some(item) = chunk.get(index) {
                                entry(ui, column_width, *item);
                            } else {
                                ui.allocate_space(egui::vec2(column_width, ENTRY_MIN_HEIGHT));
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    });
}

impl super::ScenePanel {
    pub(super) fn show_hud_bar(&self, ui: &mut egui::Ui) {
        ui.add_space(6.0);

        let min_entry_width = min_entry_width(ui).ceil();
        let content_width =
            (ui.available_width() - TITLE_COLUMN_WIDTH - TITLE_COLUMN_GAP).max(min_entry_width);
        let entry_columns = entry_columns(content_width, min_entry_width);
        let column_width = column_width(content_width, entry_columns).floor();

        for (index, section_desc) in HUD_SECTIONS.iter().enumerate() {
            if index > 0 {
                ui.add_space(SECTION_GAP);
            }
            section(
                ui,
                content_width,
                column_width,
                entry_columns,
                *section_desc,
            );
        }

        ui.add_space(6.0);
    }
}

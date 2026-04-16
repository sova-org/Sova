use eframe::egui;

use super::{MenuAction, MenuItemDef, MENU_LABEL_RIGHT_GAP, MENU_MAX_HEIGHT_FRACTION, MENU_MAX_WIDTH, MENU_MIN_WIDTH};

pub(super) enum MenuPanelClick {
    Action(MenuAction),
    OpenSubmenu(usize),
}

pub(super) struct MenuPanelResult {
    pub rect: egui::Rect,
    pub item_rects: Vec<egui::Rect>,
    pub hovered_idx: Option<usize>,
    pub clicked: Option<MenuPanelClick>,
}

fn show_item_row(
    ui: &mut egui::Ui,
    item: &MenuItemDef,
    row_width: f32,
    keyboard_selected: bool,
    allow_mouse_hover: bool,
    submenu_open: bool,
    show_mnemonic: bool,
) -> egui::Response {
    match item {
        MenuItemDef::Separator => {
            let separator_height = ui.spacing().interact_size.y * 0.25;
            let (rect, resp) = ui.allocate_exact_size(
                egui::vec2(row_width, separator_height),
                egui::Sense::hover(),
            );
            if ui.is_rect_visible(rect) {
                let y = rect.center().y;
                ui.painter()
                    .hline(rect.x_range(), y, ui.visuals().noninteractive().bg_stroke);
            }
            resp
        }
        MenuItemDef::Button {
            label,
            mnemonic,
            icon,
            enabled,
            ..
        } => text_row(
            ui,
            label,
            *mnemonic,
            *icon,
            None,
            None,
            row_width,
            *enabled,
            keyboard_selected,
            allow_mouse_hover,
            submenu_open,
            show_mnemonic,
        ),
        MenuItemDef::Checkbox {
            label,
            mnemonic,
            checked,
            shortcut_text,
            enabled,
            ..
        } => text_row(
            ui,
            label,
            *mnemonic,
            None,
            Some(*checked),
            shortcut_text.as_deref(),
            row_width,
            *enabled,
            keyboard_selected,
            allow_mouse_hover,
            submenu_open,
            show_mnemonic,
        ),
        MenuItemDef::SubMenu {
            label,
            mnemonic,
            enabled,
            ..
        } => text_row(
            ui,
            label,
            *mnemonic,
            None,
            None,
            Some("›"),
            row_width,
            *enabled,
            keyboard_selected,
            allow_mouse_hover,
            submenu_open,
            show_mnemonic,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn text_row(
    ui: &mut egui::Ui,
    label: &str,
    mnemonic: char,
    icon: Option<&'static str>,
    checkbox: Option<bool>,
    right_text: Option<&str>,
    row_width: f32,
    enabled: bool,
    keyboard_selected: bool,
    allow_mouse_hover: bool,
    submenu_open: bool,
    show_mnemonic: bool,
) -> egui::Response {
    let button_padding = ui.spacing().button_padding;
    let row_height = ui.spacing().interact_size.y;
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(row_width, row_height), egui::Sense::click());

    if !ui.is_rect_visible(rect) {
        return resp;
    }

    let mouse_hovered = allow_mouse_hover && resp.hovered();
    let visuals = if !enabled {
        &ui.visuals().widgets.noninteractive
    } else if keyboard_selected || mouse_hovered {
        &ui.visuals().widgets.hovered
    } else if submenu_open {
        &ui.visuals().widgets.open
    } else {
        &ui.visuals().widgets.inactive
    };

    if enabled && (keyboard_selected || mouse_hovered || submenu_open) {
        ui.painter()
            .rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);
    }

    let text_color = if !enabled {
        ui.visuals()
            .widgets
            .noninteractive
            .fg_stroke
            .color
            .gamma_multiply(0.5)
    } else {
        visuals.fg_stroke.color
    };

    let icon_slot = ui.spacing().icon_width + button_padding.x * 2.0;
    let mut x = rect.left() + button_padding.x;

    // Checkbox or blank left column
    if let Some(checked) = checkbox {
        if checked {
            let icon_font_id = egui::FontId::new(
                egui::TextStyle::Button.resolve(ui.style()).size,
                egui::FontFamily::Name(crate::icons::FAMILY_NAME.into()),
            );
            let galley = ui.painter().layout_no_wrap(
                crate::icons::CHECK.to_string(),
                icon_font_id,
                text_color,
            );
            let pos = egui::pos2(x, rect.center().y - galley.size().y / 2.0);
            ui.painter().galley(pos, galley, text_color);
        }
    } else if let Some(icon_str) = icon {
        let icon_font_id = egui::FontId::new(
            egui::TextStyle::Button.resolve(ui.style()).size,
            egui::FontFamily::Name(crate::icons::FAMILY_NAME.into()),
        );
        let galley = ui
            .painter()
            .layout_no_wrap(icon_str.to_string(), icon_font_id, text_color);
        let pos = egui::pos2(x, rect.center().y - galley.size().y / 2.0);
        ui.painter().galley(pos, galley, text_color);
    }
    x = rect.left() + button_padding.x + icon_slot;

    let right_galley = right_text.map(|right| {
        let right_color = if enabled && (keyboard_selected || mouse_hovered || submenu_open) {
            text_color
        } else {
            ui.visuals().weak_text_color()
        };
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        let galley = ui
            .painter()
            .layout_no_wrap(right.to_string(), font_id, right_color);
        (right_color, galley)
    });
    let label_max_x = right_galley
        .as_ref()
        .map_or(rect.right() - button_padding.x, |(_, galley)| {
            rect.right() - button_padding.x - galley.size().x - MENU_LABEL_RIGHT_GAP
        });

    // Label with optional mnemonic underline
    let lj = label_job(ui, label, mnemonic, show_mnemonic, text_color);
    let galley = ui.painter().layout_job(lj);
    let pos = egui::pos2(x, rect.center().y - galley.size().y / 2.0);
    let label_clip_rect = egui::Rect::from_min_max(
        egui::pos2(x, rect.top()),
        egui::pos2(label_max_x.max(x), rect.bottom()),
    );
    ui.painter()
        .with_clip_rect(label_clip_rect)
        .galley(pos, galley, text_color);

    // Right-aligned text (shortcut or submenu arrow)
    if let Some((right_color, galley)) = right_galley {
        let pos = egui::pos2(
            rect.right() - button_padding.x - galley.size().x,
            rect.center().y - galley.size().y / 2.0,
        );
        ui.painter().galley(pos, galley, right_color);
    }

    resp
}

pub(super) fn label_job(
    ui: &egui::Ui,
    label: &str,
    mnemonic: char,
    show_mnemonic: bool,
    color: egui::Color32,
) -> egui::text::LayoutJob {
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    if !show_mnemonic {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            label,
            0.0,
            egui::text::TextFormat {
                font_id,
                color,
                ..Default::default()
            },
        );
        return job;
    }
    layout_with_mnemonic(font_id, label, mnemonic, color)
}

fn layout_with_mnemonic(
    font_id: egui::FontId,
    label: &str,
    mnemonic: char,
    color: egui::Color32,
) -> egui::text::LayoutJob {
    let lower = mnemonic.to_ascii_lowercase();
    let found = label
        .char_indices()
        .find(|(_, c)| c.to_ascii_lowercase() == lower);

    let plain = egui::text::TextFormat {
        font_id: font_id.clone(),
        color,
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::default();

    let Some((byte_idx, ch)) = found else {
        job.append(label, 0.0, plain);
        return job;
    };

    let end = byte_idx + ch.len_utf8();

    if byte_idx > 0 {
        job.append(&label[..byte_idx], 0.0, plain.clone());
    }
    job.append(
        &label[byte_idx..end],
        0.0,
        egui::text::TextFormat {
            font_id: font_id.clone(),
            color,
            underline: egui::Stroke::new(1.0, color),
            ..Default::default()
        },
    );
    if end < label.len() {
        job.append(&label[end..], 0.0, plain);
    }
    job
}

pub(super) fn with_menu_style<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.scope(|ui| {
        egui::containers::menu::menu_style(ui.style_mut());
        add_contents(ui)
    })
    .inner
}

fn configured_menu_style(style: &egui::Style) -> egui::Style {
    let mut menu_style = style.clone();
    egui::containers::menu::menu_style(&mut menu_style);
    menu_style
}

pub(super) fn menu_popup_metrics(style: &egui::Style) -> (f32, f32) {
    let menu_style = configured_menu_style(style);
    let frame = egui::Frame::menu(&menu_style);
    (
        frame.total_margin().sum().x / 2.0 + 2.0,
        frame.total_margin().top,
    )
}

fn menu_item_height(style: &egui::Style, item: &MenuItemDef) -> f32 {
    match item {
        MenuItemDef::Separator => style.spacing.interact_size.y * 0.25,
        _ => style.spacing.interact_size.y,
    }
}

fn menu_content_height(style: &egui::Style, items: &[MenuItemDef]) -> f32 {
    items.iter().map(|item| menu_item_height(style, item)).sum()
}

fn layout_job_width(ctx: &egui::Context, job: egui::text::LayoutJob) -> f32 {
    ctx.fonts_mut(|fonts| fonts.layout_job(job).size().x)
}

fn layout_text_width(ctx: &egui::Context, font_id: egui::FontId, text: &str) -> f32 {
    ctx.fonts_mut(|fonts| {
        fonts
            .layout(
                text.to_string(),
                font_id,
                egui::Color32::WHITE,
                f32::INFINITY,
            )
            .size()
            .x
    })
}

fn menu_item_width(ctx: &egui::Context, style: &egui::Style, item: &MenuItemDef) -> f32 {
    let button_padding = style.spacing.button_padding;
    let icon_slot = style.spacing.icon_width + button_padding.x * 2.0;
    let font_id = egui::TextStyle::Button.resolve(style);
    let label_width = |label: &str, mnemonic: char| {
        layout_job_width(
            ctx,
            layout_with_mnemonic(font_id.clone(), label, mnemonic, egui::Color32::WHITE),
        )
    };
    let right_width = |text: &str| layout_text_width(ctx, font_id.clone(), text);

    let row_width = match item {
        MenuItemDef::Separator => 0.0,
        MenuItemDef::Button {
            label, mnemonic, ..
        } => icon_slot + label_width(label, *mnemonic),
        MenuItemDef::Checkbox {
            label,
            mnemonic,
            shortcut_text,
            ..
        } => {
            let shortcut_width = shortcut_text.as_deref().map_or(0.0, right_width);
            icon_slot
                + label_width(label, *mnemonic)
                + shortcut_width
                + if shortcut_text.is_some() { MENU_LABEL_RIGHT_GAP } else { 0.0 }
        }
        MenuItemDef::SubMenu {
            label, mnemonic, ..
        } => icon_slot + label_width(label, *mnemonic) + right_width("›") + MENU_LABEL_RIGHT_GAP,
    };

    (row_width + button_padding.x * 2.0).clamp(MENU_MIN_WIDTH, MENU_MAX_WIDTH)
}

fn menu_content_width(ctx: &egui::Context, style: &egui::Style, items: &[MenuItemDef]) -> f32 {
    items
        .iter()
        .map(|item| menu_item_width(ctx, style, item))
        .fold(MENU_MIN_WIDTH, f32::max)
        .clamp(MENU_MIN_WIDTH, MENU_MAX_WIDTH)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn show_menu_panel(
    ctx: &egui::Context,
    pos: egui::Pos2,
    area_id: egui::Id,
    items: &[MenuItemDef],
    base_style: &egui::Style,
    selected_idx: Option<usize>,
    show_keyboard_selection: bool,
    allow_mouse_hover: bool,
    submenu_open: impl Fn(usize) -> bool,
) -> MenuPanelResult {
    let mut item_rects = vec![egui::Rect::NOTHING; items.len()];
    let mut hovered_idx = None;
    let mut clicked = None;
    let menu_style = configured_menu_style(base_style);
    let frame = egui::Frame::menu(&menu_style);
    let content_width = menu_content_width(ctx, &menu_style, items);
    let outer_width = content_width + frame.total_margin().sum().x;
    let max_inner_height = (ctx.content_rect().height() * MENU_MAX_HEIGHT_FRACTION
        - frame.total_margin().sum().y)
        .max(menu_style.spacing.interact_size.y);
    let should_scroll = menu_content_height(&menu_style, items) > max_inner_height;

    let popup = egui::Popup::new(
        area_id,
        ctx.clone(),
        pos,
        egui::LayerId::new(egui::Order::Foreground, area_id),
    )
    .open(true)
    .kind(egui::PopupKind::Menu)
    .align(egui::RectAlign::TOP_START)
    .layout(egui::Layout::top_down_justified(egui::Align::Min))
    .style(egui::containers::menu::menu_style)
    .frame(frame)
    .width(outer_width)
    .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
    .show(|ui| {
        ui.set_min_width(content_width);
        ui.set_max_width(content_width);
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        let draw_rows = |ui: &mut egui::Ui,
                         item_rects: &mut Vec<egui::Rect>,
                         hovered_idx: &mut Option<usize>,
                         clicked: &mut Option<MenuPanelClick>| {
            ui.set_min_width(content_width);
            ui.set_max_width(content_width);
            for (i, item) in items.iter().enumerate() {
                let response = show_item_row(
                    ui,
                    item,
                    content_width,
                    show_keyboard_selection && selected_idx == Some(i),
                    allow_mouse_hover,
                    submenu_open(i),
                    true,
                );
                item_rects[i] = response.rect;

                if allow_mouse_hover && response.hovered() && item.is_navigable() {
                    *hovered_idx = Some(i);
                }

                if response.clicked() {
                    *clicked = match item {
                        MenuItemDef::SubMenu { enabled: true, .. } => {
                            Some(MenuPanelClick::OpenSubmenu(i))
                        }
                        _ => item.action().map(MenuPanelClick::Action),
                    };
                }
            }
        };

        if should_scroll {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(max_inner_height)
                .show(ui, |ui| {
                    draw_rows(ui, &mut item_rects, &mut hovered_idx, &mut clicked)
                });
        } else {
            draw_rows(ui, &mut item_rects, &mut hovered_idx, &mut clicked);
        }
    })
    .expect("menu popups anchored to fixed positions should always render");

    MenuPanelResult {
        rect: popup.response.rect,
        item_rects,
        hovered_idx,
        clicked,
    }
}

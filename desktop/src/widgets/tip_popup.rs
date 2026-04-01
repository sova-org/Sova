use eframe::egui;

pub fn show(ctx: &egui::Context, id: &str, dismissed: &mut Vec<String>) -> bool {
    if dismissed.iter().any(|d| d == id) {
        return false;
    }

    let title_key = format!("tip.{id}.title");
    let body_key = format!("tip.{id}.body");
    let title = t!(&title_key);
    let body = t!(&body_key);

    let accent = ctx.style().visuals.selection.bg_fill;
    let mut just_dismissed = false;

    let r = egui::Window::new(egui::RichText::new(""))
        .id(egui::Id::new("tip_popup"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .default_width(280.0)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.set_max_width(280.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("\u{EA6B}").strong());
                ui.label(egui::RichText::new(title.as_ref()).strong());
            });
            ui.add_space(2.0);
            ui.label(body.as_ref());
            ui.add_space(4.0);
            if ui.button(t!("tip.got_it")).clicked() {
                dismissed.push(id.to_string());
                just_dismissed = true;
            }
        });

    if let Some(resp) = r {
        let rect = resp.response.rect;
        let bar = egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height()));
        ctx.layer_painter(resp.response.layer_id)
            .rect_filled(bar, 0.0, accent);
    }

    just_dismissed
}

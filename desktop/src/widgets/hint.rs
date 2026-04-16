use eframe::egui;

#[derive(Clone)]
struct Hint {
    text: String,
    pass: u64,
}

fn id() -> egui::Id {
    egui::Id::new("contextual_hint")
}

pub fn set(ctx: &egui::Context, text: impl Into<String>) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| {
        d.insert_temp(
            id(),
            Hint {
                text: text.into(),
                pass,
            },
        )
    });
}

pub fn on_hover(ctx: &egui::Context, r: &egui::Response, text: impl Into<String>) {
    if r.hovered() {
        set(ctx, text);
    }
}

/// Render a label + control pair where both sides share the same contextual hint.
/// Returns the control's `Response` so callers can check `.changed()`.
pub fn labeled(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    hint_text: impl Into<String> + Clone,
    add_control: impl FnOnce(&mut egui::Ui) -> egui::Response,
) -> egui::Response {
    let lbl = ui.label(label);
    on_hover(ui.ctx(), &lbl, hint_text.clone());
    let r = add_control(ui);
    on_hover(ui.ctx(), &r, hint_text);
    r
}

pub fn current(ctx: &egui::Context) -> Option<String> {
    let pass = ctx.cumulative_pass_nr();
    ctx.data(|d| {
        d.get_temp::<Hint>(id())
            .filter(|h| pass.saturating_sub(h.pass) <= 1)
            .map(|h| h.text.clone())
    })
}

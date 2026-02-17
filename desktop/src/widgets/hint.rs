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

pub fn current(ctx: &egui::Context) -> Option<String> {
    let pass = ctx.cumulative_pass_nr();
    ctx.data(|d| {
        d.get_temp::<Hint>(id())
            .filter(|h| pass.saturating_sub(h.pass) <= 1)
            .map(|h| h.text.clone())
    })
}

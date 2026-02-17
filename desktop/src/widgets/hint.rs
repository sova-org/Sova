use eframe::egui;

#[derive(Clone)]
struct Hint {
    text: &'static str,
    pass: u64,
}

fn id() -> egui::Id {
    egui::Id::new("contextual_hint")
}

pub fn set(ctx: &egui::Context, text: &'static str) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| d.insert_temp(id(), Hint { text, pass }));
}

pub fn current(ctx: &egui::Context) -> Option<&'static str> {
    let pass = ctx.cumulative_pass_nr();
    ctx.data(|d| {
        d.get_temp::<Hint>(id())
            .filter(|h| pass.saturating_sub(h.pass) <= 1)
            .map(|h| h.text)
    })
}

use eframe::egui;

pub enum ConfirmAction {
    None,
    Confirmed,
    Cancelled,
}

pub struct ConfirmDialog {
    id: egui::Id,
    open: bool,
    title: String,
    message: String,
}

impl ConfirmDialog {
    pub fn new(id: impl std::hash::Hash) -> Self {
        Self {
            id: egui::Id::new(id),
            open: false,
            title: String::new(),
            message: String::new(),
        }
    }

    pub fn open(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.title = title.into();
        self.message = message.into();
        self.open = true;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn show(&mut self, ctx: &egui::Context) -> ConfirmAction {
        if !self.open {
            return ConfirmAction::None;
        }

        let mut action = ConfirmAction::None;

        let response = egui::Modal::new(self.id).show(ctx, |ui| {
            ui.set_width(300.0);

            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new(&self.title).strong());
                ui.add_space(8.0);
                ui.label(&self.message);
            });
            ui.add_space(12.0);

            ui.columns(2, |cols| {
                cols[0].vertical_centered(|ui| {
                    if ui.button(t!("common.yes").to_string()).clicked() {
                        action = ConfirmAction::Confirmed;
                    }
                });
                cols[1].vertical_centered(|ui| {
                    if ui.button(t!("common.no").to_string()).clicked() {
                        action = ConfirmAction::Cancelled;
                    }
                });
            });
        });

        if response.should_close() || matches!(action, ConfirmAction::Confirmed | ConfirmAction::Cancelled)
        {
            self.open = false;
            if matches!(action, ConfirmAction::None) {
                return ConfirmAction::Cancelled;
            }
        }

        action
    }
}

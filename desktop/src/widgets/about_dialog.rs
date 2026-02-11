use eframe::egui;

const DESCRIPTION: &str = "\
Sova is a Rust-based live coding \
environment. Sova is an instrument that \
is yours to modify. It is a free and \
open-source tool. Live code, have fun!";

const TEAM: &[&str] = &["Raphaël Forment", "Loïg Jezequel", "Tanguy Dubois"];

const LINKS: &[(&str, &str)] = &[
    ("sova.livecoding.fr", "https://sova.livecoding.fr"),
    ("cookie.paris", "https://cookie.paris"),
    ("athenor.com", "https://athenor.com"),
    ("toplap.org", "https://toplap.org"),
];

pub fn about_dialog(ctx: &egui::Context, open: &mut bool) {
    if !*open {
        return;
    }

    egui::Window::new("about_sova")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(380.0)
        .show(ctx, |ui| {
            ui.set_min_width(380.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if ui.add(egui::Button::new("✕").frame(false)).clicked() {
                    *open = false;
                }
            });

            ui.vertical_centered(|ui| {
                ui.add(
                    egui::Image::new(egui::include_image!(
                        "../../assets/icon.png"
                    ))
                    .max_size(egui::vec2(128.0, 128.0)),
                );

                ui.add_space(8.0);
                ui.heading(egui::RichText::new("Sova").size(28.0).strong());
                ui.label(
                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).weak(),
                );

                ui.add_space(12.0);
                let font_id = egui::TextStyle::Body.resolve(ui.style());
                let mut job = egui::text::LayoutJob::simple(
                    DESCRIPTION.to_string(),
                    font_id,
                    ui.visuals().text_color(),
                    ui.available_width(),
                );
                job.halign = egui::Align::Center;
                ui.label(job);

                ui.add_space(16.0);
                ui.label(egui::RichText::new("TEAM").weak());
                for name in TEAM {
                    ui.label(egui::RichText::new(*name).strong());
                }

                ui.add_space(16.0);
                ui.columns(2, |cols| {
                    for (i, (label, url)) in LINKS.iter().enumerate() {
                        let col = &mut cols[i % 2];
                        col.vertical_centered(|ui| {
                            if ui.button(format!("{} ↗", label)).clicked() {
                                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                            }
                        });
                    }
                });

                ui.add_space(8.0);
                if ui
                    .link(egui::RichText::new("AGPL-3.0 License ↗").weak())
                    .clicked()
                {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(
                        "https://www.gnu.org/licenses/agpl-3.0.html",
                    ));
                }
            });
        });
}

use eframe::egui;

const TEAM: &[&str] = &["Raphaël Forment", "Loïg Jezequel", "Tanguy Dubois"];

const LINKS: &[(&str, &str)] = &[
    ("sova.livecoding.fr", "https://sova.livecoding.fr"),
    ("cookie.paris", "https://cookie.paris"),
    ("athenor.com", "https://athenor.com"),
    ("toplap.org", "https://toplap.org"),
];

pub fn about_dialog(ctx: &egui::Context, open: &mut bool) {
    let guard_id = egui::Id::new("about_open_guard");

    if !*open {
        ctx.data_mut(|d| d.insert_temp(guard_id, false));
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        *open = false;
        return;
    }

    // Skip click-outside on the frame that opened the dialog
    let was_open = ctx.data(|d| d.get_temp::<bool>(guard_id).unwrap_or(false));
    ctx.data_mut(|d| d.insert_temp(guard_id, true));

    let resp = egui::Window::new("about_sova")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(380.0)
        .show(ctx, |ui| {
            ui.set_min_width(380.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if ui
                    .add(egui::Button::new(crate::icons::CLOSE).frame(false))
                    .clicked()
                {
                    *open = false;
                }
            });

            ui.vertical_centered(|ui| {
                ui.add(
                    egui::Image::new(egui::include_image!("../../assets/icon.png"))
                        .max_size(egui::vec2(128.0, 128.0)),
                );

                ui.add_space(8.0);
                ui.heading(egui::RichText::new("Sova").size(28.0).strong());
                ui.label(egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).weak());

                ui.add_space(12.0);
                let font_id = egui::TextStyle::Body.resolve(ui.style());
                let mut job = egui::text::LayoutJob::simple(
                    t!("about.description").into(),
                    font_id,
                    ui.visuals().text_color(),
                    ui.available_width(),
                );
                job.halign = egui::Align::Center;
                ui.label(job);

                ui.add_space(16.0);
                ui.label(egui::RichText::new(t!("about.team")).weak());
                for name in TEAM {
                    ui.label(egui::RichText::new(*name).strong());
                }

                ui.add_space(16.0);
                ui.columns(2, |cols| {
                    for (i, (label, url)) in LINKS.iter().enumerate() {
                        let col = &mut cols[i % 2];
                        let align = if i % 2 == 0 {
                            egui::Align::Max
                        } else {
                            egui::Align::Min
                        };
                        col.with_layout(egui::Layout::top_down(align), |ui| {
                            if ui
                                .button(format!("{} {}", label, crate::icons::LINK_EXTERNAL))
                                .clicked()
                            {
                                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                            }
                        });
                    }
                });

                ui.add_space(8.0);
                if ui
                    .link(
                        egui::RichText::new(format!(
                            "AGPL-3.0 License {}",
                            crate::icons::LINK_EXTERNAL
                        ))
                        .weak(),
                    )
                    .clicked()
                {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(
                        "https://www.gnu.org/licenses/agpl-3.0.html",
                    ));
                }
            });
        });

    if let Some(inner) = resp
        && was_open
        && inner.response.clicked_elsewhere()
    {
        *open = false;
    }
}

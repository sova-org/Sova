use eframe::egui;
use egui_file_dialog::FileDialog;
use sova_core::schedule::ActionTiming;
use sova_server::ClientMessage;

use crate::{
    app_types::PendingDialog,
    fonts, icons, scene_panel,
    panels::{doc_panel, keybindings_window, server_panel::ServerAction, transport_bar},
    settings::DocSide,
    theme, widgets, SovaApp,
};

impl SovaApp {
    pub(crate) fn handle_close_request(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            self.save_settings();
            if self.panels.server.is_running() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                if !self.dialogs.confirm_exit.is_open() {
                    self.dialogs
                        .confirm_exit
                        .open(t!("exit.title"), t!("exit.message"));
                }
            }
        }
    }

    pub(crate) fn handle_dialogs(&mut self, ctx: &egui::Context) {
        match self.dialogs.confirm_exit.show(ctx) {
            widgets::ConfirmAction::Confirmed => {
                self.save_settings();
                self.bridge.disconnect();
                self.panels.server.stop();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            widgets::ConfirmAction::Cancelled | widgets::ConfirmAction::None => {}
        }

        if let widgets::ConfirmAction::Confirmed = self.dialogs.confirm_reset_scene.show(ctx) {
            self.bridge
                .send(ClientMessage::ResetScene(ActionTiming::Immediate));
        }

        match self.dialogs.confirm_load_demo.show(ctx) {
            widgets::ConfirmAction::Confirmed => {
                if let Some((_, bytes)) = self.dialogs.pending_demo.take() {
                    self.load_scene_from_bytes(bytes, ActionTiming::Immediate);
                }
            }
            widgets::ConfirmAction::Cancelled => {
                self.dialogs.pending_demo = None;
            }
            widgets::ConfirmAction::None => {}
        }

        if self.session.rename_input.is_some() {
            let mut open = true;
            let mut confirmed_name: Option<String> = None;
            egui::Window::new(t!("menu.rename"))
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    let input = self.session.rename_input.as_mut().expect("dialog open guard ensures Some");
                    let r = ui.text_edit_singleline(input);
                    r.request_focus();
                    if widgets::consume_key_on_lost_focus(ui, &r, egui::Key::Enter)
                        && !input.trim().is_empty()
                    {
                        confirmed_name = Some(input.trim().to_owned());
                    }
                });
            if let Some(new_name) = confirmed_name {
                self.bridge.send(ClientMessage::SetName {
                    name: new_name.clone(),
                    password: None,
                });
                self.bridge.set_confirmed_username(new_name);
                self.session.rename_input = None;
            } else if !open {
                self.session.rename_input = None;
            }
        }

        self.dialogs.file.update(ctx);
        if let Some(path) = self.dialogs.file.take_picked() {
            match std::mem::replace(&mut self.dialogs.pending, PendingDialog::None) {
                PendingDialog::SaveScene { snapshot } => {
                    if let Ok(bytes) = serde_json::to_vec(&*snapshot)
                        && std::fs::write(&path, bytes).is_ok()
                    {
                        self.push_recent_scene(path);
                    }
                }
                PendingDialog::LoadScene { timing } => {
                    self.load_scene_from_path(&path, timing);
                }
                PendingDialog::PickSampleFolder => {
                    self.panels.audio.add_sample_path(path);
                }
                PendingDialog::None => {}
            }
        }
    }

    pub(crate) fn poll_updates(&mut self) {
        self.panels.server.poll();
        self.bridge.poll();
        if self.bridge.just_connected {
            self.bridge.just_connected = false;
            self.bridge.send(ClientMessage::SetMasterVolume(
                self.audio_ctl.effective_gain(),
            ));
        }
        let audio_running = self.bridge.audio_state().running || self.bridge.has_feedback();
        if audio_running && !self.audio_ctl.was_running {
            self.panels.scope_bar.open = true;
            self.panels.vu_meter.open = true;
        }
        self.audio_ctl.was_running = audio_running;
        self.panels.logs.poll();
    }

    pub(crate) fn render_top_bar(&mut self, ctx: &egui::Context) {
        // Paint the visuals shader first so it sits at the bottom of the
        // background layer; panels added afterwards draw on top of it.
        let clock = self.bridge.clock();
        self.panels.visuals.paint_background_central(
            ctx,
            self.prefs.appearance.visuals_enabled,
            clock.beat as f32,
            clock.tempo as f32,
            clock.phase as f32,
        );

        let menu_bar_fill = widgets::SceneOpacity::new(
            self.prefs.appearance.visuals_enabled,
            self.prefs.appearance.scene_opacity,
        )
        .panel_fill(ctx);
        egui::TopBottomPanel::top("menu_bar")
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(menu_bar_fill)
                    .inner_margin(egui::Margin {
                        left: 8,
                        right: 8,
                        top: 4,
                        bottom: 4,
                    }),
            )
            .show_separator_line(true)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    let icon = egui::Image::new(egui::include_image!("../assets/icon.png"))
                        .fit_to_exact_size(egui::vec2(20.0, 20.0));
                    let r = ui.add(egui::Button::image(icon).frame(false));
                    if r.hovered() {
                        widgets::hint::set(ctx, t!("hint.about_sova"));
                    }
                    if r.clicked() {
                        self.panels.about_open = !self.panels.about_open;
                    }
                    let menus = self.build_menus(ctx);
                    self.menu_bar.show(ui, &menus);

                    if self.bridge.is_connected()
                        && let Some(transport_bar::TransportAction::Panic) =
                            self.panels.transport_bar.show_inline(ui, ctx, &self.bridge)
                    {
                        self.audio_ctl.muted = true;
                        self.bridge.send(ClientMessage::SetMasterVolume(0.0));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let audio = self.bridge.audio_state();
                        if audio.running {
                            let cpu_pct = audio.cpu_load * 100.0;
                            let cpu_color = if cpu_pct >= 80.0 {
                                egui::Color32::from_rgb(255, 80, 80)
                            } else if cpu_pct >= 50.0 {
                                egui::Color32::from_rgb(255, 180, 50)
                            } else {
                                ui.visuals().widgets.inactive.fg_stroke.color
                            };
                            let text =
                                format!("V:{:03}  CPU {:02.0}%", audio.active_voices, cpu_pct);
                            ui.colored_label(cpu_color, text);
                        }

                        let mut vol = if self.audio_ctl.muted {
                            0.0
                        } else {
                            self.audio_ctl.master_volume
                        };
                        let slider = egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false);
                        let r = ui.add_sized([100.0, ui.available_height()], slider);
                        if r.changed() {
                            self.audio_ctl.master_volume = vol;
                            self.audio_ctl.muted = false;
                            self.bridge.send(ClientMessage::SetMasterVolume(
                                self.audio_ctl.effective_gain(),
                            ));
                        }
                        if r.hovered() {
                            widgets::hint::set(ctx, t!("hint.master_volume"));
                        }

                        let icon = if self.audio_ctl.muted || self.audio_ctl.master_volume == 0.0 {
                            icons::MUTE
                        } else {
                            icons::UNMUTE
                        };
                        let btn = ui.button(icons::rich(icon));
                        if btn.clicked() {
                            self.audio_ctl.muted = !self.audio_ctl.muted;
                            self.bridge.send(ClientMessage::SetMasterVolume(
                                self.audio_ctl.effective_gain(),
                            ));
                        }
                        if btn.hovered() {
                            let hint = if self.audio_ctl.muted {
                                t!("hint.unmute")
                            } else {
                                t!("hint.mute")
                            };
                            widgets::hint::set(ctx, hint);
                        }
                    });
                }); // end left_to_right
                if let Some(action) = self.menu_bar.take_action() {
                    self.dispatch_menu_action(action, ctx);
                }
            });
    }

    pub(crate) fn render_sidebar_and_panels(&mut self, ctx: &egui::Context) {
        if let Some((msg, _)) = self.bridge.last_error.take() {
            self.session.toasts.push(widgets::ToastLevel::Error, msg);
        }

        // Expanded sidebar is added first so it claims full window height.
        // Bottom/cheat/scope panels below will only span the remaining central column.
        let panels = &mut self.panels;
        let settings_ctx = doc_panel::SettingsContext {
            server: &mut panels.server,
            audio: &mut panels.audio,
            options: &mut panels.options,
            devices: &mut panels.devices,
            logs: &mut panels.logs,
            editor_settings: &mut self.prefs.editor,
            appearance: &mut self.prefs.appearance,
            view_mode: &mut panels.scene.view_mode,
            show_phase_bar: &mut panels.transport_bar.show_phase_bar,
        };
        let (sidebar_server_action, sidebar_appearance_changed, pick_sample_folder) = panels
            .doc
            .show_expanded_side_panel(ctx, &self.bridge, settings_ctx);
        if pick_sample_folder && matches!(self.dialogs.pending, PendingDialog::None) {
            self.dialogs.file = FileDialog::new().anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]);
            self.dialogs.file.pick_directory();
            self.dialogs.pending = PendingDialog::PickSampleFolder;
        }
        match sidebar_server_action {
            ServerAction::Start => {
                self.panels
                    .server
                    .start(self.panels.audio.generate_audio_config());
            }
            ServerAction::Stop => {
                self.bridge.disconnect();
                self.panels.server.stop();
            }
            ServerAction::None => {}
        }
        if sidebar_appearance_changed {
            theme::apply_appearance(ctx, &self.prefs.appearance);
            fonts::apply_fonts(
                ctx,
                &self.prefs.appearance.ui_font,
                &self.prefs.appearance.editor_font,
            );
        }

        let opacity = widgets::SceneOpacity::new(
            self.prefs.appearance.visuals_enabled,
            self.prefs.appearance.scene_opacity,
        );
        let bottom_panel_fill = opacity.panel_fill(ctx);
        let bar = egui::TopBottomPanel::bottom("bottom_bar")
            .frame(egui::Frame::side_top_panel(&ctx.style()).fill(bottom_panel_fill))
            .show_separator_line(true)
            .show(ctx, |ui| {
                let sample_browser_available =
                    !self.bridge.is_connected() || self.panels.server.is_running();
                widgets::bottom_bar(
                    ui,
                    &self.panels.server.info(),
                    &self.panels.client.info(&self.bridge),
                    self.panels.tools.settings.show_chat,
                    sample_browser_available && self.panels.tools.settings.show_sample_browser,
                    sample_browser_available,
                    self.panels.scene.view_mode,
                )
            })
            .inner;
        if bar.open_palette {
            self.panels.command_palette.open();
        }
        if bar.disconnect {
            self.bridge.disconnect();
        }
        if bar.toggle_chat {
            self.panels.tools.toggle_chat();
        }
        if bar.toggle_sample_browser {
            let available = !self.bridge.is_connected() || self.panels.server.is_running();
            if available {
                self.panels.tools.toggle_sample_browser();
            }
        }
        if bar.toggle_view_mode {
            self.dispatch(widgets::CommandId::ToggleViewMode);
        }

        // Preprocess visualization data once for all panels
        let scope_gen = self.bridge.scope_generation();
        let scope_data = self.bridge.scope_data();
        if !scope_data.is_empty() && scope_gen != self.viz.last_scope_gen {
            self.viz.last_scope_gen = scope_gen;
            widgets::align_trigger(&mut self.viz.aligned_scope, scope_data);
            let analyzer = self
                .viz
                .spectrum_analyzer
                .get_or_insert_with(|| widgets::SpectrumAnalyzer::new(44100.0));
            self.viz.raw_bands = analyzer.analyze(scope_data);
        }

        // VU meter must be added before scope bar so it spans full screen height
        // and the scope bar is constrained to the width beside it
        if self.panels.vu_meter.open
            && (self.bridge.audio_state().running || self.bridge.has_feedback())
        {
            let vu_side = match self.panels.doc.settings.side {
                DocSide::Left => egui::containers::panel::Side::Right,
                DocSide::Right => egui::containers::panel::Side::Left,
            };
            self.panels
                .vu_meter
                .show_side_panel(ctx, self.bridge.peak_data(), vu_side, opacity);
        }

        // Sample paths and hosting state needed for tools panel and floating windows
        let sample_paths = self.panels.audio.sample_paths();
        #[cfg(feature = "default-samples")]
        let default_sample_path = Some(self.panels.audio.default_samples_path());
        #[cfg(not(feature = "default-samples"))]
        let default_sample_path: Option<&std::path::Path> = None;
        let is_hosting = self.panels.server.is_running();

        // Tools panel (chat / sample browser) on opposite side from doc panel
        {
            let tools_side = match self.panels.doc.settings.side {
                DocSide::Left => egui::containers::panel::Side::Right,
                DocSide::Right => egui::containers::panel::Side::Left,
            };
            self.sample_browser_rect = self.panels.tools.show_side_panel(
                ctx,
                tools_side,
                &mut self.panels.chat,
                &mut self.panels.sample_browser,
                &mut self.bridge,
                default_sample_path,
                sample_paths,
                is_hosting,
                opacity,
                self.input_owner,
            );
        }

        // Scope bar as bottom panel (must be before CentralPanel)
        if self.panels.scope_bar.open
            && (self.bridge.audio_state().running || self.bridge.has_feedback())
        {
            self.panels.scope_bar.show_bottom_panel(
                ctx,
                &self.viz.aligned_scope,
                &self.viz.raw_bands,
                &self.panels.scope.settings,
                &self.panels.spectrum.settings,
                opacity,
            );
        }

        // Collapsed book-icon strip, innermost (only runs when the sidebar is not expanded)
        self.panels.doc.show_collapsed_side_panel(ctx, opacity);
    }

    pub(crate) fn render_central_panel(&mut self, ctx: &egui::Context) {
        let central_frame = if self.prefs.appearance.visuals_enabled {
            egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::TRANSPARENT)
        } else {
            egui::Frame::central_panel(&ctx.style())
        };

        if self.bridge.is_connected() {
            let sidebar_open = self.panels.doc.is_expanded()
                && self.panels.doc.mode() == doc_panel::SidebarMode::Settings;
            let panels = scene_panel::PanelVisibility {
                sidebar: sidebar_open,
                devices: self.panels.devices.open,
                scope: self.panels.scope.open,
                spectrum: self.panels.spectrum.open,
                vu_meter: self.panels.vu_meter.open,
                scope_bar: self.panels.scope_bar.open,
                logs: self.panels.doc.is_logs_open(),
                debug: self.panels.debug_open,
            };
            egui::CentralPanel::default()
                .frame(central_frame)
                .show(ctx, |ui| {
                    let sample_names = self.panels.sample_browser.sample_names();
                    self.panels.scene.show(
                        ui,
                        &self.bridge,
                        self.prefs.appearance.visuals_enabled,
                        self.prefs.appearance.scene_opacity,
                        &self.prefs.editor,
                        &sample_names,
                        self.input_owner,
                    );
                });
            // Drain pending flashes from scene panel into bridge
            let now = std::time::Instant::now();
            for (li, fi) in self.panels.scene.pending_mutation_flashes.drain(..) {
                self.bridge.mutation_flashes.insert((li, fi), now);
            }
            for ((li, fi), success) in self.panels.scene.pending_compilation_flashes.drain(..) {
                self.bridge
                    .compilation_flashes
                    .insert((li, fi), (success, now));
            }
            if panels.sidebar != sidebar_open {
                if panels.sidebar {
                    self.panels
                        .doc
                        .open_settings_tab(doc_panel::SettingsTab::Server);
                } else {
                    self.panels.doc.settings.collapsed = true;
                }
            }
            self.panels.devices.open = panels.devices;
            self.panels.scope.open = panels.scope;
            self.panels.spectrum.open = panels.spectrum;
            self.panels.vu_meter.open = panels.vu_meter;
            self.panels.scope_bar.open = panels.scope_bar;
            if panels.logs != self.panels.doc.is_logs_open() {
                self.panels
                    .doc
                    .toggle_settings_tab(doc_panel::SettingsTab::Logs);
            }
            self.panels.debug_open = panels.debug;
        } else {
            let action = egui::CentralPanel::default()
                .frame(central_frame)
                .show(ctx, |ui| {
                    self.panels.client.show_centered(
                        ui,
                        &mut self.bridge,
                        self.panels.server.is_running(),
                    )
                })
                .inner;
            if action.start_server {
                self.panels
                    .server
                    .start(self.panels.audio.generate_audio_config());
            }
            if action.stop_server {
                self.panels.server.stop();
            }
            if action.open_server_config {
                self.panels
                    .doc
                    .open_settings_tab(doc_panel::SettingsTab::Server);
            }
            if action.start_feedback
                && !self.panels.server.is_running()
                && !self.bridge.has_feedback()
            {
                self.bridge
                    .start_feedback(self.panels.audio.generate_audio_config());
            }
        }
    }

    pub(crate) fn render_floating_windows(&mut self, ctx: &egui::Context) {
        // Floating windows — detached viewports only; embedded rendering
        // is handled by the tools panel side panel above.
        self.panels
            .chat
            .show_detached_only(ctx, &mut self.bridge, &self.prefs.appearance);
        self.session.toasts.poll_chat(self.bridge.chat_messages());
        self.session.toasts.show(ctx);
        self.panels.devices.show(ctx, &self.bridge);

        let sample_paths = self.panels.audio.sample_paths();
        #[cfg(feature = "default-samples")]
        let default_sample_path = Some(self.panels.audio.default_samples_path());
        #[cfg(not(feature = "default-samples"))]
        let default_sample_path: Option<&std::path::Path> = None;
        let is_hosting = self.panels.server.is_running();

        self.panels.sample_browser.show_detached_only(
            ctx,
            &self.bridge,
            default_sample_path,
            sample_paths,
            &self.prefs.appearance,
            is_hosting,
        );

        self.panels
            .scope
            .show(ctx, &self.viz.aligned_scope, &self.prefs.appearance);
        self.panels
            .spectrum
            .show(ctx, &self.viz.raw_bands, &self.prefs.appearance);

        // Single repaint request for all visualization panels
        let any_viz_open =
            self.panels.scope.open || self.panels.spectrum.open || self.panels.scope_bar.open;
        if any_viz_open && (self.bridge.audio_state().running || self.bridge.has_feedback()) {
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }

        self.panels.visuals.show_editor(ctx, &self.prefs.editor);

        if self.panels.visuals.take_pending_broadcast() {
            self.bridge.send_hydra_code(self.panels.visuals.code());
        }
        if self.panels.visuals.shared
            && let Some((sender, code)) = self.bridge.take_remote_hydra()
        {
            self.panels.visuals.remote_sender = Some(sender);
            self.panels.visuals.apply_remote_code(&code);
        }
    }

    pub(crate) fn render_overlays(&mut self, ctx: &egui::Context) {
        keybindings_window::show_debug_window(ctx, &mut self.panels.debug_open);
        keybindings_window::show_keybindings_window(
            ctx,
            &mut self.panels.keybindings_open,
            self.panels.scene.view_mode,
        );
        widgets::about_dialog(ctx, &mut self.panels.about_open);

        self.panels
            .command_palette
            .update_states(&widgets::PanelStates {
                sidebar: self.panels.doc.is_expanded()
                    && self.panels.doc.mode() == doc_panel::SidebarMode::Settings,
                devices: self.panels.devices.open,
                scope: self.panels.scope.open,
                spectrum: self.panels.spectrum.open,
                vu_meter: self.panels.vu_meter.open,
                scope_bar: self.panels.scope_bar.open,
                chat: self.panels.tools.settings.show_chat || self.panels.chat.detached,
                logs: self.panels.doc.is_logs_open(),
                debug: self.panels.debug_open,
                keybindings: self.panels.keybindings_open,
                about: self.panels.about_open,
                sample_browser: (self.panels.tools.settings.show_sample_browser
                    && (!self.bridge.is_connected() || self.panels.server.is_running()))
                    || self.panels.sample_browser.detached,
                documentation: !self.panels.doc.settings.collapsed,
                visuals: self.panels.visuals.open,
            });
        match self.panels.command_palette.show(ctx) {
            widgets::PaletteAction::Execute(cmd) => self.dispatch(cmd),
            widgets::PaletteAction::None => {}
        }

        let had_interaction = ctx.input(|i| {
            i.pointer.any_click()
                || i.pointer.any_released()
                || i.pointer.is_decidedly_dragging()
                || !i.events.is_empty()
        });
        if had_interaction {
            let current = self.build_settings();
            if current != self.last_saved_settings {
                self.settings_dirty_since
                    .get_or_insert(std::time::Instant::now());
            } else {
                self.settings_dirty_since = None;
            }
        }
        if let Some(since) = self.settings_dirty_since
            && since.elapsed() >= std::time::Duration::from_secs(1)
        {
            self.save_settings();
        }
    }
}

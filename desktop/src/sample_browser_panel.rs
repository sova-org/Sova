use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui;

use crate::client_bridge::ClientBridge;
use crate::sample_browser::{SampleBrowserState, TreeLineKind, resolve_sample_path};
use crate::settings::AppearanceSettings;
use crate::widgets::{self, Waveform};
use sova_server::ClientMessage;

struct PreviewData {
    key: String,
    mono_samples: Vec<f32>,
    channels: u8,
    duration_secs: f32,
}

struct DecodeResult {
    key: String,
    mono_samples: Vec<f32>,
    channels: u8,
    duration_secs: f32,
}

pub struct SampleBrowserPanel {
    pub open: bool,
    pub detached: bool,
    state: Option<SampleBrowserState>,
    last_paths: Vec<PathBuf>,
    default_path: Option<PathBuf>,
    preview: Option<PreviewData>,
    decode_rx: Option<mpsc::Receiver<DecodeResult>>,
    pending_key: Option<String>,
    begin: f64,
}

impl SampleBrowserPanel {
    pub fn new() -> Self {
        Self {
            open: false,
            detached: false,
            state: None,
            last_paths: Vec::new(),
            default_path: None,
            preview: None,
            decode_rx: None,
            pending_key: None,
            begin: 0.0,
        }
    }

    pub fn sample_names(&self) -> Vec<String> {
        self.state.as_ref().map(|s| s.tree.sample_names()).unwrap_or_default()
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        default_path: Option<&std::path::Path>,
        sample_paths: &[PathBuf],
        appearance: &AppearanceSettings,
        is_hosting: bool,
    ) {
        if !self.open {
            return;
        }

        if !is_hosting && bridge.is_connected() {
            self.open = false;
            return;
        }

        let dp_changed = self.default_path.as_deref() != default_path;
        if self.last_paths != sample_paths || dp_changed {
            self.last_paths = sample_paths.to_vec();
            self.default_path = default_path.map(PathBuf::from);
            if sample_paths.is_empty() && default_path.is_none() {
                self.state = None;
            } else {
                self.state = Some(SampleBrowserState::new(
                    default_path,
                    sample_paths,
                ));
            }
            self.preview = None;
        }

        // Poll background decode
        if let Some(rx) = &self.decode_rx
            && let Ok(result) = rx.try_recv()
        {
            self.pending_key = None;
            self.preview = Some(PreviewData {
                key: result.key,
                mono_samples: result.mono_samples,
                channels: result.channels,
                duration_secs: result.duration_secs,
            });
            self.decode_rx = None;
        }

        if self.detached {
            self.show_detached(ctx, bridge, sample_paths, appearance, is_hosting);
        } else {
            self.show_embedded(ctx, bridge, sample_paths, is_hosting);
        }
    }

    fn browser_content(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        sample_paths: &[PathBuf],
        show_popout: bool,
        is_hosting: bool,
    ) {
        let Some(state) = &mut self.state else {
            ui.colored_label(egui::Color32::GRAY, t!("sample_browser.no_paths"));
            return;
        };

        // Search bar (always visible), with optional pop-out button
        let search_id = ui.id().with("sample_search");
        let search_resp = ui
            .horizontal(|ui| {
                if show_popout {
                    let r = ui
                        .button(crate::icons::POPOUT)
                        .on_hover_text(t!("common.pop_out"));
                    if r.hovered() {
                        crate::widgets::hint::set(ui.ctx(), t!("sample_browser.hint.detach"));
                    }
                    if r.clicked() {
                        self.detached = true;
                    }
                }
                let r = ui.add(
                    egui::TextEdit::singleline(&mut state.search_query)
                        .id(search_id)
                        .hint_text(t!("sample_browser.search"))
                        .desired_width(ui.available_width()),
                );
                if r.hovered() {
                    crate::widgets::hint::set(ui.ctx(), t!("sample_browser.hint.search"));
                }
                r
            })
            .inner;
        if search_resp.changed() {
            state.update_search();
        }

        ui.separator();

        let search_focused = search_resp.has_focus();

        // Handle keyboard input
        let prev_cursor = state.cursor;
        let (activate, focus_search) = handle_keyboard(ui, state, search_focused);
        let cursor_changed = state.cursor != prev_cursor;
        if focus_search {
            search_resp.request_focus();
        }

        let row_height = 18.0;
        let avail = ui.available_height() - 80.0;
        let visible_rows = (avail / row_height).max(5.0) as usize;

        let mut clicked_file: Option<(String, usize)> = None;
        let mut new_cursor: Option<usize> = None;
        let mut should_toggle = false;
        let mut preview_request: Option<(String, usize)> = None;
        let mut seek_request = false;

        egui::ScrollArea::vertical()
            .max_height(visible_rows as f32 * row_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let entries = state.entries();
                if entries.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, t!("sample_browser.no_entries"));
                    return;
                }

                for (i, entry) in entries.iter().enumerate() {
                    let selected = i == state.cursor;
                    let indent = entry.depth as f32 * 16.0;
                    let is_file = matches!(entry.kind, TreeLineKind::File);

                    let (rect, resp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), row_height),
                        egui::Sense::click(),
                    );

                    if selected {
                        ui.painter()
                            .rect_filled(rect, 0.0, ui.visuals().selection.bg_fill);
                    } else if resp.hovered() {
                        ui.painter()
                            .rect_filled(rect, 0.0, ui.visuals().widgets.hovered.bg_fill);
                    }

                    let icon_x = rect.left() + 4.0 + indent;
                    let text_x = icon_x + 16.0;

                    match &entry.kind {
                        TreeLineKind::Root { expanded } | TreeLineKind::Folder { expanded } => {
                            let openness = if *expanded { 1.0f32 } else { 0.0 };
                            let center = egui::pos2(icon_x + 5.0, rect.center().y);
                            let s = 4.0;
                            let tri_rect =
                                egui::Rect::from_center_size(center, egui::vec2(s * 2.0, s * 2.0));
                            let mut points = vec![
                                tri_rect.left_top(),
                                tri_rect.right_top(),
                                tri_rect.center_bottom(),
                            ];
                            use std::f32::consts::TAU;
                            let rotation = egui::emath::Rot2::from_angle(egui::emath::remap(
                                openness,
                                0.0..=1.0,
                                -TAU / 4.0..=0.0,
                            ));
                            for p in &mut points {
                                *p = center + rotation * (*p - center);
                            }
                            let color = ui.visuals().strong_text_color();
                            ui.painter().add(egui::Shape::convex_polygon(
                                points,
                                color,
                                egui::Stroke::NONE,
                            ));
                        }
                        TreeLineKind::File => {
                            let cx = icon_x + 5.0;
                            let cy = rect.center().y;
                            let s = 4.0;
                            let play_color = if selected || resp.hovered() {
                                ui.visuals().selection.bg_fill
                            } else {
                                ui.visuals().weak_text_color()
                            };
                            ui.painter().add(egui::Shape::convex_polygon(
                                vec![
                                    egui::pos2(cx - s * 0.5, cy - s),
                                    egui::pos2(cx + s, cy),
                                    egui::pos2(cx - s * 0.5, cy + s),
                                ],
                                play_color,
                                egui::Stroke::NONE,
                            ));
                        }
                    }

                    let color = if selected {
                        ui.visuals().selection.stroke.color
                    } else if entry.is_default
                        && matches!(entry.kind, TreeLineKind::Root { .. })
                    {
                        ui.visuals().weak_text_color()
                    } else if is_file {
                        ui.visuals().text_color()
                    } else {
                        ui.visuals().strong_text_color()
                    };

                    ui.painter().text(
                        egui::pos2(text_x, rect.min.y + 1.0),
                        egui::Align2::LEFT_TOP,
                        &entry.label,
                        egui::FontId::monospace(12.0),
                        color,
                    );

                    if resp.clicked() {
                        new_cursor = Some(i);
                        if !is_file {
                            should_toggle = true;
                        } else {
                            clicked_file = Some((entry.folder.clone(), entry.index));
                        }
                    }

                    if selected && cursor_changed {
                        resp.scroll_to_me(None);
                    }
                }
            });

        if let Some(cursor) = new_cursor {
            state.cursor = cursor;
            if should_toggle {
                state.toggle_expand();
            }
        }

        // Handle file click
        if let Some((folder, index)) = clicked_file {
            preview_request = Some((folder, index));
        }

        // Handle keyboard activate
        if activate {
            let info = state
                .current_entry()
                .map(|e| (e.kind.clone(), e.folder.clone(), e.index));
            if let Some((kind, folder, index)) = info {
                match &kind {
                    TreeLineKind::Folder { .. } | TreeLineKind::Root { .. } => {
                        state.toggle_expand();
                    }
                    TreeLineKind::File => {
                        preview_request = Some((folder, index));
                    }
                }
            }
        }

        // Waveform preview
        if self.preview.is_some() || self.pending_key.is_some() {
            ui.separator();
        }

        if let Some(ref preview) = self.preview {
            let ch_label = if preview.channels == 1 {
                t!("sample_browser.mono")
            } else {
                t!("sample_browser.stereo")
            };
            let r = ui.label(format!("{:.1}s · {}", preview.duration_secs, ch_label));
            if r.hovered() {
                crate::widgets::hint::set(ui.ctx(), t!("audio.hint.sample_info"));
            }

            let color = ui.visuals().selection.bg_fill;
            ui.add_space(2.0);
            let click_pos = Waveform::new(&preview.mono_samples, color)
                .normalize(true)
                .num_bins(512)
                .cursor(Some(self.begin as f32))
                .show(ui);
            if let Some(pos) = click_pos {
                self.begin = pos as f64;
                seek_request = true;
            }
        } else if self.pending_key.is_some() {
            ui.spinner();
        }

        // Process file click: decode waveform + play from beginning
        if let Some((folder, index)) = preview_request {
            let found = self.state.as_ref().and_then(|state| {
                state
                    .entries()
                    .iter()
                    .find(|e| {
                        matches!(e.kind, TreeLineKind::File)
                            && e.folder == folder
                            && e.index == index
                    })
                    .cloned()
            });
            if let Some(entry) = found {
                self.begin = 0.0;
                self.trigger_preview(&entry, sample_paths, ctx);
                if is_hosting && bridge.is_connected() {
                    bridge.send(ClientMessage::PreviewSample {
                        folder: entry.folder,
                        index: entry.index,
                        begin: 0.0,
                    });
                }
            }
        }

        // Process waveform seek: replay current sample from clicked position
        if seek_request && is_hosting && bridge.is_connected() {
            let found = self.preview.as_ref().and_then(|preview| {
                let parts: Vec<&str> = preview.key.splitn(2, ':').collect();
                if parts.len() != 2 {
                    return None;
                }
                self.state.as_ref().and_then(|state| {
                    state
                        .entries()
                        .iter()
                        .find(|e| {
                            matches!(e.kind, TreeLineKind::File)
                                && e.folder == parts[0]
                                && e.label == parts[1]
                        })
                        .map(|e| (e.folder.clone(), e.index))
                })
            });
            if let Some((folder, index)) = found {
                bridge.send(ClientMessage::PreviewSample {
                    folder,
                    index,
                    begin: self.begin,
                });
            }
        }
    }

    fn show_embedded(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        sample_paths: &[PathBuf],
        is_hosting: bool,
    ) {
        let mut open = self.open;
        egui::Window::new(t!("sample_browser.title"))
            .open(&mut open)
            .resizable(true)
            .collapsible(true)
            .default_size([300.0, 500.0])
            .show(ctx, |ui| {
                self.browser_content(ui, ctx, bridge, sample_paths, true, is_hosting);
            });
        self.open = open;
    }

    fn show_detached(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        sample_paths: &[PathBuf],
        appearance: &AppearanceSettings,
        is_hosting: bool,
    ) {
        let mut open = self.open;
        let mut detached = self.detached;
        widgets::show_detached_viewport(
            ctx,
            &mut open,
            &mut detached,
            "sample_browser_viewport",
            &t!("sample_browser.detached_title"),
            [300.0, 500.0],
            appearance,
            |ui| self.browser_content(ui, ctx, bridge, sample_paths, false, is_hosting),
        );
        self.open = open;
        self.detached = detached;
    }

    fn trigger_preview(
        &mut self,
        entry: &crate::sample_browser::TreeLine,
        sample_paths: &[PathBuf],
        ctx: &egui::Context,
    ) {
        let key = format!("{}:{}", entry.folder, entry.label);

        if self.preview.as_ref().is_some_and(|p| p.key == key) {
            return;
        }
        if self.pending_key.as_ref().is_some_and(|k| k == &key) {
            return;
        }

        let mut all_paths: Vec<PathBuf> = Vec::new();
        if let Some(dp) = &self.default_path {
            all_paths.push(dp.clone());
        }
        all_paths.extend_from_slice(sample_paths);

        let Some(path) = resolve_sample_path(&all_paths, entry) else {
            return;
        };

        let (tx, rx) = mpsc::channel();
        self.decode_rx = Some(rx);
        self.pending_key = Some(key.clone());

        let ctx = ctx.clone();
        std::thread::spawn(
            move || match doux::sampling::decode_sample_file(&path, 44100.0) {
                Ok(data) => {
                    let channels = data.channels;
                    let frame_count = data.frame_count as usize;
                    let mono = if channels > 1 {
                        let ch = channels as usize;
                        (0..frame_count)
                            .map(|i| {
                                let start = i * ch;
                                let end = (start + ch).min(data.frames.len());
                                data.frames[start..end].iter().sum::<f32>() / ch as f32
                            })
                            .collect()
                    } else {
                        data.frames.to_vec()
                    };
                    let duration = frame_count as f32 / 44100.0;
                    let _ = tx.send(DecodeResult {
                        key,
                        mono_samples: mono,
                        channels,
                        duration_secs: duration,
                    });
                    ctx.request_repaint();
                }
                Err(e) => {
                    eprintln!("Failed to decode sample: {e}");
                }
            },
        );
    }
}

fn handle_keyboard(
    ui: &mut egui::Ui,
    state: &mut SampleBrowserState,
    search_focused: bool,
) -> (bool, bool) {
    let search_id = ui.id().with("sample_search");
    let other_focus = ui.memory(|m| m.focused().is_some_and(|id| id != search_id));
    if other_focus {
        return (false, false);
    }

    let mut activate = false;
    let mut focus_search = false;

    ui.input(|i| {
        if search_focused {
            if i.key_pressed(egui::Key::Escape) {
                state.clear_search();
                ui.memory_mut(|m| m.surrender_focus(search_id));
            }
            return;
        }

        let ctrl = i.modifiers.command;

        if i.key_pressed(egui::Key::ArrowUp) || i.key_pressed(egui::Key::K) {
            let n = if ctrl { 10 } else { 1 };
            state.move_up(n);
        }
        if i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::J) {
            let n = if ctrl { 10 } else { 1 };
            state.move_down(n, 30);
        }
        if i.key_pressed(egui::Key::PageUp) {
            state.move_up(20);
        }
        if i.key_pressed(egui::Key::PageDown) {
            state.move_down(20, 30);
        }
        if i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::ArrowRight) {
            activate = true;
        }
        if i.key_pressed(egui::Key::ArrowLeft) {
            state.collapse_at_cursor();
        }
        if i.key_pressed(egui::Key::Slash) {
            focus_search = true;
        }
        if i.key_pressed(egui::Key::Escape) && !state.search_query.is_empty() {
            state.clear_filter();
        }
    });

    (activate, focus_search)
}

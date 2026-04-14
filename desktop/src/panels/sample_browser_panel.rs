mod keyboard;
mod preview;
mod tree;

use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui;

use crate::InputOwner;
use crate::client_bridge::ClientBridge;
use crate::sample_browser::{SampleBrowserState, TreeLineKind};
use crate::settings::AppearanceSettings;
use crate::widgets::{self, Waveform};
use sova_server::ClientMessage;

use preview::{DecodeResult, PreviewData};

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
        self.state
            .as_ref()
            .map(|s| s.tree.sample_names())
            .unwrap_or_default()
    }

    /// Updates the sample tree and polls the background decode channel.
    /// Must be called every frame the browser is active, before rendering content.
    pub fn poll(&mut self, default_path: Option<&std::path::Path>, sample_paths: &[PathBuf]) {
        let dp_changed = self.default_path.as_deref() != default_path;
        if self.last_paths != sample_paths || dp_changed {
            self.last_paths = sample_paths.to_vec();
            self.default_path = default_path.map(PathBuf::from);
            if sample_paths.is_empty() && default_path.is_none() {
                self.state = None;
            } else {
                self.state = Some(SampleBrowserState::new(default_path, sample_paths));
            }
            self.preview = None;
        }

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
    }

    /// Renders only the detached OS viewport when `detached` is true.
    /// Called from the floating windows section instead of `show()`.
    pub fn show_detached_only(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        default_path: Option<&std::path::Path>,
        sample_paths: &[PathBuf],
        appearance: &AppearanceSettings,
        is_hosting: bool,
    ) {
        if !self.detached {
            return;
        }
        if !is_hosting && bridge.is_connected() {
            self.detached = false;
            return;
        }
        self.poll(default_path, sample_paths);
        self.open = true;
        // Detached viewport has no panel conflicts — it always owns its input.
        self.show_detached(
            ctx,
            bridge,
            sample_paths,
            appearance,
            is_hosting,
            InputOwner::SampleBrowser,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn browser_content(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        sample_paths: &[PathBuf],
        show_popout: bool,
        is_hosting: bool,
        input_owner: InputOwner,
    ) {
        let Some(state) = &mut self.state else {
            ui.colored_label(egui::Color32::GRAY, t!("sample_browser.no_paths"));
            return;
        };

        let search_id = ui.id().with("sample_search");
        let mut search_resp = None;
        egui::TopBottomPanel::bottom(ui.id().with("sample_search_bar"))
            .show_separator_line(true)
            .show_inside(ui, |ui| {
                search_resp = Some(
                    ui.horizontal(|ui| {
                        if show_popout {
                            let r = ui
                                .button(crate::icons::rich(crate::icons::POPOUT))
                                .on_hover_text(t!("common.pop_out"));
                            if r.hovered() {
                                crate::widgets::hint::set(
                                    ui.ctx(),
                                    t!("sample_browser.hint.detach"),
                                );
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
                    .inner,
                );
            });
        let Some(search_resp) = search_resp else {
            return;
        };
        if search_resp.changed() {
            state.update_search();
        }

        let search_focused = search_resp.has_focus();

        // Handle keyboard input
        let prev_cursor = state.cursor;
        let (activate, focus_search) =
            keyboard::handle_keyboard(ui, state, search_focused, input_owner);
        let cursor_changed = state.cursor != prev_cursor;
        if focus_search {
            search_resp.request_focus();
        }

        let row_height = 18.0;
        let avail = ui.available_height() - 80.0;
        let visible_rows = (avail / row_height).max(5.0) as usize;

        let mut preview_request: Option<(String, usize)> = None;
        let mut seek_request = false;

        let tree = tree::render_tree(ui, state, row_height, visible_rows, cursor_changed);

        if let Some(cursor) = tree.new_cursor {
            state.cursor = cursor;
            if tree.should_toggle {
                state.toggle_expand();
            }
        }

        if let Some((folder, index)) = tree.clicked_file {
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

    fn show_detached(
        &mut self,
        ctx: &egui::Context,
        bridge: &ClientBridge,
        sample_paths: &[PathBuf],
        appearance: &AppearanceSettings,
        is_hosting: bool,
        input_owner: InputOwner,
    ) {
        let mut open = self.open;
        let mut detached = self.detached;
        widgets::show_detached_viewport(
            ctx,
            &mut open,
            &mut detached,
            &t!("sample_browser.detached_title"),
            [300.0, 500.0],
            appearance,
            |ui| {
                self.browser_content(
                    ui,
                    ctx,
                    bridge,
                    sample_paths,
                    false,
                    is_hosting,
                    input_owner,
                )
            },
        );
        self.open = open;
        self.detached = detached;
    }

}

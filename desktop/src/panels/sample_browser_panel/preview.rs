use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui;
use sova_core::log_eprintln;

use crate::sample_browser::resolve_sample_path;

#[derive(Clone)]
pub(super) struct PreviewData {
    pub key: String,
    pub mono_samples: Vec<f32>,
    pub channels: u8,
    pub duration_secs: f32,
}

pub(super) struct DecodeResult {
    pub key: String,
    pub mono_samples: Vec<f32>,
    pub channels: u8,
    pub duration_secs: f32,
}

impl super::SampleBrowserPanel {
    pub(super) fn trigger_preview(
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
                    log_eprintln!("Failed to decode sample: {e}");
                }
            },
        );
    }
}

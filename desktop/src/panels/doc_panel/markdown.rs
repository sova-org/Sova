use eframe::egui;
use egui::text::{LayoutJob, LayoutSection, TextWrapping};
use egui::TextFormat;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use sova_core::scene::script::Script;
use sova_core::schedule::SchedulerMessage;
use sova_server::ClientMessage;

use crate::widgets::syntax_highlight::{CompiledSyntax, SyntaxTheme};

use super::{find_clicked_hook, MarkdownRunner};

fn is_runnable(info: &str, lang: &str) -> bool {
    let tag = info.trim().to_ascii_lowercase();
    if tag.is_empty() {
        return true;
    }
    if tag == lang {
        return true;
    }
    // Opt-out tags for output samples, ASCII diagrams, etc.
    if matches!(tag.as_str(), "text" | "txt" | "output") {
        return false;
    }
    // Aliases: existing Cagire docs use ```forth fences.
    matches!((lang, tag.as_str()), ("cagire", "forth"))
}

/// Render a green/red status pill for a snippet run result.
pub(crate) fn show_run_status_pill(ui: &mut egui::Ui, status: &Result<String, String>) {
    ui.add_space(4.0);
    let (bg, fg, text) = match status {
        Ok(s) => (
            egui::Color32::from_rgb(20, 40, 20),
            egui::Color32::from_rgb(120, 220, 120),
            s,
        ),
        Err(s) => (
            egui::Color32::from_rgb(50, 20, 20),
            egui::Color32::from_rgb(220, 100, 100),
            s,
        ),
    };
    egui::Frame::NONE.fill(bg).inner_margin(6.0).show(ui, |ui| {
        ui.colored_label(fg, text);
    });
}

/// Render markdown with syntax-highlighted code blocks.
/// Splits on ``` fences, renders prose via CommonMarkViewer and code blocks
/// as syntax-highlighted labels in a dark frame.
/// Returns the slug of the first clicked cross-reference link, if any.
pub(crate) fn show_highlighted_markdown(
    ui: &mut egui::Ui,
    cache: &mut CommonMarkCache,
    md: &str,
    syntax: Option<&CompiledSyntax>,
    theme: &SyntaxTheme,
    mut runner: Option<&mut MarkdownRunner<'_>>,
) -> Option<String> {
    let font_id = egui::FontId::monospace(13.0);
    let text_color = ui.visuals().text_color();
    let bg = ui.visuals().extreme_bg_color;

    let mut clicked_link: Option<String> = None;
    let mut rest = md;
    let mut section_id = 0u32;
    while let Some(fence_start) = rest.find("```") {
        let prose = &rest[..fence_start];
        if !prose.trim().is_empty() {
            ui.push_id(section_id, |ui| {
                CommonMarkViewer::new().show(ui, cache, prose);
            });
            section_id += 1;
            if clicked_link.is_none() {
                clicked_link = find_clicked_hook(cache);
            }
        }

        // Skip the opening ``` and capture the optional info string up to the newline
        let after_fence = &rest[fence_start + 3..];
        let (info, after_tag) = match after_fence.find('\n') {
            Some(nl) => (&after_fence[..nl], &after_fence[nl + 1..]),
            None => {
                // Malformed: no closing fence
                rest = after_fence;
                continue;
            }
        };

        // Find closing ```
        let (code, remainder) = match after_tag.find("```") {
            Some(end) => {
                let code = &after_tag[..end];
                let skip = end + 3;
                let rem = &after_tag[skip..];
                // Skip trailing newline after closing fence
                let rem = rem.strip_prefix('\n').unwrap_or(rem);
                (code, rem)
            }
            None => {
                // No closing fence: treat remainder as code
                (after_tag, "")
            }
        };

        let code = code.strip_suffix('\n').unwrap_or(code);

        let runnable = runner
            .as_deref()
            .is_some_and(|r| is_runnable(info, r.lang_name));
        let connected = runner.as_deref().is_some_and(|r| r.bridge.is_connected());

        ui.add_space(6.0);
        let frame_response = egui::Frame::NONE
            .fill(bg)
            .inner_margin(egui::Margin {
                left: 12,
                right: 8,
                top: 8,
                bottom: 8,
            })
            .show(ui, |ui| {
                let job = build_highlighted_job(code, &font_id, text_color, syntax, theme);
                ui.add(egui::Label::new(job).selectable(true));
                if runnable {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn = egui::Button::new(t!("doc.run").as_ref());
                            if ui.add_enabled(connected, btn).clicked()
                                && let Some(r) = runner.as_deref_mut()
                            {
                                r.bridge.send(ClientMessage::SchedulerControl(
                                    SchedulerMessage::RunSnippet(
                                        Script::new(code.to_owned(), r.lang_name.to_owned()),
                                        1.0,
                                    ),
                                ));
                            }
                        });
                    });
                }
            });
        let rect = frame_response.response.rect;
        let accent = ui.visuals().selection.bg_fill;
        ui.painter().line_segment(
            [rect.left_top(), rect.left_bottom()],
            egui::Stroke::new(3.0, accent),
        );

        ui.add_space(6.0);

        rest = remainder;
    }

    // Remaining prose after last code block
    if !rest.trim().is_empty() {
        ui.push_id(section_id, |ui| {
            CommonMarkViewer::new().show(ui, cache, rest);
        });
        if clicked_link.is_none() {
            clicked_link = find_clicked_hook(cache);
        }
    }

    clicked_link
}

fn build_highlighted_job(
    code: &str,
    font_id: &egui::FontId,
    text_color: egui::Color32,
    syntax: Option<&CompiledSyntax>,
    theme: &SyntaxTheme,
) -> LayoutJob {
    let default_fmt = TextFormat::simple(font_id.clone(), text_color);
    let mut job = LayoutJob {
        text: code.to_owned(),
        wrap: TextWrapping {
            max_width: f32::INFINITY,
            ..Default::default()
        },
        ..Default::default()
    };

    if let Some(cs) = syntax {
        let mut pos = 0;
        for (range, cat) in cs.tokenize(code) {
            if range.start > pos {
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: pos..range.start,
                    format: default_fmt.clone(),
                });
            }
            job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: range.clone(),
                format: TextFormat::simple(font_id.clone(), theme.color(cat)),
            });
            pos = range.end;
        }
        if pos < code.len() {
            job.sections.push(LayoutSection {
                leading_space: 0.0,
                byte_range: pos..code.len(),
                format: default_fmt,
            });
        }
    } else {
        job.sections.push(LayoutSection {
            leading_space: 0.0,
            byte_range: 0..code.len(),
            format: default_fmt,
        });
    }

    job
}

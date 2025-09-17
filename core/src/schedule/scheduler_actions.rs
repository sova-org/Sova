use crate::{
    log_eprintln,
    scene::{Scene, Line, script::Script},
    schedule::{
        message::SchedulerMessage, notification::SchedulerNotification,
        scheduler_state::DuplicatedFrameData,
    },
    transcoder::Transcoder,
};
use crossbeam_channel::Sender;
use std::sync::Arc;

pub struct ActionProcessor;

impl ActionProcessor {
    pub fn process_scene_modifications(
        action: SchedulerMessage,
        scene: &mut Scene,
        update_notifier: &Sender<SchedulerNotification>,
        transcoder: &Transcoder,
    ) -> bool {
        match action {
            SchedulerMessage::EnableFrames(line, frames, _) => {
                Self::enable_frames(scene, line, &frames, update_notifier);
                true
            }
            SchedulerMessage::DisableFrames(line, frames, _) => {
                Self::disable_frames(scene, line, &frames, update_notifier);
                true
            }
            SchedulerMessage::UploadScript(line, frame, script, _) => {
                Self::upload_script(scene, line, frame, script, transcoder, update_notifier);
                true
            }
            SchedulerMessage::UpdateLineFrames(line, vec, _) => {
                scene.add_line_if_empty();
                scene.mut_line(line).unwrap().set_frames(vec);
                let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
                true
            }
            SchedulerMessage::InsertFrame(line, position, value, _) => {
                Self::insert_frame(scene, line, position, value, update_notifier);
                true
            }
            SchedulerMessage::RemoveFrame(line, position, _) => {
                Self::remove_frame(scene, line, position, update_notifier);
                true
            }
            SchedulerMessage::RemoveLine(index, _) => {
                Self::remove_line(scene, index, update_notifier);
                true
            }
            SchedulerMessage::SetLine(index, line, _) => {
                Self::set_line(scene, index, line, update_notifier);
                true
            }
            SchedulerMessage::SetLineStartFrame(line_index, start_frame, _) => {
                Self::set_line_start_frame(scene, line_index, start_frame, update_notifier);
                true
            }
            SchedulerMessage::SetLineEndFrame(line_index, end_frame, _) => {
                Self::set_line_end_frame(scene, line_index, end_frame, update_notifier);
                true
            }
            SchedulerMessage::SetLineLength(line_idx, length_opt, _) => {
                Self::set_line_length(scene, line_idx, length_opt, update_notifier);
                true
            }
            SchedulerMessage::SetLineSpeedFactor(line_idx, speed_factor, _) => {
                Self::set_line_speed_factor(scene, line_idx, speed_factor, update_notifier);
                true
            }
            SchedulerMessage::AddLine => {
                let new_line = Line::new(vec![1.0]);
                Self::add_line(scene, new_line, update_notifier);
                true
            }
            SchedulerMessage::InternalDuplicateFrame {
                target_line_idx,
                target_insert_idx,
                frame_length,
                is_enabled,
                script: script_arc_opt,
                timing: _,
            } => {
                Self::duplicate_frame(
                    scene,
                    target_line_idx,
                    target_insert_idx,
                    frame_length,
                    is_enabled,
                    script_arc_opt,
                    update_notifier,
                );
                true
            }
            SchedulerMessage::InternalDuplicateFrameRange {
                target_line_idx,
                target_insert_idx,
                frames_data,
                timing: _,
            } => {
                Self::duplicate_frame_range(
                    scene,
                    target_line_idx,
                    target_insert_idx,
                    frames_data,
                    update_notifier,
                );
                true
            }
            SchedulerMessage::InternalRemoveFramesMultiLine {
                lines_and_indices,
                timing: _,
            } => {
                Self::remove_frames_multi_line(scene, lines_and_indices, update_notifier);
                true
            }
            SchedulerMessage::InternalInsertDuplicatedBlocks {
                duplicated_data,
                target_line_idx,
                target_frame_idx,
                timing: _,
            } => {
                Self::insert_duplicated_blocks(
                    scene,
                    duplicated_data,
                    target_line_idx,
                    target_frame_idx,
                    update_notifier,
                );
                true
            }
            SchedulerMessage::SetFrameName(line_idx, frame_idx, name, _) => {
                Self::set_frame_name(scene, line_idx, frame_idx, name, update_notifier);
                true
            }
            SchedulerMessage::SetScriptLanguage(line_idx, frame_idx, lang, _) => {
                Self::set_script_language(
                    scene,
                    line_idx,
                    frame_idx,
                    lang,
                    transcoder,
                    update_notifier,
                );
                true
            }
            SchedulerMessage::SetFrameRepetitions(line_idx, frame_idx, repetitions, _) => {
                Self::set_frame_repetitions(
                    scene,
                    line_idx,
                    frame_idx,
                    repetitions,
                    update_notifier,
                );
                true
            }
            _ => false,
        }
    }

    fn enable_frames(
        scene: &mut Scene,
        line_idx: usize,
        frames: &[usize],
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        line.enable_frames(frames);
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn disable_frames(
        scene: &mut Scene,
        line_idx: usize,
        frames: &[usize],
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        line.disable_frames(frames);
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn upload_script(
        scene: &mut Scene,
        line_idx: usize,
        frame: usize,
        mut script: Script,
        transcoder: &Transcoder,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        scene.add_line_if_empty();
        if transcoder.has_compiler(script.lang()) {
            transcoder.compile_script(&mut script);
        }
        let line = scene.mut_line(line_idx).unwrap();
        line.set_script(frame, script.clone());
        // let _ = update_notifier.send(SchedulerNotification::UploadedScript(
        //     line_idx, frame, script,
        // ));
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn insert_frame(
        scene: &mut Scene,
        line_idx: usize,
        position: usize,
        value: f64,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        scene.add_line_if_empty();
        scene
            .mut_line(line_idx)
            .unwrap()
            .insert_frame(position, value);
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn remove_frame(
        scene: &mut Scene,
        line_idx: usize,
        position: usize,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        line.remove_frame(position);
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn remove_line(
        scene: &mut Scene,
        index: usize,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        scene.remove_line(index);
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn add_line(scene: &mut Scene, line: Line, update_notifier: &Sender<SchedulerNotification>) {
        scene.add_line(line);
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn set_line(
        scene: &mut Scene,
        index: usize,
        line: Line,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        scene.set_line(index, line);
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn set_line_start_frame(
        scene: &mut Scene,
        line_idx: usize,
        start_frame: Option<usize>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        line.start_frame = start_frame;
        line.make_consistent();
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn set_line_end_frame(
        scene: &mut Scene,
        line_idx: usize,
        end_frame: Option<usize>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        line.end_frame = end_frame;
        line.make_consistent();
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn set_line_length(
        scene: &mut Scene,
        line_idx: usize,
        length_opt: Option<f64>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        line.custom_length = length_opt;
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn set_line_speed_factor(
        scene: &mut Scene,
        line_idx: usize,
        speed_factor: f64,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        line.speed_factor = if speed_factor > 0.0 {
            speed_factor
        } else {
            1.0
        };
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn duplicate_frame(
        scene: &mut Scene,
        target_line_idx: usize,
        target_insert_idx: usize,
        frame_length: f64,
        is_enabled: bool,
        script_arc_opt: Option<Arc<Script>>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(target_line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        line.insert_frame(target_insert_idx, frame_length);
        if is_enabled {
            line.enable_frame(target_insert_idx);
        } else {
            line.disable_frame(target_insert_idx);
        }
        if let Some(script_arc) = script_arc_opt {
            let mut script_to_insert = (*script_arc).clone();
            script_to_insert.index = target_insert_idx;
            line.set_script(target_insert_idx, script_to_insert);
        } else {
            line.set_script(target_insert_idx, Default::default());
        }
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn duplicate_frame_range(
        scene: &mut Scene,
        target_line_idx: usize,
        target_insert_idx: usize,
        frames_data: Vec<DuplicatedFrameData>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(target_line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        let mut current_insert_idx = target_insert_idx;
        for frame_data in frames_data {
            line.insert_frame(current_insert_idx, frame_data.length);
            if frame_data.is_enabled {
                line.enable_frame(current_insert_idx);
            } else {
                line.disable_frame(current_insert_idx);
            }
            if let Some(script_arc) = frame_data.script {
                let mut script_to_insert = (*script_arc).clone();
                script_to_insert.index = current_insert_idx;
                line.set_script(current_insert_idx, script_to_insert);
            } else {
                line.set_script(current_insert_idx, Script::default());
            }
            line.set_frame_name(current_insert_idx, frame_data.name);
            line.frame_repetitions[current_insert_idx] = frame_data.repetitions.max(1);
            current_insert_idx += 1;
        }
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn remove_frames_multi_line(
        scene: &mut Scene,
        lines_and_indices: Vec<(usize, Vec<usize>)>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let mut any_modification = false;
        for (line_idx, frames) in lines_and_indices {
            if let Some(line) = scene.lines.get_mut(line_idx) {
                let current_n_frames = line.n_frames();
                let requested_to_remove = frames.len();

                if current_n_frames > 0 && requested_to_remove >= current_n_frames {
                    log_eprintln!(
                        "[!] Scheduler: Denied removing {} frames from line {} (would empty line).",
                        requested_to_remove,
                        line_idx
                    );
                    continue;
                }

                let mut indices_to_remove = frames.clone();
                indices_to_remove.sort_unstable_by(|a, b| b.cmp(a));

                for index in indices_to_remove {
                    if index < line.n_frames() {
                        line.remove_frame(index);
                        any_modification = true;
                    } else {
                        log_eprintln!(
                            "[!] Scheduler: InternalRemoveFramesMultiLine attempted to remove invalid index {} from line {}",
                            index,
                            line_idx
                        );
                    }
                }

                if any_modification {
                    line.make_consistent();
                }
            } else {
                log_eprintln!(
                    "[!] Scheduler: InternalRemoveFramesMultiLine received for invalid line index {}",
                    line_idx
                );
            }
        }

        if any_modification {
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        }
    }

    fn insert_duplicated_blocks(
        scene: &mut Scene,
        duplicated_data: Vec<Vec<DuplicatedFrameData>>,
        target_line_idx: usize,
        target_frame_idx: usize,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let mut any_modification = false;
        for (col_offset, column_data) in duplicated_data.into_iter().enumerate() {
            let current_target_line_idx = target_line_idx + col_offset;

            if current_target_line_idx < scene.lines.len() {
                if let Some(line) = scene.lines.get_mut(current_target_line_idx) {
                    let mut current_insert_idx = target_frame_idx;
                    for frame_data in column_data {
                        line.insert_frame(current_insert_idx, frame_data.length);
                        if frame_data.is_enabled {
                            line.enable_frame(current_insert_idx);
                        } else {
                            line.disable_frame(current_insert_idx);
                        }
                        if let Some(script_arc) = frame_data.script {
                            let mut script_to_insert = (*script_arc).clone();
                            script_to_insert.index = current_insert_idx;
                            line.set_script(current_insert_idx, script_to_insert);
                        } else {
                            line.set_script(current_insert_idx, Default::default());
                        }
                        line.set_frame_name(current_insert_idx, frame_data.name);
                        line.frame_repetitions[current_insert_idx] = frame_data.repetitions.max(1);
                        current_insert_idx += 1;
                        any_modification = true;
                    }
                }
            } else {
                log_eprintln!(
                    "[!] Scheduler: InternalInsertDuplicatedBlocks skipped invalid target line index {}",
                    current_target_line_idx
                );
            }
        }

        if any_modification {
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        }
    }

    fn set_frame_name(
        scene: &mut Scene,
        line_idx: usize,
        frame_idx: usize,
        name: Option<String>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        line.set_frame_name(frame_idx, name);
        let _ = update_notifier.send(SchedulerNotification::UpdatedLine(line_idx, line.clone()));
    }

    fn set_script_language(
        scene: &mut Scene,
        line_idx: usize,
        frame_idx: usize,
        lang: String,
        transcoder: &Transcoder,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        if let Some(script) = line.scripts.get_mut(frame_idx) {
            let script_mut = Arc::make_mut(script);
            script_mut.set_lang(lang);
            if transcoder.has_compiler(script_mut.lang()) {
                transcoder.compile_script(script_mut);
            }
            let _ =
                update_notifier.send(SchedulerNotification::UpdatedLine(line_idx, line.clone()));
        } else {
            log_eprintln!(
                "[!] Scheduler::set_script_language: Script not found for frame {} in line {}",
                frame_idx,
                line_idx
            );
        }
    }

    fn set_frame_repetitions(
        scene: &mut Scene,
        line_idx: usize,
        frame_idx: usize,
        repetitions: usize,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        let Some(line) = scene.mut_line(line_idx) else {
            log_eprintln!("[!] Scheduler: Scene is empty !");
            return;
        };
        if frame_idx < line.frame_repetitions.len() {
            line.frame_repetitions[frame_idx] = repetitions.max(1);
            let _ =
                update_notifier.send(SchedulerNotification::UpdatedLine(line_idx, line.clone()));
        } else {
            log_eprintln!(
                "[!] Scheduler::set_frame_repetitions: Invalid frame index {} for line {}",
                frame_idx,
                line_idx
            );
        }
    }
}

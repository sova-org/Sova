use std::sync::{Arc, mpsc::Sender};
use crate::{
    scene::{Scene, line::Line, script::Script},
    schedule::{
        message::SchedulerMessage,
        notification::SchedulerNotification,
        scheduler_state::DuplicatedFrameData,
    },
};

pub struct ActionProcessor;

impl ActionProcessor {
    pub fn process_scene_modifications(
        action: SchedulerMessage,
        scene: &mut Scene,
        update_notifier: &Sender<SchedulerNotification>,
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
                Self::upload_script(scene, line, frame, script, update_notifier);
                true
            }
            SchedulerMessage::UpdateLineFrames(line, vec, _) => {
                scene.mut_line(line).set_frames(vec);
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
            SchedulerMessage::SetSceneLength(length, _) => {
                scene.set_length(length);
                let _ = update_notifier.send(SchedulerNotification::SceneLengthChanged(length));
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
                Self::set_script_language(scene, line_idx, frame_idx, lang, update_notifier);
                true
            }
            SchedulerMessage::SetFrameRepetitions(line_idx, frame_idx, repetitions, _) => {
                Self::set_frame_repetitions(scene, line_idx, frame_idx, repetitions, update_notifier);
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
        if let Some(line) = scene.lines.get_mut(line_idx) {
            line.enable_frames(frames);
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: EnableFrames received for invalid line index {}",
                line_idx
            );
        }
    }

    fn disable_frames(
        scene: &mut Scene,
        line_idx: usize,
        frames: &[usize],
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        if let Some(line) = scene.lines.get_mut(line_idx) {
            line.disable_frames(frames);
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: DisableFrames received for invalid line index {}",
                line_idx
            );
        }
    }

    fn upload_script(
        scene: &mut Scene,
        line: usize,
        frame: usize,
        script: Script,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        scene.mut_line(line).set_script(frame, script);
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn insert_frame(
        scene: &mut Scene,
        line_idx: usize,
        position: usize,
        value: f64,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        if let Some(line) = scene.lines.get_mut(line_idx) {
            line.insert_frame(position, value);
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: InsertFrame received for invalid line index {}",
                line_idx
            );
        }
    }

    fn remove_frame(
        scene: &mut Scene,
        line_idx: usize,
        position: usize,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        if let Some(line) = scene.lines.get_mut(line_idx) {
            line.remove_frame(position);
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: RemoveFrame received for invalid line index {}",
                line_idx
            );
        }
    }

    fn remove_line(
        scene: &mut Scene,
        index: usize,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        scene.remove_line(index);
        let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
    }

    fn add_line(
        scene: &mut Scene,
        line: Line,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
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
        line_index: usize,
        start_frame: Option<usize>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        if let Some(line) = scene.lines.get_mut(line_index) {
            line.start_frame = start_frame;
            line.make_consistent();
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: SetLineStartFrame received for invalid line index {}",
                line_index
            );
        }
    }

    fn set_line_end_frame(
        scene: &mut Scene,
        line_index: usize,
        end_frame: Option<usize>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        if let Some(line) = scene.lines.get_mut(line_index) {
            line.end_frame = end_frame;
            line.make_consistent();
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: SetLineEndFrame received for invalid line index {}",
                line_index
            );
        }
    }

    fn set_line_length(
        scene: &mut Scene,
        line_idx: usize,
        length_opt: Option<f64>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        if let Some(line) = scene.lines.get_mut(line_idx) {
            line.custom_length = length_opt;
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: SetLineLength received for invalid line index {}",
                line_idx
            );
        }
    }

    fn set_line_speed_factor(
        scene: &mut Scene,
        line_idx: usize,
        speed_factor: f64,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        if let Some(line) = scene.lines.get_mut(line_idx) {
            line.speed_factor = if speed_factor > 0.0 {
                speed_factor
            } else {
                1.0
            };
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: SetLineSpeedFactor received for invalid line index {}",
                line_idx
            );
        }
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
        if let Some(line) = scene.lines.get_mut(target_line_idx) {
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
                let default_script = Script::new(
                    "".to_string(),
                    Default::default(),
                    "bali".to_string(),
                    target_insert_idx,
                );
                line.set_script(target_insert_idx, default_script);
            }
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: InternalDuplicateFrame received for invalid line index {}",
                target_line_idx
            );
        }
    }

    fn duplicate_frame_range(
        scene: &mut Scene,
        target_line_idx: usize,
        target_insert_idx: usize,
        frames_data: Vec<DuplicatedFrameData>,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        if let Some(line) = scene.lines.get_mut(target_line_idx) {
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
                    let default_script = Script::new(
                        "".to_string(),
                        Default::default(),
                        "bali".to_string(),
                        current_insert_idx,
                    );
                    line.set_script(current_insert_idx, default_script);
                }
                line.set_frame_name(current_insert_idx, frame_data.name);
                line.frame_repetitions[current_insert_idx] = frame_data.repetitions.max(1);
                current_insert_idx += 1;
            }
            let _ = update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: InternalDuplicateFrameRange received for invalid line index {}",
                target_line_idx
            );
        }
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
                    eprintln!(
                        "[!] Scheduler: Denied removing {} frames from line {} (would empty line).",
                        requested_to_remove, line_idx
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
                        eprintln!(
                            "[!] Scheduler: InternalRemoveFramesMultiLine attempted to remove invalid index {} from line {}",
                            index, line_idx
                        );
                    }
                }

                if any_modification {
                    line.make_consistent();
                }
            } else {
                eprintln!(
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
                            let default_script = Script::new(
                                "".to_string(),
                                Default::default(),
                                "bali".to_string(),
                                current_insert_idx,
                            );
                            line.set_script(current_insert_idx, default_script);
                        }
                        line.set_frame_name(current_insert_idx, frame_data.name);
                        line.frame_repetitions[current_insert_idx] = frame_data.repetitions.max(1);
                        current_insert_idx += 1;
                        any_modification = true;
                    }
                }
            } else {
                eprintln!(
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
        if let Some(line) = scene.lines.get_mut(line_idx) {
            line.set_frame_name(frame_idx, name.clone());
            let _ = update_notifier.send(SchedulerNotification::UpdatedLine(line_idx, line.clone()));
        } else {
            eprintln!(
                "[!] Scheduler::set_frame_name: Invalid line index {}",
                line_idx
            );
        }
    }

    fn set_script_language(
        scene: &mut Scene,
        line_idx: usize,
        frame_idx: usize,
        lang: String,
        update_notifier: &Sender<SchedulerNotification>,
    ) {
        if let Some(line) = scene.lines.get_mut(line_idx) {
            if let Some(script_pos) = line.scripts.iter().position(|s| s.index == frame_idx) {
                let mut script_clone = (*line.scripts[script_pos]).clone();
                script_clone.lang = lang;
                line.scripts[script_pos] = Arc::new(script_clone);
                let _ = update_notifier.send(SchedulerNotification::UpdatedLine(line_idx, line.clone()));
            } else {
                eprintln!(
                    "[!] Scheduler::set_script_language: Script not found for frame {} in line {}",
                    frame_idx, line_idx
                );
            }
        } else {
            eprintln!(
                "[!] Scheduler::set_script_language: Invalid line index {}",
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
        if let Some(line) = scene.lines.get_mut(line_idx) {
            if frame_idx < line.frame_repetitions.len() {
                line.frame_repetitions[frame_idx] = repetitions.max(1);
                let _ = update_notifier.send(SchedulerNotification::UpdatedLine(line_idx, line.clone()));
            } else {
                eprintln!(
                    "[!] Scheduler::set_frame_repetitions: Invalid frame index {} for line {}",
                    frame_idx, line_idx
                );
            }
        } else {
            eprintln!(
                "[!] Scheduler::set_frame_repetitions: Invalid line index {}",
                line_idx
            );
        }
    }
}
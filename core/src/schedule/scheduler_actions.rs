use crate::{
    scene::{Frame, Scene},
    schedule::{
        message::SchedulerMessage, notification::SovaNotification
    },
    lang::Transcoder,
};
use crossbeam_channel::Sender;
use std::collections::BTreeSet;

pub struct ActionProcessor;

impl ActionProcessor {
    pub fn process_scene_modifications(
        action: SchedulerMessage,
        scene: &mut Scene,
        update_notifier: &Sender<SovaNotification>,
        transcoder: &Transcoder,
    ) {
        match action {
            SchedulerMessage::SetLines(lines, _) => {
                let mut updated = lines.clone();
                let mut upd_index = BTreeSet::new();
                let previous_len = scene.n_lines();
                for (i, line) in lines {
                    upd_index.insert(i);
                    scene.set_line(i, line);
                    transcoder.process_line(i, scene.line(i).unwrap());
                }
                for new in previous_len..scene.n_lines() {
                    if upd_index.contains(&new) {
                        continue;
                    }
                    updated.push((new, scene.line(new).unwrap().clone()))
                }
                let _ = update_notifier.send(
                    SovaNotification::UpdatedLines(updated)
                );
            },
            SchedulerMessage::ConfigureLines(mut lines, _) => {
                let mut upd_index = BTreeSet::new();
                let previous_len = scene.n_lines();
                for (i, line) in lines.iter() {
                    upd_index.insert(*i);
                    scene.line_mut(*i).configure(line);
                }
                for new in previous_len..scene.n_lines() {
                    if upd_index.contains(&new) {
                        continue;
                    }
                    lines.push((new, scene.line(new).unwrap().configuration()))
                }
                let _ = update_notifier.send(
                    SovaNotification::UpdatedLineConfigurations(lines)
                );
            },
            SchedulerMessage::AddLine(i, line, _) => {
                scene.insert_line(i, line.clone());
                transcoder.process_line(i, scene.line(i).unwrap());
                let _ = update_notifier.send(
                    SovaNotification::AddedLine(i, line)
                );
            },
            SchedulerMessage::RemoveLine(index, _) => {
                scene.remove_line(index);
                let _ = update_notifier.send(
                    SovaNotification::RemovedLine(index)
                );
            }
            SchedulerMessage::SetFrames(frames, _) => {
                Self::set_frames(scene, frames, update_notifier, transcoder);
            },
            SchedulerMessage::AddFrame(line_id, frame_id, frame, _) => {
                let updated = frame.clone();
                let line = scene.line_mut(line_id);
                line.insert_frame(frame_id, frame);
                transcoder.process_script(line_id, frame_id, line.frame(frame_id).unwrap().script());
                let _ = update_notifier.send(
                    SovaNotification::AddedFrame(line_id, frame_id, updated)
                );
            },
            SchedulerMessage::RemoveFrame(line, position, _) => {
                scene.line_mut(line).remove_frame(position);
                let _ = update_notifier.send(
                    SovaNotification::RemovedFrame(line, position)
                );
            },
            SchedulerMessage::CompilationUpdate(line_id, frame_id, id, state) => {
                if !scene.has_frame(line_id, frame_id) {
                    return;
                }
                if scene.get_frame_mut(line_id, frame_id).update_compilation_state(id, state.clone()) {
                    let _ = update_notifier.send(
                        SovaNotification::CompilationUpdated(line_id, frame_id, id, state)
                    );
                }
            }
            // Handled earlier by scheduler
            SchedulerMessage::TransportStart(_) | SchedulerMessage::TransportStop(_)
            | SchedulerMessage::SetTempo(_, _)
            | SchedulerMessage::SetScene(_, _)
            | SchedulerMessage::DeviceMessage(_, _, _)
            | SchedulerMessage::Shutdown => (),
        }
    }

    fn set_frames(
        scene: &mut Scene,
        frames: Vec<(usize, usize, Frame)>,
        update_notifier: &Sender<SovaNotification>,
        transcoder: &Transcoder,
    ) {
        let mut updated = frames.clone();
        let mut upd_index = BTreeSet::new();
        let previous_lens : Vec<usize> = scene.lines.iter().map(|l| l.n_frames()).collect();
        for (line_id, frame_id, frame) in frames {
            upd_index.insert((line_id, frame_id));
            let line = scene.line_mut(line_id);
            line.set_frame(frame_id, frame);
            transcoder.process_script(line_id, frame_id, line.frame(frame_id).unwrap().script());
        }
        for (line_id, line) in scene.lines.iter().enumerate() {
            for (frame_id, frame) in line.frames.iter().enumerate() {
                if line_id >= previous_lens.len() || frame_id >= previous_lens[line_id] {
                    if upd_index.contains(&(line_id, frame_id)) {
                        continue;
                    }
                    updated.push((line_id, frame_id, frame.clone()));
                }
            }
        }
        let _ = update_notifier.send(
            SovaNotification::UpdatedFrames(updated)
        );
    }

}

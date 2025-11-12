use crate::{
    lang::Transcoder,
    scene::{Frame, Scene},
    schedule::{message::SchedulerMessage, notification::SovaNotification},
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
        feedback: &Sender<SchedulerMessage>,
    ) {
        match action {
            SchedulerMessage::SetLines(lines, _) => {
                let mut updated = lines.clone();
                let mut upd_index = BTreeSet::new();
                let previous_len = scene.n_lines();
                for (i, line) in lines {
                    upd_index.insert(i);
                    scene.set_line(i, line);
                    transcoder.process_line(i, scene.line(i).unwrap(), feedback.clone());
                }
                for new in previous_len..scene.n_lines() {
                    if upd_index.contains(&new) {
                        continue;
                    }
                    updated.push((new, scene.line(new).unwrap().clone()))
                }
                let _ = update_notifier.send(SovaNotification::UpdatedLines(updated));
            }
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
                let _ = update_notifier.send(SovaNotification::UpdatedLineConfigurations(lines));
            }
            SchedulerMessage::AddLine(i, line, _) => {
                scene.insert_line(i, line.clone());
                transcoder.process_line(i, scene.line(i).unwrap(), feedback.clone());
                let _ = update_notifier.send(SovaNotification::AddedLine(i, line));
            }
            SchedulerMessage::RemoveLine(index, _) => {
                scene.remove_line(index);
                let _ = update_notifier.send(SovaNotification::RemovedLine(index));
            }
            SchedulerMessage::GoToFrame(line_id, frame_id, _) => {
                let line = scene.line_mut(line_id);
                line.go_to_frame(frame_id, 0);
                let _ = update_notifier.send(SovaNotification::FramePositionChanged(
                    scene.positions().collect(),
                ));
            }
            SchedulerMessage::SetFrames(frames, _) => {
                Self::set_frames(scene, frames, update_notifier, transcoder, feedback);
            }
            SchedulerMessage::AddFrame(line_id, frame_id, frame, _) => {
                let updated = frame.clone();
                let line = scene.line_mut(line_id);
                let pos = line.position();
                line.insert_frame(frame_id, frame);
                transcoder.process_script(
                    line_id,
                    frame_id,
                    line.frame(frame_id).unwrap().script(),
                    feedback.clone(),
                );
                let _ =
                    update_notifier.send(SovaNotification::AddedFrame(line_id, frame_id, updated));
                if pos != line.position() {
                    let _ = update_notifier.send(SovaNotification::FramePositionChanged(
                        scene.positions().collect(),
                    ));
                }
            }
            SchedulerMessage::RemoveFrame(line_index, position, _) => {
                let line = scene.line_mut(line_index);
                let pos = line.position();
                line.remove_frame(position);
                let _ = update_notifier.send(SovaNotification::RemovedFrame(line_index, position));
                if pos != line.position() {
                    let _ = update_notifier.send(SovaNotification::FramePositionChanged(
                        scene.positions().collect(),
                    ));
                }
            }
            SchedulerMessage::SetScript(line_id, frame_id, script, _) => {
                let frame = scene.get_frame_mut(line_id, frame_id);
                frame.set_script(script);
                transcoder.process_script(line_id, frame_id, frame.script(), feedback.clone());
                let _ = update_notifier.send(SovaNotification::UpdatedFrames(vec![(
                    line_id,
                    frame_id,
                    frame.clone(),
                )]));
            }
            SchedulerMessage::CompilationUpdate(line_id, frame_id, id, state) => {
                if !scene.has_frame(line_id, frame_id) {
                    return;
                }

                let light = state.lightened();

                // Only transmit the status using the notification system, to reduce bandwidth
                let notif = SovaNotification::CompilationUpdated(line_id, frame_id, id, light);

                if scene
                    .get_frame_mut(line_id, frame_id)
                    .update_compilation_state(id, state)
                {
                    let _ = update_notifier.send(notif);
                }
            }
            // Handled earlier by scheduler
            SchedulerMessage::TransportStart(_)
            | SchedulerMessage::TransportStop(_)
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
        feedback: &Sender<SchedulerMessage>,
    ) {
        let mut updated = frames.clone();
        let mut upd_index = BTreeSet::new();
        let previous_lens: Vec<usize> = scene.lines.iter().map(|l| l.n_frames()).collect();
        for (line_id, frame_id, frame) in frames {
            upd_index.insert((line_id, frame_id));
            let line = scene.line_mut(line_id);
            line.set_frame(frame_id, frame);
            transcoder.process_script(
                line_id,
                frame_id,
                line.frame(frame_id).unwrap().script(),
                feedback.clone(),
            );
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
        let _ = update_notifier.send(SovaNotification::UpdatedFrames(updated));
    }
}

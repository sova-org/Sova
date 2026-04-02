use crate::{
    clock::Clock,
    scene::{
        Frame, Scene,
        script::{Script, ScriptExecution},
    },
    schedule::{message::SchedulerMessage, notification::SovaNotification},
    vm::{LanguageCenter, event::ConcreteEvent},
};
use crossbeam_channel::Sender;
use std::collections::BTreeSet;

pub struct ActionProcessor;

impl ActionProcessor {
    pub fn process_scene_modifications(
        action: SchedulerMessage,
        scene: &mut Scene,
        update_notifier: &Sender<SovaNotification>,
        languages: &LanguageCenter,
        feedback: &Sender<SchedulerMessage>,
        clock: &Clock,
        scratchpad: &mut Vec<(ScriptExecution, f64)>,
    ) {
        match action {
            SchedulerMessage::SetScenePrelude(scripts) => {
                scene.prelude = scripts;
                let mut execs = scene
                    .trigger_prelude(languages, clock.micros())
                    .map(|exec| (exec, 1.0))
                    .collect();
                scratchpad.append(&mut execs);
                let _ = update_notifier
                    .send(SovaNotification::UpdatedScenePrelude(scene.prelude.clone()));
            }
            SchedulerMessage::SetLines(lines, _) => {
                let mut updated = lines.clone();
                let mut upd_index = BTreeSet::new();
                let previous_len = scene.n_lines();
                for (i, line) in lines {
                    upd_index.insert(i);
                    scene.set_line(i, line);
                    languages.process_line(i, scene.line(i).unwrap(), feedback.clone());
                }
                for new in previous_len..scene.n_lines() {
                    if upd_index.contains(&new) {
                        continue;
                    }
                    updated.push((new, scene.line(new).unwrap().clone()))
                }
                let _ = update_notifier.send(SovaNotification::UpdatedLines(updated));
            }
            SchedulerMessage::SetSceneMode(mode, _) => {
                scene.mode = mode;
                let _ = update_notifier.send(SovaNotification::UpdatedSceneMode(mode));
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
                languages.process_line(i, scene.line(i).unwrap(), feedback.clone());
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
                Self::set_frames(scene, frames, update_notifier, languages, feedback);
            }
            SchedulerMessage::AddFrame(line_id, frame_id, frame, _) => {
                let updated = frame.clone();
                let line = scene.line_mut(line_id);
                let pos = line.position();
                let script = frame.script().clone();
                line.insert_frame(frame_id, frame);
                languages.process_script(line_id, frame_id, script, feedback.clone());
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
                let frame = scene.frame_mut(line_id, frame_id);
                frame.set_script(script.clone());
                languages.process_script(line_id, frame_id, script, feedback.clone());
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
                    .frame_mut(line_id, frame_id)
                    .update_compilation_state(id, state)
                {
                    let _ = update_notifier.send(notif);
                }
            }
            SchedulerMessage::StartLine(line_id, _) => {
                scene.line_mut(line_id).start();
            }
            SchedulerMessage::StartLineAt(line_id, frame_id, _) => {
                scene.line_mut(line_id).start_at(frame_id);
            }
            SchedulerMessage::GetAnnotations => {
                let _ = update_notifier.send(SovaNotification::Annotations(scene.annotations()));
            }
            // Handled earlier by scheduler
            SchedulerMessage::TransportStart(_)
            | SchedulerMessage::TransportStop(_)
            | SchedulerMessage::SetTempo(_, _)
            | SchedulerMessage::SetQuantum(_, _)
            | SchedulerMessage::SetScene(_, _)
            | SchedulerMessage::DeviceMessage(_, _, _)
            | SchedulerMessage::RunSnippet(_, _)
            | SchedulerMessage::Shutdown => (),
        }
    }

    fn set_frames(
        scene: &mut Scene,
        frames: Vec<(usize, usize, Frame)>,
        update_notifier: &Sender<SovaNotification>,
        languages: &LanguageCenter,
        feedback: &Sender<SchedulerMessage>,
    ) {
        let mut updated = frames.clone();
        let mut upd_index = BTreeSet::new();
        let previous_lens: Vec<usize> = scene.lines.iter().map(|l| l.n_frames()).collect();
        for (line_id, frame_id, frame) in frames {
            upd_index.insert((line_id, frame_id));
            let line = scene.line_mut(line_id);
            let script = frame.script().clone();
            line.set_frame(frame_id, frame);
            languages.process_script(line_id, frame_id, script, feedback.clone());
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

    pub fn process_internal_event(
        scene: &mut Scene,
        event: ConcreteEvent,
        update_notifier: &Sender<SovaNotification>,
        languages: &LanguageCenter,
        feedback: &Sender<SchedulerMessage>,
    ) {
        match event {
            ConcreteEvent::ExecuteFrame(l_i, f_i) => {
                if !scene.has_frame(l_i, f_i) {
                    return;
                }
                scene.line_mut(l_i).start_at(f_i);
                let positions = scene.positions().collect();
                let _ = update_notifier.send(SovaNotification::FramePositionChanged(positions));
            }
            ConcreteEvent::SetFrameEnabled(l_i, f_i, en) => {
                if !scene.has_frame(l_i, f_i) {
                    return;
                }
                let frame = scene.frame_mut(l_i, f_i);
                frame.enabled = en;
                let _ = update_notifier.send(SovaNotification::UpdatedFrames(vec![(
                    l_i,
                    f_i,
                    frame.clone(),
                )]));
            }
            ConcreteEvent::SetFrameDuration(l_i, f_i, dur) => {
                if !scene.has_frame(l_i, f_i) {
                    return;
                }
                let frame = scene.frame_mut(l_i, f_i);
                frame.duration = dur;
                let _ = update_notifier.send(SovaNotification::UpdatedFrames(vec![(
                    l_i,
                    f_i,
                    frame.clone(),
                )]));
            }
            ConcreteEvent::SetLineLooping(l_i, looping) => {
                if scene.n_lines() <= l_i {
                    return;
                }
                let line = scene.line_mut(l_i);
                line.looping = looping;
                let _ =
                    update_notifier.send(SovaNotification::UpdatedLines(vec![(l_i, line.clone())]));
            }
            ConcreteEvent::SetLineTrailing(l_i, trailing) => {
                if scene.n_lines() <= l_i {
                    return;
                }
                let line = scene.line_mut(l_i);
                line.trailing = trailing;
                let _ =
                    update_notifier.send(SovaNotification::UpdatedLines(vec![(l_i, line.clone())]));
            }
            ConcreteEvent::SetLineManual(l_i, manual) => {
                if scene.n_lines() <= l_i {
                    return;
                }
                let line = scene.line_mut(l_i);
                line.manual = manual;
                let _ =
                    update_notifier.send(SovaNotification::UpdatedLines(vec![(l_i, line.clone())]));
            }
            ConcreteEvent::SetLineSpeedFactor(l_i, sp) => {
                if scene.n_lines() <= l_i {
                    return;
                }
                let line = scene.line_mut(l_i);
                line.speed_factor = sp;
                let _ =
                    update_notifier.send(SovaNotification::UpdatedLines(vec![(l_i, line.clone())]));
            }
            ConcreteEvent::SetFrame(l_i, f_i, lang, txt) => {
                let frame = scene.frame_mut(l_i, f_i); // NO SAFETY ! Will insert if too big
                let script = Script::new(txt, lang);
                frame.set_script(script.clone());
                languages.process_script(l_i, f_i, script, feedback.clone());
                let _ = update_notifier.send(SovaNotification::UpdatedFrames(vec![(
                    l_i,
                    f_i,
                    frame.clone(),
                )]));
            }
            ConcreteEvent::KillExecutions(l_i, f_i) => {
                if !scene.has_frame(l_i, f_i) {
                    return;
                }
                let frame = scene.frame_mut(l_i, f_i);
                frame.kill_executions();
            }
            _ => (),
        }
    }
}

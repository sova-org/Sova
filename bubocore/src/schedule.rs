// Doit faire traduction (Event, TimeSpan) en (ProtocolMessage, SyncTime)

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError},
        Arc,
    },
    thread::JoinHandle,
    time::Duration, usize,
};

use serde::{Deserialize, Serialize};
use thread_priority::ThreadBuilder;

use crate::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    lang::variable::VariableStore,
    scene::{
        script::{Script, ScriptExecution},
        Scene, Line,
    },
    protocol::TimedMessage,
    shared_types::GridSelection,
};

pub const SCHEDULED_DRIFT: SyncTime = 30_000;

/// Specifies when a scheduler action should be applied.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionTiming {
    /// Apply the action immediately upon processing.
    Immediate,
    /// Apply the action at the start of the next scene loop (quantized to scene length).
    EndOfScene,
    /// Apply the action when the clock beat reaches or exceeds this value.
    AtBeat(u64), // Using u64 for beats to simplify comparison/storage
}

impl Default for ActionTiming {
    fn default() -> Self {
        ActionTiming::Immediate
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulerMessage {
    /// Upload a new scene to the scheduler.
    UploadScene(Scene),
    /// Enable multiple frames in a line.
    EnableFrames(usize, Vec<usize>, ActionTiming),
    /// Disable multiple frames in a line.
    DisableFrames(usize, Vec<usize>, ActionTiming),
    /// Upload a script to a specific line/frame.
    UploadScript(usize, usize, Script, ActionTiming),
    /// Update the frames vector for a line.
    UpdateLineFrames(usize, Vec<f64>, ActionTiming),
    /// Insert a frame with a given value at a specific position in a line.
    InsertFrame(usize, usize, f64, ActionTiming),
    /// Remove the frame at a specific position in a line.
    RemoveFrame(usize, usize, ActionTiming), 
    /// Add a new line to the scene.
    AddLine,
    /// Remove a line at a specific index.
    RemoveLine(usize, ActionTiming),
    /// Set a line at a specific index.
    SetLine(usize, Line, ActionTiming),
    /// Set the start frame for a line.
    SetLineStartFrame(usize, Option<usize>, ActionTiming),
    /// Set the end frame for a line.
    SetLineEndFrame(usize, Option<usize>, ActionTiming),
    /// Set the entire scene.
    SetScene(Scene, ActionTiming),
    /// Set the scene length.
    SetSceneLength(usize, ActionTiming),
    /// Set the master tempo.
    SetTempo(f64, ActionTiming),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum SchedulerNotification {
    #[default]
    Nothing,
    UpdatedScene(Scene),
    UpdatedLine(usize, Line),
    EnableFrames(usize, Vec<usize>),
    DisableFrames(usize, Vec<usize>),
    UploadedScript(usize, usize, Script),
    UpdatedLineFrames(usize, Vec<f64>),
    AddedLine(Line),
    RemovedLine(usize),
    Log(TimedMessage),
    TempoChanged(f64),
    ClientListChanged(Vec<String>),
    ChatReceived(String, String),
    FramePositionChanged(Vec<usize>),
    /// Indicates a peer's grid selection has changed.
    PeerGridSelectionChanged(String, GridSelection), // (username, selection)
    /// Indicates a peer started editing a frame.
    PeerStartedEditingFrame(String, usize, usize), // (username, line_idx, frame_idx)
    /// Indicates a peer stopped editing a frame.
    PeerStoppedEditingFrame(String, usize, usize), // (username, line_idx, frame_idx)
    /// Indicates the scene length has changed.
    SceneLengthChanged(usize),
}

/// A pending action to be applied at a specific time.
#[derive(Debug, Clone)]
struct DeferredAction {
    action: SchedulerMessage,
    timing: ActionTiming,
    target_beat: Option<u64>, // Stores calculated target beat for EndOfSceneLoop
}

pub struct Scheduler {
    pub scene: Scene,
    pub global_vars: VariableStore,

    pub executions: Vec<ScriptExecution>,

    world_iface: Sender<TimedMessage>,
    devices: Arc<DeviceMap>,
    clock: Clock,

    message_source: Receiver<SchedulerMessage>,

    update_notifier: Sender<SchedulerNotification>,

    next_wait: Option<SyncTime>,
    processed_scene_modification: bool,
    deferred_actions: Vec<DeferredAction>,
    last_beat: f64,
}

impl Scheduler {
    pub fn create(
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
    ) -> (JoinHandle<()>, Sender<SchedulerMessage>, Receiver<SchedulerNotification>) {
        let (tx, rx) = mpsc::channel();
        let (p_tx, p_rx) = mpsc::channel();

        let handle = ThreadBuilder::default()
            .name("BuboCore-scheduler")
            .spawn(move |_| {
                let mut sched = Scheduler::new(clock_server.into(), devices, world_iface, rx, p_tx);
                sched.do_your_thing();
            })
            .expect("Unable to start Scheduler");
        (handle, tx, p_rx)
    }

    pub fn new(
        clock: Clock,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
        receiver: Receiver<SchedulerMessage>,
        update_notifier: Sender<SchedulerNotification>,
    ) -> Scheduler {
        Scheduler {
            world_iface,
            scene: Default::default(),
            global_vars: HashMap::new(),
            executions: Vec::new(),
            devices,
            clock,
            message_source: receiver,
            update_notifier,
            next_wait: None,
            processed_scene_modification: false,
            deferred_actions: Vec::new(),
            last_beat: 0.0,
        }
    }

    // Calculates the current frame index, iteration, start time, and remaining time for a line,
    // based on the global scene length acting as the loop boundary.
    // Note: Does not take &self to avoid borrow conflicts when iterating mutably over lines.
    fn frame_index(clock: &Clock, scene_length: usize, line : &Line, date: SyncTime) -> (usize, usize, SyncTime, SyncTime) {
        let scene_length_beats = scene_length as f64;
        if scene_length_beats <= 0.0 {
             return (usize::MAX, usize::MAX, SyncTime::MAX, SyncTime::MAX); // Avoid division by zero
        }

        let current_absolute_beat = clock.beat_at_date(date); // Use clock arg
        if current_absolute_beat < 0.0 {
            return (usize::MAX, usize::MAX, SyncTime::MAX, SyncTime::MAX);
        }

        // Calculate beat position within the scene loop
        let beat_in_scene_loop = current_absolute_beat % scene_length_beats;
        let scene_loop_iteration = current_absolute_beat.div_euclid(scene_length_beats) as usize;

        // Determine the sequence of frames to check based on line's start/end frames
        let effective_start_frame = line.get_effective_start_frame();
        let effective_num_frames = line.get_effective_num_frames();

        if effective_num_frames == 0 {
            return (usize::MAX, scene_loop_iteration, SyncTime::MAX, SyncTime::MAX); // No frames in line's effective range
        }

        let mut cumulative_beats_in_line = 0.0;
        for frame_idx_in_range in 0..effective_num_frames {
            let absolute_frame_index = effective_start_frame + frame_idx_in_range;
            
            // Safe division for speed factor
            let speed_factor = if line.speed_factor == 0.0 { 1.0 } else { line.speed_factor };
            let frame_len_beats = line.frame_len(absolute_frame_index) / speed_factor;

            if frame_len_beats <= 0.0 { continue; } // Skip zero/negative length frames

            let frame_end_beat_in_line = cumulative_beats_in_line + frame_len_beats;

            // Check if the beat_in_scene_loop falls within this frame's position *relative* to the start of the line's effective sequence
            if beat_in_scene_loop >= cumulative_beats_in_line && beat_in_scene_loop < frame_end_beat_in_line {
                // Found the active frame
                let absolute_beat_at_scene_loop_start = scene_loop_iteration as f64 * scene_length_beats;
                let frame_start_beat_absolute = absolute_beat_at_scene_loop_start + cumulative_beats_in_line;
                let start_date = clock.date_at_beat(frame_start_beat_absolute); // Use clock arg

                let remaining_beats_in_frame = frame_end_beat_in_line - beat_in_scene_loop;
                let remaining_micros = clock.beats_to_micros(remaining_beats_in_frame); // Use clock arg

                // Calculate remaining time until next scene loop boundary
                let remaining_beats_in_scene = scene_length_beats - beat_in_scene_loop;
                let remaining_micros_in_scene = clock.beats_to_micros(remaining_beats_in_scene); // Use clock arg

                // The actual delay until the *next* event is the minimum of frame end and scene end
                let next_event_delay = remaining_micros.min(remaining_micros_in_scene);

                return (absolute_frame_index, scene_loop_iteration, start_date, next_event_delay);
            }

            cumulative_beats_in_line += frame_len_beats;
        }

        // If the loop finishes, beat_in_scene_loop is past the end of the line's effective frames
        // Calculate delay until the next scene loop start
        let remaining_beats_in_scene = scene_length_beats - beat_in_scene_loop;
        let remaining_micros_in_scene = clock.beats_to_micros(remaining_beats_in_scene); // Use clock arg
        return (usize::MAX, scene_loop_iteration, SyncTime::MAX, remaining_micros_in_scene);
    }

    pub fn change_scene(&mut self, mut scene: Scene) {
        let date = self.theoretical_date();
        scene.make_consistent();
        let clock_ref = &self.clock; // Get immutable ref
        let scene_len = scene.length(); // Get length from the incoming scene
        for line in scene.lines_iter_mut() {
            let (frame, iter, _, _) = Self::frame_index(clock_ref, scene_len, line, date); // Pass clock_ref and scene_len
            line.current_frame = frame;
            line.current_iteration = iter;
            line.first_iteration_index = iter;
        }
        self.scene = scene;
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    } 

    /// Applies the actual state change from a SchedulerMessage.
    /// Assumes the timing condition has already been met.
    fn apply_action(&mut self, action: SchedulerMessage) {
        match action {
            // --- Actions that modify the scene --- 
            SchedulerMessage::EnableFrames(line, frames, _) => self.enable_frames(line, &frames),
            SchedulerMessage::DisableFrames(line, frames, _) => self.disable_frames(line, &frames),
            SchedulerMessage::UploadScript(line, frame, script, _) => self.upload_script(line, frame, script),
            SchedulerMessage::UpdateLineFrames(line, vec, _) => {
                self.scene.mut_line(line).set_frames(vec);
                // Send full scene update on frame change for now
                let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
            }
            SchedulerMessage::InsertFrame(line, position, value, _) => self.insert_frame(line, position, value),
            SchedulerMessage::RemoveFrame(line, position, _) => self.remove_frame(line, position),
            SchedulerMessage::RemoveLine(index, _) => self.remove_line(index),
            SchedulerMessage::SetLine(index, line, _) => self.set_line(index, line),
            SchedulerMessage::SetLineStartFrame(line_index, start_frame, _) => {
                 if let Some(line) = self.scene.lines.get_mut(line_index) {
                     line.start_frame = start_frame;
                     line.make_consistent();
                     let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                 } else {
                     eprintln!("[!] Scheduler: SetLineStartFrame received for invalid line index {}", line_index);
                 }
            }
            SchedulerMessage::SetLineEndFrame(line_index, end_frame, _) => {
                 if let Some(line) = self.scene.lines.get_mut(line_index) {
                     line.end_frame = end_frame;
                     line.make_consistent();
                     let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                 } else {
                     eprintln!("[!] Scheduler: SetLineEndFrame received for invalid line index {}", line_index);
                 }
            }
            SchedulerMessage::SetSceneLength(length, _) => {
                self.scene.set_length(length);
                let _ = self.update_notifier.send(SchedulerNotification::SceneLengthChanged(length));
            }
             // --- Actions that modify the clock --- 
            SchedulerMessage::SetTempo(tempo, _) => {
                self.clock.set_tempo(tempo);
                // Clock changes notify immediately through its own mechanism? Or scheduler notification?
                // Let's stick with scheduler notification for now.
                let _ = self.update_notifier.send(SchedulerNotification::TempoChanged(tempo));
            }
             // --- Actions handled elsewhere or always immediate --- 
            SchedulerMessage::UploadScene(scene) => self.change_scene(scene), // Always immediate
            SchedulerMessage::SetScene(scene, timing) => {
                self.change_scene(scene.clone());
                if timing == ActionTiming::Immediate {
                    let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(scene.clone()));
                }
            }
            SchedulerMessage::AddLine => { // Always immediate
                let new_line = Line::new(vec![1.0]);
                self.add_line(new_line);
            }
        }
        self.processed_scene_modification = true; // Flag that *some* modification occurred
    }

    pub fn process_message(&mut self, msg: SchedulerMessage) {
        let timing = match &msg {
            SchedulerMessage::EnableFrames(_, _, t) |
            SchedulerMessage::DisableFrames(_, _, t) |
            SchedulerMessage::UploadScript(_, _, _, t) |
            SchedulerMessage::UpdateLineFrames(_, _, t) |
            SchedulerMessage::InsertFrame(_, _, _, t) |
            SchedulerMessage::RemoveFrame(_, _, t) |
            SchedulerMessage::RemoveLine(_, t) |
            SchedulerMessage::SetLine(_, _, t) |
            SchedulerMessage::SetLineStartFrame(_, _, t) |
            SchedulerMessage::SetLineEndFrame(_, _, t) |
            SchedulerMessage::SetSceneLength(_, t) |
            SchedulerMessage::SetTempo(_, t) => *t,
            SchedulerMessage::SetScene(_, t) => *t,
            SchedulerMessage::UploadScene(_) | SchedulerMessage::AddLine => ActionTiming::Immediate,
        };

        if timing == ActionTiming::Immediate {
            self.apply_action(msg);
        } else {
            let current_beat = self.clock.beat().floor() as u64;
            let scene_len_beats = self.scene.length() as u64;
            let target_beat = if timing == ActionTiming::EndOfScene && scene_len_beats > 0 {
                Some(((current_beat / scene_len_beats) + 1) * scene_len_beats)
            } else { None }; // AtBeat timing doesn't need pre-calculation here

            self.deferred_actions.push(DeferredAction { action: msg, timing, target_beat });
             println!("Deferred action: {:?}, target: {:?}", self.deferred_actions.last().unwrap().action, self.deferred_actions.last().unwrap().timing); // Debug log
        }
    }

    pub fn set_line(&mut self, index: usize, line: Line) {
        self.scene.set_line(index, line);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn upload_script(&mut self, line: usize, frame: usize, script: Script) {
        self.scene.mut_line(line).set_script(frame, script);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn remove_line(&mut self, index: usize) {
        self.scene.remove_line(index);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn add_line(&mut self, line: Line) {
        self.scene.add_line(line);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn disable_frame(&mut self, line: usize, frame: usize) {
        self.scene.mut_line(line).disable_frame(frame);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }
    
    pub fn enable_frame(&mut self, line: usize, frame: usize) {
        self.scene.mut_line(line).enable_frame(frame);
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn disable_frames(&mut self, line_idx: usize, frames: &[usize]) {
        if let Some(line) = self.scene.lines.get_mut(line_idx) {
            line.disable_frames(frames);
            let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
        } else {
            eprintln!("[!] Scheduler: DisableFrames received for invalid line index {}", line_idx);
        }
    }

    pub fn enable_frames(&mut self, line_idx: usize, frames: &[usize]) {
        if let Some(line) = self.scene.lines.get_mut(line_idx) {
            line.enable_frames(frames);
            let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
        } else {
            eprintln!("[!] Scheduler: EnableFrames received for invalid line index {}", line_idx);
        }
    }

    pub fn do_your_thing(&mut self) {
        let start_date = self.clock.micros();
        println!("[+] Starting scheduler at {start_date}");
        loop {
            self.processed_scene_modification = false;
            self.clock.capture_app_state();

            // Receive incoming messages
            if let Some(timeout) = self.next_wait {
                let duration = Duration::from_micros(timeout);
                match self.message_source.recv_timeout(duration) {
                    Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => (),
                    Ok(msg) => self.process_message(msg),
                }
            } else {
                match self.message_source.try_recv() {
                    Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => (),
                    Ok(msg) => self.process_message(msg),
                }
            }

            let current_micros = self.clock.micros();
            let current_beat = self.clock.beat_at_date(current_micros);

            // Process deferred actions
            let scene_len_beats = self.scene.length() as f64;
            let mut applied_deferred = false;
            let mut indices_to_apply = Vec::new();

            // Step 1: Identify actions to apply
            for (index, deferred) in self.deferred_actions.iter().enumerate() {
                 let should_apply = match deferred.timing {
                    ActionTiming::Immediate => false, // Should not be in this list
                    ActionTiming::AtBeat(target) => current_beat >= target as f64,
                    ActionTiming::EndOfScene => {
                        if scene_len_beats <= 0.0 { false } // Avoid division by zero
                        else {
                            // Check if the beat crossed the scene length boundary since last iteration
                            (self.last_beat % scene_len_beats) > (current_beat % scene_len_beats)
                        }
                    }
                };
                 if should_apply {
                    indices_to_apply.push(index);
                }
            }

            // Step 2: Apply identified actions (if any)
            if !indices_to_apply.is_empty() {
                let actions_to_run: Vec<SchedulerMessage> = indices_to_apply.iter()
                    .map(|&index| self.deferred_actions[index].action.clone())
                    .collect();
                
                for action in actions_to_run {
                     println!("Applying deferred action: {:?}", action); // Debug log
                    self.apply_action(action);
                }
                applied_deferred = true;

                // Step 3: Remove applied actions (using retain with index check)
                 let mut current_index = 0;
                 self.deferred_actions.retain(|_| {
                     let keep = !indices_to_apply.contains(&current_index);
                     current_index += 1;
                     keep
                 });
            }

            // Update last_beat for next loop's EndOfSceneLoop check
            self.last_beat = current_beat;

            // Calculate frame indices and schedule script executions
            let date = self.theoretical_date();
            let clock_ref = &self.clock; // Get immutable ref before mutable loop
            let scene_len = self.scene.length(); // Get length before mutable loop
            let mut next_frame_delay = SyncTime::MAX;
            let mut current_positions = Vec::with_capacity(self.scene.n_lines());
            let mut positions_changed = false;

            for line in self.scene.lines_iter_mut() { // Mutable borrow starts here
                let (frame, iter, scheduled_date, track_frame_delay) = Self::frame_index(clock_ref, scene_len, line, date); // Pass clock_ref and scene_len
                next_frame_delay = std::cmp::min(next_frame_delay, track_frame_delay);

                current_positions.push(frame);

                let has_changed_frame = (frame != line.current_frame) || (iter != line.current_iteration);

                if has_changed_frame {
                    line.frames_passed += 1;
                    positions_changed = true;
                }

                if frame < usize::MAX && has_changed_frame && line.is_frame_enabled(frame) {
                    let script = Arc::clone(&line.scripts[frame]);
                    self.executions.push(ScriptExecution::execute_at(script, line.index, scheduled_date));
                    line.current_frame = frame;
                    line.frames_executed += 1;
                }
                line.current_iteration = iter;
            }

            if positions_changed && !self.processed_scene_modification { 
                let _ = self.update_notifier.send(SchedulerNotification::FramePositionChanged(current_positions));
            }

            // Run script execution logic
            let next_exec_delay = self.execution_loop();

            // Determine next loop wait time
            let next_delay = std::cmp::min(next_exec_delay, next_frame_delay);
            if next_delay > 0 {
                self.next_wait = Some(next_delay);
            } else {
                self.next_wait = None;
            }
        }
        println!("[-] Exiting scheduler...");
        for (_, (_, device)) in self.devices.output_connections.lock().unwrap().iter() {
            device.flush();
        }
    }

    #[inline]
    pub fn theoretical_date(&self) -> SyncTime {
        self.clock.micros() + SCHEDULED_DRIFT
    }

    #[inline]
    pub fn kill_all(&mut self) {
        self.executions.clear();
    }

    fn execution_loop(&mut self) -> SyncTime {
        if self.scene.n_lines() == 0 {
            return SyncTime::MAX;
        }

        let scheduled_date = self.theoretical_date();
        // TODO: Read MIDI input controller values
        let mut next_timeout = SyncTime::MAX;

        self.executions.retain_mut(|exec| {
            if !exec.is_ready(scheduled_date) {
                next_timeout = std::cmp::min(next_timeout, exec.remaining_before(scheduled_date));
                return true;
            }
            next_timeout = 0;
            if let Some((event, date)) = exec.execute_next(&self.clock, &mut self.global_vars, self.scene.mut_lines()) {
                let messages = self.devices.map_event(event, date);
                for message in messages {
                    //let _ = self.update_notifier.send(SchedulerNotification::Log(message.clone()));
                    let _ = self.world_iface.send(message);
                }
            }
            !exec.has_terminated()
        });
        next_timeout
    }

    pub fn insert_frame(&mut self, line_idx: usize, position: usize, value: f64) {
        if let Some(line) = self.scene.lines.get_mut(line_idx) {
            line.insert_frame(position, value);
            let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
        } else {
            eprintln!("[!] Scheduler: InsertFrame received for invalid line index {}", line_idx);
        }
    }

    pub fn remove_frame(&mut self, line_idx: usize, position: usize) {
        if let Some(line) = self.scene.lines.get_mut(line_idx) {
            line.remove_frame(position);
            let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
        } else {
            eprintln!("[!] Scheduler: RemoveFrame received for invalid line index {}", line_idx);
        }
    }

}

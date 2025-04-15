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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulerMessage {
    /// Upload a new scene to the scheduler.
    UploadScene(Scene),
    /// Enable multiple frames in a line.
    EnableFrames(usize, Vec<usize>),
    /// Disable multiple frames in a line.
    DisableFrames(usize, Vec<usize>),
    /// Upload a script to a specific line/frame.
    UploadScript(usize, usize, Script),
    /// Update the frames vector for a line.
    UpdateLineFrames(usize, Vec<f64>),
    /// Insert a frame with a given value at a specific position in a line.
    InsertFrame(usize, usize, f64),
    /// Remove the frame at a specific position in a line.
    RemoveFrame(usize, usize), 
    /// Add a new line to the scene.
    AddLine,
    /// Remove a line at a specific index.
    RemoveLine(usize),
    /// Set a line at a specific index.
    SetLine(usize, Line),
    /// Set the start frame for a line.
    SetLineStartFrame(usize, Option<usize>),
    /// Set the end frame for a line.
    SetLineEndFrame(usize, Option<usize>),
    /// Set the entire scene.
    SetScene(Scene),
    /// Set the scene length.
    SetSceneLength(usize),
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
        }
    }

    fn frame_index(clock : &Clock, line : &Line, date: SyncTime) -> (usize, usize, SyncTime, SyncTime) {
        // Use the effective range defined by start_frame and end_frame
        let effective_start_frame = line.get_effective_start_frame();
        let effective_num_frames = line.get_effective_num_frames();

        if effective_num_frames == 0 {
            return (usize::MAX, usize::MAX, SyncTime::MAX, SyncTime::MAX); // No frames to play
        }

        let effective_beats_len : f64 = line.effective_beats_len();

        if effective_beats_len <= 0.0 {
             return (usize::MAX, usize::MAX, SyncTime::MAX, SyncTime::MAX); // Avoid division by zero or negative length
        }

        let beat = clock.beat_at_date(date);
        if beat < 0.0 {
            return (usize::MAX, usize::MAX, SyncTime::MAX, SyncTime::MAX);
        }

        // Calculate beat within the effective loop length
        let beat_in_loop = beat % (effective_beats_len / line.speed_factor);
        let loop_iteration = beat.div_euclid(effective_beats_len / line.speed_factor) as usize;

        // Calculate the beat offset corresponding to the start of the effective range
        // This assumes frames before start_frame exist and have lengths.
        let line_start_beat_in_loop = beat - beat_in_loop; // Beat corresponding to the start of the current loop iteration

        let mut current_beat_in_effective_range = beat_in_loop;

        // Iterate through the frames *within the effective range*
        for frame_idx_in_range in 0..effective_num_frames {
            let absolute_frame_index = effective_start_frame + frame_idx_in_range;
            let frame_len_beats = line.frame_len(absolute_frame_index) / line.speed_factor; // Use absolute index to get length

            if current_beat_in_effective_range <= frame_len_beats {
                // Found the current frame within the effective range
                // Calculate the absolute start beat of this frame within the current loop iteration
                let frame_start_beat_absolute = line_start_beat_in_loop
                                              + (line.frames[effective_start_frame..absolute_frame_index].iter().sum::<f64>() / line.speed_factor);

                let start_date = clock.date_at_beat(frame_start_beat_absolute);
                let remaining_micros = clock.beats_to_micros(frame_len_beats - current_beat_in_effective_range);

                return (absolute_frame_index, loop_iteration, start_date, remaining_micros);
            }

            // Move to the next frame in the effective range
            current_beat_in_effective_range -= frame_len_beats;
        }

        // Should theoretically not be reached if effective_beats_len > 0
        eprintln!("[!] Scheduler::frame_index fell through loop unexpectedly. Beat: {}, Loop Beat: {}, Effective Length: {}", beat, beat_in_loop, effective_beats_len);
        return (usize::MAX, usize::MAX, SyncTime::MAX, SyncTime::MAX);
    }

    pub fn change_scene(&mut self, mut scene: Scene) {
        let date = self.theoretical_date();
        scene.make_consistent();
        for line in scene.lines_iter_mut() {
            let (frame, iter, _, _) = Self::frame_index(&self.clock, line, date);
            line.current_frame = frame;
            line.current_iteration = iter;
            line.first_iteration_index = iter;
        }
        self.scene = scene;
        let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    } 

    pub fn process_message(&mut self, msg: SchedulerMessage) {
        // Flag is reset at start of do_your_thing loop
        match msg {
            SchedulerMessage::UploadScene(scene) => {
                self.change_scene(scene);
                self.processed_scene_modification = true;
            }
            SchedulerMessage::EnableFrames(line, frames) => {
                self.enable_frames(line, &frames);
                self.processed_scene_modification = true;
            }
            SchedulerMessage::DisableFrames(line, frames) => {
                self.disable_frames(line, &frames);
                self.processed_scene_modification = true;
            }
            SchedulerMessage::UploadScript(line, frame, script) => {
                self.upload_script(line, frame, script);
                self.processed_scene_modification = true;
            }
            SchedulerMessage::UpdateLineFrames(line, vec) => {
                self.scene.mut_line(line).set_frames(vec);
                let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                self.processed_scene_modification = true;
            }
            SchedulerMessage::InsertFrame(line, position, value) => {
                self.insert_frame(line, position, value);
                self.processed_scene_modification = true;
            }
            SchedulerMessage::RemoveFrame(line, position) => {
                self.remove_frame(line, position);
                self.processed_scene_modification = true;
            }
            SchedulerMessage::AddLine => {
                let new_line = Line::new(vec![1.0]);
                self.add_line(new_line);
                self.processed_scene_modification = true;
            },
            SchedulerMessage::RemoveLine(index) => {
                self.remove_line(index);
                self.processed_scene_modification = true;
            }
            SchedulerMessage::SetLine(index, line) => {
                self.set_line(index, line);
                self.processed_scene_modification = true;
            }
            SchedulerMessage::SetLineStartFrame(line_index, start_frame) => {
                 if let Some(line) = self.scene.lines.get_mut(line_index) {
                     line.start_frame = start_frame;
                     line.make_consistent();
                     let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                     self.processed_scene_modification = true;
                 } else {
                     eprintln!("[!] Scheduler: SetLineStartFrame received for invalid line index {}", line_index);
                 }
            }
            SchedulerMessage::SetLineEndFrame(line_index, end_frame) => {
                 if let Some(line) = self.scene.lines.get_mut(line_index) {
                     line.end_frame = end_frame;
                     line.make_consistent();
                     let _ = self.update_notifier.send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                     self.processed_scene_modification = true;
                 } else {
                     eprintln!("[!] Scheduler: SetLineEndFrame received for invalid line index {}", line_index);
                 }
            }
            SchedulerMessage::SetScene(scene) => {
                self.change_scene(scene);
                self.processed_scene_modification = true;
            }
            SchedulerMessage::SetSceneLength(length) => {
                self.scene.set_length(length);
                let _ = self.update_notifier.send(SchedulerNotification::SceneLengthChanged(length));
                self.processed_scene_modification = true;
            }
        };
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

            let date = self.theoretical_date();

            let mut next_frame_delay = SyncTime::MAX;
            let mut current_positions = Vec::with_capacity(self.scene.n_lines());
            let mut positions_changed = false;

            for line in self.scene.lines_iter_mut() {
                let (frame, iter, scheduled_date, track_frame_delay) = Self::frame_index(&self.clock, line, date);
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

            let next_exec_delay = self.execution_loop();

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

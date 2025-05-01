// Doit faire traduction (Event, TimeSpan) en (ProtocolMessage, SyncTime)

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError},
    },
    thread::JoinHandle,
    time::Duration,
    usize,
};

use serde::{Deserialize, Serialize};
use thread_priority::ThreadBuilder;

use crate::{
    clock::{Clock, ClockServer, SyncTime},
    device_map::DeviceMap,
    lang::event::ConcreteEvent,
    lang::variable::VariableStore,
    protocol::message::TimedMessage,
    scene::{
        Line, Scene,
        script::{Script, ScriptExecution},
    },
    shared_types::DeviceInfo,
    shared_types::GridSelection,
};

// Helper struct for InternalDuplicateFrame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicatedFrameData {
    pub length: f64,
    pub is_enabled: bool,
    pub script: Option<Arc<Script>>,
    pub name: Option<String>, // Added frame name
    pub repetitions: usize, // Added frame repetitions
}

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
    /// Set a custom loop length for a specific line.
    SetLineLength(usize, Option<f64>, ActionTiming),
    /// Set the playback speed factor for a specific line.
    SetLineSpeedFactor(usize, f64, ActionTiming),
    /// Request the transport to start playback at the specified timing.
    TransportStart(ActionTiming),
    /// Request the transport to stop playback at the specified timing.
    TransportStop(ActionTiming),
    /// Set the name for a specific frame.
    SetFrameName(usize, usize, Option<String>, ActionTiming), // line_idx, frame_idx, name, timing
    /// Update the language identifier for a specific frame's script.
    SetScriptLanguage(usize, usize, String, ActionTiming), // line_idx, frame_idx, lang, timing
    /// Set the number of repetitions for a specific frame.
    SetFrameRepetitions(usize, usize, usize, ActionTiming), // line_idx, frame_idx, repetitions, timing
    /// Internal: Duplicate a frame (used by server handler)
    InternalDuplicateFrame {
        target_line_idx: usize,
        target_insert_idx: usize,
        frame_length: f64,
        is_enabled: bool,
        script: Option<Arc<Script>>,
        timing: ActionTiming,
    },
    /// Internal: Duplicate a range of frames (used by server handler)
    InternalDuplicateFrameRange {
        target_line_idx: usize,
        target_insert_idx: usize,
        frames_data: Vec<DuplicatedFrameData>,
        timing: ActionTiming,
    },
    /// Internal: Remove frames across potentially multiple lines.
    InternalRemoveFramesMultiLine {
        lines_and_indices: Vec<(usize, Vec<usize>)>,
        timing: ActionTiming,
    },
    /// Internal: Insert blocks of duplicated frame data.
    InternalInsertDuplicatedBlocks {
        // Vec<Vec<...>>: Outer Vec = columns, Inner Vec = rows within that column
        duplicated_data: Vec<Vec<DuplicatedFrameData>>,
        target_line_idx: usize,  // Top-left line index for insertion
        target_frame_idx: usize, // Top-left frame index for insertion
        timing: ActionTiming,
    },
}

/// Enum representing notifications broadcast by the Scheduler.
#[derive(Debug, Clone, Default)]
pub enum SchedulerNotification {
    #[default]
    Nothing,
    UpdatedScene(Scene),
    UpdatedLine(usize, Line),
    TempoChanged(f64),
    Log(TimedMessage),
    TransportStarted,
    TransportStopped,
    /// Current frame position for each playing line (line_idx, frame_idx, repetition_idx)
    FramePositionChanged(Vec<(usize, usize, usize)>),
    /// List of connected clients changed.
    ClientListChanged(Vec<String>),
    /// A chat message was received from a client.
    ChatReceived(String, String), // (sender_name, message)
    /// Enable specific frames in a line
    EnableFrames(usize, Vec<usize>),
    /// Disable specific frames in a line
    DisableFrames(usize, Vec<usize>),
    /// Uploaded script to a line/frame
    UploadedScript(usize, usize, Script),
    /// Set line frames
    UpdatedLineFrames(usize, Vec<f64>),
    /// Added a line
    AddedLine(Line),
    /// Removed a line
    RemovedLine(usize),
    /// A peer updated their grid selection.
    PeerGridSelectionChanged(String, GridSelection),
    /// A peer started editing a specific frame.
    PeerStartedEditingFrame(String, usize, usize),
    /// A peer stopped editing a specific frame.
    PeerStoppedEditingFrame(String, usize, usize),
    /// The total length of the scene (in lines) changed.
    SceneLengthChanged(usize),
    /// The list of available/connected devices changed.
    DeviceListChanged(Vec<DeviceInfo>),
}

/// Internal playback state for the scheduler
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlaybackState {
    Stopped,
    Starting(f64), // Waiting to start at the target beat
    Playing,
}

/// A pending action to be applied at a specific time.
#[derive(Debug, Clone)]
struct DeferredAction {
    action: SchedulerMessage,
    timing: ActionTiming,
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
    playback_state: PlaybackState,             // Track internal state
    shared_atomic_is_playing: Arc<AtomicBool>, // Added shared atomic
}

impl Scheduler {
    pub fn create(
        clock_server: Arc<ClockServer>,
        devices: Arc<DeviceMap>,
        world_iface: Sender<TimedMessage>,
        shared_atomic_is_playing: Arc<AtomicBool>,
    ) -> (
        JoinHandle<()>,
        Sender<SchedulerMessage>,
        Receiver<SchedulerNotification>,
    ) {
        let (tx, rx) = mpsc::channel();
        let (p_tx, p_rx) = mpsc::channel();

        let shared_atomic_clone = shared_atomic_is_playing.clone(); // Clone for the thread

        let handle = ThreadBuilder::default()
            .name("BuboCore-scheduler")
            .spawn(move |_| {
                let mut sched = Scheduler::new(
                    clock_server.into(),
                    devices,
                    world_iface,
                    rx,
                    p_tx,
                    shared_atomic_clone, // Pass the clone to new
                );
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
        shared_atomic_is_playing: Arc<AtomicBool>, // Accept the shared atomic
    ) -> Scheduler {
        Scheduler {
            world_iface,
            scene: Default::default(),
            global_vars: VariableStore::new(),
            executions: Vec::new(),
            devices,
            clock,
            message_source: receiver,
            update_notifier,
            next_wait: None,
            processed_scene_modification: false,
            deferred_actions: Vec::new(),
            last_beat: 0.0,
            playback_state: PlaybackState::Stopped, // Initialize
            shared_atomic_is_playing,               // Store the shared atomic
        }
    }

    // Calculates the current frame index, iteration, start time, and remaining time for a line,
    // based on the global scene length acting as the loop boundary.
    // Note: Does not take &self to avoid borrow conflicts when iterating mutably over lines.
    fn frame_index(
        clock: &Clock,
        scene_length: usize,
        line: &Line,
        date: SyncTime,
    ) -> (usize, usize, usize, SyncTime, SyncTime) {
        // Determine effective loop length: custom line length or global scene length
        let effective_loop_length_beats = line.custom_length.unwrap_or(scene_length as f64);

        if effective_loop_length_beats <= 0.0 {
            return (usize::MAX, usize::MAX, 0, SyncTime::MAX, SyncTime::MAX); // Avoid division by zero
        }

        let current_absolute_beat = clock.beat_at_date(date); // Use clock arg
        if current_absolute_beat < 0.0 {
            return (usize::MAX, usize::MAX, 0, SyncTime::MAX, SyncTime::MAX);
        }

        // Calculate beat position within the line's effective loop
        let beat_in_effective_loop = current_absolute_beat % effective_loop_length_beats;
        let loop_iteration = current_absolute_beat.div_euclid(effective_loop_length_beats) as usize;

        // Determine the sequence of frames to check based on line's start/end frames
        let effective_start_frame = line.get_effective_start_frame();
        let effective_num_frames = line.get_effective_num_frames();

        if effective_num_frames == 0 {
            return (usize::MAX, loop_iteration, 0, SyncTime::MAX, SyncTime::MAX); // No frames in line's effective range
        }

        let mut cumulative_beats_in_line = 0.0;
        for frame_idx_in_range in 0..effective_num_frames {
            let absolute_frame_index = effective_start_frame + frame_idx_in_range;

            // Safe division for speed factor
            let speed_factor = if line.speed_factor == 0.0 {
                1.0
            } else {
                line.speed_factor
            };
            let single_rep_len_beats = line.frame_len(absolute_frame_index) / speed_factor;
            let total_repetitions = line
                .frame_repetitions
                .get(absolute_frame_index)
                .copied()
                .unwrap_or(1)
                .max(1); // Ensure at least 1 repetition
            let total_frame_len_beats = single_rep_len_beats * total_repetitions as f64;

            if single_rep_len_beats <= 0.0 {
                continue;
            } // Skip zero/negative length frames

            let frame_end_beat_in_line = cumulative_beats_in_line + total_frame_len_beats;

            // Check if the beat_in_effective_loop falls within this frame's position (including repetitions)
            // *relative* to the start of the line's effective sequence
            if beat_in_effective_loop >= cumulative_beats_in_line
                && beat_in_effective_loop < frame_end_beat_in_line
            {
                // Found the active frame

                // Calculate which repetition we are currently in (0-based)
                let beat_within_frame = beat_in_effective_loop - cumulative_beats_in_line;
                let current_repetition_index = (beat_within_frame / single_rep_len_beats)
                    .floor()
                    .max(0.0) as usize;
                // Clamp to max possible index
                let current_repetition_index = current_repetition_index.min(total_repetitions - 1);

                // Calculate the start date of the *first* repetition of this frame in this loop iteration
                let absolute_beat_at_loop_start =
                    loop_iteration as f64 * effective_loop_length_beats;
                let frame_first_rep_start_beat_absolute =
                    absolute_beat_at_loop_start + cumulative_beats_in_line;
                let start_date = clock.date_at_beat(frame_first_rep_start_beat_absolute); // Use clock arg

                // Calculate remaining time until the end of the *current* repetition
                let current_rep_end_beat_in_line = cumulative_beats_in_line
                    + (single_rep_len_beats * (current_repetition_index + 1) as f64);
                let remaining_beats_in_rep = current_rep_end_beat_in_line - beat_in_effective_loop;
                let remaining_micros_in_rep = clock.beats_to_micros(remaining_beats_in_rep);

                // Calculate remaining time until next effective loop boundary
                let remaining_beats_in_loop = effective_loop_length_beats - beat_in_effective_loop;
                let remaining_micros_in_loop = clock.beats_to_micros(remaining_beats_in_loop);

                // The actual delay until the *next* event is the minimum of current rep end and loop end
                let next_event_delay = remaining_micros_in_rep.min(remaining_micros_in_loop);

                return (
                    absolute_frame_index,
                    loop_iteration,
                    current_repetition_index, // Return 0-based index
                    start_date, // Start date of the first repetition
                    next_event_delay,
                );
            }

            cumulative_beats_in_line += total_frame_len_beats; // Add total length for this frame
        }

        // If the loop finishes, beat_in_effective_loop is past the end of the line's effective frames
        // Calculate delay until the next effective loop start
        let remaining_beats_in_loop = effective_loop_length_beats - beat_in_effective_loop;
        let remaining_micros_in_loop = clock.beats_to_micros(remaining_beats_in_loop); // Use clock arg
        return (
            usize::MAX,
            loop_iteration,
            0, // Default repetition index when outside frames
            SyncTime::MAX,
            remaining_micros_in_loop,
        );
    }

    pub fn change_scene(&mut self, mut scene: Scene) {
        let date = self.theoretical_date();
        scene.make_consistent();
        let scene_len = scene.length(); // Get length before mutable borrow
        for line in scene.lines_iter_mut() {
            let (frame, iter, _rep, _, _) = Self::frame_index(&self.clock, scene_len, line, date);
            line.current_frame = frame;
            line.current_iteration = iter;
            line.first_iteration_index = iter;
            line.current_repetition = 0; // Reset repetition on scene change
        }
        // Clear any pending executions from the old scene
        self.executions.clear();

        // Queue executions for initially active frames in the new scene
        for line in scene.lines.iter() {
            // Iterate immutably now
            let (frame, _, _, scheduled_date, _) =
                Self::frame_index(&self.clock, scene_len, line, date);
            if frame < usize::MAX && line.is_frame_enabled(frame) {
                let script = Arc::clone(&line.scripts[frame]);
                // Schedule execution slightly ahead to align with scheduler's drift
                self.executions.push(ScriptExecution::execute_at(
                    script,
                    line.index,
                    scheduled_date,
                ));
            }
        }

        self.scene = scene;
        // Notify clients about the completely new scene state
        let _ = self
            .update_notifier
            .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    /// Applies the actual state change from a SchedulerMessage.
    /// Assumes the timing condition has already been met.
    fn apply_action(&mut self, action: SchedulerMessage) {
        match action {
            // --- Actions that modify the scene ---
            SchedulerMessage::EnableFrames(line, frames, _) => self.enable_frames(line, &frames),
            SchedulerMessage::DisableFrames(line, frames, _) => self.disable_frames(line, &frames),
            SchedulerMessage::UploadScript(line, frame, script, _) => {
                self.upload_script(line, frame, script)
            }
            SchedulerMessage::UpdateLineFrames(line, vec, _) => {
                self.scene.mut_line(line).set_frames(vec);
                // Send full scene update on frame change for now
                let _ = self
                    .update_notifier
                    .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
            }
            SchedulerMessage::InsertFrame(line, position, value, _) => {
                self.insert_frame(line, position, value)
            }
            SchedulerMessage::RemoveFrame(line, position, _) => self.remove_frame(line, position),
            SchedulerMessage::RemoveLine(index, _) => self.remove_line(index),
            SchedulerMessage::SetLine(index, line, _) => self.set_line(index, line),
            SchedulerMessage::SetLineStartFrame(line_index, start_frame, _) => {
                if let Some(line) = self.scene.lines.get_mut(line_index) {
                    line.start_frame = start_frame;
                    line.make_consistent();
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                } else {
                    eprintln!(
                        "[!] Scheduler: SetLineStartFrame received for invalid line index {}",
                        line_index
                    );
                }
            }
            SchedulerMessage::SetLineEndFrame(line_index, end_frame, _) => {
                if let Some(line) = self.scene.lines.get_mut(line_index) {
                    line.end_frame = end_frame;
                    line.make_consistent();
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                } else {
                    eprintln!(
                        "[!] Scheduler: SetLineEndFrame received for invalid line index {}",
                        line_index
                    );
                }
            }
            SchedulerMessage::SetSceneLength(length, _) => {
                self.scene.set_length(length);
                let _ = self
                    .update_notifier
                    .send(SchedulerNotification::SceneLengthChanged(length));
            }
            SchedulerMessage::SetLineLength(line_idx, length_opt, _) => {
                if let Some(line) = self.scene.lines.get_mut(line_idx) {
                    line.custom_length = length_opt;
                    // Send full scene update notification when line length changes
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                } else {
                    eprintln!(
                        "[!] Scheduler: SetLineLength received for invalid line index {}",
                        line_idx
                    );
                }
            }
            SchedulerMessage::SetLineSpeedFactor(line_idx, speed_factor, _) => {
                if let Some(line) = self.scene.lines.get_mut(line_idx) {
                    // Basic validation: ensure speed factor is positive
                    line.speed_factor = if speed_factor > 0.0 {
                        speed_factor
                    } else {
                        1.0
                    };
                    // Send full scene update notification
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                } else {
                    eprintln!(
                        "[!] Scheduler: SetLineSpeedFactor received for invalid line index {}",
                        line_idx
                    );
                }
            }
            // --- Handle Transport Control --- //
            SchedulerMessage::TransportStart(_) => {
                // Calculate the target start time (next quantum)
                let current_micros = self.clock.micros();
                let current_beat = self.clock.beat_at_date(current_micros);
                let quantum = self.clock.quantum();
                let start_beat = ((current_beat / quantum).floor() + 1.0) * quantum;
                let start_micros = self.clock.date_at_beat(start_beat);

                println!(
                    "[SCHEDULER] Requesting transport start via Link at beat {} ({} micros)",
                    start_beat, start_micros
                );

                // Request Link to start playing at the calculated time
                self.clock
                    .session_state
                    .set_is_playing(true, start_micros as u64);
                self.clock.commit_app_state();
                // Send notification immediately when processing the command
                let _ = self
                    .update_notifier
                    .send(SchedulerNotification::TransportStarted);
                // Note: Atomic is set in the state machine transition
            }
            SchedulerMessage::TransportStop(_) => {
                let now_micros = self.clock.micros();
                println!("[SCHEDULER] Requesting transport stop via Link now");

                // Request Link to stop playing now
                self.clock
                    .session_state
                    .set_is_playing(false, now_micros as u64);
                self.clock.commit_app_state();

                // Also clear any pending executions immediately when stopped
                self.executions.clear();
                // Send notification immediately when processing the command
                let _ = self
                    .update_notifier
                    .send(SchedulerNotification::TransportStopped);
                // Update shared atomic immediately for stop command
                self.shared_atomic_is_playing
                    .store(false, Ordering::Relaxed);
            }
            // --- Actions that modify the clock ---
            SchedulerMessage::SetTempo(tempo, _) => {
                self.clock.set_tempo(tempo);
                // Clock changes notify immediately through its own mechanism? Or scheduler notification?
                // Let's stick with scheduler notification for now.
                let _ = self
                    .update_notifier
                    .send(SchedulerNotification::TempoChanged(tempo));
            }
            // --- Actions handled elsewhere or always immediate ---
            SchedulerMessage::UploadScene(scene) => self.change_scene(scene), // Always immediate
            SchedulerMessage::SetScene(scene, timing) => {
                self.change_scene(scene.clone());
                if timing == ActionTiming::Immediate {
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::UpdatedScene(scene.clone()));
                }
            }
            SchedulerMessage::AddLine => {
                // Always immediate
                let new_line = Line::new(vec![1.0]);
                self.add_line(new_line);
            }
            SchedulerMessage::InternalDuplicateFrame {
                target_line_idx,
                target_insert_idx,
                frame_length,
                is_enabled,
                script: script_arc_opt,
                timing: _,
            } => {
                if let Some(line) = self.scene.lines.get_mut(target_line_idx) {
                    // 1. Insert the frame length
                    line.insert_frame(target_insert_idx, frame_length);
                    // 2. Set enabled/disabled state
                    if is_enabled {
                        line.enable_frame(target_insert_idx);
                    } else {
                        line.disable_frame(target_insert_idx);
                    }
                    // 3. Insert the script
                    if let Some(script_arc) = script_arc_opt {
                        // Clone the inner Script
                        let mut script_to_insert = (*script_arc).clone();
                        script_to_insert.index = target_insert_idx; // Set correct index
                        line.set_script(target_insert_idx, script_to_insert);
                    } else {
                        // If no script was provided (e.g., source was empty), insert a default/empty one
                        let default_script = Script::new(
                            "".to_string(),
                            Default::default(),
                            "bali".to_string(),
                            target_insert_idx,
                        );
                        line.set_script(target_insert_idx, default_script);
                    }
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                } else {
                    eprintln!(
                        "[!] Scheduler: InternalDuplicateFrame received for invalid line index {}",
                        target_line_idx
                    );
                }
            }
            SchedulerMessage::InternalDuplicateFrameRange {
                target_line_idx,
                target_insert_idx,
                frames_data,
                timing: _,
            } => {
                if let Some(line) = self.scene.lines.get_mut(target_line_idx) {
                    let mut current_insert_idx = target_insert_idx;
                    for frame_data in frames_data {
                        // Insert frame length
                        line.insert_frame(current_insert_idx, frame_data.length);
                        // Set enabled/disabled state
                        if frame_data.is_enabled {
                            line.enable_frame(current_insert_idx);
                        } else {
                            line.disable_frame(current_insert_idx);
                        }
                        // Insert script
                        if let Some(script_arc) = frame_data.script {
                            // Clone the inner Script, not the Arc
                            let mut script_to_insert = (*script_arc).clone();
                            script_to_insert.index = current_insert_idx; // Set correct index
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
                        // --- Add frame repetition setting --- 
                        line.frame_repetitions[current_insert_idx] = frame_data.repetitions.max(1); // Ensure at least 1
                        // ----------------------------------
                        current_insert_idx += 1; // Increment index for the next frame
                    }
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                } else {
                    eprintln!(
                        "[!] Scheduler: InternalDuplicateFrameRange received for invalid line index {}",
                        target_line_idx
                    );
                }
            }
            SchedulerMessage::InternalRemoveFramesMultiLine {
                lines_and_indices,
                timing: _,
            } => {
                let mut any_modification = false;
                // Iterate through each line specified in the message
                for (line_idx, frames) in lines_and_indices {
                    if let Some(line) = self.scene.lines.get_mut(line_idx) {
                        // --- Check if deletion would empty the line ---
                        let current_n_frames = line.n_frames();
                        let requested_to_remove = frames.len();

                        if current_n_frames > 0 && requested_to_remove >= current_n_frames {
                            eprintln!(
                                "[!] Scheduler: Denied removing {} frames from line {} (would empty line).",
                                requested_to_remove, line_idx
                            );
                            // Skip this line, continue to the next if any
                            continue;
                        }

                        // Clone the indices vector for sorting and iteration
                        let mut indices_to_remove = frames.clone();

                        // Sort indices in descending order to avoid shifting issues during removal
                        println!(
                            "[SCHED DEBUG] InternalRemoveFramesMultiLine: Received indices {:?} for line {}",
                            frames, line_idx
                        );
                        indices_to_remove.sort_unstable_by(|a, b| b.cmp(a));
                        println!(
                            "[SCHED DEBUG] InternalRemoveFramesMultiLine: Sorted indices to remove: {:?}",
                            indices_to_remove
                        );

                        for index in indices_to_remove {
                            // Check bounds again just in case
                            println!(
                                "[SCHED DEBUG]   Attempting remove index: {}, current n_frames: {}",
                                index,
                                line.n_frames()
                            ); // Log before remove
                            if index < line.n_frames() {
                                line.remove_frame(index);
                                any_modification = true; // Mark that something changed
                            } else {
                                eprintln!(
                                    "[!] Scheduler: InternalRemoveFramesMultiLine attempted to remove invalid index {} from line {}",
                                    index, line_idx
                                );
                            }
                        }

                        // After removing all for this line, ensure consistency
                        if any_modification {
                            // Only call if something was actually removed from this line
                            line.make_consistent();
                        }
                    } else {
                        eprintln!(
                            "[!] Scheduler: InternalRemoveFramesMultiLine received for invalid line index {}",
                            line_idx
                        );
                    }
                }

                // Send a single notification AFTER processing all lines, if any modification occurred
                if any_modification {
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                }
            }
            SchedulerMessage::InternalInsertDuplicatedBlocks {
                duplicated_data,
                target_line_idx,
                target_frame_idx,
                timing: _,
            } => {
                let mut any_modification = false;
                for (col_offset, column_data) in duplicated_data.into_iter().enumerate() {
                    let current_target_line_idx = target_line_idx + col_offset;

                    // Ensure the target line exists before attempting to modify it
                    if current_target_line_idx < self.scene.lines.len() {
                        if let Some(line) = self.scene.lines.get_mut(current_target_line_idx) {
                            let mut current_insert_idx = target_frame_idx;
                            for frame_data in column_data {
                                line.insert_frame(current_insert_idx, frame_data.length);
                                if frame_data.is_enabled {
                                    line.enable_frame(current_insert_idx);
                                } else {
                                    line.disable_frame(current_insert_idx);
                                }
                                if let Some(script_arc) = frame_data.script {
                                    // Clone the inner Script, not the Arc
                                    let mut script_to_insert = (*script_arc).clone();
                                    script_to_insert.index = current_insert_idx; // Set correct index
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
                                // --- Add frame name setting ---
                                line.set_frame_name(current_insert_idx, frame_data.name); // Use frame_data.name here
                                // --- Add frame repetition setting ---
                                line.frame_repetitions[current_insert_idx] = frame_data.repetitions.max(1);
                                // ----------------------------------
                                // ------------------------------
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
                    let _ = self
                        .update_notifier
                        .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
                }
            }
            SchedulerMessage::SetFrameName(line_idx, frame_idx, name, _) => {
                self.set_frame_name(line_idx, frame_idx, name);
            }
            SchedulerMessage::SetScriptLanguage(line_idx, frame_idx, lang, _) => {
                if let Some(line) = self.scene.lines.get_mut(line_idx) {
                    // Find the script Arc's position with the matching index
                    if let Some(script_pos) = line.scripts.iter().position(|s| s.index == frame_idx)
                    {
                        // Clone the Script data out of the Arc
                        let mut script_clone = (*line.scripts[script_pos]).clone(); // Use single deref *, then clone Script

                        // Modify the clone
                        script_clone.lang = lang;

                        // Replace the old Arc with a new one containing the modified clone
                        line.scripts[script_pos] = Arc::new(script_clone);

                        self.processed_scene_modification = true;
                        // Send notification that the line (containing the script) updated
                        let _ = self
                            .update_notifier
                            .send(SchedulerNotification::UpdatedLine(line_idx, line.clone()));
                        // TODO: Potentially recompile if needed immediately or notify transcoder?
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
            SchedulerMessage::SetFrameRepetitions(line_idx, frame_idx, repetitions, _) => {
                if let Some(line) = self.scene.lines.get_mut(line_idx) {
                    if frame_idx < line.frame_repetitions.len() {
                        line.frame_repetitions[frame_idx] = repetitions.max(1); // Ensure at least 1 repetition
                        self.processed_scene_modification = true;
                        let _ = self.update_notifier.send(SchedulerNotification::UpdatedLine(line_idx, line.clone()));
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
        self.processed_scene_modification = true; // Flag that *some* modification occurred
    }

    pub fn process_message(&mut self, msg: SchedulerMessage) {
        let timing = match &msg {
            SchedulerMessage::EnableFrames(_, _, t)
            | SchedulerMessage::DisableFrames(_, _, t)
            | SchedulerMessage::UploadScript(_, _, _, t)
            | SchedulerMessage::UpdateLineFrames(_, _, t)
            | SchedulerMessage::InsertFrame(_, _, _, t)
            | SchedulerMessage::RemoveFrame(_, _, t)
            | SchedulerMessage::RemoveLine(_, t)
            | SchedulerMessage::SetLine(_, _, t)
            | SchedulerMessage::SetLineStartFrame(_, _, t)
            | SchedulerMessage::SetLineEndFrame(_, _, t)
            | SchedulerMessage::SetSceneLength(_, t)
            | SchedulerMessage::SetTempo(_, t)
            | SchedulerMessage::SetLineLength(_, _, t)
            | SchedulerMessage::SetLineSpeedFactor(_, _, t) => *t,
            SchedulerMessage::SetScene(_, t) => *t,
            SchedulerMessage::UploadScene(_) | SchedulerMessage::AddLine => ActionTiming::Immediate,
            SchedulerMessage::TransportStart(t) | SchedulerMessage::TransportStop(t) => *t,
            SchedulerMessage::InternalDuplicateFrame { timing, .. } => *timing,
            SchedulerMessage::InternalDuplicateFrameRange { timing, .. } => *timing,
            SchedulerMessage::InternalRemoveFramesMultiLine { timing, .. } => *timing,
            SchedulerMessage::InternalInsertDuplicatedBlocks { timing, .. } => *timing,
            SchedulerMessage::SetFrameName(_, _, _, t) => *t,
            SchedulerMessage::SetScriptLanguage(_, _, _, t) => *t,
            SchedulerMessage::SetFrameRepetitions(_, _, _, t) => *t,
        };

        if timing == ActionTiming::Immediate {
            self.apply_action(msg);
        } else {
            let current_beat = self.clock.beat().floor() as u64;
            let scene_len_beats = self.scene.length() as u64;
            let _target_beat = if timing == ActionTiming::EndOfScene && scene_len_beats > 0 {
                Some(((current_beat / scene_len_beats) + 1) * scene_len_beats)
            } else {
                None
            }; // AtBeat timing doesn't need pre-calculation here

            self.deferred_actions.push(DeferredAction {
                action: msg,
                timing,
            });
            println!(
                "Deferred action: {:?}, target: {:?}",
                self.deferred_actions.last().unwrap().action,
                self.deferred_actions.last().unwrap().timing
            ); // Debug log
        }
    }

    pub fn set_line(&mut self, index: usize, line: Line) {
        self.scene.set_line(index, line);
        let _ = self
            .update_notifier
            .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn upload_script(&mut self, line: usize, frame: usize, script: Script) {
        self.scene.mut_line(line).set_script(frame, script);
        let _ = self
            .update_notifier
            .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn remove_line(&mut self, index: usize) {
        self.scene.remove_line(index);
        let _ = self
            .update_notifier
            .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn add_line(&mut self, line: Line) {
        self.scene.add_line(line);
        let _ = self
            .update_notifier
            .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn disable_frame(&mut self, line: usize, frame: usize) {
        self.scene.mut_line(line).disable_frame(frame);
        let _ = self
            .update_notifier
            .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn enable_frame(&mut self, line: usize, frame: usize) {
        self.scene.mut_line(line).enable_frame(frame);
        let _ = self
            .update_notifier
            .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
    }

    pub fn disable_frames(&mut self, line_idx: usize, frames: &[usize]) {
        if let Some(line) = self.scene.lines.get_mut(line_idx) {
            line.disable_frames(frames);
            let _ = self
                .update_notifier
                .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: DisableFrames received for invalid line index {}",
                line_idx
            );
        }
    }

    pub fn enable_frames(&mut self, line_idx: usize, frames: &[usize]) {
        if let Some(line) = self.scene.lines.get_mut(line_idx) {
            line.enable_frames(frames);
            let _ = self
                .update_notifier
                .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: EnableFrames received for invalid line index {}",
                line_idx
            );
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
            let mut _applied_deferred;
            let mut indices_to_apply = Vec::new();

            // Step 1: Identify actions to apply
            for (index, deferred) in self.deferred_actions.iter().enumerate() {
                let should_apply = match deferred.timing {
                    ActionTiming::Immediate => false, // Should not be in this list
                    ActionTiming::AtBeat(target) => current_beat >= target as f64,
                    ActionTiming::EndOfScene => {
                        if scene_len_beats <= 0.0 {
                            false
                        }
                        // Avoid division by zero
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
                let actions_to_run: Vec<SchedulerMessage> = indices_to_apply
                    .iter()
                    .map(|&index| self.deferred_actions[index].action.clone())
                    .collect();

                for action in actions_to_run {
                    println!("Applying deferred action: {:?}", action); // Debug log
                    self.apply_action(action);
                }
                _applied_deferred = true;

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

            // Check Ableton Link's current playing state
            let link_is_playing = self.clock.session_state.is_playing();

            // --- Scheduler State Machine ---
            match self.playback_state {
                PlaybackState::Stopped => {
                    if link_is_playing {
                        // Transition: Stopped -> Starting
                        let quantum = self.clock.quantum();
                        let target_beat = ((current_beat / quantum).floor() + 1.0) * quantum;
                        println!(
                            "[SCHEDULER] Link is playing, scheduler was stopped. Waiting for beat {:.4} to start.",
                            target_beat
                        );
                        self.playback_state = PlaybackState::Starting(target_beat);
                        self.next_wait = Some(1_000); // Check frequently
                    } else {
                        // Still stopped
                        self.next_wait = Some(100_000);
                    }
                }
                PlaybackState::Starting(target_beat) => {
                    if link_is_playing {
                        if current_beat >= target_beat {
                            // Transition: Starting -> Playing (Target beat reached)
                            println!(
                                "[SCHEDULER] Target beat {:.4} reached. Starting playback.",
                                target_beat
                            );

                            // Reset scene state
                            for line in self.scene.lines_iter_mut() {
                                line.current_frame = usize::MAX;
                                line.current_iteration = 0;
                                line.first_iteration_index = 0;
                                line.frames_passed = 0;
                                line.frames_executed = 0;
                            }
                            self.executions.clear();

                            // Calculate the precise start time (current time at alignment)
                            let start_date = self.clock.date_at_beat(target_beat); // Use target beat time
                            // Schedule initial scripts for the target start time
                            let scene_len = self.scene.length();
                            for line in self.scene.lines.iter() {
                                let (frame, iter, rep, _scheduled_date, _) =
                                    Self::frame_index(&self.clock, scene_len, line, start_date);
                                if frame == line.get_effective_start_frame()
                                    && line.is_frame_enabled(frame)
                                    && iter == 0 && rep == 0
                                {
                                    let script = Arc::clone(&line.scripts[frame]);
                                    self.executions.push(ScriptExecution::execute_at(
                                        script, line.index, start_date,
                                    ));
                                    println!(
                                        "[SCHEDULER] Queued script for Line {} Frame {} at start",
                                        line.index, frame
                                    );
                                }
                            }

                            // DO NOT run playback logic in this cycle. Let the next cycle handle it.
                            // Remove notification from state transition, it's now sent when command is processed
                            // let _ = self.update_notifier.send(SchedulerNotification::TransportStarted);
                            self.playback_state = PlaybackState::Playing;
                            // Update shared atomic: We are now playing
                            self.shared_atomic_is_playing.store(true, Ordering::Relaxed);
                            self.next_wait = None; // Allow normal calculation below
                            self.processed_scene_modification = true;
                        } else {
                            // Still waiting for target beat
                            self.next_wait = Some(1_000); // Check frequently
                        }
                    } else {
                        // Link stopped while we were waiting to start
                        println!(
                            "[SCHEDULER] Link stopped while waiting to start. Returning to Stopped state."
                        );
                        self.playback_state = PlaybackState::Stopped;
                        // Update shared atomic: We are now stopped
                        self.shared_atomic_is_playing
                            .store(false, Ordering::Relaxed);
                        if !self.executions.is_empty() {
                            self.executions.clear();
                        } // Clear just in case
                        self.next_wait = Some(100_000);
                    }
                }
                PlaybackState::Playing => {
                    if link_is_playing {
                        // --- Main Playback Logic ---
                        // Run only if playing normally (was playing last cycle)
                        // No, run if state is Playing and Link is Playing
                        let date = self.theoretical_date();
                        let clock_ref = &self.clock; // Get immutable ref before mutable loop
                        let scene_len = self.scene.length(); // Get length before mutable loop
                        let mut next_frame_delay = SyncTime::MAX;
                        let mut current_positions = Vec::with_capacity(self.scene.n_lines());
                        let mut positions_changed = false;

                        for line in self.scene.lines_iter_mut() {
                            // Mutable borrow starts here
                            let (frame, iter, rep, scheduled_date, track_frame_delay) =
                                Self::frame_index(clock_ref, scene_len, line, date); // Pass clock_ref and scene_len
                            next_frame_delay = std::cmp::min(next_frame_delay, track_frame_delay);

                            // Store frame and repetition index
                            current_positions.push((frame, rep));

                            let has_changed =
                                (frame != line.current_frame)
                                || (iter != line.current_iteration)
                                || (rep != line.current_repetition);

                            if has_changed {
                                // Only increment passed if frame or iteration changed, not just repetition
                                if frame != line.current_frame || iter != line.current_iteration {
                                    line.frames_passed += 1;
                                }
                                positions_changed = true;
                            }

                            // Queue script if frame/iter/rep changed and frame is valid/enabled
                            if frame < usize::MAX
                                && has_changed
                                && line.is_frame_enabled(frame)
                            {
                                let script = Arc::clone(&line.scripts[frame]);
                                self.executions.push(ScriptExecution::execute_at(
                                    script,
                                    line.index,
                                    scheduled_date, // Use start date of the first repetition
                                ));
                                // Only increment executed if frame or iteration changed
                                if frame != line.current_frame || iter != line.current_iteration {
                                     line.frames_executed += 1;
                                }
                            }
                            // Update state *after* checks
                            line.current_frame = frame;
                            line.current_iteration = iter;
                            line.current_repetition = rep;
                        }

                        if positions_changed && !self.processed_scene_modification {
                            // Correctly map index `i` (line_idx) and frame `f` and repetition `r`
                            let frame_updates: Vec<(usize, usize, usize)> = current_positions
                                .iter()
                                .enumerate() // Get index `i` along with frame/rep tuple `&(f, r)`
                                .map(|(i, &(f, r))| (i, f, r)) // Create the tuple (line_idx, frame_idx, rep_idx)
                                .collect();
                            let _ = self
                                .update_notifier
                                .send(SchedulerNotification::FramePositionChanged(frame_updates));
                        }

                        // Run script execution logic
                        let next_exec_delay = self.execution_loop();

                        // Determine next loop wait time based on playback events
                        let next_delay = std::cmp::min(next_exec_delay, next_frame_delay);
                        if next_delay > 0 {
                            self.next_wait = Some(next_delay);
                        } else {
                            self.next_wait = None;
                        }
                    } else {
                        // Transition: Playing -> Stopped (Link stopped externally)
                        println!(
                            "[SCHEDULER] Link stopped. Stopping playback and clearing executions."
                        );
                        self.playback_state = PlaybackState::Stopped;
                        // Update shared atomic: We are now stopped
                        self.shared_atomic_is_playing
                            .store(false, Ordering::Relaxed);
                        if !self.executions.is_empty() {
                            self.executions.clear();
                        }
                        // Send notification here as well, as it wasn't initiated by a command
                        let _ = self
                            .update_notifier
                            .send(SchedulerNotification::TransportStopped);
                        self.next_wait = Some(100_000);
                        self.processed_scene_modification = true;
                    }
                }
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
        let mut next_timeout = SyncTime::MAX;

        self.executions.retain_mut(|exec| {
            if !exec.is_ready(scheduled_date) {
                next_timeout = std::cmp::min(next_timeout, exec.remaining_before(scheduled_date));
                return true;
            }

            next_timeout = 0;
            if let Some((event, date)) = exec.execute_next(
                &self.clock,
                &mut self.global_vars,
                self.scene.mut_lines(),
                self.devices.clone(),
            ) {
                let maybe_slot_id: Option<usize> = match event {
                    ConcreteEvent::MidiNote(_, _, _, _, id)
                    | ConcreteEvent::MidiControl(_, _, _, id)
                    | ConcreteEvent::MidiProgram(_, _, id)
                    | ConcreteEvent::MidiAftertouch(_, _, _, id)
                    | ConcreteEvent::MidiChannelPressure(_, _, id)
                    | ConcreteEvent::MidiSystemExclusive(_, id)
                    | ConcreteEvent::MidiStart(id)
                    | ConcreteEvent::MidiStop(id)
                    | ConcreteEvent::MidiReset(id)
                    | ConcreteEvent::MidiContinue(id)
                    | ConcreteEvent::MidiClock(id) => Some(id),
                    ConcreteEvent::Dirt { device_id: id, .. } => Some(id),
                    ConcreteEvent::Osc { device_id: id, .. } => Some(id),
                    ConcreteEvent::Nop => None,
                };

                if let Some(slot_id) = maybe_slot_id {
                    let messages =
                        self.devices
                            .map_event_for_slot_id(slot_id, event, date, &self.clock);
                    for message in messages {
                        let _ = self.world_iface.send(message);
                    }
                } // Nop events (maybe_slot_id == None) are ignored.
            }

            !exec.has_terminated()
        });

        next_timeout
    }

    pub fn insert_frame(&mut self, line_idx: usize, position: usize, value: f64) {
        if let Some(line) = self.scene.lines.get_mut(line_idx) {
            line.insert_frame(position, value);
            let _ = self
                .update_notifier
                .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: InsertFrame received for invalid line index {}",
                line_idx
            );
        }
    }

    pub fn remove_frame(&mut self, line_idx: usize, position: usize) {
        if let Some(line) = self.scene.lines.get_mut(line_idx) {
            line.remove_frame(position);
            let _ = self
                .update_notifier
                .send(SchedulerNotification::UpdatedScene(self.scene.clone()));
        } else {
            eprintln!(
                "[!] Scheduler: RemoveFrame received for invalid line index {}",
                line_idx
            );
        }
    }

    pub fn set_frame_name(&mut self, line_idx: usize, frame_idx: usize, name: Option<String>) {
        if let Some(line) = self.scene.lines.get_mut(line_idx) {
            line.set_frame_name(frame_idx, name.clone());
            self.processed_scene_modification = true;
            self.update_notifier
                .send(SchedulerNotification::UpdatedLine(line_idx, line.clone()))
                .unwrap();
            // Optionally, add a more specific notification like FrameNameChanged if needed later.
        } else {
            eprintln!(
                "[!] Scheduler::set_frame_name: Invalid line index {}",
                line_idx
            );
        }
    }
}

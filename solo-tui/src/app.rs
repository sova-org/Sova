use std::{collections::HashMap, sync::Arc};

use crate::{
    event::{AppEvent, Event, EventHandler},
    page::Page,
    widgets::{log_widget::LogWidget, scene_widget::SceneWidget},
};
use crossbeam_channel::{Receiver, Sender};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use sova_core::{
    LogMessage, Scene, clock::{Clock, ClockServer}, lang::variable::VariableValue, protocol::DeviceInfo, scene::Frame, schedule::{ActionTiming, SchedulerMessage, SovaNotification}
};

pub struct AppState {
    pub running: bool,
    pub scene_image: Scene,
    pub global_vars: HashMap<String, VariableValue>,
    pub playing: bool,
    pub positions: Vec<(usize, usize)>,
    pub clock: Clock,
    pub devices: Vec<DeviceInfo>,
    pub page: Page,
    pub selected: (usize, usize),
    pub events: EventHandler,
}

impl AppState {
    pub fn selected_frame(&self) -> Option<&Frame> {
        self.scene_image.get_frame(self.selected.0, self.selected.1)
    }
}

/// Application.
pub struct App {
    pub sched_iface: Sender<SchedulerMessage>,
    pub state: AppState,
    pub scene_widget: SceneWidget,
    pub log_widget: LogWidget,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(
        sched_iface: Sender<SchedulerMessage>,
        sched_update: Receiver<SovaNotification>,
        log_rx: Receiver<LogMessage>,
        clock_server: Arc<ClockServer>,
    ) -> Self {
        App {
            sched_iface,
            state: AppState {
                running: Default::default(),
                scene_image: Default::default(),
                global_vars: Default::default(),
                playing: Default::default(),
                positions: Default::default(),
                clock: clock_server.into(),
                devices: Default::default(),
                page: Default::default(),
                selected: Default::default(),
                events: EventHandler::new(sched_update, log_rx),
            },
            scene_widget: SceneWidget::default(),
            log_widget: LogWidget::default(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.state.running = true;
        self.log(LogMessage::info("Starting app...".to_owned()));
        while self.state.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.state.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event)
                    if key_event.kind == crossterm::event::KeyEventKind::Press =>
                {
                    self.handle_key_event(key_event)?
                }
                _ => {}
            },
            Event::App(app_event) => self.handle_app_event(app_event)?,
            Event::Notification(notif) => self.handle_notification(notif)?,
        }
        Ok(())
    }

    pub fn handle_app_event(&mut self, event: AppEvent) -> color_eyre::Result<()> {
        match event {
            AppEvent::SchedulerControl(msg) => {
                let _ = self.sched_iface.send(msg);
            }
            AppEvent::Right => self.state.page.right(),
            AppEvent::Left => self.state.page.left(),
            AppEvent::Up => self.state.page.up(),
            AppEvent::Down => self.state.page.down(),
            AppEvent::Quit => self.quit(),
        }
        Ok(())
    }

    pub fn handle_notification(&mut self, notif: SovaNotification) -> color_eyre::Result<()> {
        match notif {
            SovaNotification::Nothing | SovaNotification::TempoChanged(_) => (),
            SovaNotification::UpdatedScene(scene) => self.state.scene_image = scene,
            SovaNotification::UpdatedLines(items) => {
                for (index, line) in items {
                    self.state.scene_image.set_line(index, line);
                }
            }
            SovaNotification::UpdatedLineConfigurations(items) => {
                for (index, line) in items {
                    self.state.scene_image.line_mut(index).configure(&line);
                }
            }
            SovaNotification::AddedLine(index, line) => {
                self.state.scene_image.insert_line(index, line)
            }
            SovaNotification::RemovedLine(index) => self.state.scene_image.remove_line(index),
            SovaNotification::UpdatedFrames(items) => {
                for (line_index, frame_index, frame) in items {
                    self.state
                        .scene_image
                        .line_mut(line_index)
                        .set_frame(frame_index, frame);
                }
            }
            SovaNotification::AddedFrame(line_index, frame_index, frame) => self
                .state
                .scene_image
                .line_mut(line_index)
                .insert_frame(frame_index, frame),
            SovaNotification::RemovedFrame(line_index, frame_index) => self
                .state
                .scene_image
                .line_mut(line_index)
                .remove_frame(frame_index),
            SovaNotification::CompilationUpdated(line_index, frame_index, _, state) => {
                let frame = self
                    .state
                    .scene_image
                    .line_mut(line_index)
                    .frame_mut(frame_index);
                *frame.compilation_state_mut() = state;
            }
            SovaNotification::TransportStarted => self.state.playing = true,
            SovaNotification::TransportStopped => self.state.playing = false,
            SovaNotification::FramePositionChanged(positions) => self.state.positions = positions,
            SovaNotification::GlobalVariablesChanged(values) => self.state.global_vars = values,
            SovaNotification::Log(msg) => self.log(msg),
            SovaNotification::DeviceListChanged(devices) => self.state.devices = devices,
            SovaNotification::ClientListChanged(_)
            | SovaNotification::ChatReceived(_, _)
            | SovaNotification::PeerStartedEditingFrame(_, _, _)
            | SovaNotification::PeerStoppedEditingFrame(_, _, _) => (),
        }
        Ok(())
    }

    fn log(&mut self, msg: LogMessage) {
        self.log_widget.add_log(msg);
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc => self.state.events.send(AppEvent::Quit),

            KeyCode::Up if key_event.modifiers == KeyModifiers::CONTROL => {
                self.state.events.send(AppEvent::Up);
            }
            KeyCode::Down if key_event.modifiers == KeyModifiers::CONTROL => {
                self.state.events.send(AppEvent::Down);
            }
            KeyCode::Left if key_event.modifiers == KeyModifiers::CONTROL => {
                self.state.events.send(AppEvent::Left);
            }
            KeyCode::Right if key_event.modifiers == KeyModifiers::CONTROL => {
                self.state.events.send(AppEvent::Right);
            }

            KeyCode::Char(' ') if key_event.modifiers == KeyModifiers::CONTROL => {
                let event = if self.state.playing {
                    SchedulerMessage::TransportStop(ActionTiming::Immediate)
                } else {
                    SchedulerMessage::TransportStart(ActionTiming::Immediate)
                };
                self.state.events.send(event.into())
            }

            _ => match self.state.page {
                Page::Scene => self
                    .scene_widget
                    .process_event(&mut self.state, key_event)?,
                _ => (),
            },
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {
        self.state.clock.capture_app_state();
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.state.running = false;
    }
}

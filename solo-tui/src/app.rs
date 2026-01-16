use std::{collections::HashMap, sync::Arc};

use crate::{
    event::{AppEvent, Event, EventHandler, TICK_FPS},
    notification::Notification,
    page::Page,
    popup::{Popup, PopupValue},
    widgets::{
        configure_widget::ConfigureWidget, devices_widget::DevicesWidget, edit_widget::EditWidget,
        log_widget::LogWidget, scene_widget::SceneWidget, time_widget::TimeWidget,
    },
};
use arboard::Clipboard;
use crossbeam_channel::{Receiver, Sender};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use sova_core::{
    LogMessage, Scene,
    clock::{Clock, ClockServer},
    device_map::DeviceMap,
    protocol::DeviceInfo,
    scene::Frame,
    schedule::{ActionTiming, SchedulerMessage, SovaNotification, playback::PlaybackState},
    vm::{LanguageCenter, variable::VariableValue},
};

pub struct AppState {
    pub running: bool,
    pub scene_image: Scene,
    pub global_vars: HashMap<String, VariableValue>,
    pub playing: PlaybackState,
    pub positions: Vec<Vec<(usize, usize)>>,
    pub clock: Clock,
    pub devices: Vec<DeviceInfo>,
    pub page: Page,
    pub selected: (usize, usize),
    pub events: EventHandler,
    pub device_map: Arc<DeviceMap>,
    pub languages: Arc<LanguageCenter>,
    pub clipboard: Option<Clipboard>,
}

impl AppState {
    pub fn selected_frame(&self) -> Option<&Frame> {
        self.scene_image.get_frame(self.selected.0, self.selected.1)
    }

    pub fn refresh_devices(&mut self) {
        self.devices = self.device_map.device_list();
    }
}

/// Application.
pub struct App {
    pub sched_iface: Sender<SchedulerMessage>,
    pub state: AppState,
    pub scene_widget: SceneWidget,
    pub edit_widget: EditWidget,
    pub devices_widget: DevicesWidget,
    pub log_widget: LogWidget,
    pub popup: Popup,
    pub notification: Notification,
    frame_counter: u16,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(
        sched_iface: Sender<SchedulerMessage>,
        sched_update: Receiver<SovaNotification>,
        log_rx: Receiver<LogMessage>,
        clock_server: Arc<ClockServer>,
        device_map: Arc<DeviceMap>,
        languages: Arc<LanguageCenter>,
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
                clipboard: Clipboard::new().map(|x| Some(x)).unwrap_or_default(),
                device_map,
                languages,
            },
            scene_widget: SceneWidget::default(),
            edit_widget: EditWidget::default(),
            devices_widget: DevicesWidget::default(),
            log_widget: LogWidget::default(),
            popup: Popup::default(),
            notification: Notification::new(),
            frame_counter: 0,
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
            AppEvent::Popup(title, content, value, callback) => {
                self.popup.open(title, content, value, callback)
            }
            AppEvent::ChangeScript => self.edit_widget.open(&self.state),
            AppEvent::Info(text) => self.notification.info(text),
            AppEvent::Positive(text) => self.notification.positive(text),
            AppEvent::Negative(text) => self.notification.negative(text),
            AppEvent::Quit => self.quit(),
        }
        Ok(())
    }

    pub fn handle_notification(&mut self, notif: SovaNotification) -> color_eyre::Result<()> {
        match notif {
            SovaNotification::Tick
            | SovaNotification::TempoChanged(_)
            | SovaNotification::QuantumChanged(_) => (),
            SovaNotification::UpdatedScene(scene) => self.state.scene_image = scene,
            SovaNotification::UpdatedGlobalMode(m) => self.state.scene_image.set_global_mode(m),
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
                if state.is_err() {
                    self.state
                        .events
                        .send(AppEvent::Negative(state.to_string()));
                }
                let frame = self
                    .state
                    .scene_image
                    .line_mut(line_index)
                    .frame_mut(frame_index);
                *frame.compilation_state_mut() = state;
            }
            SovaNotification::PlaybackStateChanged(state) => self.state.playing = state,
            SovaNotification::FramePositionChanged(positions) => {
                self.state.positions = positions
            }
            SovaNotification::GlobalVariablesChanged(values) => self.state.global_vars = values,
            SovaNotification::Log(msg) => self.log(msg),
            SovaNotification::DeviceListChanged(devices) => self.state.devices = devices,
            SovaNotification::ClientListChanged(_)
            | SovaNotification::ChatReceived(_, _)
            | SovaNotification::PeerStartedEditingFrame(_, _, _)
            | SovaNotification::PeerStoppedEditingFrame(_, _, _)
            | SovaNotification::ScopeData(_) => (),
        }
        Ok(())
    }

    fn log(&mut self, msg: LogMessage) {
        self.log_widget.add_log(msg);
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if self.popup.showing {
            self.popup.process_event(&mut self.state, key_event);
            return Ok(());
        }

        match key_event.code {
            KeyCode::Esc => {
                self.state.events.send(AppEvent::Popup(
                    "Exit Sova ?".to_owned(),
                    "Are you sure you want to quit ?".to_owned(),
                    PopupValue::Bool(false),
                    Box::new(|state, x| {
                        if bool::from(x) {
                            state.events.send(AppEvent::Quit)
                        }
                    }),
                ));
            }

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
                let event = if self.state.playing.is_playing() {
                    SchedulerMessage::TransportStop(ActionTiming::Immediate)
                } else {
                    SchedulerMessage::TransportStart(ActionTiming::Immediate)
                };
                self.state.events.send(event.into())
            }

            _ => match self.state.page {
                Page::Scene => self.scene_widget.process_event(&mut self.state, key_event),
                Page::Edit => self.edit_widget.process_event(&mut self.state, key_event),
                Page::Devices => self
                    .devices_widget
                    .process_event(&mut self.state, key_event),
                Page::Time => TimeWidget::process_event(&mut self.state, key_event),
                Page::Logs => self.log_widget.process_event(key_event),
                Page::Configure => ConfigureWidget::process_event(&mut self.state, key_event),
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
        if self.frame_counter == 0 {
            self.state.refresh_devices();
        }
        self.frame_counter = (self.frame_counter + 1) % (TICK_FPS as u16);
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.state.running = false;
    }
}

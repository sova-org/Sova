use std::collections::HashMap;

use crate::event::{AppEvent, Event, EventHandler};
use crossbeam_channel::{Receiver, Sender};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use sova_core::{lang::variable::VariableValue, schedule::{ActionTiming, SchedulerMessage, SovaNotification}, Scene};

/// Application.
#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub events: EventHandler,
    pub sched_iface: Sender<SchedulerMessage>,
    scene_image: Scene,
    global_vars: HashMap<String, VariableValue>,
    playing: bool,
    positions: Vec<(usize, usize)>
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(sched_iface: Sender<SchedulerMessage>, sched_update: Receiver<SovaNotification>) -> Self {
        App {
            running: false,
            events: EventHandler::new(sched_update),
            sched_iface,
            scene_image: Default::default(),
            global_vars: Default::default(),
            playing: false,
            positions: Default::default(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event)
                    if key_event.kind == crossterm::event::KeyEventKind::Press =>
                {
                    self.handle_key_event(key_event)?
                }
                _ => {}
            }
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
            AppEvent::NextPage => todo!(),
            AppEvent::PreviousPage => todo!(),
            AppEvent::Quit => self.quit(),
        }
        Ok(())
    }

    pub fn handle_notification(&mut self, notif: SovaNotification) -> color_eyre::Result<()> {
        match notif {
            SovaNotification::Nothing => (),
            SovaNotification::UpdatedScene(scene) => self.scene_image = scene,
            SovaNotification::UpdatedLines(items) => {
                for (index, line) in items {
                    self.scene_image.set_line(index, line);
                }
            }
            SovaNotification::UpdatedLineConfigurations(items) => {
                for (index, line) in items {
                    self.scene_image.line_mut(index).configure(&line);
                }
            }
            SovaNotification::AddedLine(index, line) => 
                self.scene_image.insert_line(index, line),
            SovaNotification::RemovedLine(index) => 
                self.scene_image.remove_line(index),
            SovaNotification::UpdatedFrames(items) => {
                for (line_index, frame_index, frame) in items {
                    self.scene_image.line_mut(line_index).set_frame(frame_index, frame);
                }
            },
            SovaNotification::AddedFrame(line_index, frame_index, frame) => 
                self.scene_image.line_mut(line_index).insert_frame(frame_index, frame),
            SovaNotification::RemovedFrame(line_index, frame_index) => 
                self.scene_image.line_mut(line_index).remove_frame(frame_index),
            SovaNotification::CompilationUpdated(line_index, frame_index, _, state) => {
                let frame = self.scene_image.line_mut(line_index).frame_mut(frame_index);
                *frame.compilation_state_mut() = state;
            },
            SovaNotification::TempoChanged(_) => todo!(),
            SovaNotification::Log(log_message) => todo!(),
            SovaNotification::TransportStarted => self.playing = true,
            SovaNotification::TransportStopped => self.playing = false,
            SovaNotification::FramePositionChanged(positions) => self.positions = positions,
            SovaNotification::DeviceListChanged(device_infos) => todo!(),
            SovaNotification::ClientListChanged(_) |
                SovaNotification::ChatReceived(_, _) |
                SovaNotification::PeerStartedEditingFrame(_, _, _) |
                SovaNotification::PeerStoppedEditingFrame(_, _, _) => (),
            SovaNotification::GlobalVariablesChanged(values) => 
                self.global_vars = values,
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char(' ') if key_event.modifiers == KeyModifiers::CONTROL => {
                let event = if self.playing {
                    SchedulerMessage::TransportStop(ActionTiming::Immediate)
                } else {
                    SchedulerMessage::TransportStart(ActionTiming::Immediate)
                };
                self.events.send(event.into())
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

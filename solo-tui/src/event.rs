use color_eyre::eyre::WrapErr;
use crossbeam_channel::{select, unbounded, Receiver, Sender};
use ratatui::crossterm::event::{self, Event as CrosstermEvent};
use sova_core::{LogMessage, schedule::{SchedulerMessage, SovaNotification}};
use std::{
    thread,
    time::{Duration, Instant},
};

use crate::{app::AppState, popup::PopupValue};

const TICK_FPS: f64 = 30.0;

pub enum Event {
    Tick,
    Crossterm(CrosstermEvent),
    App(AppEvent),
    Notification(SovaNotification)
}

impl From<AppEvent> for Event {
    fn from(value: AppEvent) -> Self {
        Event::App(value)
    }
}
impl From<SovaNotification> for Event {
    fn from(value: SovaNotification) -> Self {
        Event::Notification(value)
    }
}

impl From<LogMessage> for Event {
    fn from(value: LogMessage) -> Self {
        Event::Notification(SovaNotification::Log(value))
    }
}

pub enum AppEvent {
    SchedulerControl(SchedulerMessage),
    Right,
    Left,
    Up,
    Down,
    Popup(String, String, PopupValue, Box<dyn FnOnce(&mut AppState, PopupValue) + Send>),
    ChangeScript,
    Info(String),
    Positive(String),
    Negative(String),
    Quit,
}

impl From<SchedulerMessage> for AppEvent {
    fn from(value: SchedulerMessage) -> Self {
        AppEvent::SchedulerControl(value)
    }
}
/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    notifications: Receiver<SovaNotification>,
    log_rx: Receiver<LogMessage>
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new(notifications: Receiver<SovaNotification>, log_rx: Receiver<LogMessage>) -> Self {
        let (sender, receiver) = unbounded();
        let actor = EventThread::new(sender.clone());
        thread::spawn(|| actor.run());
        Self { sender, receiver, notifications, log_rx }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    pub fn next(&self) -> color_eyre::Result<Event> {
        select! {
            recv(self.receiver) -> ev => Ok(ev?),
            recv(self.notifications) -> notif => Ok(notif?.into()),
            recv(self.log_rx) -> log => Ok(log?.into())
        }
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&mut self, app_event: AppEvent) {
        let _ = self.sender.send(app_event.into());
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventThread {
    /// Event sender channel.
    sender: Sender<Event>,
}

impl EventThread {
    /// Constructs a new instance of [`EventThread`].
    fn new(sender: Sender<Event>) -> Self {
        Self { sender }
    }

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    fn run(self) -> color_eyre::Result<()> {
        let tick_interval = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut last_tick = Instant::now();
        loop {
            // emit tick events at a fixed rate
            let timeout = tick_interval.saturating_sub(last_tick.elapsed());
            if timeout == Duration::ZERO {
                last_tick = Instant::now();
                self.send(Event::Tick);
            }
            // poll for crossterm events, ensuring that we don't block the tick interval
            if event::poll(timeout).wrap_err("failed to poll for crossterm events")? {
                let event = event::read().wrap_err("failed to read crossterm event")?;
                self.send(Event::Crossterm(event));
            }
        }
    }

    fn send(&self, event: Event) {
        let _ = self.sender.send(event);
    }
}

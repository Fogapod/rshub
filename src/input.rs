use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};

#[derive(Debug)]
pub enum UserInput {
    Quit,

    // directions
    Up,
    Down,
    Left,
    Right,
    // gotos
    Top,
    Bottom,
    // actions
    Back,
    Enter,
    Delete,
    // misc
    Tab,
    Help,
    Refresh,
    // custom
    Char(char),
}

impl UserInput {
    fn from(event: Event) -> Option<Self> {
        match event {
            Event::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Char('c' | 'C'),
                    modifiers: KeyModifiers::CONTROL,
                } => Some(Self::Quit),
                KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                } => Some(Self::Char(c)),
                KeyEvent {
                    code: KeyCode::Left,
                    ..
                } => Some(Self::Left),
                KeyEvent {
                    code: KeyCode::Right,
                    ..
                } => Some(Self::Right),
                KeyEvent {
                    code: KeyCode::Up, ..
                } => Some(Self::Up),
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => Some(Self::Down),
                KeyEvent {
                    code: KeyCode::Home,
                    ..
                } => Some(Self::Top),
                KeyEvent {
                    code: KeyCode::End, ..
                } => Some(Self::Bottom),
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => Some(Self::Back),
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => Some(Self::Enter),
                KeyEvent {
                    code: KeyCode::Delete | KeyCode::Backspace,
                    ..
                } => Some(Self::Delete),
                KeyEvent {
                    code: KeyCode::Tab, ..
                } => Some(Self::Tab),
                KeyEvent {
                    code: KeyCode::F(1),
                    ..
                } => Some(Self::Help),
                KeyEvent {
                    code: KeyCode::F(5),
                    ..
                } => Some(Self::Refresh),
                _ => None,
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => Some(Self::Up),
                MouseEventKind::ScrollDown => Some(Self::Down),
                _ => None,
            },
            _ => None,
        }
    }
}

pub(crate) enum EventOrTick<I> {
    Input(I),
    Tick,
}

pub(crate) fn spawn_input_thread(interval: Duration) -> mpsc::Receiver<EventOrTick<UserInput>> {
    let (tx, rx) = mpsc::channel();

    thread::Builder::new()
        .name("input".to_owned())
        .spawn(move || loop {
            let event = if event::poll(interval).unwrap() {
                if let Some(valid_input) = UserInput::from(event::read().unwrap()) {
                    EventOrTick::Input(valid_input)
                } else {
                    EventOrTick::Tick
                }
            } else {
                EventOrTick::Tick
            };

            if let Err(e) = tx.send(event) {
                log::error!("failed to send input event, probably closed channel: {}", e);
                break;
            }
        })
        .expect("unable to spawn input thread");

    rx
}

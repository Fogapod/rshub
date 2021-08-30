use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event};

#[derive(Debug)]
pub enum UserInput<'a> {
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
    // custom
    Char(&'a char),
}

pub(crate) enum EventOrTick<I> {
    Input(I),
    Tick,
}

pub(crate) fn spawn_input_thread(interval: Duration) -> mpsc::Receiver<EventOrTick<Event>> {
    let (tx, rx) = mpsc::channel();

    thread::Builder::new()
        .name("input".to_owned())
        .spawn(move || loop {
            let event = if event::poll(interval).unwrap() {
                EventOrTick::Input(event::read().unwrap())
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

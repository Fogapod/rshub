use std::time::Duration;

use std::thread;

use crossterm::event::{self, Event};

#[derive(Debug)]
pub enum UserInput {
    Up,
    Down,
    Left,
    Right,
    Back,
    Enter,
    Tab,
    Char(char),
}

pub(crate) enum EventOrTick<I> {
    Input(I),
    Tick,
}

pub(crate) fn spawn_input_thread(
    interval: u64,
    tx: std::sync::mpsc::Sender<EventOrTick<Event>>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("input".to_owned())
        .spawn(move || {
            let tick_rate = Duration::from_millis(interval);

            loop {
                let event = if event::poll(tick_rate).unwrap() {
                    EventOrTick::Input(event::read().unwrap())
                } else {
                    EventOrTick::Tick
                };

                if let Err(e) = tx.send(event) {
                    log::error!("failed to send input event, probably closed channel: {}", e);
                    break;
                }
            }
        })
        .expect("unable to spawn input thread")
}

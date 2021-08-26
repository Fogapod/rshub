use std::time::{Duration, Instant};

use std::thread;

use crossterm::event::{self, Event as CEvent};

#[derive(Debug)]
pub enum UserInput {
    Up,
    Down,
    Left,
    Right,
    Back,
    Enter,
}

pub(crate) enum Event<I> {
    Input(I),
    Tick,
}

pub(crate) fn spawn_input_thread(
    interval: u64,
    tx: std::sync::mpsc::Sender<Event<crossterm::event::KeyEvent>>,
) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("input".to_owned())
        .spawn(move || {
            let tick_rate = Duration::from_millis(interval);
            let mut last_tick = Instant::now();

            loop {
                // poll for tick rate duration, if no events, sent tick event.
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                if event::poll(timeout).unwrap() {
                    if let CEvent::Key(key) = event::read().unwrap() {
                        tx.send(Event::Input(key)).unwrap();
                    }
                }
                if last_tick.elapsed() >= tick_rate {
                    tx.send(Event::Tick).unwrap();
                    last_tick = Instant::now();
                }
            }
        })
        .expect("unable to spawn input thread")
}

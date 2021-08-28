mod app;
mod constants;
mod datatypes;
mod geolocation;
mod input;
mod states;
mod views;
mod waitable_mutex;

use std::{env, io, sync::mpsc};

use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use log::LevelFilter;

use crate::input::{spawn_input_thread, EventOrTick};

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        // cannot show cursor without terminal instance
        cleanup_terminal::<CrosstermBackend<io::Stdout>>(None);
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
}

fn cleanup_terminal<B: Backend>(terminal: Option<&mut Terminal<B>>) {
    let mut stdout = io::stdout();

    disable_raw_mode().unwrap();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).unwrap();

    if let Some(terminal) = terminal {
        terminal.show_cursor().unwrap();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    better_panic::install();

    // TODO: temp file, platform specific
    // TODO: configure level
    simple_logging::log_to_file("test.log", LevelFilter::Debug).unwrap();

    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        let data_dir =
            { env::var("XDG_DATA_HOME").unwrap_or_else(|_| "~/.local/share".to_string()) };

        log::debug!("data dir: {}", data_dir);
    }

    let mut app = app::App::new();

    let handles = app.spawn_threads();

    setup_panic_hook();

    let mut terminal = {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);

        Terminal::new(backend)?
    };

    let (tx, rx) = mpsc::channel();
    spawn_input_thread(250, tx);

    loop {
        // TODO: only draw when something changed
        terminal.draw(|f| app.draw(f))?;

        match rx.recv()? {
            EventOrTick::Input(event) => app.on_input(&event),
            EventOrTick::Tick => {}
        }

        if app.stopped {
            break;
        }
    }

    drop(rx);

    for handle in handles {
        log::info!("joining {:?}", handle.thread().name());
        handle.join().unwrap();
    }

    cleanup_terminal(Some(&mut terminal));

    Ok(())
}

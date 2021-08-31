mod app;
mod config;
mod constants;
mod datatypes;
mod errors;
mod input;
mod states;
mod views;

use std::io;
use std::sync::atomic::Ordering;
use std::time::Duration;

use tui::{backend::CrosstermBackend, Terminal};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use log::LevelFilter;

use crate::config::AppConfig;
use crate::input::{spawn_input_thread, EventOrTick};

fn setup_panic_hook() {
    #[cfg(not(debug_assertions))]
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        cleanup_terminal(None);

        #[cfg(debug_assertions)]
        {
            better_panic::Settings::auto().create_panic_handler()(panic_info);
        }
        #[cfg(not(debug_assertions))]
        {
            original_hook(panic_info);
        }
    }));
}

fn cleanup_terminal(terminal: Option<&mut Terminal<CrosstermBackend<io::Stdout>>>) {
    disable_raw_mode().unwrap();

    let mut stdout = io::stdout();

    // FIXME: if mouse is outside terminal, it is not released properly and garbage
    // is printed after panic
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).unwrap();

    if let Some(terminal) = terminal {
        terminal.show_cursor().unwrap();
    }
}

const fn verbosity_to_log_level(verbosity: u32) -> LevelFilter {
    let mut verbosity = verbosity;

    #[cfg(debug_assertions)]
    {
        // jump straight to debug
        verbosity += 3;
    }

    match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}
fn setup_logger(config: &AppConfig) -> Result<(), io::Error> {
    simple_logging::log_to_file(
        &config.dirs.log_file,
        verbosity_to_log_level(config.verbose),
    )
}

fn _main() -> Result<(), Box<dyn std::error::Error>> {
    let config: AppConfig = AppConfig::new()?;

    setup_logger(&config)?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let mut app = rt.block_on(app::App::new(config));

    let mut terminal = {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);

        Terminal::new(backend)?
    };

    {
        let rx = spawn_input_thread(Duration::from_millis(200));

        loop {
            // TODO: only draw when something changed
            terminal.draw(|f| {
                rt.block_on(app.draw(f));
            })?;

            match rx.recv()? {
                EventOrTick::Input(event) => rt.block_on(app.on_input(&event)),
                EventOrTick::Tick => {}
            }

            if app.stopped {
                log::info!("app stopped, cleaning up");

                cleanup_terminal(Some(&mut terminal));

                break;
            }

            if app.panicked.load(Ordering::Relaxed) {
                // IMPORTANT: do not cleanup terminal, this is done in panic hook
                log::error!("app panicked, cleaning up");

                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_panic_hook();

    // TODO: actual error processing
    _main()?;

    Ok(())
}

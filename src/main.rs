mod app;
mod config;
mod constants;
mod datatypes;
mod geolocation;
mod input;
mod states;
mod views;

use std::io;
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
    std::panic::set_hook(Box::new(|panic_info| {
        // cannot show cursor without terminal instance
        cleanup_terminal(None);

        #[cfg(debug_assertions)]
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
}

fn cleanup_terminal(terminal: Option<&mut Terminal<CrosstermBackend<io::Stdout>>>) {
    let mut stdout = io::stdout();

    disable_raw_mode().unwrap();
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
        config.args.log_file.clone().unwrap_or_else(|| {
            config
                .data_dir
                // TODO: rotate by count or date or something
                .join(format!("{}.log", env!("CARGO_PKG_NAME")))
        }),
        verbosity_to_log_level(config.args.verbose),
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
                break;
            }
        }
    }

    cleanup_terminal(Some(&mut terminal));

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    better_panic::install();

    setup_panic_hook();

    // TODO: actual error processing
    _main()?;

    Ok(())
}

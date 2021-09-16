use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use log::LevelFilter;

use tui::{backend::CrosstermBackend, Terminal};

pub fn create_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    Terminal::new(backend)
}

pub fn cleanup_terminal(terminal: Option<&mut Terminal<CrosstermBackend<io::Stdout>>>) {
    disable_raw_mode().unwrap();

    let mut stdout = io::stdout();

    // FIXME: if mouse is outside terminal, it is not released properly and garbage
    // is printed after panic
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture).unwrap();

    if let Some(terminal) = terminal {
        terminal.show_cursor().unwrap();
    }
}

pub const fn verbosity_to_log_level(verbosity: u32) -> LevelFilter {
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

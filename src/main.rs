mod app;
mod config;
mod constants;
mod datatypes;
mod input;
mod states;
mod utils;
mod views;

use std::io;

use log::LevelFilter;

use crate::config::AppConfig;
use crate::utils::{cleanup_terminal, create_terminal, verbosity_to_log_level};

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

fn setup_logger(config: &AppConfig) -> Result<(), io::Error> {
    simplelog::WriteLogger::init(
        verbosity_to_log_level(config.verbose),
        simplelog::ConfigBuilder::default()
            .set_level_padding(simplelog::LevelPadding::Right)
            .set_thread_level(LevelFilter::Off)
            .add_filter_ignore_str("mio")
            .add_filter_ignore_str("want")
            .add_filter_ignore_str("rustls")
            .add_filter_ignore_str("reqwest")
            .add_filter_ignore_str("tokio_util")
            .build(),
        std::fs::File::create(&config.dirs.log_file)?,
    )
    .expect("creating logger");

    Ok(())
}

fn _main() -> Result<(), Box<dyn std::error::Error>> {
    let config: AppConfig = AppConfig::new()?;

    setup_logger(&config)?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let mut app = app::App::new(config);

    let mut terminal = create_terminal()?;

    rt.block_on(app.run(&mut terminal));

    cleanup_terminal(Some(&mut terminal));

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_panic_hook();

    // TODO: actual error processing
    _main()?;

    Ok(())
}

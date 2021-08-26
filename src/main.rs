mod app;
mod geolocation;
mod types;
mod ui;

use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle},
    time,
};
use std::{
    env, io,
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};

use parking_lot::{Condvar, Mutex, RwLock};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use log::LevelFilter;

use crate::app::{App, ServersState};
use crate::geolocation::{ip_to_location, Location, IP};
use crate::types::{Server, ServerListData};

const SERVER_LIST_URL: &str = "https://api.unitystation.org/serverlist";
const GITHUB_REPO_URL: &str = "https://github.com/unitystation/unitystation";

type StopLock = Arc<(Mutex<bool>, Condvar)>;

enum Event<I> {
    Input(I),
    Tick,
}

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

pub fn spawn_location_fetch_thread(
    interval: u64,
    locations: Arc<RwLock<HashMap<IP, Location>>>,
    stop_lock: StopLock,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("location_queue".to_owned())
        .spawn(move || loop {
            let duration = time::Duration::from_secs(interval);

            let location = ip_to_location(IP::Local, &locations).unwrap();

            {
                let locations = locations.read();
                log::info!("{:?}", &locations);
            };

            let mut lock = stop_lock.0.lock();

            if !stop_lock.1.wait_for(&mut lock, duration).timed_out() {
                break;
            }
        })
        .expect("failed to build thread")
}

pub fn spawn_server_fetch_thread(
    interval: u64,
    servers: Arc<RwLock<ServersState>>,
    stop_lock: StopLock,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("server_fetch".to_owned())
        .spawn(move || {
            let duration = time::Duration::from_secs(interval);

            let loop_body = move || {
                servers.write().clear_error();

                let req = match reqwest::blocking::get(SERVER_LIST_URL) {
                    Ok(req) => req,
                    Err(err) => {
                        log::error!("{}", err);
                        servers
                            .write()
                            .set_error(&format!("error making request: {}", err));
                        return;
                    }
                };

                let resp = match req.json::<ServerListData>() {
                    Ok(resp) => resp,
                    Err(err) => {
                        log::error!("{}", err);
                        servers
                            .write()
                            .set_error(&format!("error decoding response: {}", err));
                        return;
                    }
                };

                {
                    let mut servers = servers.write();
                    servers.update(resp);
                    log::info!("{:?}", &servers);
                }
            };

            loop {
                loop_body();

                let mut lock = stop_lock.0.lock();

                if !stop_lock.1.wait_for(&mut lock, duration).timed_out() {
                    break;
                }
            }
        })
        .expect("failed to build thread")
}

fn spawn_threads(app: &mut App) -> Vec<JoinHandle<()>> {
    vec![
        spawn_server_fetch_thread(20, app.servers.clone(), app.stop_lock.clone()),
        spawn_location_fetch_thread(
            5,
            app.servers.read().locations.clone(),
            app.stop_lock.clone(),
        ),
    ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    better_panic::install();

    // TODO: temp file, platform specific
    // TODO: configure level
    simple_logging::log_to_file("test.log", LevelFilter::Debug).unwrap();

    #[cfg(any(target_os = "linux", target_os = "android"))]
    let data_dir = env::var("XDG_DATA_HOME").unwrap_or_else(|_| "~/.local/share".to_string());

    log::debug!("data dir: {}", data_dir);

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let tick_rate = Duration::from_millis(250);
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
    });

    let mut app = app::App::new();

    let handles = spawn_threads(&mut app);

    setup_panic_hook();

    let mut terminal = {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?; //, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);

        Terminal::new(backend)?
    };

    loop {
        // if app.state_changed() {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        // }

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    terminal.draw(|f| ui::draw_exit_view(f, &mut app))?;

                    break;
                }
                KeyCode::Left => {
                    app.on_left();
                }
                KeyCode::Right => {
                    app.on_right();
                }
                KeyCode::Up => {
                    app.on_up();
                }
                KeyCode::Down => {
                    app.on_down();
                }
                KeyCode::Esc | KeyCode::Backspace => {
                    app.on_back();
                }
                KeyCode::Enter => {
                    app.on_enter();
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    {
        log::warn!("waiting stop lock");
        let mut stop = app.stop_lock.0.lock();
        *stop = true;
        log::warn!("set stop lock");
        app.stop_lock.1.notify_all();
        log::warn!("notified stop lock");
    }

    for handle in handles {
        log::info!("joining {:?}", handle.thread().name());
        handle.join().unwrap();
    }

    cleanup_terminal(Some(&mut terminal));

    Ok(())
}

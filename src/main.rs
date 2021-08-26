mod app;
mod geolocation;
mod input;
mod types;
mod views;

use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle},
    time,
};
use std::{env, io, sync::mpsc};

use parking_lot::{Condvar, Mutex, RwLock};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crossterm::{
    event::{DisableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use log::LevelFilter;

use crate::app::App;
use crate::geolocation::{ip_to_location, Location, IP};
use crate::input::{spawn_input_thread, Event, UserInput};
use crate::types::{Server, ServerListData};

const SERVER_LIST_URL: &str = "https://api.unitystation.org/serverlist";
// const GITHUB_REPO_URL: &str = "https://github.com/unitystation/unitystation";

type StopLock = Arc<(Mutex<bool>, Condvar)>;

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

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn update_servers(servers: &mut HashMap<String, Server>, data: ServerListData) {
    let mut existing = servers.clone();

    for sv in data.servers {
        if let Some(sv_existing) = servers.get_mut(&sv.ip) {
            existing.remove(&sv.ip);

            if calculate_hash(&sv_existing.data) != calculate_hash(&sv) {
                sv_existing.updated = true;
            }

            sv_existing.offline = false;
            sv_existing.data = sv;
        } else {
            servers.insert(sv.ip.clone(), Server::new(&sv));
        }
    }

    for ip in existing.keys() {
        servers.get_mut(ip).unwrap().offline = true;
    }
}

pub fn spawn_server_fetch_thread(
    interval: u64,
    servers: Arc<RwLock<HashMap<String, Server>>>,
    stop_lock: StopLock,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("server_fetch".to_owned())
        .spawn(move || {
            let duration = time::Duration::from_secs(interval);

            let loop_body = move || {
                //servers.write().clear_error();

                let req = match reqwest::blocking::get(SERVER_LIST_URL) {
                    Ok(req) => req,
                    Err(err) => {
                        log::error!("{}", err);
                        // servers
                        //     .write()
                        //     .set_error(&format!("error making request: {}", err));
                        return;
                    }
                };

                let resp = match req.json::<ServerListData>() {
                    Ok(resp) => resp,
                    Err(err) => {
                        log::error!("{}", err);
                        // servers
                        //     .write()
                        //     .set_error(&format!("error decoding response: {}", err));
                        return;
                    }
                };

                {
                    let mut servers = servers.write();
                    update_servers(&mut servers, resp);
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
        spawn_server_fetch_thread(20, app.state.servers.clone(), app.state.stop_lock.clone()),
        // spawn_location_fetch_thread(
        //     5,
        //     app.servers.read().locations.clone(),
        //     app.stop_lock.clone(),
        // ),
    ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    better_panic::install();

    // TODO: temp file, platform specific
    // TODO: configure level
    simple_logging::log_to_file("test.log", LevelFilter::Debug).unwrap();

    #[cfg(any(target_os = "linux", target_os = "android"))]
    let data_dir = { env::var("XDG_DATA_HOME").unwrap_or_else(|_| "~/.local/share".to_string()) };

    log::debug!("data dir: {}", data_dir);

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

    let (tx, rx) = mpsc::channel();
    spawn_input_thread(250, tx);

    loop {
        // if app.state_changed() {
        terminal.draw(|f| app.draw(f))?;
        // }

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    // terminal.draw(|f| ui::draw_exit_view(f, &mut app))?;

                    break;
                }
                KeyCode::Left => {
                    app.on_input(&UserInput::Left);
                }
                KeyCode::Right => {
                    app.on_input(&UserInput::Right);
                }
                KeyCode::Up => {
                    app.on_input(&UserInput::Up);
                }
                KeyCode::Down => {
                    app.on_input(&UserInput::Down);
                }
                KeyCode::Esc | KeyCode::Backspace => {
                    app.on_input(&UserInput::Back);
                }
                KeyCode::Enter => {
                    app.on_input(&UserInput::Enter);
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    {
        log::warn!("waiting stop lock");
        let mut stop = app.state.stop_lock.0.lock();
        *stop = true;
        log::warn!("set stop lock");
        app.state.stop_lock.1.notify_all();
        log::warn!("notified stop lock");
    }

    for handle in handles {
        log::info!("joining {:?}", handle.thread().name());
        handle.join().unwrap();
    }

    cleanup_terminal(Some(&mut terminal));

    Ok(())
}

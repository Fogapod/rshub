use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use parking_lot::{Condvar, Mutex, RwLock};
use tui::widgets::TableState;

use crate::geolocation::{Location, IP};
use crate::types::{Server, ServerListData};
use crate::ui::{CommitsTab, InstallationsTab, ServersTab, Tab};

pub trait Window {
    fn on_up(&mut self, app: &mut App) -> bool {
        false
    }

    fn on_down(&mut self, app: &mut App) -> bool {
        false
    }

    fn on_left(&mut self, app: &mut App) -> bool {
        app.tabs.previous();
        true
    }

    fn on_right(&mut self, app: &mut App) -> bool {
        app.tabs.next();
        true
    }

    fn on_enter(&mut self, app: &mut App) -> bool {
        false
    }

    fn on_back(&mut self, app: &mut App) -> bool {
        false
    }
}

type StopLock = Arc<(Mutex<bool>, Condvar)>;

pub(crate) struct TabsState {
    pub tabs: Vec<Box<dyn Tab>>,
    pub index: usize,
    _state_changed: bool,
}

impl Window for TabsState {}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl TabsState {
    pub fn new(tabs: Vec<Box<dyn Tab>>) -> Self {
        Self {
            tabs,
            index: 0,
            _state_changed: true,
        }
    }

    pub fn next(&mut self) {
        if !self.tabs.is_empty() {
            self._state_changed = true;

            self.index = (self.index + 1) % self.tabs.len();
        }
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self._state_changed = true;
            self.index -= 1;
        } else if self.index != 0 {
            self._state_changed = true;
            self.index = self.tabs.len() - 1;
        }

        // count is 0
    }

    pub(crate) fn state_chaned(&mut self) -> bool {
        if self._state_changed {
            self._state_changed = false;
            true
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct ServersState {
    pub servers: HashMap<String, Server>,
    pub locations: Arc<RwLock<HashMap<IP, Location>>>,
    pub index: usize,
    pub error: Option<String>,
    pub(crate) state: TableState,
    _state_changed: bool,
}

impl Default for ServersState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServersState {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            locations: Arc::new(RwLock::new(HashMap::new())),
            index: 0,
            error: None,
            state: TableState::default(),
            _state_changed: true,
        }
    }

    pub fn count(&self) -> usize {
        self.servers.len()
    }

    pub fn next(&mut self) {
        if self.count() == 0 {
            self.state.select(None);

            return;
        }

        self._state_changed = true;

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.count() - 1 {
                    // jump back to last item
                    self.count() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.count() == 0 {
            self.state.select(None);

            return;
        }

        self._state_changed = true;

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn update(&mut self, data: ServerListData) {
        let mut existing = self.servers.clone();

        for sv in data.servers {
            if let Some(sv_existing) = self.servers.get_mut(&sv.ip) {
                existing.remove(&sv.ip);

                if calculate_hash(&sv_existing.data) != calculate_hash(&sv) {
                    sv_existing.updated = true;
                    self._state_changed = true;
                }

                sv_existing.offline = false;
                sv_existing.data = sv;
            } else {
                self.servers
                    .insert(sv.ip.clone(), Server::new(&sv, &self.locations));
            }
        }

        for ip in existing.keys() {
            self.servers.get_mut(ip).unwrap().offline = true;
        }
    }

    pub fn set_error(&mut self, error: &str) {
        self._state_changed = true;

        self.error = Some(error.to_owned());
    }

    pub fn clear_error(&mut self) {
        self._state_changed = true;

        self.error = None;
    }

    pub(crate) fn state_chaned(&mut self) -> bool {
        if self._state_changed {
            self._state_changed = false;
            true
        } else {
            false
        }
    }
}

impl Window for ServersState {}

pub struct App<'a> {
    pub servers: Arc<RwLock<ServersState>>,
    pub(crate) tabs: TabsState,
    pub(crate) stop_lock: StopLock,
    window_stack: Vec<&'a dyn Window>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let tabs_view = TabsState::new(vec![
            Box::new(ServersTab {}),
            Box::new(InstallationsTab {}),
            Box::new(CommitsTab {}),
        ]);
        Self {
            servers: Arc::new(RwLock::new(ServersState::new())),
            tabs: tabs_view,
            stop_lock: Arc::new((Mutex::new(false), Condvar::new())),
            window_stack: vec![], //vec![&tabs_view],
        }
    }

    pub(crate) fn state_changed(&mut self) -> bool {
        self.tabs.state_chaned() || self.servers.write().state_chaned()
    }

    pub(crate) fn on_left(&mut self) {
        //for window in self.window_stack.iter().rev() {
        //   if window.on_left(self) {
        //      return;
        //    }
        //}

        self.tabs.previous();
    }

    pub(crate) fn on_right(&mut self) {
        //for window in self.window_stack.iter().rev() {
        //    if window.on_right(self) {
        //        return;
        //    }
        //}

        self.tabs.next();
    }

    pub(crate) fn on_up(&mut self) {
        if self.tabs.index == 0 {
            self.servers.write().previous();
        }
    }

    pub(crate) fn on_down(&mut self) {
        if self.tabs.index == 0 {
            self.servers.write().next();
        }
    }

    pub(crate) fn on_back(&mut self) {
        if self.tabs.index == 0 {
            self.servers.write().unselect();
        }
    }

    pub(crate) fn on_enter(&mut self) {}
}

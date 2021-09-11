pub mod commits;
pub mod events;
pub mod help;
pub mod servers;
pub mod tabs;
pub mod versions;
#[cfg(feature = "geolocation")]
pub mod world;

use std::io;
use std::sync::Arc;

use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::terminal::Frame;

use crate::app::AppAction;
use crate::datatypes::hotkey::HotKey;
use crate::input::UserInput;
use crate::states::AppState;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ViewType {
    Tab,
    #[cfg(feature = "geolocation")]
    World,
    Help,
}

pub trait Draw {
    fn draw(&self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect, app: Arc<AppState>);
}

#[async_trait::async_trait]
pub trait Input {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction>;
}

pub trait HotKeys {
    fn hotkeys(&self) -> Vec<HotKey> {
        Vec::new()
    }
}

pub trait Name {
    fn name(&self) -> String;
}

pub trait AppView: Draw + Input + HotKeys + Name {}

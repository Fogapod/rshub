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
use crate::input::UserInput;
use crate::states::help::HotKey;
use crate::states::AppState;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ViewType {
    Tab,
    #[cfg(feature = "geolocation")]
    World,
    Help,
}

#[async_trait::async_trait]
pub trait Drawable {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: Arc<AppState>,
    );
}

#[async_trait::async_trait]
pub trait InputProcessor {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction>;
}

pub trait HotKeys {
    fn hotkeys(&self) -> Vec<HotKey> {
        Vec::new()
    }
}

pub trait Named {
    fn name(&self) -> String;
}

pub trait AppView: Drawable + InputProcessor + HotKeys + Named {}

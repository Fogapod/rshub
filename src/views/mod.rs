pub mod commits;
pub mod installations;
pub mod servers;
pub mod tabs;
pub mod world;

use std::io;
use std::sync::Arc;

use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::terminal::Frame;

use crate::app::AppAction;
use crate::input::UserInput;
use crate::states::AppState;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ViewType {
    Tab,
    World,
}

#[async_trait::async_trait]
pub trait Drawable {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &AppState,
    );
}

#[async_trait::async_trait]
pub trait InputProcessor {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction>;
}

pub trait AppView: Drawable + InputProcessor {}

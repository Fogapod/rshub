pub mod commits;
pub mod installations;
pub mod servers;
pub mod tabs;

use std::io;

use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::terminal::Frame;

use crate::app::AppAction;
use crate::input::UserInput;
use crate::states::AppState;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum ViewType {
    Tab,
}

pub trait Drawable {
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect, app: &AppState);
}

pub trait InputProcessor {
    fn on_input(&mut self, _: &UserInput, _: &AppState) -> Option<AppAction> {
        None
    }
}

pub trait AppView: Drawable + InputProcessor {}

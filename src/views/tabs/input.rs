use std::sync::Arc;

use crate::app::AppAction;
use crate::input::UserInput;
use crate::states::AppState;
use crate::views::Input;

use super::tab::Tab;
use super::Tabs;

#[async_trait::async_trait]
impl Input for Tabs {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction> {
        match input {
            UserInput::Char('s' | 'S') => {
                self.select_tab(Tab::Servers);
                None
            }
            UserInput::Char('v' | 'V') => {
                self.select_tab(Tab::Versions);
                None
            }
            UserInput::Char('c' | 'C') => {
                self.select_tab(Tab::Commits);
                None
            }
            UserInput::Tab => {
                self.selection.select_next(Tab::tab_count());
                None
            }
            // cannot move this to function because of match limitation for arms
            // even if they implement same trait
            _ => match self.selected_tab() {
                Tab::Servers => app.servers.on_input(input, app).await,
                Tab::Versions => app.versions.on_input(input, app).await,
                Tab::Commits => app.commits.on_input(input, app).await,
            },
        }
    }
}

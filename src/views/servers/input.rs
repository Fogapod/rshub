use std::sync::Arc;

use crate::app::AppAction;

use crate::input::UserInput;
use crate::states::AppState;
use crate::views::{Input, ViewType};

use super::Servers;

#[async_trait::async_trait]
impl Input for Servers {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction> {
        match input {
            #[cfg(feature = "geolocation")]
            UserInput::Char('m' | 'M') => Some(AppAction::OpenView(ViewType::World)),
            UserInput::Char('i' | 'I') => {
                if let Some(i) = self.state.read().selection.selected() {
                    Some(AppAction::InstallVersion(
                        self.state.read().items[i].version.clone(),
                    ))
                } else {
                    None
                }
            }
            UserInput::Enter => {
                if let Some(i) = self.state.read().selection.selected() {
                    let server = &self.state.read().items[i];

                    Some(AppAction::ConnectToServer {
                        version: server.version.clone(),
                        address: server.address.clone(),
                    })
                } else {
                    None
                }
            }
            _ => self.state.write().selection.on_input(input, self.count()),
        }
    }
}

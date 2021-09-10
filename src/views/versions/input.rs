use std::sync::Arc;

use crate::app::AppAction;
use crate::input::UserInput;
use crate::states::AppState;
use crate::views::Input;

use super::Versions;

#[async_trait::async_trait]
impl Input for Versions {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction> {
        match input {
            UserInput::Refresh => {
                let mut versions = self.state.write().await;

                versions.refresh(app.clone()).await;

                if let Some(i) = self.selection.selected() {
                    if i >= versions.count() {
                        self.selection.unselect();
                    }
                }

                None
            }
            UserInput::Char('i' | 'I') => {
                if let Some(i) = self.selection.selected() {
                    Some(AppAction::InstallVersion(
                        self.state.read().await.items[i].version.clone(),
                    ))
                } else {
                    None
                }
            }
            UserInput::Char('d' | 'D') => {
                if let Some(i) = self.selection.selected() {
                    Some(AppAction::UninstallVersion(
                        self.state.read().await.items[i].version.clone(),
                    ))
                } else {
                    None
                }
            }
            UserInput::Char('a' | 'A') => {
                if let Some(i) = self.selection.selected() {
                    Some(AppAction::AbortVersionInstallation(
                        self.state.read().await.items[i].version.clone(),
                    ))
                } else {
                    None
                }
            }
            UserInput::Enter => {
                if let Some(i) = self.selection.selected() {
                    Some(AppAction::LaunchVersion(
                        self.state.read().await.items[i].version.clone(),
                    ))
                } else {
                    None
                }
            }
            _ => self
                .selection
                .on_input(input, self.state.read().await.count()),
        }
    }
}

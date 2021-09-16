use std::sync::Arc;

use crate::states::AppState;

#[derive(Copy, Clone)]
pub enum Tab {
    Servers,
    Versions,
    Commits,
}

impl Tab {
    pub fn name(&self, app: Arc<AppState>) -> String {
        match self {
            Self::Servers => {
                format!("servers [{}]", app.servers.count())
            }
            Self::Versions => {
                format!("versions [{}]", app.versions.count())
            }
            Self::Commits => format!("commits [{}]", app.commits.count()),
        }
    }

    pub const fn all() -> [Self; 3] {
        [Self::Servers {}, Self::Versions {}, Self::Commits {}]
    }

    pub const fn tab_count() -> usize {
        Self::all().len()
    }
}

impl From<Tab> for usize {
    fn from(value: Tab) -> usize {
        match value {
            Tab::Servers => 0,
            Tab::Versions => 1,
            Tab::Commits => 2,
        }
    }
}

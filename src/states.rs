mod app;
mod commits;
mod locations;
mod servers;

pub use app::AppState;
pub use commits::CommitState;
pub use locations::LocationsState;
pub use servers::ServersState;

use std::sync::Arc;

pub type SharedAppState = Arc<AppState>;
pub type SharedCommitState = Arc<CommitState>;
pub type SharedLocationsState = Arc<LocationsState>;
pub type SharedServersState = Arc<ServersState>;

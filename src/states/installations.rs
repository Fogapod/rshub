use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::datatypes::installation::{Installation, InstallationAction};

pub struct InstallationsState {
    pub items: Vec<Installation>,
    queue: mpsc::UnboundedSender<InstallationAction>,
}

impl InstallationsState {
    pub async fn new() -> Arc<RwLock<Self>> {
        let (tx, rx) = mpsc::unbounded_channel();

        let instance = Arc::new(RwLock::new(Self {
            items: Vec::new(),
            queue: tx,
        }));

        tokio::task::spawn(Self::installation_handler_task(instance.clone(), rx));

        instance
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub async fn installation_handler_task(
        _installations: Arc<RwLock<InstallationsState>>,
        mut rx: mpsc::UnboundedReceiver<InstallationAction>,
    ) {
        while let Some(installation) = rx.recv().await {
            log::info!("instalaltion event: {:?}", installation);
        }
    }
}

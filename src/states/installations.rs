use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::datatypes::installation::Installation;

pub struct InstallationsState {
    pub items: RwLock<Vec<Installation>>,
    queue: mpsc::UnboundedSender<Installation>,
}

impl InstallationsState {
    pub async fn new() -> Arc<Self> {
        let (tx, rx) = mpsc::unbounded_channel();

        let instance = Arc::new(Self {
            items: RwLock::new(Vec::new()),
            queue: tx,
        });

        tokio::task::spawn(Self::rename_me_task(instance.clone(), rx));

        instance
    }

    pub async fn count(&self) -> usize {
        self.items.read().await.len()
    }

    pub async fn rename_me_task(
        _installations: Arc<InstallationsState>,
        mut rx: mpsc::UnboundedReceiver<Installation>,
    ) {
        while let Some(installation) = rx.recv().await {
            log::info!("instalaltion event: {:?}", installation);
        }
    }
}

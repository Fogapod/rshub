use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;

use tokio::sync::mpsc;

use anyhow::{Context, Error};

use crate::config::AppConfig;
use crate::datatypes::geolocation::{Location, LocationJson, IP};
use crate::states::app::{AppState, TaskResult};

pub struct LocationsState {
    pub items: HashMap<IP, Location>,
    queue: mpsc::UnboundedSender<IP>,
    queue_recv: Option<mpsc::UnboundedReceiver<IP>>,
}

impl LocationsState {
    pub fn new(_: &AppConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            items: HashMap::new(),
            queue: tx,
            queue_recv: Some(rx),
        }
    }

    pub async fn run(&mut self, app: Arc<AppState>) {
        if app.config.offline {
            return;
        }

        let queue_recv = if let Some(queue_recv) = self.queue_recv.take() {
            queue_recv
        } else {
            log::error!("installation state: queue receiver already taken");
            return;
        };

        app.watch_task(tokio::task::spawn(Self::location_fetch_task(
            app.clone(),
            queue_recv,
        )))
        .await;
    }

    pub async fn resolve(&mut self, ip: &IP) {
        if self.items.get(ip).is_some() {
            return;
        }

        self.queue.send(ip.to_owned()).expect("closed channel");
    }

    async fn location_fetch_task(
        app: Arc<AppState>,
        mut rx: mpsc::UnboundedReceiver<IP>,
    ) -> TaskResult {
        while let Some(ip) = rx.recv().await {
            log::debug!("resolving location: {:?}", ip);

            app.watch_task(tokio::spawn(Self::fetch_location(Arc::clone(&app), ip)))
                .await;
        }

        Ok(())
    }

    async fn fetch_location(app: Arc<AppState>, ip: IP) -> TaskResult {
        let mut request = app.client.get(format!("{}/json", app.config.geo_provider));

        if let IP::Remote(ref ip) = ip {
            request = request.query(&[("ip", ip)])
        }

        let location = request
            .send()
            .await
            .with_context(|| "sending location request")?
            .error_for_status()?
            .json::<LocationJson>()
            .await
            .with_context(|| "parsing location request")?;

        let location = Location::try_from(&location).map_err(Error::msg)?;

        log::debug!("resolved location: {:?} -> {:?}", ip, location);

        app.locations.write().items.insert(ip, location);

        Ok(())
    }
}

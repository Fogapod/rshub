use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use anyhow::Result;

use crate::config::AppConfig;
use crate::datatypes::geolocation::{Location, IP};
use crate::states::app::{AppState, TaskResult};

pub struct LocationsState {
    pub items: HashMap<IP, Location>,
    queue: mpsc::UnboundedSender<IP>,
    queue_recv: Option<mpsc::UnboundedReceiver<IP>>,
    geo_provider: reqwest::Url,
}

impl LocationsState {
    pub async fn new(config: &AppConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            items: HashMap::new(),
            queue: tx,
            queue_recv: Some(rx),
            geo_provider: config.geo_provider.clone(),
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

    pub async fn resolve(&mut self, ip: IP) -> Result<()> {
        {
            if self.items.get(&ip).is_some() {
                return Ok(());
            }
        }

        self.queue.send(ip)?;

        Ok(())
    }

    async fn location_fetch_task(
        app: Arc<AppState>,
        mut rx: mpsc::UnboundedReceiver<IP>,
    ) -> TaskResult {
        let geo_provider = app.locations.read().await.geo_provider.clone();
        let errors = Arc::new(RwLock::new(Vec::new()));

        while let Some(ip) = rx.recv().await {
            log::debug!("resolving location: {:?}", ip);

            let client = app.client.clone();
            let locations = app.locations.clone();
            let geo_provider = geo_provider.clone();
            let errors = errors.clone();

            tokio::spawn(async move {
                let mut request = client.get(format!("{}/json", geo_provider));

                if let IP::Remote(ref ip) = ip {
                    request = request.query(&[("ip", ip)])
                }

                let response = request.send().await;

                let response = match response {
                    Ok(response) => response,
                    Err(err) => {
                        log::error!("error sending location request: {}", &err);
                        errors.write().await.push(err);
                        return;
                    }
                };

                let location = match response.json::<Location>().await {
                    Ok(location) => location,
                    Err(err) => {
                        log::error!("error parsing location request: {}", &err);
                        errors.write().await.push(err);
                        return;
                    }
                };

                log::debug!("resolved location: {:?} -> {:?}", ip, location);

                if location.is_valid() {
                    locations.write().await.items.insert(ip, location);
                } else {
                    log::error!("bad location: {:?}", location);
                }
            });
        }

        Ok(())
    }
}

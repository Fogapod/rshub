use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::datatypes::geolocation::{Location, IP};
use crate::states::app::{TaskQueue, TaskResult};

pub struct LocationsState {
    pub items: HashMap<IP, Location>,
    queue: mpsc::UnboundedSender<IP>,
    geo_provider: reqwest::Url,
}

impl LocationsState {
    pub async fn new(config: &AppConfig, tasks: TaskQueue) -> Arc<RwLock<Self>> {
        let (tx, rx) = mpsc::unbounded_channel::<IP>();

        let instance = Arc::new(RwLock::new(Self {
            items: HashMap::new(),
            queue: tx,
            geo_provider: config.geo_provider.clone(),
        }));

        tasks
            .read()
            .await
            .send(tokio::task::spawn(Self::location_fetch_task(
                instance.clone(),
                rx,
            )))
            .expect("spawn location task");

        instance
    }

    pub async fn resolve(&mut self, ip: IP) -> Result<(), Box<dyn std::error::Error>> {
        {
            if self.items.get(&ip).is_some() {
                return Ok(());
            }
        }

        self.queue.send(ip)?;

        Ok(())
    }

    async fn location_fetch_task(
        locations: Arc<RwLock<Self>>,
        mut rx: mpsc::UnboundedReceiver<IP>,
    ) -> TaskResult {
        let client = Arc::new(
            reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("creating client"),
        );

        let geo_provider = locations.read().await.geo_provider.clone();
        let errors = Arc::new(RwLock::new(Vec::new()));

        while let Some(ip) = rx.recv().await {
            log::debug!("resolving location: {:?}", ip);

            let client = client.clone();
            let locations = locations.clone();
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

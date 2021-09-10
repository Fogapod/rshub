use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;

use crate::constants::SERVER_LIST_URL;
#[cfg(feature = "geolocation")]
use crate::datatypes::geolocation::IP;
use crate::datatypes::server::ServerListJson;
use crate::states::app::TaskResult;
use crate::states::AppState;

pub async fn server_fetch_task(app: Arc<AppState>) -> TaskResult {
    #[cfg(feature = "geolocation")]
    app.locations.write().await.resolve(&IP::Local).await;

    async fn loop_body(app: AppState) -> anyhow::Result<()> {
        let data = app
            .client
            .get(SERVER_LIST_URL)
            .send()
            .await
            .with_context(|| "sending server list request")?
            .error_for_status()?
            .json::<ServerListJson>()
            .await
            .with_context(|| "parsing server list response")?;

        app.servers.write().await.update(app.clone(), data).await;

        Ok(())
    }

    let interval = Duration::from_secs(app.config.update_interval);
    loop {
        if let Err(err) = loop_body(app).await {
            app.events.read().await.error(err).await;
        }

        tokio::time::sleep(interval).await;
    }
}

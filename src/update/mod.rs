use std::sync::Arc;
use std::time::Duration;
use log::info;
use tokio::time::sleep;
use crate::state::AppState;

pub async fn run(app_state: Arc<AppState>) -> Result<(), String> {
    loop {
        let suppliers = app_state
            .get_suppliers(300, 1)
            .await?;
        for s in suppliers {
            info!("{}", s);
        }

        info!("sleep 10 sec");
        sleep(Duration::from_secs(10)).await;
    }
}

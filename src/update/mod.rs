use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;
use log::{info, warn};
use tokio::time::sleep;
use crate::state::AppState;
use crate::wb::calculate_and_set_price;

const PAUSE: u64 = 1 * 60;

pub async fn run(state: Arc<AppState>) -> Result<(), String> {
    loop {
        let suppliers = match state.get_suppliers(300, 1).await {
            Ok(suppliers) => suppliers,
            Err(e) => {
                return Err(format!("Failed to get suppliers: {}", e));
            }
        };

        for supplier in suppliers {
            if let (Some(wb_id), Some(wb_jwt)) = (supplier.wb_id, supplier.wb_jwt.as_ref()) {
                match calculate_and_set_price(
                    supplier.wb_id,
                    wb_jwt,
                    join_all(
                        supplier
                            .goods
                            .iter()
                            .map(|(_, good)| async {
                                state.task_manager.remove_task(good.id).await;
                                good.clone()
                            }))
                        .await,
                ).await {
                    Ok((_, __, handle)) => {
                        if let Some(handle) = handle {
                            state.task_manager.add_task(wb_id, handle).await;
                        }
                        info!("Processed supplier: {}", supplier);
                    }
                    Err(err) => warn!("Failed background update sid={:?}: {}", supplier.wb_id, err)
                };
            }
        }

        info!("Sleeping for {} seconds", PAUSE);
        sleep(Duration::from_secs(PAUSE)).await;
    }
}


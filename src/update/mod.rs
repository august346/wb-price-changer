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
            if let Some(wb_jwt) = supplier.wb_jwt.as_ref() {
                let goods = match state.get_goods(&supplier.api_key).await {
                    Ok(goods) => goods,
                    Err(e) => {
                        warn!("Failed to fetch goods for supplier {}: {}", supplier.api_key, e);
                        continue;
                    }
                };

                if let Err(err) = calculate_and_set_price(
                    supplier.wb_id,
                    wb_jwt,
                    goods
                ).await {
                    warn!("Failed background update sid={:?}: {}", supplier.wb_id, err)
                };
            }
        }

        info!("Sleeping for {} seconds", PAUSE);
        sleep(Duration::from_secs(PAUSE)).await;
    }
}

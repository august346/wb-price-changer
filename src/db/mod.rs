pub mod supplier;
pub mod product;

use std::collections::HashMap;
use tokio::sync::Mutex;
use crate::db::supplier::Supplier;
use crate::utils;

pub struct DB {
    suppliers: Mutex<HashMap<String, Supplier>>
}

impl DB {
    pub fn new() -> Self {
        Self { suppliers: Mutex::new(HashMap::new()) }
    }

    pub async fn get_supplier(&self, api_key: &str) -> Result<Option<Supplier>, String> {
        let suppliers = self.suppliers.lock().await;
        match suppliers.get(api_key) {
            None => Ok(None),
            Some(supplier) => Ok(Some(supplier.clone()))
        }
    }

    pub async fn create_supplier(&self) -> Result<Supplier, String> {
        let api_key = utils::generate_uuid_str();
        let supplier = Supplier::new(&api_key).await?;

        let mut suppliers = self.suppliers.lock().await;

        if suppliers.insert(api_key, supplier.clone()).is_some() {
            return Err("apikey already exists".to_string());
        }

        Ok(supplier)
    }

    pub async fn set_wb_jwt(&self, api_key: &str, jwt: &str) -> Result<(), String> {
        let mut suppliers = self.suppliers.lock().await;
        if let Some(sup) = suppliers.get_mut(api_key) {
            sup.wb_jwt = Some(jwt.to_string());
            return Ok(());
        }

        Err("api_key not found".to_string())
    }

    pub async fn get_suppliers(&self, limit: usize, page: usize) -> Result<Vec<Supplier>, String> {
        let suppliers = self.suppliers.lock().await;

        let start_index = (page - 1) * limit;

        let mut suppliers_to_return: Vec<Supplier> = Vec::new();

        for (i, (_, supplier)) in suppliers.iter().enumerate() {
            if i >= start_index && suppliers_to_return.len() < limit {
                suppliers_to_return.push(supplier.clone());
            }
        }

        Ok(suppliers_to_return)
    }
}
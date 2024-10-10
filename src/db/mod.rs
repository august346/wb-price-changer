pub mod supplier;
pub mod product;

use std::collections::HashMap;
use sqlx::PgPool;
use crate::db::product::Product;
use crate::db::supplier::Supplier;
use crate::utils;

pub struct DB {
    client: PgPool
}

impl DB {
    pub fn new(client: PgPool) -> Self {
        Self { client }
    }

    pub async fn get_supplier(&self, api_key: &str) -> Result<Option<Supplier>, String> {
        Supplier::get(&self.client, api_key).await
    }

    pub async fn create_supplier(&self) -> Result<Supplier, String> {
        let api_key = utils::generate_uuid_str();
        Supplier::create(&self.client, api_key).await
    }

    pub async fn set_wb_jwt(&self, api_key: &str, jwt: &str) -> Result<(), String> {
        Supplier::set_wb_jwt(&self.client, api_key, jwt).await
    }

    pub async fn set_wb_id(&self, api_key: &str, wb_id: i32) -> Result<(), String> {
        Supplier::set_wb_id(&self.client, api_key, wb_id).await
    }

    pub async fn add_goods(&self, api_key: &str, products: &Vec<Product>) -> Result<(), String> {
        Product::create_many(&self.client, api_key, products).await
    }

    pub async fn get_suppliers(&self, limit: usize, page: usize) -> Result<Vec<Supplier>, String> {
        let offset = (page - 1) * limit;

        Supplier::list(&self.client, limit, offset).await
    }
}
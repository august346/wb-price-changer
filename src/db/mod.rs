pub mod supplier;
pub mod product;

use sqlx::PgPool;
use crate::db::product::Product;
use crate::db::supplier::Supplier; p
use sqlx::types::Uuid; 

pub struct DB {
    client: PgPoolh
}

impl DB {
    pub fn new(client: PgPool) -> Self {
        Self { client }
    }

    pub async fn get_supplier(&self, api_key: &Uuid) -> Result<Option<Supplier>, String> {
        Supplier::get(&self.client, api_key).await
    }

    pub async fn create_supplier(&self) -> Result<Supplier, String> {
        Supplier::create(&self.client).await
    }

    pub async fn set_wb_jwt(&self, api_key: &Uuid, jwt: &str) -> Result<(), String> {
        Supplier::set_wb_jwt(&self.client, api_key, jwt).await
    }

    pub async fn set_wb_id(&self, api_key: &Uuid, wb_id: i32) -> Result<(), String> {
        Supplier::set_wb_id(&self.client, api_key, wb_id).await
    }

    pub async fn add_goods(&self, api_key: &Uuid, products: &Vec<Product>) -> Result<(), String> {
        Product::create_many(&self.client, api_key, products).await
    }

    pub async fn get_suppliers(&self, limit: usize, page: usize) -> Result<Vec<Supplier>, String> {
        let offset = (page - 1) * limit;

        Supplier::list(&self.client, limit as i64, offset as i64).await
    }

    pub async fn get_goods(&self, api_key: &Uuid) -> Result<Vec<Product>, String> {
        Product::get_by_apikey(&self.client, api_key).await
    }

    pub async fn count_by_apikey(&self, api_key: &Uuid) -> Result<i64, String> {
        Product::count_by_apikey(&self.client, api_key).await
    }
}
pub mod supplier;
pub mod product;

use sqlx::{Error, PgPool, types::Uuid};
use sqlx::migrate::MigrateError;
use crate::db::product::Product;
use crate::db::supplier::Supplier;
use crate::utils;

pub struct DB {
    client: PgPool,
}

impl DB {
    pub async fn new(db_url: &str) -> Result<Self, String> {
        let client = PgPool::connect(db_url)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get db client"))?;

        Ok(Self { client })
    }

    pub async fn run_migrations(&self) -> Result<(), MigrateError> {
        sqlx::migrate!("./migrations")
            .run(&self.client)
            .await
    }

    pub async fn get_supplier(&self, api_key: &Uuid) -> Result<Option<Supplier>, Error> {
        Supplier::get(&self.client, api_key).await
    }

    pub async fn create_supplier(&self) -> Result<Supplier, Error> {
        Supplier::create(&self.client).await
    }

    pub async fn set_wb_jwt(&self, api_key: &Uuid, jwt: &str) -> Result<(), Error> {
        Supplier::set_wb_jwt(&self.client, api_key, jwt).await
    }

    pub async fn set_wb_id(&self, api_key: &Uuid, wb_id: i32) -> Result<(), Error> {
        Supplier::set_wb_id(&self.client, api_key, wb_id).await
    }

    pub async fn add_goods(&self, api_key: &Uuid, products: &Vec<Product>) -> Result<(), Error> {
        Product::create_many(&self.client, api_key, products).await
    }

    pub async fn get_suppliers(&self, limit: usize, page: usize) -> Result<Vec<Supplier>, Error> {
        let offset = (page - 1) * limit;

        Supplier::list(&self.client, limit as i64, offset as i64).await
    }

    pub async fn get_goods(&self, api_key: &Uuid) -> Result<Vec<Product>, Error> {
        Product::get_by_apikey(&self.client, api_key).await
    }

    pub async fn count_by_apikey(&self, api_key: &Uuid) -> Result<i64, Error> {
        Product::count_by_apikey(&self.client, api_key).await
    }
}
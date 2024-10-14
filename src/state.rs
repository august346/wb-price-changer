use crate::db::DB;
use crate::db::product::Product;
use crate::db::supplier::Supplier;
use uuid::Uuid;
use crate::utils;

pub struct AppState {
    db: DB,
}

impl AppState {
    pub async fn setup_app_state(db_url: &str) -> Result<AppState, String> {
        Ok(AppState {
            db: DB::new(db_url).await?
        })
    }

    pub async fn run_migrations(&self) -> Result<(), String> {
        self.db.run_migrations()
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create supplier"))
    }

    pub async fn get_supplier(&self, api_key: &Uuid) -> Result<Supplier, String> {
        match self.db.get_supplier(api_key).await {
            Ok(Some(supplier)) => Ok(supplier),
            Ok(None) => Err("Invalid api key".to_string()),
            Err(err) => Err(utils::make_err(Box::new(err), "get supplier"))
        }
    }

    pub async fn create_supplier(&self) -> Result<Supplier, String> {
        self.db.create_supplier()
            .await
            .map_err(|err| utils::make_err(Box::new(err), "create supplier"))
    }

    pub async fn set_wb_jwt(&self, api_key: &Uuid, jwt: &str) -> Result<(), String> {
        self.db.set_wb_jwt(api_key, jwt)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "set wb_jwt"))
    }

    pub async fn get_suppliers(&self, limit: usize, page: usize) -> Result<Vec<Supplier>, String> {
        self.db.get_suppliers(limit, page)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get suppliers"))
    }

    pub async fn set_wb_id(&self, api_key: &Uuid, wb_id: i32) -> Result<(), String> {
        self.db.set_wb_id(api_key, wb_id)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "set wb id"))
    }

    pub async fn add_goods(&self, api_key: &Uuid, products: &Vec<Product>) -> Result<(), String> {
        self.db.add_goods(api_key, products)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "add goods"))
    }

    pub async fn get_goods(&self, api_key: &Uuid) -> Result<Vec<Product>, String> {
        self.db.get_goods(api_key)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "get goods"))
    }

    pub async fn count_by_apikey(&self, api_key: &Uuid) -> Result<i64, String> {
        self.db.count_by_apikey(api_key)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "count by apikey"))
    }

    pub async fn delete_by_id_and_api_key(&self, id: i32, api_key: &Uuid) -> Result<(), String> {
        self.db.delete_by_id_and_api_key(id, api_key)
            .await
            .map_err(|err| utils::make_err(Box::new(err), "delete by id and apikey"))
    }
}

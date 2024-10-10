use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::debug;
use crate::db::DB;
use crate::db::product::Product;
use crate::db::supplier::Supplier;
use sqlx::types::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub task_manager: Arc<TaskManager>,
    db: Arc<DB>,
}

pub struct TaskManager {
    tasks: Mutex<HashMap<i32, JoinHandle<()>>>,
}

impl TaskManager {
    fn new() -> Self {
        TaskManager {
            tasks: Mutex::new(HashMap::new()),
        }
    }

    pub async fn remove_task(&self, id: i32) {
        let mut tasks = self.tasks.lock().await;
        if let Some(existing_task) = tasks.remove(&id) {
            if !existing_task.is_finished() {
                existing_task.abort();
                debug!("task_id={id} aborted");
            }
        }
    }

    pub async fn add_task(&self, id: i32, handle: JoinHandle<()>) {
        let mut tasks = self.tasks.lock().await;
        tasks.insert(id, handle);
        debug!("task_id={id} inserted");
    }
}

impl AppState {
    pub async fn setup_app_state(client: PgPool) -> Result<AppState, String> {
        Ok(AppState {
            task_manager: Arc::new(TaskManager::new()),
            db: Arc::new(DB::new(client))
        })
    }

    pub async fn run_migrations(&self) -> Result<(), String> {
        Ok(())
    }

    pub async fn get_supplier(&self, api_key: &Uuid) -> Result<Supplier, String> {
        match self.db.get_supplier(api_key).await {
            Ok(Some(supplier)) => Ok(supplier),
            Ok(None) => Err("Invalid api key".to_string()),
            Err(err) => Err(err)
        }
    }

    pub async fn create_supplier(&self) -> Result<Supplier, String> {
        self.db.create_supplier().await
    }

    pub async fn set_wb_jwt(&self, api_key: &Uuid, jwt: &str) -> Result<(), String> {
        self.db.set_wb_jwt(api_key, jwt).await
    }

    pub async fn get_suppliers(&self, limit: usize, page: usize) -> Result<Vec<Supplier>, String> {
        self.db.get_suppliers(limit, page).await
    }

    pub async fn set_wb_id(&self, api_key: &Uuid, wb_id: i32) -> Result<(), String> {
        self.db.set_wb_id(api_key, wb_id).await
    }

    pub async fn add_goods(&self, api_key: &Uuid, products: &Vec<Product>) -> Result<(), String> {
        self.db.add_goods(api_key, products).await
    }

    pub async fn get_goods(&self, api_key: &Uuid) -> Result<Vec<Product>, String> {
        self.db.get_goods(api_key).await
    }

    pub async fn count_by_apikey(&self, api_key: &Uuid) -> Result<i64, String> {
        self.db.count_by_apikey(api_key).await
    }
}

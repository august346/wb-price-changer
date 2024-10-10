use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct Supplier {
    pub api_key: String,
    pub wb_id: Option<i32>,
    pub wb_jwt: Option<String>,
    pub goods: HashMap<i32, Product>
}

impl Display for Supplier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Supplier(api_key={}, wb_id={}, len(goods)={} ",
            self.api_key,
            self.wb_id.unwrap_or_default(),
            self.goods.len()
        )
    }
}

impl Supplier {
    pub async fn list(pool: &PgPool, limit: usize, offset: usize) -> Result<Vec<Supplier>, String> {    
        let result = sqlx::query_as!(
            Supplier,
            r#"
            SELECT api_key, wb_id, wb_jwt FROM suppliers
            ORDER BY api_key
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await;
    
        match result {
            Ok(suppliers) => Ok(suppliers),
            Err(e) => Err(format!("Error fetching suppliers: {:?}", e)),
        }
    }

    pub async fn get(client: &PgPool, api_key: &str) -> Result<Option<Supplier>, String> {
        let result = sqlx::query_as!(
            Supplier,
            r#"
            SELECT api_key, wb_id, wb_jwt FROM suppliers WHERE api_key = $1
            "#,
            api_key
        )
        .fetch_optional(client)
        .await;
    
        match result {
            Ok(Some(supplier)) => Ok(Some(supplier)),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Error fetching supplier: {:?}", e)),
        }
    }
    
    pub async fn create(client: &PgPool, api_key: String) -> Result<Supplier, String> {        
        let result = sqlx::query_as!(
            Supplier,
            r#"
            INSERT INTO suppliers (api_key) VALUES ($1)
            "#,
            api_key
        )
        .fetch(client)
        .await;
    
        match result {
            Ok(_) => Ok(supplier),
            Err(e) => Err(format!("Error creating supplier: {:?}", e)),
        }
    }   

    pub async fn set_wb_jwt(client: &PgPool, api_key: &str, jwt: &str) -> Result<(), String> {
        let result = sqlx::query!(
            r#"
            UPDATE suppliers SET wb_jwt = $1 WHERE api_key = $2
            "#,
            jwt,
            api_key
        )
        .execute(client)
        .await;
    
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error updating wb_jwt: {:?}", e)),
        }
    }

    pub async fn set_wb_id(client: &PgPool, api_key: &str, wb_id: i32) -> Result<(), String> {
        let result = sqlx::query!(
            r#"
            UPDATE suppliers SET wb_id = $1 WHERE api_key = $2
            "#,
            wb_id,
            api_key
        )
        .execute(client)
        .await;
    
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Error updating wb_id: {:?}", e)),
        }
    }
}

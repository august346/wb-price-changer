use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use sqlx::PgPool;
use sqlx::types::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)] 
pub struct Supplier {
    pub api_key: Uuid, 
    pub wb_id: Option<i32>,
    pub wb_jwt: Option<String>,
}

impl Display for Supplier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Supplier(api_key={}, wb_id={})",
            self.api_key,
            self.wb_id.unwrap_or_default(),
        )
    }
}

impl Supplier {
    pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Supplier>, String> {
        sqlx::query_as!(
            Supplier,
            r#"
            SELECT api_key, wb_id, wb_jwt FROM suppliers
            ORDER BY api_key
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Error fetching suppliers: {:?}", e))
    }

    pub async fn get(client: &PgPool, api_key: &Uuid) -> Result<Option<Supplier>, String> {
        sqlx::query_as!(
            Supplier,
            r#"
            SELECT api_key, wb_id, wb_jwt FROM suppliers WHERE api_key = $1
            "#,
            api_key
        )
        .fetch_optional(client)
        .await
        .map_err(|e| format!("Error fetching supplier: {:?}", e))
    }
    
    pub async fn create(client: &PgPool) -> Result<Supplier, String> {
        sqlx::query_as!(
            Supplier,
            r#"
            INSERT INTO suppliers DEFAULT VALUES 
            RETURNING *
            "#, 
        )
        .fetch_one(client)
        .await
        .map_err(|e| format!("Error creating supplier: {:?}", e))
    }

    pub async fn set_wb_jwt(client: &PgPool, api_key: &Uuid, jwt: &str) -> Result<(), String> {
        sqlx::query!(
            r#"
            UPDATE suppliers SET wb_jwt = $1 WHERE api_key = $2
            "#,
            jwt,
            api_key
        )
        .execute(client)
        .await
        .map_err(|e| format!("Error updating wb_jwt: {:?}", e))?;

        Ok(())
    }

    pub async fn set_wb_id(client: &PgPool, api_key: &Uuid, wb_id: i32) -> Result<(), String> {
        sqlx::query!(
            r#"
            UPDATE suppliers SET wb_id = $1 WHERE api_key = $2
            "#,
            wb_id,
            api_key
        )
        .execute(client)
        .await
        .map_err(|e| format!("Error updating wb_id: {:?}", e))?;

        Ok(())
    }
}

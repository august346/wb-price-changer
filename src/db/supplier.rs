use std::fmt::{Display, Formatter};
use sqlx::{Error, PgPool, types::Uuid};

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
    pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Supplier>, Error> {
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
    }

    pub async fn get(client: &PgPool, api_key: &Uuid) -> Result<Option<Supplier>, Error> {
        sqlx::query_as!(
            Supplier,
            r#"
            SELECT api_key, wb_id, wb_jwt FROM suppliers WHERE api_key = $1
            "#,
            api_key
        )
            .fetch_optional(client)
            .await
    }

    pub async fn create(client: &PgPool) -> Result<Supplier, Error> {
        sqlx::query_as!(
            Supplier,
            r#"
            INSERT INTO suppliers DEFAULT VALUES 
            RETURNING *
            "#, 
        )
            .fetch_one(client)
            .await
    }

    pub async fn set_wb_jwt(client: &PgPool, api_key: &Uuid, jwt: &str) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE suppliers SET wb_jwt = $1 WHERE api_key = $2
            "#,
            jwt,
            api_key
        )
            .execute(client)
            .await?;

        Ok(())
    }

    pub async fn set_wb_id(client: &PgPool, api_key: &Uuid, wb_id: i32) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE suppliers SET wb_id = $1 WHERE api_key = $2
            "#,
            wb_id,
            api_key
        )
            .execute(client)
            .await?;

        Ok(())
    }
}

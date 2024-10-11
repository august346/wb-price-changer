use serde::{Deserialize, Serialize};
use sqlx::{Error, PgPool, types::Uuid};

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct Product {
    pub id: i32,
    pub price: i32,
}

impl Product {
    pub fn new(id: i32, price: i32) -> Self {
        Self { id, price }
    }

    pub async fn create_many(client: &PgPool, api_key: &Uuid, products: &[Product]) -> Result<(), Error> {
        let mut transaction = client.begin().await?;

        for product in products {
            sqlx::query!(
            r#"
            INSERT INTO products (id, price, supplier_api_key)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET price = $2
            "#,
            product.id,
            product.price,
            api_key
        )
                .execute(&mut *transaction)
                .await?;
        }

        transaction.commit().await?;
        Ok(())
    }


    pub async fn get_by_apikey(client: &PgPool, api_key: &Uuid) -> Result<Vec<Product>, Error> {
        sqlx::query_as!(
            Product,
            r#"
            SELECT id, price FROM products
            WHERE supplier_api_key = $1
            "#,
            api_key
        )
            .fetch_all(client)
            .await
    }

    pub async fn count_by_apikey(client: &PgPool, api_key: &Uuid) -> Result<i64, Error> {
        match sqlx::query!(
            r#"
            SELECT COUNT(*) FROM products
            WHERE supplier_api_key = $1
            "#,
            api_key
        )
            .fetch_one(client)
            .await
            .map(|record| record.count) {
            Ok(Some(count)) => Ok(count),
            Ok(None) => Ok(0),
            Err(err) => Err(err)
        }
    }
}

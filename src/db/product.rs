use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Product {
    pub id: i32,
    pub price: i32
}

impl Product {
    pub fn new(id: i32, price: i32) -> Self {
        Self { id, price }
    }

    pub async fn create_many(client: &PgPool, api_key: &str, products: &Vec<Product>) -> Result<(), String> {
        let mut transaction = client.begin().await.map_err(|e| e.to_string())?;
    
        for product in products {
            let result = sqlx::query!(
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
            .execute(&mut transaction)
            .await;
    
            if let Err(e) = result {
                transaction.rollback().await.map_err(|e| e.to_string())?;
                return Err(format!("Error adding goods: {:?}", e));
            }
        }
    
        transaction.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }    
}
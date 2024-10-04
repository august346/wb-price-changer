use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Product {
    pub id: i32,
    pub price: i32
}
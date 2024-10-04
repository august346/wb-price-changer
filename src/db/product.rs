use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Product {
    pub id: i32,
    pub price: i32
}

impl Product {
    pub fn new(id: i32, price: i32) -> Self {
        Self { id, price }
    }
}
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Product {
    pub id: i32,
    pub price: i32
}

impl Product {
    pub fn new(id: i32, price: i32) -> Self {
        Self { id, price }
    }
}
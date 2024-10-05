use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use crate::db::product::Product;

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
    pub async fn new(api_key: &str) -> Result<Self, String> {
        let goods = HashMap::new();
        Ok(Self { api_key: api_key.to_string(), wb_id: None, wb_jwt: None, goods })
    }

    pub async fn add_goods(&mut self, products: &Vec<Product>) -> Result<(), String> {
        for p in products {
            self.goods.insert(p.id, p.clone());
        }

        Ok(())
    }
}

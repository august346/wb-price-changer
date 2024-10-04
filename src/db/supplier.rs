use crate::db::product::Product;

#[derive(Clone)]
pub struct Supplier {
    pub api_key: String,
    wb_id: Option<i64>,
    pub wb_jwt: Option<String>,
    good: Vec<Product>
}

impl Supplier {
    pub async fn new(api_key: &str) -> Result<Self, String> {
        Ok(Self { api_key: api_key.to_string(), wb_id: None, wb_jwt: None, good: vec![] })
    }
}

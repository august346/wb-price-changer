use std::time::Duration;
use reqwest::Client;
use serde::Deserialize;
use std::io::Write;
use serde_json::Value;
use tempfile::NamedTempFile;
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use crate::calc::count_new_basic;
use crate::db::product::Product;
use crate::utils;

const JQ_QUERY: &str = r#"{
    supplier_id: (.data.products[0].supplierId // null),
    total: (.data.total // null),
    prices: [.data.products[] | {id: .id, basic: .sizes[0].price.basic, total: .sizes[0].price.total}]
}"#;

pub async fn calculate_and_set_price(
    supplier_id: Option<i32>,
    token: &str,
    products: Vec<Product>,
) -> Result<(Option<i32>, Vec<Product>, JoinHandle<()>), String> {
    let prices_page = get_prices(supplier_id, products.iter().map(|p| p.id).collect::<Vec<i32>>())
        .await
        .map_err(|err| utils::make_err(err, "get prices"))?;

    let updated_products: Vec<(i32, Product)> = prices_page
        .prices
        .iter()
        .zip(products.iter())
        .map(|(product_price, product)| {
            let target_price = product.price;
            let (discounted, new_price) = count_new_basic(target_price, product_price.total, product_price.basic);
            (discounted, Product::new(product_price.id, new_price))
        })
        .collect();

    let to_update: Vec<Product> = updated_products.iter().map(|(_, p)| p.clone()).collect();
    set_price(token, to_update.clone())
        .await
        .map_err(|_| "Error setting price.".to_string())?;

    let token_clone = token.to_string();
    let handle = tokio::spawn(async move {
        sleep(Duration::from_secs(10)).await;

        let _ = one_more_try(prices_page.supplier_id, &token_clone, updated_products).await;
    });

    Ok((prices_page.supplier_id, to_update, handle))
}

pub async fn get_prices(supplier_id: Option<i32>, id_list: Vec<i32>) -> Result<ProductPricesPage, Box<dyn std::error::Error>> {
    match id_list.len() {
        0 => Ok(ProductPricesPage::default()),
        1 => get_one_price(id_list[0]).await,
        _ => Ok(
            get_supplier_catalog(
                supplier_id
                    .ok_or_else(
                        || "Not available to get many prices without supplier_id".to_string()
                    )?, None, None)
                .await?
                .with_goods(id_list)
        )
    }
}

async fn get_one_price(id: i32) -> Result<ProductPricesPage, Box<dyn std::error::Error>> {
    let url = format!("https://card.wb.ru/cards/v2/detail?curr=rub&dest=-1257786&nm={}", id);
    let data: Value = Client::new()
        .get(&url)
        .timeout(Duration::from_secs(60))
        .send()
        .await?
        .json()
        .await?;

    Ok(parse_json(data).await?)
}

#[derive(Deserialize, Debug, Default)]
pub struct ProductPricesPage {
    supplier_id: Option<i32>,
    total: Option<i32>,
    prices: Vec<ProductPrice>,
}

impl ProductPricesPage {
    fn with_goods(&self, id_list: Vec<i32>) -> Self {
        Self {
            supplier_id: self.supplier_id,
            total: self.total,
            prices: self
                .prices
                .clone()
                .into_iter()
                .filter(|product_price| id_list.contains(&product_price.id))
                .collect(),
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ProductPrice {
    id: i32,
    basic: i32,
    total: i32,
}

pub async fn get_supplier_catalog(supplier: i32, limit: Option<i32>, page: Option<i32>) -> Result<ProductPricesPage, String> {
    let catalog_url = "https://catalog.wb.ru/sellers/v2/catalog";
    let url = format!(
        "{catalog_url}?curr=rub&dest=-1257786&sort=newly&supplier={supplier}&limit={}&page={}",
        limit.unwrap_or(300),
        page.unwrap_or(1)
    );
    let data = Client::new()
        .get(&url)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|err| utils::make_err(Box::new(err), "get catalog"))?
        .json()
        .await
        .map_err(|err| utils::make_err(Box::new(err), "parse get catalog response"))?;

    parse_json(data).await
}

async fn parse_json(data: Value) -> Result<ProductPricesPage, String> {
    let mut temp_file = NamedTempFile::new()
        .map_err(|err| utils::make_err(Box::new(err), "create temporary file"))?;
    writeln!(temp_file, "{}", data)
        .map_err(|err| utils::make_err(Box::new(err), "write to temporary file"))?;

    let output = Command::new("jq")
        .arg(JQ_QUERY)
        .arg(temp_file.path())
        .output()
        .await
        .map_err(|err| utils::make_err(Box::new(err), "execute jq"))?;

    let filtered_json = String::from_utf8_lossy(&output.stdout);
    Ok(serde_json::from_str(&filtered_json)
        .map_err(|err| utils::make_err(Box::new(err), "parse filtered JSON"))?)
}

pub async fn set_price(token: &str, products: Vec<Product>) -> Result<(), reqwest::Error> {
    let data = products.iter()
        .map(|product| serde_json::json!(
            { "nmID": product.id, "price": product.price }
        ))
        .collect::<Vec<_>>();

    Client::new()
        .post("https://discounts-prices-api.wildberries.ru/api/v2/upload/task")
        .timeout(Duration::from_secs(60))
        .header("Authorization", token)
        .json(&serde_json::json!({ "data": data }))
        .send()
        .await?;

    Ok(())
}

async fn one_more_try(supplier_id: Option<i32>, token: &str, updated_products: Vec<(i32, Product)>) -> Result<(), String> {
    let prices_pate = get_prices(
        supplier_id,
        updated_products
            .iter()
            .map(|(_, p)| p.id)
            .collect::<Vec<i32>>())
        .await
        .map_err(|_| "Error retrieving prices.".to_string())?;

    let products: Vec<Product> = prices_pate
        .prices
        .iter()
        .zip(updated_products.iter())
        .filter(
            |(product_price, (discounted, _))| product_price.total / 100 != *discounted)
        .map(|(product_price, (_, product))| {
            let (_, new_price) = count_new_basic(product.price, product_price.total, product_price.basic);
            Product::new(product_price.id, new_price)
        })
        .collect();

    if !products.is_empty() {
        set_price(token, products)
            .await
            .map_err(|_| "Error setting price.".to_string())?;
    }

    Ok(())
}
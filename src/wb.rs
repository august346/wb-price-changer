use std::time::Duration;
use reqwest::Client;
use serde::Deserialize;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio::process::Command;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tracing::info;
use crate::calc::count_new_basic;
use crate::db::product::Product;
use crate::utils;

pub async fn calculate_and_set_price(id: i32, target_price: i32, jwt: &str) -> Result<(i32, JoinHandle<()>), String> {
    let Price { basic, total } = get_prices(id).await.map_err(|_| "Error retrieving prices.".to_string())?;
    let (discounted, new_price) = count_new_basic(target_price, total, basic);

    set_price(jwt, vec![Product::new(id, new_price)]).await.map_err(|_| "Error setting price.".to_string())?;

    let id_clone = id;
    let jwt_clone = jwt.to_string();
    let handle = tokio::spawn(async move {
        sleep(Duration::from_secs(10)).await;

        let _ = one_more_try(id_clone, &jwt_clone, target_price, discounted).await;
    });

    Ok((new_price, handle))
}

#[derive(Deserialize)]
pub struct Price {
    pub basic: i32,
    pub total: i32,
}

pub async fn get_prices(id: i32) -> Result<Price, reqwest::Error> {
    let url = format!("https://card.wb.ru/cards/v2/detail?curr=rub&dest=-1257786&nm={}", id);
    let res: serde_json::Value = Client::new()
        .get(&url)
        .timeout(Duration::from_secs(60))
        .send()
        .await?
        .json()
        .await?;
    let price = res["data"]["products"][0]["sizes"][0]["price"].clone();
    Ok(serde_json::from_value(price).unwrap())
}

#[derive(Deserialize, Debug)]
struct FilteredResponse {
    total: u32,
    products: Vec<FilteredProduct>,
}

#[derive(Deserialize, Debug)]
struct FilteredProduct {
    id: u64,
    basic: u64,
    total: u64,
}

pub async fn get_supplier_catalog(supplier: i32, limit: Option<i32>, page: Option<i32>) -> Result<(), String> {
    let catalog_url = "https://catalog.wb.ru/sellers/v2/catalog";
    let url = format!(
        "{catalog_url}?curr=rub&dest=-1257786&sort=newly&supplier={supplier}&limit={}&page={}",
        limit.unwrap_or(300),
        page.unwrap_or(1)
    );
    let data: serde_json::Value = Client::new()
        .get(&url)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|err| utils::make_err(Box::new(err), "get catalog"))?
        .json()
        .await
        .map_err(|err| utils::make_err(Box::new(err), "parse get catalog response"))?;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    writeln!(temp_file, "{}", data).expect("Failed to write to temporary file");

    let output = Command::new("jq")
        .arg("{ total: .data.total, products: [.data.products[] | {id: .id, basic: .sizes[0].price.basic, total: .sizes[0].price.total}]}")
        .arg(temp_file.path())
        .output()
        .await
        .expect("Failed to execute jq");

    let filtered_json = String::from_utf8_lossy(&output.stdout);
    let filtered_response: FilteredResponse =
        serde_json::from_str(&filtered_json).expect("Failed to parse filtered JSON");

    info!("{:?}", filtered_response);

    Ok(())
}

pub async fn set_price(token: &str, products: Vec<Product>) -> Result<(), reqwest::Error> {
    let data = products.iter()
        .map(|product| serde_json::json!({ "nmID": product.id, "price": product.price }))
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

async fn one_more_try(id: i32, jwt: &str, target: i32, discounted: i32) -> Result<(), String> {
    let prices = get_prices(id)
        .await
        .map_err(|err| utils::make_err(Box::new(err), "1mt get price"))?;
    if prices.total / 100 != discounted {
        let (_, updated_price) = count_new_basic(target, prices.total, prices.basic);
        set_price(jwt, vec![Product::new(id, updated_price)])
            .await
            .map_err(|err| utils::make_err(Box::new(err), "1mt set price"))?;
    }

    Ok(())
}
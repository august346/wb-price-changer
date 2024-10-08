use std::env;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

pub fn make_err(err: Box<dyn std::error::Error>, process: &str) -> String {
    format!("Failed {}: {:?}", process, err)
}

pub fn get_env_var(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("{} must be set", key))
}

pub fn get_env_or(key: &str, default: String) -> Result<String, String> {
    get_env_var(key).or(Ok(default))
}

pub fn generate_uuid_str() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Debug, Deserialize)]
struct Claims {
    exp: usize,
}

pub fn get_jwt_expire(jwt: &str) -> Result<usize, String> {
    let mut val = Validation::default();
    val.insecure_disable_signature_validation();
    val.validate_exp = false;

    let token_data = decode::<Claims>(
        jwt,
        &DecodingKey::from_secret("".as_ref()),
        &val,
    ).map_err(|err| make_err(Box::new(err), "decode jwt"))?;

    Ok(token_data.claims.exp)
}

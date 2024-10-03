use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn create_jwt(user_id: &str) -> Result<String, String> {
    let expiration_time = (Utc::now() + Duration::hours(1)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration_time,
    };

    let secret = env::var("JWT_SECRET").expect("Jwt secret not found");
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &encoding_key).map_err(|err| err.to_string())
}

pub fn verify_jwt(token: &str) -> Result<Claims, String> {
    let secret = env::var("JWT_SECRET").expect("Jwt secret not found");
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();
    let token_data =
        decode::<Claims>(token, &decoding_key, &validation).map_err(|err| err.to_string())?;

    let exp = token_data.claims.exp;
    if Utc::now().timestamp() as usize > exp {
        return Err("Token expired".to_string());
    }
    Ok(token_data.claims)
}

use crate::config::configuration;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iss: String,
    pub iat: usize,
    pub nfb: usize,
}

/******************************************/
// Creating JWT token
/******************************************/
pub fn create_jwt(user_id: &str) -> Result<String, String> {
    let config = configuration::Settings::new().expect("Failed to load configurations");
    let expiration_time = (Utc::now() + Duration::hours(1)).timestamp() as usize;
    let issued_at = Utc::now().timestamp() as usize;
    let not_before = issued_at + 10;
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration_time,
        iss: "ecommerce".to_string(),
        iat: issued_at,
        nfb: not_before,
    };

    // let secret = env::var("JWT_SECRET").expect("Jwt secret not found");
    let encoding_key = EncodingKey::from_secret(config.jwt.secret.as_ref());
    encode(&Header::default(), &claims, &encoding_key).map_err(|err| err.to_string())
}

/******************************************/
// Verifying JWT token
/******************************************/

pub fn verify_jwt(token: &str) -> Result<Claims, String> {
    let config = configuration::Settings::new().expect("Failed to load configurations");
    let decoding_key = DecodingKey::from_secret(config.jwt.secret.as_ref());
    let mut validation = Validation::default();
    let token_data =
        decode::<Claims>(token, &decoding_key, &validation).map_err(|err| err.to_string())?;

    let exp = token_data.claims.exp;
    if Utc::now().timestamp() as usize > exp {
        return Err("Token expired".to_string());
    }
    if token_data.claims.iss != "ecommerce" {
        return Err("Invalid issuer".to_string());
    }
    let iat = token_data.claims.iat;
    if iat > Utc::now().timestamp() as usize {
        return Err("Token issued in the future".to_string());
    }
    let nfb = token_data.claims.nfb;
    if nfb > Utc::now().timestamp() as usize {
        return Err("Token not valid yet".to_string());
    }

    Ok(token_data.claims)
}

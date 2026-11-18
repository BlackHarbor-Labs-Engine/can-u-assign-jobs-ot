use bcrypt::hash;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Algorithm, TokenData};
use chrono::{Utc, Duration};
use crate::database; use sqlx::SqlitePool;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token error: {0}")]
    TokenError(String),
    #[error("Database error: {0}")]
    DbError(#[from] crate::database::DatabaseError),
}

// ClaimsData must be public and outside the impl block to be accessible by both functions
#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimsData {
    pub sub: String, // Subject (username)
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at time
}

pub struct AuthService;

impl AuthService {
    fn get_secret_key() -> String {
        env::var("JWT_SECRET_KEY").unwrap_or_else(|_| "your-secret-key".to_string())
    }

    pub fn hash_password(password: &str) -> String {
        let hash_bytes = hash(password.as_bytes(), 12).unwrap();
        hex::encode(hash_bytes)
    }

    pub fn verify_password(plain_password: &str, hashed_password: &str) -> bool {
        bcrypt::verify(plain_password, hashed_password).unwrap_or(false)
    }

    pub async fn register_user(pool: &SqlitePool, username: &str, email: &str, password: &str) -> Result<i64, AuthError> { 
        let hashed_password = AuthService::hash_password(password);
        database::create_user(pool, username, email, &hashed_password)
            .await
            .map_err(AuthError::DbError)
    }

    pub fn create_token(username: &str) -> Result<String, AuthError> {
        let secret_key = Self::get_secret_key();
        
        let expiration = Utc::now() + Duration::minutes(30);
        let claims = ClaimsData {
            sub: username.to_string(),
            exp: expiration.timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
        };
        
        let token = encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret_key.as_bytes()))
            .map_err(|e| AuthError::TokenError(e.to_string()))?;
        
        Ok(token)
    }

    pub fn verify_token(token: &str) -> Result<String, AuthError> {
        let secret_key = Self::get_secret_key();
        let decoding_key = DecodingKey::from_secret(secret_key.as_bytes());
        let validation = jsonwebtoken::Validation::new(Algorithm::HS256);
        
        let token_data: TokenData<ClaimsData> = decode::<ClaimsData>(token, &decoding_key, &validation)
            .map_err(|e| AuthError::TokenError(e.to_string()))?;
        
        // Check expiration
        let current_time = Utc::now().timestamp();
        if token_data.claims.exp < current_time.try_into().unwrap() {
            return Err(AuthError::TokenError("Token expired".to_string()));
        }
        
        Ok(token_data.claims.sub)
    }
}

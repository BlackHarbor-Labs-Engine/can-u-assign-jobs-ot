use bcrypt::{hash, verify};
use serde::{Deserialize, Serialize};
use std::env; // Keep this for secret key
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token error: {0}")]
    TokenError(String),
}

pub struct AuthService;

impl AuthService {
    fn get_secret_key() -> String {
        env::var("JWT_SECRET_KEY").unwrap_or_else(|_| "your-secret-key".to_string())
    }

    pub fn hash_password(password: &str) -> String {
        let salt_bytes = [0u8; 16];
        hash(password.as_bytes(), 12)
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }

    pub fn verify_password(plain_password: &str, hashed_password: &str) -> bool {
        bcrypt::verify(plain_password, hashed_password).unwrap_or(false)
    }

    pub fn create_token(username: &str) -> Result<String, AuthError> {
        let secret_key = Self::get_secret_key();
        
        use jsonwebtoken::{encode, Header, EncodingKey, Claims}; 
        
        #[derive(Debug, Serialize, Deserialize)]
        struct ClaimsData {
            sub: String, // Subject (username)
            exp: usize,  // Expiration time
            iat: usize,  // Issued at time
        }

        let expiration = chrono::Utc::now() + chrono::Duration::minutes(30);
        let claims = ClaimsData {
            sub: username.to_string(),
            exp: expiration.timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };

        let token = encode(&Header::new(jsonwebtoken::Algorithm::HS256), &claims, &EncodingKey::from_secret(secret_key.as_bytes()))
            .map_err(|e| AuthError::TokenError(e.to_string()))?;

        Ok(token)
    }

    pub fn verify_token(token: &str) -> Result<String, AuthError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthError::TokenError("Invalid token format".to_string()));
        }

        // Decode claims
        let claims_str = base64_decode(parts[1])
            .map_err(|_| AuthError::TokenError("Failed to decode claims".to_string()))?;
        
        let claims: serde_json::Value = serde_json::from_str(&claims_str)
            .map_err(|e| AuthError::TokenError(format!("Invalid claims: {}", e)))?;

        // Verify expiration
        let exp = claims.get("exp")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| AuthError::TokenError("Missing or invalid exp claim".to_string()))?;

        if chrono::Utc::now().timestamp() > exp {
            return Err(AuthError::TokenError("Token expired".to_string()));
        }

        // Get subject
        claims.get("sub")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AuthError::TokenError("Missing sub claim".to_string()))
    }
}

// Simple base64 encoding/decoding using ring
// Removed custom base64_encode function
    // Use simple URL-safe base64 encoding without external crate
    // Removed custom base64 constants as we use jsonwebtoken crate
    
    let mut result = String::new();
    let bytes = data.into_bytes();
    
    let mut i = 0;
    while i < bytes.len() {
        let chunk = [bytes[i], 
            if i + 1 < bytes.len() { bytes[i + 1] } else { 0 },
            if i + 2 < bytes.len() { bytes[i + 2] } else { 0 }];
        
        result.push(BASE64_CHARS[(chunk[0] >> 2) as usize] as char);
        result.push(BASE64_CHARS[((chunk[0] & 3) << 4 | (chunk[1] >> 4)) as usize] as char);
        if i + 1 < bytes.len() {
            result.push(BASE64_CHARS[((chunk[1] & 15) << 2 | (chunk[2] >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        if i + 2 < bytes.len() {
            result.push(BASE64_CHARS[(chunk[2] & 63) as usize] as char);
        } else {
            result.push('=');
        }
        
        i += 3;
    }
    
    result
}

// Removed custom base64_decode function
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let mut result = Vec::new();
    let bytes = data.as_bytes();
    
    let mut i = 0;
    while i < bytes.len() && bytes[i] != b'=' {
        let chunk = [
            BASE64_CHARS.iter().position(|&c| c == bytes[i]).ok_or("Invalid char")? as u8,
            if i + 1 < bytes.len() { 
                BASE64_CHARS.iter().position(|&c| c == bytes[i+1]).ok_or("Invalid char")? as u8 
            } else { 0 },
            if i + 2 < bytes.len() && bytes[i+2] != b'=' {
                BASE64_CHARS.iter().position(|&c| c == bytes[i+2]).ok_or("Invalid char")? as u8
            } else { 0 },
            if i + 3 < bytes.len() && bytes[i+3] != b'=' {
                BASE64_CHARS.iter().position(|&c| c == bytes[i+3]).ok_or("Invalid char")? as u8
            } else { 0 },
        ];

        result.push((chunk[0] << 2 | (chunk[1] >> 4)) as u8);
        if i + 2 < bytes.len() && bytes[i+2] != b'=' {
            result.push(((chunk[1] & 0x0F) << 4 | (chunk[2] >> 2)) as u8);
        }
        if i + 3 < bytes.len() && bytes[i+3] != b'=' {
            result.push(((chunk[2] & 0x03) << 6 | chunk[3]) as u8);
        }
        
        i += 4;
    }
    
    Ok(String::from_utf8(result).map_err(|_| "Invalid UTF-8".to_string())?)
}

// Removed custom sign_hmac_sha256 function
    use ring::hmac::{sign, Key};
    
    let key = Key::new(ring::hmac::HMAC_SHA256, key.as_bytes());
    
    let signature = sign(&key, data.as_bytes());
    
    // Convert to base64-like string
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let bytes = signature.as_ref();
    let mut result = String::new();
    
    for &byte in bytes {
        result.push(BASE64_CHARS[(byte >> 4) as usize] as char);
        result.push(BASE64_CHARS[(byte & 0x0F) as usize] as char);
    }
    
    result
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
}

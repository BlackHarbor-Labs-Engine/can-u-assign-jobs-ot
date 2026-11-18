mod models;
mod services {
    pub mod auth;
}
mod database;

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{post, get},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use models::{UserRegister, UserLogin};
use services::auth::AuthService;

type AppState = Arc<Mutex<()>>;

#[tokio::main]
async fn main() {
    // Initialize database
    if let Err(e) = database::init_db().await {
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    // Create app state
    let state = Arc::new(Mutex::new(()));

    // Build router
    let app = Router::new()
        .route("/", get(root))
        .route("/auth/register", post(register_user))
        .route("/auth/token", post(login_for_access_token))
        .with_state(state);

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    println!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Welcome to Harbour Backend Services API"
}

async fn register_user(
    State(_state): State<Arc<Mutex<()>>>,
    Json(payload): Json<UserRegister>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = database::get_pool();
    
    // Check if user already exists
    let existing_user = sqlx::query(
        "SELECT id FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e.to_string()})))?;

    if existing_user.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({"error": "Username already registered"}),
        ));
    }

    // Hash password and create user
    let hashed_password = AuthService::hash_password(&payload.password);
    
    database::create_user(&payload.username, &payload.email, &hashed_password)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e.to_string()})))?;

    Ok(json!({"message": "User created successfully"}))
}

async fn login_for_access_token(
    State(_state): State<Arc<Mutex<()>>>,
    Json(payload): Json<UserLogin>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = database::get_pool();
    
    // Get user from database
    let user_row = sqlx::query(
        "SELECT hashed_password FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e.to_string()})))?;

    let user_row = match user_row {
        Some(row) => row,
        None => return Err((
            StatusCode::UNAUTHORIZED,
            json!({"error": "Invalid credentials"}),
        )),
    };

    // Verify password
    let hashed_password: String = user_row.get("hashed_password");
    
    if !AuthService::verify_password(&payload.password, &hashed_password) {
        return Err((
            StatusCode::UNAUTHORIZED,
            json!({"error": "Invalid credentials"}),
        ));
    }

    // Create JWT token
    let access_token = AuthService::create_token(&payload.username)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, json!({"error": e.to_string()})))?;

    Ok(json!({
        "access_token": access_token,
        "token_type": "bearer"
    }))
}

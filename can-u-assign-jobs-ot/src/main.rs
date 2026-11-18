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
    if let Err(e) = database::init_db("data.db").await {
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
    match sqlx::query(
        "SELECT id FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(_)) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Username already registered"})),
        )),
        Ok(None) => {
            // Hash password and create user
            let hashed_password = AuthService::hash_password(&payload.password);
            
            match database::create_user(&payload.username, &payload.email, &hashed_password).await {
                Ok(_) => Ok(Json(json!({"message": "User created successfully"}))),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

async fn login_for_access_token(
    State(_state): State<Arc<Mutex<()>>>,
    Json(payload): Json<UserLogin>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pool = database::get_pool();
    
    // Get user from database
    match sqlx::query(
        "SELECT hashed_password FROM users WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(pool)
    .await
    {
        Ok(None) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid credentials"})),
        )),
        Ok(Some(row)) => {
            // Import Row trait for get method
            use sqlx::Row;
            
            // Verify password
            let hashed_password: String = row.get("hashed_password");
            
            if !AuthService::verify_password(&payload.password, &hashed_password) {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid credentials"})),
                ));
            }

            // Create JWT token
            match AuthService::create_token(&payload.username) {
                Ok(access_token) => Ok(Json(json!({
                    "access_token": access_token,
                    "token_type": "bearer"
                }))),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
            }
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

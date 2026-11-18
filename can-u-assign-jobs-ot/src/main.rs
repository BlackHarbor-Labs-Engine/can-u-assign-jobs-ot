use std::sync::{Arc, Mutex};
use axum::{extract::{State, Json}, http::StatusCode, routing::post, Router};
use serde::{Deserialize, Serialize};

// Import models from the crate root
use crate::models::{UserLogin, LoginResponse};

// Define the shared application state
pub struct AppState(Arc<Mutex<()>>);

// The function now uses LoginResponse and provides a functional path
async fn login_for_access_token(
    State(_state): State<Arc<Mutex<()>>>, // State is extracted here
    payload: Json<UserLogin>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<String>)> {
    // Validation check
    if payload.username.is_empty() || payload.password.is_empty() {
        // Fix: Convert &str to String for Json payload
        return Err((StatusCode::BAD_REQUEST, Json("Username and password required.".to_string())));
    }

    // Success path: return a dummy token
    Ok(Json(LoginResponse { token: format!("jwt_for_{}", payload.username) }))
}

#[tokio::main]
async fn main() {
    // Setup shared state
    let shared_state = Arc::new(Mutex::new(()));
    
    // Setup router, passing the Arc<Mutex<()>> directly to with_state
    let app = Router::new()
        .route("/login", post(login_for_access_token))
        .with_state(shared_state);

    // Run server (minimal placeholder for actual server binding)
    println!("Server running on http://127.0.0.1:3000");
    // In a real app: axum::Server::bind(&"127.0.0.1:3000").serve(app.into_make_service()).await.unwrap();
}

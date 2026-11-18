
use axum::{extract::{State, Json}, http::StatusCode, routing::post, Router};


// Declare modules here to make them visible to the binary crate
mod models;
mod services;
mod database;
mod repositories;

// Import necessary types from modules
use crate::models::{UserLogin, LoginResponse};
use crate::services::auth::AuthService;


// Define the shared application state
#[derive(Clone)]
pub struct AppState(sqlx::SqlitePool);

// The login handler now uses the DB pool and AuthService
async fn login_for_access_token(
    State(_pool): State<sqlx::SqlitePool>,
    payload: Json<UserLogin>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<String>)> {
    // 1. Validation check
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json("Username and password required.".to_string())));
    }

    // 2. Fetch user from DB
    let user_opt = database::get_user_by_username(&payload.username).await.map_err(|e| {
        eprintln!("DB Error during login: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json("Database error.".to_string()))
    })?;

    let user = match user_opt {
        Some(u) => u,
        None => return Err((StatusCode::UNAUTHORIZED, Json("Invalid credentials.".to_string()))),
    };

    // 3. Verify password
    if !AuthService::verify_password(&payload.password, &user.hashed_password) {
        return Err((StatusCode::UNAUTHORIZED, Json("Invalid credentials.".to_string())));
    }

    // 4. Create token
    match AuthService::create_token(&user.username) {
        Ok(token) => Ok(Json(LoginResponse { token })),
        Err(e) => {
            eprintln!("Token creation error: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json("Token generation failed.".to_string())))
        }
    }
}

#[tokio::main]
async fn main() {
    // --- Initialization Phase ---
    // In a real scenario, this URL would come from env vars.
    let database_url = "sqlite:can_u_assign_jobs_ot.db";
    
    // Initialize DB Pool
    let pool = match database::init_db(database_url).await {
        Ok(_) => { println!("Database initialized successfully."); database::get_pool().clone() },
        Err(e) => { eprintln!("FATAL: Database initialization failed: {:?}", e); std::process::exit(1); }
    };

    // --- Server Setup Phase ---
    // Setup router, passing the SqlitePool directly to with_state
    let app: Router<AppState> = Router::new().with_state(pool.clone()).with_state(AppState(pool.clone()))
        .route("/login", post(login_for_access_token))
        .with_state(pool);

    // Run server (minimal placeholder for actual server binding)
    println!("Server running on http://127.0.0.1:3000");
    // In a real app: axum::Server::bind(&"127.0.0.1:3000").serve(app.into_make_service()).await.unwrap();
}

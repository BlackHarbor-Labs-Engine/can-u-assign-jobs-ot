use can_u_assign_jobs_ot::database::{init_db, create_user, get_user_by_username};
use can_u_assign_jobs_ot::services::auth::AuthService;

#[tokio::test]
async fn test_pool_creation() {
    // Use in-memory database for testing
    
    let pool = init_db(":memory:").await.expect("Failed to initialize database");
    
    // Verify pool is set (no global pool to check, just ensuring init succeeded)
    let _ = pool;
}

#[tokio::test]
async fn test_create_user() {
    // Use in-memory database for testing
    
    let pool = init_db(":memory:").await.expect("Failed to initialize database");

    let user_id = create_user(&pool, "testuser", "test@example.com", "hashed_password_123").await
    .expect("Failed to create user");

    assert!(user_id > 0, "User ID should be positive");

    // Verify user was created
    let user = get_user_by_username(&pool, "testuser")
        .await
        .expect("Failed to get user")
        .expect("User should exist");

    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
}

#[tokio::test]
async fn test_auth_flow() {
    let pool = init_db(":memory:").await.expect("Failed to initialize database");
    let username = "auth_tester";
    let password = "securepassword123";
    let email = "auth@test.com";

    // 1. Register User
    let user_id = AuthService::register_user(&pool, username, email, password).await.expect("User registration failed");

    // 2. Create Token
    let token = AuthService::create_token(username).expect("Token creation failed");

    // 3. Verify Token (Success Case)
    let verified_user = AuthService::verify_token(&token).expect("Token verification failed");
    assert_eq!(verified_user, username, "Token subject mismatch");

    // 4. Test Wrong Password (Simulated Login)
    let wrong_password_result = AuthService::verify_password("wrongpassword", &AuthService::hash_password(password));
    assert!(!wrong_password_result, "Should fail verification with wrong password");

    // 5. Test Invalid Token
    let invalid_token_result = AuthService::verify_token("not.a.real.token");
    assert!(invalid_token_result.is_err(), "Should fail verification with invalid token");
}

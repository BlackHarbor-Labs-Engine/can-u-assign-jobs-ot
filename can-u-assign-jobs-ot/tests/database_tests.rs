use std::env;
use std::path::Path;

#[tokio::test]
async fn test_pool_creation() {
    // Use a temporary database file for testing
    let temp_db = "test_temp.db";
    
    // Clean up any existing test database
    if Path::new(temp_db).exists() {
        std::fs::remove_file(temp_db).unwrap();
    }

    // Initialize the database
    can_u_assign_jobs_ot::database::init_db(&format!("sqlite:{}", temp_db))
        .await
        .expect("Failed to initialize database");

    // Verify pool is set
    let pool = can_u_assign_jobs_ot::database::get_pool();
    assert!(pool.is_connected().await, "Database pool should be connected");

    // Clean up
    std::fs::remove_file(temp_db).unwrap_or_default();
}

#[tokio::test]
async fn test_create_user() {
    let temp_db = "test_temp_users.db";
    
    if Path::new(temp_db).exists() {
        std::fs::remove_file(temp_db).unwrap();
    }

    can_u_assign_jobs_ot::database::init_db(&format!("sqlite:{}", temp_db))
        .await
        .expect("Failed to initialize database");

    let user_id = can_u_assign_jobs_ot::database::create_user(
        "testuser",
        "test@example.com",
        "hashed_password_123"
    )
    .await
    .expect("Failed to create user");

    assert!(user_id > 0, "User ID should be positive");

    // Verify user was created
    let user = can_u_assign_jobs_ot::database::get_user_by_username("testuser")
        .await
        .expect("Failed to get user")
        .expect("User should exist");

    assert_eq!(user.get::<String, _>(1), "testuser");
    assert_eq!(user.get::<String, _>(2), "test@example.com");

    std::fs::remove_file(temp_db).unwrap_or_default();
}

#[tokio::test]
async fn test_job_repository_create() {
    let temp_db = "test_temp_jobs.db";
    
    if Path::new(temp_db).exists() {
        std::fs::remove_file(temp_db).unwrap();
    }

    can_u_assign_jobs_ot::database::init_db(&format!("sqlite:{}", temp_db))
        .await
        .expect("Failed to initialize database");

    let repo = can_u_assign_jobs_ot::repositories::job_repository::JobRepositoryImpl;

    // First create a user
    let user_id = can_u_assign_jobs_ot::database::create_user(
        "jobuser",
        "job@example.com",
        "hashed_password_456"
    )
    .await
    .expect("Failed to create user");

    // Create a job
    let job_id = repo.create_job("Test Job", "This is a test job description", user_id)
        .await
        .expect("Failed to create job");

    assert!(job_id > 0, "Job ID should be positive");

    // Verify job was created
    let jobs = repo.get_jobs_by_user(user_id)
        .await
        .expect("Failed to get jobs");

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].title, "Test Job");
    assert_eq!(jobs[0].description, "This is a test job description");
    assert_eq!(jobs[0].user_id, Some(user_id));

    std::fs::remove_file(temp_db).unwrap_or_default();
}

#[tokio::test]
async fn test_job_repository_assign() {
    let temp_db = "test_temp_assign.db";
    
    if Path::new(temp_db).exists() {
        std::fs::remove_file(temp_db).unwrap();
    }

    can_u_assign_jobs_ot::database::init_db(&format!("sqlite:{}", temp_db))
        .await
        .expect("Failed to initialize database");

    let repo = can_u_assign_jobs_ot::repositories::job_repository::JobRepositoryImpl;

    // Create two users
    let user1_id = can_u_assign_jobs_ot::database::create_user(
        "assignuser1",
        "assign1@example.com",
        "hashed_password_789"
    )
    .await
    .expect("Failed to create user 1");

    let user2_id = can_u_assign_jobs_ot::database::create_user(
        "assignuser2",
        "assign2@example.com",
        "hashed_password_abc"
    )
    .await
    .expect("Failed to create user 2");

    // Create a job for user1
    let job_id = repo.create_job("Assignable Job", "This job can be assigned", user1_id)
        .await
        .expect("Failed to create job");

    // Assign the job to user2
    repo.assign_job_to_user(job_id, user2_id)
        .await
        .expect("Failed to assign job");

    // Verify assignment worked
    let jobs = repo.get_jobs_by_user(user2_id)
        .await
        .expect("Failed to get jobs for user2");

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, job_id);
    assert_eq!(jobs[0].user_id, Some(user2_id));

    // Verify original user no longer has the job
    let user1_jobs = repo.get_jobs_by_user(user1_id)
        .await
        .expect("Failed to get jobs for user1");

    assert_eq!(user1_jobs.len(), 0);

    std::fs::remove_file(temp_db).unwrap_or_default();
}

#[tokio::test]
async fn test_multiple_jobs_per_user() {
    let temp_db = "test_temp_multi.db";
    
    if Path::new(temp_db).exists() {
        std::fs::remove_file(temp_db).unwrap();
    }

    can_u_assign_jobs_ot::database::init_db(&format!("sqlite:{}", temp_db))
        .await
        .expect("Failed to initialize database");

    let repo = can_u_assign_jobs_ot::repositories::job_repository::JobRepositoryImpl;

    let user_id = can_u_assign_jobs_ot::database::create_user(
        "multiuser",
        "multi@example.com",
        "hashed_password_multi"
    )
    .await
    .expect("Failed to create user");

    // Create multiple jobs
    for i in 1..=5 {
        repo.create_job(&format!("Job {}", i), &format!("Description {}", i), user_id)
            .await
            .expect("Failed to create job");
    }

    let jobs = repo.get_jobs_by_user(user_id)
        .await
        .expect("Failed to get jobs");

    assert_eq!(jobs.len(), 5);

    std::fs::remove_file(temp_db).unwrap_or_default();
}

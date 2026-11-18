use std::path::Path;
use sqlx::Row;
use can_u_assign_jobs_ot::repositories::job_repository::JobRepository;
use can_u_assign_jobs_ot::database::{init_db, create_user, get_user_by_username};

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
async fn test_job_repository_create() {
    // Use in-memory database for testing
    
    let pool = init_db(":memory:").await.expect("Failed to initialize database");

    let repo = can_u_assign_jobs_ot::repositories::job_repository::JobRepositoryImpl;

    // First create a user
    let user_id = create_user(&pool, "jobuser", "job@example.com", "hashed_password_456").await
    .expect("Failed to create user");

    // Create a job
    let job_id = repo.create_job("Test Job", "This is a test job description", user_id)
        .await
        .expect("Failed to create job");

    assert!(job_id > 0, "Job ID should be positive");

    // Verify job was created
    let jobs = repo.get_jobs_by_user(&pool, user_id)
        .await
        .expect("Failed to get jobs");

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].title, "Test Job");
    assert_eq!(jobs[0].description, "This is a test job description");
    assert_eq!(jobs[0].user_id, Some(user_id));
}

#[tokio::test]
async fn test_job_repository_assign() {
    // Use in-memory database for testing
    
    let pool = init_db(":memory:").await.expect("Failed to initialize database");

    let repo = can_u_assign_jobs_ot::repositories::job_repository::JobRepositoryImpl;

    // Create two users
    let user1_id = create_user(&pool, "assignuser1", "assign1@example.com", "hashed_password_789").await
    .expect("Failed to create user 1");

    let user2_id = create_user(&pool, "assignuser2", "assign2@example.com", "hashed_password_abc").await
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
    let jobs = repo.get_jobs_by_user(&pool, user2_id)
        .await
        .expect("Failed to get jobs for user2");

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, job_id);
    assert_eq!(jobs[0].user_id, Some(user2_id));

    // Verify original user no longer has the job
    let user1_jobs = repo.get_jobs_by_user(&pool, user1_id)
        .await
        .expect("Failed to get jobs for user1");

    assert_eq!(user1_jobs.len(), 0);
}

#[tokio::test]
async fn test_multiple_jobs_per_user() {
    // Use in-memory database for testing
    
    let pool = init_db(":memory:").await.expect("Failed to initialize database");

    let repo = can_u_assign_jobs_ot::repositories::job_repository::JobRepositoryImpl;

    let user_id = create_user(&pool, "multiuser", "multi@example.com", "hashed_password_multi").await
    .expect("Failed to create user");

    // Create multiple jobs
    for i in 1..=5 {
        repo.create_job(&format!("Job {}", i), &format!("Description {}", i), user_id)
            .await
            .expect("Failed to create job");
    }

    let jobs = repo.get_jobs_by_user(&pool, user_id)
        .await
        .expect("Failed to get jobs");

    assert_eq!(jobs.len(), 5);
}

use serde::{Serialize, Deserialize}; // Assuming necessary traits are derived

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    // Add other fields as necessary
}

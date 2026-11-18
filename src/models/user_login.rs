use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

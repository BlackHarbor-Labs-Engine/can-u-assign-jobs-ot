use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRegister {
    pub username: String,
    pub email: String,
    pub password: String,
}

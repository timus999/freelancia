use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub email:String,
    pub password: String,
}


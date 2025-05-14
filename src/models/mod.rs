use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]

pub struct Job {
    pub id: u32,
    pub title: String,
    pub budget: u32,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct Freelancer{
    pub id:u32,
    pub name:String,
    pub skills:Vec<String>,
    pub rating: f32,
}
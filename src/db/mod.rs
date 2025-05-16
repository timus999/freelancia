use sqlx::{Pool, Sqlite, SqlitePool};
use std::env;


pub async fn init_pool() -> Result<Pool<Sqlite>, sqlx::Error>{
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    SqlitePool::connect(&database_url).await
}


use axum::{Router};
use sqlx::SqlitePool;
use freelancia_backend::routes;
use dotenvy::dotenv;
use std::env;

// mod routes;
// mod handlers;
// mod models;
#[tokio::main]

async fn main(){

    //load from .env
    dotenv().ok();

    //connect to db
    let db_url = env::var("DATABASE_URL").expect("databaseurl must be set in .env"); // or "sqlite:///full/path/to/test.db"
    let pool = match SqlitePool::connect(&db_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to connect to DB: {e}");
            return;
        }
    };
    println!("connected to database");
    //Define the route
    // let app = routes::create_routes(); 
    let app = Router::new()
                .merge(routes::create_routes())
                .merge(routes::auth_routes(pool)); 
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    //set the address

    //start the server 
    axum::serve(listener, app)
        .await
        .unwrap();
}
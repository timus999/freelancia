use axum;
use sqlx::SqlitePool;
use freelancia_backend::routes;

// mod routes;
// mod handlers;
// mod models;
#[tokio::main]

async fn main(){

    //connect to db
    let db_url = "sqlite:///home/timus/Desktop/rust_programming/blockchain/freelancia_backend/test.db"; // or "sqlite:///full/path/to/test.db"
    let pool = match SqlitePool::connect(db_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to connect to DB: {e}");
            return;
        }
    };
    println!("connected to database");
    //Define the route
    // let app = routes::create_routes(); 
    let app2 = routes::auth_routes(pool); 
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    //set the address

    //start the server 
    axum::serve(listener, app2)
        .await
        .unwrap();
}
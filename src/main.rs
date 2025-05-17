use axum::{Router};
use freelancia_backend::{db, routes};
use dotenvy::dotenv;

#[tokio::main]

async fn main(){

    //load from .env
    dotenv().ok();

    //connect to db
    let pool = db::init_pool()
        .await
        .expect("Failed to connect to database");
    println!("connected to database");
    //Define the route
    // let app = routes::create_routes(); 
    let app = Router::new()
                .nest("/api",routes::create_routes(pool.clone()))
                .nest("/auth", routes::auth_routes(pool.clone()));
                
    
    let listener= tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    //set the address

    //start the server 
    axum::serve(listener, app)
        .await
        .unwrap();
}
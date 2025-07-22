use axum::Router;
use dotenvy::dotenv;
use freelancia_backend::handlers::auth::cleanup_blacklisted_tokens;
use freelancia_backend::{db, routes};
use http::{
    header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Method,
};
use tokio::time::{interval, Duration};
use tower_http::cors::CorsLayer;

#[tokio::main]

async fn main() {
    //load from .env
    dotenv().ok();

    //connect to db
    let pool = db::init_pool()
        .await
        .expect("Failed to connect to database");
    println!("connected to database");

    // Spawn cleanup task
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(3600)); // Every hour
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_blacklisted_tokens(&pool_clone).await {
                eprintln!("Token cleanup failed: {}", e);
            }
        }
    });

    // Define CORS layer
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);
    //Define the route
    // let app = routes::create_routes();
    let app = Router::new()
        .nest("/api", routes::create_routes(pool.clone()))
        .nest("/api", routes::auth_routes(pool.clone()))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    //set the address

    //start the server
    axum::serve(listener, app).await.unwrap();
}


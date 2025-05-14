use axum;

mod routes;
mod handlers;
mod models;
#[tokio::main]

async fn main(){
    //Define the route
    let app = routes::create_routes();  
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    //set the address

    //start the server 
    axum::serve(listener, app)
        .await
        .unwrap();
}
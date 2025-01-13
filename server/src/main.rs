mod router;
mod routes {
    pub mod healthcheck;
    pub mod karaoke;
}
mod state;
mod queue;
mod ytdlp;
mod actors {
    pub mod request;
    pub mod videodl;
}

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use tower_http::cors::CorsLayer;

use crate::router::create_router_with_state;

#[tokio::main]
async fn main() {
    let cors_layer = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router_with_state().await.unwrap().layer(cors_layer);

    println!("Server started. Please listen on 127.0.0.1:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();                 
}

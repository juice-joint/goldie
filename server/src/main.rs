mod router;
mod routes {
    pub mod healthcheck;
    pub mod karaoke;
}
mod state;
mod ytdlp;
mod actors {
    pub mod video_downloader;
    pub mod song_coordinator;
}

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use tower_http::cors::{Any, CorsLayer};

use crate::router::create_router_with_state;

#[tokio::main]
async fn main() {
    let cors_layer = CorsLayer::new()
        .allow_origin(Any) // Allows all origins
        .allow_methods(Any) // Allows all HTTP methods
        .allow_headers(Any); // Allows all headers

    let app = create_router_with_state().await.unwrap().layer(cors_layer);

    println!("Server started. Please listen on 127.0.0.1:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();                 
}

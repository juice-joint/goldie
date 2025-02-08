mod router;
mod routes {
    pub mod admin;
    pub mod healthcheck;
    pub mod karaoke;
    pub mod sys;
    pub mod streaming;
}
mod state;
mod actors {
    pub mod video_downloader;
    pub mod song_coordinator;
}

mod lib {
    pub mod os;
    pub mod file_storage;
    pub mod pitch_shifter;
    pub mod yt_downloader;
}

use tower_http::cors::{Any, CorsLayer};

use crate::router::create_router_with_state;

#[tokio::main]
async fn main() {
    let cors_layer = CorsLayer::new()
        .allow_origin(Any) // Allows all origins
        .allow_methods(Any) // Allows all HTTP methods
        .allow_headers(Any); // Allows all headers

    let app = create_router_with_state().await.layer(cors_layer);

    println!("Server started. Please listen on 127.0.0.1:8000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();                 
}
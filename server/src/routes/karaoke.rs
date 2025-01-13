// use std::sync::{Arc, Mutex};

// use axum::{debug_handler, extract::{ Json, State }, http::StatusCode, response::{IntoResponse, Result, Response}};
// use serde::Deserialize;

// use crate::{actors::request::RequestActorHandle, queue::{ PlayableSong }, state::AppState, ytdlp::{Ytdlp, YtdlpError}};

use std::sync::Arc;

use axum::{
    body::Body, extract::State, http::{header::{self, ACCEPT_RANGES}, HeaderMap, StatusCode}, response::{IntoResponse, Response}, Json
};
use axum_extra::{headers, TypedHeader};
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::actors::request::RequestActorHandle;

#[derive(Deserialize)]
pub struct QueueSong {
    yt_link: String,
}

// impl IntoResponse for YtdlpError {
//     fn into_response(self) -> axum::response::Response {
//         return axum::response::Response::new("hi".into());
//     }
// }

// #[debug_handler(state = AppState)]
// pub async fn queue_song(
//     State(request_actor_handle): State<Arc<RequestActorHandle>>,
//     Json(payload): Json<QueueSong>,
// ) -> Result<impl IntoResponse, YtdlpError> {
//     println!("helo beanie 1");

//     println!("ytlkink {}", payload.yt_link);
    

//     println!("{:?}", request_actor_handle.queue_song().await);
//     // let url = String::from("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
//     // let _video_path = ytdlp
//     //     .fetcher
//     //     .download_video_from_url(url, "my-video.mp4")
//     //     .await
//     //     .map_err(|error| {
//     //         eprintln!("error downloading video: {}", error);
//     //         YtdlpError::SomethingWentWrong(error.to_string())
//     //     })?;

//     println!("helo beanie 2");

//     Ok((StatusCode::OK, [("x-foo", "bar")], "Hello, World!"))
// }

// pub async fn play_next_song(
//     State(request_actor_handle): State<Arc<RequestActorHandle>>,
//     Json(payload): Json<QueueSong>
// ) -> impl IntoResponse {


//     (StatusCode::OK, [("x-foo", "bar")], "Hello, World!")
// }

pub async fn here_video(
    State(request_actor_handle): State<Arc<RequestActorHandle>>,
    headers: HeaderMap
) -> Result<Response<Body>, StatusCode> {
    // Open the file
    println!("helo beanie");

    let wants_html = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/html"))
        .unwrap_or(false);
    // If Accept header includes text/html, send the HTML page
    if wants_html
    {
        let html = format!(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Video Player</title>
                <style>
                    body {{ 
                        background: #1a1a1a;
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        height: 100vh;
                        margin: 0;
                    }}
                    video {{
                        max-width: 90%;
                        box-shadow: 0 0 20px rgba(0,0,0,0.3);
                    }}
                </style>
            </head>
            <body>
                <video width="1280" height="720" controls autoplay>
                    <source src="?raw=true" type="video/mp4">
                    Your browser does not support the video tag.
                </source>
                </video>
            </body>
            </html>
        "#);

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .body(Body::from(html))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    }


    let file = File::open("assets/video.mp4")
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    // Create a stream from the file
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Build the response with appropriate headers
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4") // Adjust content type as needed
        .header(ACCEPT_RANGES, "bytes")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

// pub async fn play_song(State(shared_state): State<SharedState>) -> impl IntoResponse {
//     let mut state = shared_state.lock().await;

//     match state.song_coordinator.pop() {
//         Some(next_song) => {
//             state.song_coordinator.set_current(next_song);
//         },
//         None => {}
//     }

//     println!("{:?}", state.song_coordinator.);
// }

// pub async fn end_song(State(shared_state): State<SharedState>) -> impl IntoResponse {
//     let state = shared_state.lock().await;

//     let next_song = state.song_coordinator.pop();
//     state.song_coordinator.set_current(next_song);

//     println!("{:?}", state.song_queue);
// }

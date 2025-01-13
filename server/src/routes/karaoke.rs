use std::sync::{Arc, Mutex};

use axum::{debug_handler, extract::{ Json, State }, http::StatusCode, response::{IntoResponse, Result}};
use serde::Deserialize;

use crate::{actors::request::RequestActorHandle, queue::{ PlayableSong, SongCoordinator }, state::AppState, ytdlp::{Ytdlp, YtdlpError}};

#[derive(Deserialize)]
pub struct QueueSong {
    yt_link: String,
}

impl IntoResponse for YtdlpError {
    fn into_response(self) -> axum::response::Response {
        return axum::response::Response::new("hi".into());
    }
}

#[debug_handler(state = AppState)]
pub async fn queue_song(
    State(song_coordinator_state): State<Arc<Mutex<SongCoordinator>>>,
    State(ytdlp): State<Arc<Ytdlp>>,
    State(request_actor_handle): State<Arc<RequestActorHandle>>,
    Json(payload): Json<QueueSong>,
) -> Result<impl IntoResponse, YtdlpError> {
    println!("helo beanie 1");

    println!("ytlkink {}", payload.yt_link);
    

    println!("{:?}", request_actor_handle.queue_song().await);
    // let url = String::from("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    // let _video_path = ytdlp
    //     .fetcher
    //     .download_video_from_url(url, "my-video.mp4")
    //     .await
    //     .map_err(|error| {
    //         eprintln!("error downloading video: {}", error);
    //         YtdlpError::SomethingWentWrong(error.to_string())
    //     })?;

    println!("helo beanie 2");

    let mut song_coordinator = song_coordinator_state.lock().unwrap();
    let new_song = PlayableSong::new("test");
    song_coordinator.queue(new_song);

    println!("{:?}", song_coordinator.current());

    Ok((StatusCode::OK, [("x-foo", "bar")], "Hello, World!"))
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

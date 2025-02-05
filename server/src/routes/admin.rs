use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tokio::sync;

use crate::actors::song_coordinator::{KeyDownResponse, KeyUpResponse, SongActorHandle};

use super::karaoke::SseEvent;

pub async fn toggle_playback(
    State(sse_broadcaster): State<Arc<sync::broadcast::Sender<SseEvent>>>
) -> Result<impl IntoResponse, StatusCode> {
    let _ = sse_broadcaster.send(SseEvent::TogglePlayback);
    Ok(StatusCode::ACCEPTED)
}

pub async fn key_up(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> Result<impl IntoResponse, StatusCode> {
    let song_actor_response = song_actor_handle.key_up().await;
    match song_actor_response {
        KeyUpResponse::Success(current_key ) => {
            Ok((StatusCode::OK, Json(current_key)))
        }
        KeyUpResponse::Fail => {
            Err(StatusCode::NOT_MODIFIED)
        }
    } 
}

pub async fn key_down(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> Result<impl IntoResponse, StatusCode> {
    let song_actor_response = song_actor_handle.key_down().await;
    match song_actor_response {
        KeyDownResponse::Success(current_key ) => {
            Ok((StatusCode::OK, Json(current_key)))
        }
        KeyDownResponse::Fail => {
            Err(StatusCode::NOT_MODIFIED)
        }
    } 
}
use std::sync::Arc;

use axum::{debug_handler, extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use tokio::sync;
use uuid::Uuid;

use crate::{actors::song_coordinator::SongActorHandle, state::AppState};

use super::karaoke::SseEvent;

pub async fn toggle_playback(
    State(sse_broadcaster): State<Arc<sync::broadcast::Sender<SseEvent>>>,
) -> Result<impl IntoResponse, StatusCode> {
    let _ = sse_broadcaster.send(SseEvent::TogglePlayback);
    Ok(StatusCode::ACCEPTED)
}

pub async fn key_up(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> Result<impl IntoResponse, StatusCode> {
    let song_actor_response = song_actor_handle.key_up().await;
    match song_actor_response {
        Ok(current_key) => Ok((StatusCode::OK, Json(current_key))),
        Err(_) => Err(StatusCode::NOT_MODIFIED),
    }
}

pub async fn key_down(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("HUHFAW");
    let song_actor_response = song_actor_handle.key_down().await;
    match song_actor_response {
        Ok(current_key) => Ok((StatusCode::OK, Json(current_key))),
        Err(_) => Err(StatusCode::NOT_MODIFIED),
    }
}

pub async fn get_key(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
) -> Result<impl IntoResponse, StatusCode> {
    let song_actor_response = song_actor_handle.get_key().await;
    match song_actor_response {
        Ok(current_key) => { 
            println!("{:?}", current_key);
            Ok((StatusCode::OK, Json(current_key))) 
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct RepositionSongRequest {
    song_uuid: String,
    position: usize,
}

#[debug_handler(state = AppState)]
pub async fn reposition_song(
    State(song_actor_handle): State<Arc<SongActorHandle>>,
    Json(payload): Json<RepositionSongRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let song_uuid = Uuid::parse_str(&payload.song_uuid).map_err(|_| StatusCode::BAD_REQUEST)?;
    let position = payload.position;

    let song_actor_response = song_actor_handle.reposition_song(song_uuid, position).await;
    match song_actor_response {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::NOT_MODIFIED),
    }
}

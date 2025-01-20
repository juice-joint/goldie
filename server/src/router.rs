
use std::sync::{Arc, Mutex};

use axum::routing::post;
use axum::{
    routing::get,
    Router,
};
use tokio::sync;

use crate::actors::song_coordinator::SongActorHandle;
use crate::actors::video_downloader::VideoDlActorHandle;
use crate::routes::karaoke::{current_song, here_video, play_next_song, queue_song, song_list, sse};
use crate::ytdlp::YtdlpError;
use crate::{routes::healthcheck::healthcheck, state::AppState, ytdlp::Ytdlp};

pub async fn create_router_with_state() -> Result<Router, YtdlpError> {
    let ytdlp = Ytdlp::new().await?;

    let (sse_broadcaster, _) = sync::broadcast::channel(10);
    let sse_broadcaster = Arc::new(sse_broadcaster);

    let song_actor_handle = Arc::new(SongActorHandle::new(sse_broadcaster.clone()));

    let videodl_actor = Arc::new(VideoDlActorHandle::new(ytdlp.clone()));

    let app_state = AppState::new(song_actor_handle, videodl_actor, sse_broadcaster.clone());

    Ok(Router::new()
            .route("/api/healthcheck", get(healthcheck))
            .route("/queue_song", post(queue_song))
            .route("/play_next", post(play_next_song))
            .route("/song_list", get(song_list))
            .route("/current_song", get(current_song))
            .route("/assets/{video}", get(here_video))
            .route("/sse", get(sse))
            // .route("/end_song", get(end_song))
            // .route("/play_song", get(play_song))
            .with_state(app_state))
}

use std::sync::Arc;


use axum::routing::post;
use axum::{
    routing::get,
    Router,
};
use tokio::sync;

use crate::routes::admin::{get_key, remove_song, reposition_song};
use crate::{actors::song_coordinator::SongActorHandle, routes::admin::{key_down, key_up, toggle_playback}};
use crate::actors::video_downloader::VideoDlActorHandle;
use crate::lib::yt_downloader::YtDownloader;
use crate::routes::karaoke::{current_song, play_next_song, queue_song, song_list, sse};
use crate::routes::streaming::{serve_dash_file, serve_media_file};
use crate::routes::sys::server_ip;
use crate::{routes::healthcheck::healthcheck, state::AppState};

pub async fn create_router_with_state() -> Router {
    let yt_downloader = Arc::new(YtDownloader {});

    let (sse_broadcaster, _) = sync::broadcast::channel(10);
    let sse_broadcaster = Arc::new(sse_broadcaster);

    let song_actor_handle = Arc::new(SongActorHandle::new(sse_broadcaster.clone()));
    let videodl_actor = Arc::new(VideoDlActorHandle::new(String::from("./assets"), yt_downloader));

    let app_state = AppState::new(song_actor_handle, videodl_actor, sse_broadcaster.clone());

    Router::new()
            .route("/api/healthcheck", get(healthcheck))
            .route("/server_ip",get(server_ip))
            .route("/queue_song", post(queue_song))
            .route("/play_next", post(play_next_song))
            .route("/song_list", get(song_list))
            .route("/current_song", get(current_song))
            .route("/dash/{song_name}/{file}", get(serve_dash_file))
            .route("/dash/{song_name}/{stream}/{file}", get(serve_media_file))
            .route("/sse", get(sse))
            .route("/toggle_playback", post(toggle_playback))
            .route("/key_up", post(key_up))
            .route("/key_down", post(key_down))
            .route("/get_key", get(get_key))
            .route("/reposition_song", post(reposition_song))
            .route("/remove_song", post(remove_song))
            .with_state(app_state)
}
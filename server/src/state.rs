use std::sync::Arc;

use axum::extract::FromRef;
use tokio::sync;

use crate::{actors::{song_coordinator::SongActorHandle, video_downloader::VideoDlActorHandle}, routes::karaoke::SseEvent};

#[derive(Clone)]
pub struct AppState {
    pub song_actor_handle: Arc<SongActorHandle>,
    pub videodl_actor_handle: Arc<VideoDlActorHandle>,
    pub sse_broadcaster: Arc<sync::broadcast::Sender<SseEvent>>
}

impl AppState {
    pub fn new(
        song_actor_handle: Arc<SongActorHandle>,
        videodl_actor_handle: Arc<VideoDlActorHandle>,
        sse_broadcaster: Arc<sync::broadcast::Sender<SseEvent>>
    ) -> Self {
        AppState {
            song_actor_handle,
            videodl_actor_handle,
            sse_broadcaster
        }
    }
}

impl FromRef<AppState> for Arc<SongActorHandle> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.song_actor_handle.clone()
    }
}

impl FromRef<AppState> for Arc<VideoDlActorHandle> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.videodl_actor_handle.clone()
    }
}

impl FromRef<AppState> for Arc<sync::broadcast::Sender<SseEvent>> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.sse_broadcaster.clone()
    }
}


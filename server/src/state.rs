use std::sync::{Arc, Mutex};

use axum::extract::FromRef;

use crate::{actors::request::RequestActorHandle, queue::SongCoordinator, ytdlp::Ytdlp};

#[derive(Clone)]
pub struct AppState {
    pub song_coordinator: Arc<Mutex<SongCoordinator>>,
    pub ytdlp: Arc<Ytdlp>,
    pub request_actor_handle: Arc<RequestActorHandle>
}

impl AppState {
    pub fn new(
        song_coordinator: Arc<Mutex<SongCoordinator>>,
        ytdlp: Arc<Ytdlp>, 
        request_actor_handle: Arc<RequestActorHandle>
    ) -> Self {
        AppState {
            song_coordinator,
            ytdlp,
            request_actor_handle
        }
    }
}

impl FromRef<AppState> for Arc<Mutex<SongCoordinator>> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.song_coordinator.clone()
    }
}

impl FromRef<AppState> for Arc<Ytdlp> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.ytdlp.clone()
    }
}

impl FromRef<AppState> for Arc<RequestActorHandle> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.request_actor_handle.clone()
    }
}
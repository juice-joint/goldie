use std::sync::Arc;

use axum::extract::FromRef;
use tokio::sync;

use crate::{actors::request::RequestActorHandle, queue::PlayableSong};

#[derive(Clone)]
pub struct AppState {
    pub request_actor_handle: Arc<RequestActorHandle>,
    pub sse_broadcaster: Arc<sync::broadcast::Sender<PlayableSong>>
}

impl AppState {
    pub fn new(
        request_actor_handle: Arc<RequestActorHandle>,
        sse_broadcaster: Arc<sync::broadcast::Sender<PlayableSong>>
    ) -> Self {
        AppState {
            request_actor_handle,
            sse_broadcaster
        }
    }
}

impl FromRef<AppState> for Arc<RequestActorHandle> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.request_actor_handle.clone()
    }
}

impl FromRef<AppState> for Arc<sync::broadcast::Sender<PlayableSong>> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.sse_broadcaster.clone()
    }
}


use std::sync::Arc;

use axum::extract::FromRef;

use crate::actors::request::RequestActorHandle;

#[derive(Clone)]
pub struct AppState {
    pub request_actor_handle: Arc<RequestActorHandle>
}

impl AppState {
    pub fn new(
        request_actor_handle: Arc<RequestActorHandle>
    ) -> Self {
        AppState {
            request_actor_handle
        }
    }
}

impl FromRef<AppState> for Arc<RequestActorHandle> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.request_actor_handle.clone()
    }
}
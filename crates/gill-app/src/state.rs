use crate::instance::InstanceHandle;
use async_session::MemoryStore;
use axum::extract::FromRef;
use oauth2::basic::BasicClient;

#[derive(Clone)]
pub struct AppState {
    pub store: MemoryStore,
    pub oauth_client: BasicClient,
    pub instance: InstanceHandle,
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

impl FromRef<AppState> for BasicClient {
    fn from_ref(state: &AppState) -> Self {
        state.oauth_client.clone()
    }
}

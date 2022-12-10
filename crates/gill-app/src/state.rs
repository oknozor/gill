use async_session::MemoryStore;
use axum::extract::FromRef;
use oauth2::basic::BasicClient;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::Theme;
use crate::instance::InstanceHandle;

#[derive(Clone)]
pub struct AppState {
    pub store: MemoryStore,
    pub oauth_client: BasicClient,
    pub syntax_set: SyntaxSet,
    pub theme: Theme,
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

impl FromRef<AppState> for SyntaxSet {
    fn from_ref(state: &AppState) -> Self {
        state.syntax_set.clone()
    }
}

impl FromRef<AppState> for Theme {
    fn from_ref(state: &AppState) -> Self {
        state.theme.clone()
    }
}


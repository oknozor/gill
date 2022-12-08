use crate::apub::object::user::{ApubUser, Person, PersonAcceptedActivities};
use crate::error::AppError;
use crate::oauth::{oauth_client, AppState};
use crate::{api, apub, view};
use activitypub_federation::core::axum::{
    inbox::receive_activity, json::ApubJson, verify_request_payload, DigestVerified,
};
use activitypub_federation::data::Data;
use activitypub_federation::deser::context::WithContext;
use activitypub_federation::traits::ApubObject;
use activitypub_federation::{
    core::object_id::ObjectId, InstanceSettings, LocalInstance, UrlVerifier,
};
use async_session::MemoryStore;
use axum::async_trait;
use axum::body::Body;
use axum::extract::{OriginalUri, State};
use axum::headers::HeaderMap;
use axum::http::{Method, Request};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post;
use axum::{body, middleware, Extension, Json, Router};
use gill_ipc::listener::IPCListener;
use http::StatusCode;
use reqwest::Client;
use sqlx::PgPool;
use std::io;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_http::ServiceBuilderExt;
use url::Url;

use crate::syntax::{load_syntax, load_theme};

pub type InstanceHandle = Arc<Instance>;

pub struct Instance {
    pub local_instance: LocalInstance,
    db: PgPool,
}

/// Use this to store your federation blocklist, or a database connection needed to retrieve it.
#[derive(Clone)]
struct MyUrlVerifier();

#[async_trait]
impl UrlVerifier for MyUrlVerifier {
    // TODO: check against known instance
    async fn verify(&self, url: &Url) -> Result<(), &'static str> {
        if url.domain() == Some("malicious.com") {
            Err("malicious domain")
        } else {
            Ok(())
        }
    }
}

impl Instance {
    pub fn new(hostname: String, db: PgPool) -> Result<InstanceHandle, AppError> {
        let settings = InstanceSettings::builder()
            .debug(true)
            .url_verifier(Box::new(MyUrlVerifier()))
            .build()?;

        let local_instance = LocalInstance::new(hostname, Client::default().into(), settings);

        let instance = Arc::new(Instance { local_instance, db });

        Ok(instance)
    }

    pub fn local_instance(&self) -> &LocalInstance {
        &self.local_instance
    }

    pub fn database(&self) -> &PgPool {
        &self.db
    }

    pub async fn listen(instance: &InstanceHandle) -> anyhow::Result<()> {
        let hostname = instance.local_instance.hostname();
        let instance = instance.clone();
        let serve_dir =
            axum::routing::get_service(ServeDir::new("assets")).handle_error(handle_error);
        let store = MemoryStore::new();
        let oauth_client = oauth_client();
        let syntax_set = load_syntax();
        let theme = load_theme();
        let db = instance.db.clone();
        let app_state = AppState {
            store,
            oauth_client,
            syntax_set,
            theme,
        };

        let app = Router::new()
            .nest("/api/v1/", api::router())
            .nest_service("/apub", apub::router(instance))
            .nest_service("/", view::router(app_state))
            .nest_service("/assets/", serve_dir)
            .layer(TraceLayer::new_for_http())
            .layer(Extension(db))
            .into_make_service();

        // run it
        let addr = hostname
            .to_socket_addrs()?
            .next()
            .expect("Failed to lookup domain name");

        let app = axum::Server::bind(&addr).serve(app);

        let ipc = IPCListener;
        let _ = tokio::join!(app, ipc.listen());

        Ok(())
    }
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

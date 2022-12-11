use crate::error::AppError;
use crate::oauth::oauth_client;
use crate::{api, apub, view};

use activitypub_federation::{InstanceSettings, LocalInstance, UrlVerifier};
use async_session::MemoryStore;
use axum::async_trait;

use axum::response::IntoResponse;

use axum::routing::get;
use axum::{Extension, Router};
use gill_ipc::listener::IPCListener;
use http::StatusCode;
use reqwest::Client;
use sqlx::PgPool;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tower_http::services::ServeDir;

use tower_http::trace::TraceLayer;

use crate::state::AppState;
use gill_settings::SETTINGS;
use url::Url;

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
        let instance = instance.clone();
        let store = MemoryStore::new();
        let oauth_client = oauth_client();
        let syntax_set = syntect::dumps::from_binary(include_bytes!("syntax/syntax.bin"));
        let theme = syntect::dumps::from_binary(include_bytes!("syntax/theme.bin"));
        let db = instance.db.clone();
        let app_state = AppState {
            store,
            oauth_client,
            syntax_set,
            theme,
            instance: instance.clone(),
        };

        let app = Router::new()
            .nest_service(
                "/assets",
                axum::routing::get_service(ServeDir::new("assets")).handle_error(handle_error),
            )
            .route(
                "/.well-known/webfinger",
                get(crate::webfinger::webfinger).with_state(app_state.clone()),
            )
            .nest("/api/v1/", api::router())
            .nest_service("/apub", apub::router(instance))
            .nest_service("/", view::router(app_state.clone()))
            .layer(TraceLayer::new_for_http())
            .layer(Extension(db))
            .into_make_service();

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), SETTINGS.port);
        let app = axum::Server::bind(&addr).serve(app);

        let ipc = IPCListener;
        let _ = tokio::join!(app, ipc.listen());

        Ok(())
    }
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

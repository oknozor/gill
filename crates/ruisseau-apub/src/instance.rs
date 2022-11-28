use crate::error::Error;
use activitypub_federation::core::axum::{
    inbox::receive_activity, json::ApubJson, verify_request_payload, DigestVerified,
};
use activitypub_federation::data::Data;
use activitypub_federation::deser::context::WithContext;
use activitypub_federation::traits::ApubObject;
use activitypub_federation::{
    core::object_id::ObjectId, InstanceSettings, LocalInstance, UrlVerifier,
};
use axum::async_trait;
use axum::body::Body;
use axum::extract::{OriginalUri, State};
use axum::headers::HeaderMap;
use axum::http::{Method, Request};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post;
use axum::{body, middleware, Extension, Json, Router};
use reqwest::Client;
use sqlx::PgPool;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::ServiceBuilderExt;
use url::Url;

use crate::object::user::{ApubUser, Person, PersonAcceptedActivities};

pub type InstanceHandle = Arc<Instance>;

pub struct Instance {
    local_instance: LocalInstance,
    db: PgPool,
}

/// Use this to store your federation blocklist, or a database connection needed to retrieve it.
#[derive(Clone)]
struct MyUrlVerifier();

#[async_trait]
impl UrlVerifier for MyUrlVerifier {
    async fn verify(&self, url: &Url) -> Result<(), &'static str> {
        if url.domain() == Some("malicious.com") {
            Err("malicious domain")
        } else {
            Ok(())
        }
    }
}

impl Instance {
    pub fn new(hostname: String, db: PgPool) -> Result<InstanceHandle, Error> {
        let settings = InstanceSettings::builder()
            .debug(true)
            .url_verifier(Box::new(MyUrlVerifier()))
            .build()?;

        let local_instance =
            LocalInstance::new(hostname.clone(), Client::default().into(), settings);

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
        let app = Router::new()
            .route("/inbox", post(http_post_user_inbox))
            .layer(
                ServiceBuilder::new()
                    .map_request_body(body::boxed)
                    .layer(middleware::from_fn(verify_request_payload)),
            )
            .route("/objects/:user_name", get(http_get_user))
            .with_state(instance)
            .layer(TraceLayer::new_for_http());

        // run it
        let addr = hostname
            .to_socket_addrs()?
            .next()
            .expect("Failed to lookup domain name");

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}

async fn http_get_user(
    State(data): State<InstanceHandle>,
    request: Request<Body>,
) -> Result<ApubJson<WithContext<Person>>, Error> {
    let hostname: String = data.local_instance.hostname().to_string();
    let request_url = format!("http://{}{}", hostname, &request.uri());
    let url = Url::parse(&request_url).expect("Failed to parse url");
    let user = ObjectId::<ApubUser>::new(url)
        .dereference_local(&data)
        .await?
        .into_apub(&data)
        .await?;

    Ok(ApubJson(WithContext::new_default(user)))
}

async fn http_post_user_inbox(
    headers: HeaderMap,
    method: Method,
    OriginalUri(uri): OriginalUri,
    State(data): State<InstanceHandle>,
    Extension(digest_verified): Extension<DigestVerified>,
    Json(activity): Json<WithContext<PersonAcceptedActivities>>,
) -> impl IntoResponse {
    receive_activity::<WithContext<PersonAcceptedActivities>, ApubUser, InstanceHandle>(
        digest_verified,
        activity,
        &data.clone().local_instance,
        &Data::new(data),
        headers,
        method,
        uri,
    )
    .await
}

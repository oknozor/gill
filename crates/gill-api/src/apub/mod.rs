use crate::apub::object::user::{ApubUser, Person, PersonAcceptedActivities};
use crate::error::AppError;
use crate::instance::{Instance, InstanceHandle};
use activitypub_federation::core::axum::inbox::receive_activity;
use activitypub_federation::core::axum::json::ApubJson;
use activitypub_federation::core::axum::{verify_request_payload, DigestVerified};
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::data::Data;
use activitypub_federation::deser::context::WithContext;
use activitypub_federation::traits::ApubObject;
use axum::body::Body;
use axum::extract::{OriginalUri, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{body, middleware, Extension, Json, Router};
use http::{HeaderMap, Method, Request};

use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

use url::Url;

pub mod activities;
pub mod object;

pub fn router(instance: Arc<Instance>) -> Router {
    let public = Router::new().route("/users/:user_name", get(http_get_user));

    let private = Router::new()
        .route("/:user/inbox", post(http_post_user_inbox))
        .layer(
            ServiceBuilder::new()
                .map_request_body(body::boxed)
                .layer(middleware::from_fn(verify_request_payload)),
        );

    public.merge(private).with_state(instance)
}

async fn http_get_user(
    State(data): State<InstanceHandle>,
    request: Request<Body>,
) -> Result<ApubJson<WithContext<Person>>, AppError> {
    let hostname: String = data.local_instance.hostname().to_string();
    let request_url = format!("http://{}/apub{}", hostname, &request.uri());
    let url = Url::parse(&request_url).expect("Failed to parse url");
    let user = ObjectId::<ApubUser>::new(url)
        .dereference_local(&data)
        .await?
        .into_apub(&data)
        .await;

    let user = WithContext::new_default(user?);
    println!("{:?}", user);
    Ok(ApubJson(user))
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

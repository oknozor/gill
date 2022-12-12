use crate::apub::object::user::{ApubUser, PersonAcceptedActivities, UserWrapper};
use crate::error::AppError;
use crate::instance::{Instance, InstanceHandle};
use activitypub_federation::core::axum::inbox::receive_activity;
use activitypub_federation::core::axum::json::ApubJson;
use activitypub_federation::core::axum::{verify_request_payload, DigestVerified};

use activitypub_federation::data::Data;
use activitypub_federation::deser::context::WithContext;
use activitypub_federation::traits::ApubObject;

use axum::extract::{OriginalUri, Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{body, middleware, Extension, Json, Router};
use http::{HeaderMap, Method};

use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

use crate::apub::object::repository::{
    ApubRepository, RepositoryAcceptedActivities, RepositoryWrapper,
};

pub mod activities;
pub mod object;

pub fn router(instance: Arc<Instance>) -> Router {
    let public = Router::new()
        .route("/users/:user", get(user))
        .route("/users/:user/repositories/:repository", get(repository));

    let private = Router::new()
        .route("/users/:user/inbox", post(user_inbox))
        .route(
            "/users/:user/repositories/:repository/inbox",
            post(repository_inbox),
        )
        .layer(
            ServiceBuilder::new()
                .map_request_body(body::boxed)
                .layer(middleware::from_fn(verify_request_payload)),
        );

    public.merge(private).with_state(instance)
}

async fn user(
    Path(user): Path<String>,
    State(data): State<InstanceHandle>,
) -> Result<ApubJson<WithContext<ApubUser>>, AppError> {
    let object_id = UserWrapper::activity_pub_id_from_namespace(&user)?;
    let user = object_id
        .dereference_local(&data)
        .await?
        .into_apub(&data)
        .await;
    Ok(ApubJson(WithContext::new_default(user?)))
}

async fn repository(
    State(data): State<InstanceHandle>,
    Path((user, repository)): Path<(String, String)>,
) -> Result<ApubJson<WithContext<ApubRepository>>, AppError> {
    let object_id = RepositoryWrapper::activity_pub_id_from_namespace(&user, &repository)?;
    let repository = object_id.dereference_local(&data).await?;
    let repository = repository.into_apub(&data).await;
    let repository = WithContext::new_default(repository?);
    Ok(ApubJson(repository))
}

async fn user_inbox(
    headers: HeaderMap,
    method: Method,
    OriginalUri(uri): OriginalUri,
    State(data): State<InstanceHandle>,
    Extension(digest_verified): Extension<DigestVerified>,
    Json(activity): Json<WithContext<PersonAcceptedActivities>>,
) -> impl IntoResponse {
    receive_activity::<WithContext<PersonAcceptedActivities>, UserWrapper, InstanceHandle>(
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

async fn repository_inbox(
    headers: HeaderMap,
    method: Method,
    OriginalUri(uri): OriginalUri,
    State(data): State<InstanceHandle>,
    Extension(digest_verified): Extension<DigestVerified>,
    Json(activity): Json<WithContext<RepositoryAcceptedActivities>>,
) -> impl IntoResponse {
    receive_activity::<WithContext<RepositoryAcceptedActivities>, UserWrapper, InstanceHandle>(
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

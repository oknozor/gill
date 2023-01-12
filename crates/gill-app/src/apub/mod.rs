use crate::error::AppResult;
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

use crate::apub::ticket::ApubTicket;

use repository::{ApubRepository, RepositoryAcceptedActivities};

use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;

use uuid::Uuid;

use crate::apub::ticket::comment::ApubIssueComment;
use crate::domain::issue::comment::IssueComment;
use crate::domain::issue::Issue;
use crate::domain::repository::Repository;
use crate::domain::user::User;
use user::{ApubUser, PersonAcceptedActivities};

pub mod commit;
pub mod common;
pub mod repository;
pub mod ticket;
pub mod user;

pub fn router(instance: Arc<Instance>) -> Router {
    let public = Router::new()
        .route("/users/:user", get(user))
        .route("/users/:user/repositories/:repository", get(repository))
        .route(
            "/users/:user/repositories/:repository/issues/:number",
            get(issue),
        )
        .route(
            "/users/:user/repositories/:repository/issues/:number/comments/:uuid",
            get(comment),
        );

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
) -> AppResult<ApubJson<WithContext<ApubUser>>> {
    let object_id = User::activity_pub_id_from_namespace(&user)?;
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
) -> AppResult<ApubJson<WithContext<ApubRepository>>> {
    let object_id = Repository::activity_pub_id_from_namespace(&user, &repository)?;
    let repository = object_id.dereference_local(&data).await?;
    let repository = repository.into_apub(&data).await;
    let repository = WithContext::new_default(repository?);
    Ok(ApubJson(repository))
}

async fn issue(
    State(data): State<InstanceHandle>,
    Path((user, repository, issue_number)): Path<(String, String, i32)>,
) -> AppResult<ApubJson<WithContext<ApubTicket>>> {
    let object_id = Issue::activity_pub_id_from_namespace(&user, &repository, issue_number)?;
    let ticket = object_id.dereference_local(&data).await?;
    let ticket = ticket.into_apub(&data).await;
    let ticket = WithContext::new_default(ticket?);
    Ok(ApubJson(ticket))
}

async fn comment(
    State(data): State<InstanceHandle>,
    Path((user, repository, issue_number, uuid)): Path<(String, String, i32, Uuid)>,
) -> AppResult<ApubJson<WithContext<ApubIssueComment>>> {
    let object_id =
        IssueComment::activity_pub_id_from_namespace(&user, &repository, issue_number, uuid)?;
    let comment = object_id.dereference_local(&data).await?;
    let comment = comment.into_apub(&data).await;
    let comment = WithContext::new_default(comment?);
    Ok(ApubJson(comment))
}

async fn user_inbox(
    headers: HeaderMap,
    method: Method,
    OriginalUri(uri): OriginalUri,
    State(data): State<InstanceHandle>,
    Extension(digest_verified): Extension<DigestVerified>,
    Json(activity): Json<WithContext<PersonAcceptedActivities>>,
) -> impl IntoResponse {
    receive_activity::<WithContext<PersonAcceptedActivities>, User, InstanceHandle>(
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
    receive_activity::<WithContext<RepositoryAcceptedActivities>, User, InstanceHandle>(
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

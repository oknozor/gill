use crate::error::AppError;
use crate::instance::{Instance, InstanceHandle};
use activitypub_federation::core::axum::inbox::receive_activity;
use activitypub_federation::core::axum::json::ApubJson;
use activitypub_federation::core::axum::{verify_request_payload, DigestVerified};

use activitypub_federation::data::Data;
use activitypub_federation::deser::context::WithContext;
use activitypub_federation::traits::{ActivityHandler, ApubObject};

use axum::extract::{OriginalUri, Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{body, middleware, Extension, Json, Router};
use http::{HeaderMap, Method};

use crate::apub::ticket::{ApubTicket, IssueWrapper};
use activitypub_federation::core::activity_queue::send_activity;
use activitypub_federation::core::signatures::PublicKey;
use activitypub_federation::LocalInstance;
use axum::async_trait;
use repository::{ApubRepository, RepositoryAcceptedActivities, RepositoryWrapper};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;
use tracing::log::info;
use url::{ParseError, Url};
use user::{ApubUser, PersonAcceptedActivities, UserWrapper};

pub mod commit;
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

async fn issue(
    State(data): State<InstanceHandle>,
    Path((user, repository, issue_number)): Path<(String, String, i32)>,
) -> Result<ApubJson<WithContext<ApubTicket>>, AppError> {
    let object_id = IssueWrapper::activity_pub_id_from_namespace(&user, &repository, issue_number)?;
    let ticket = object_id.dereference_local(&data).await?;
    let ticket = ticket.into_apub(&data).await;
    let ticket = WithContext::new_default(ticket?);
    Ok(ApubJson(ticket))
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum CreateOrUpdateType {
    Create,
    Update,
}

#[async_trait]
pub trait GillApubObject {
    fn view_uri(&self) -> String;

    fn followers_url(&self) -> Result<Url, AppError>;

    async fn followers(&self, db: &InstanceHandle) -> Result<Vec<Url>, AppError>;

    fn local_id(&self) -> i32;

    fn activity_pub_id(&self) -> &str;

    fn public_key_with_owner(&self) -> Result<PublicKey, ParseError>;

    fn private_key(&self) -> Option<String>;

    fn activity_pub_id_as_url(&self) -> Result<Url, ParseError> {
        Url::parse(self.activity_pub_id())
    }

    async fn send<Activity>(
        &self,
        activity: Activity,
        recipients: Vec<Url>,
        local_instance: &LocalInstance,
    ) -> Result<(), <Activity as ActivityHandler>::Error>
    where
        Activity: ActivityHandler + Serialize + Send + Sync,
        <Activity as ActivityHandler>::Error:
            From<anyhow::Error> + From<serde_json::Error> + From<AppError> + From<ParseError>,
    {
        let activity = WithContext::new_default(activity);
        send_activity(
            activity,
            self.public_key_with_owner()?,
            self.private_key().expect("has private key"),
            recipients,
            local_instance,
        )
        .await?;
        Ok(())
    }
}

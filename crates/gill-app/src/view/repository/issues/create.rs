use crate::domain::issue::CreateIssueCommand;
use crate::error::AppError;
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::Form;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CreateIssueForm {
    pub title: String,
    pub content: String,
}

pub async fn create(
    connected_user: Option<Oauth2User>,
    Path((owner, repository)): Path<(String, String)>,
    State(state): State<AppState>,
    Form(form): Form<CreateIssueForm>,
) -> Result<Redirect, AppError> {
    let db = state.instance.database();
    let Some(user) = get_connected_user(db, connected_user).await else {
        return Err(AppError::Unauthorized);
    };

    let command = CreateIssueCommand::from(form);
    command
        .execute(&repository, &owner, user, &state.instance)
        .await?;
    Ok(Redirect::to(&format!("/{owner}/{repository}/issues")))
}

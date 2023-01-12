use crate::domain::issue::comment::create::CreateIssueCommentCommand;
use crate::error::{AppError, AppResult};
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::{Extension, Form};

use gill_authorize_derive::authorized;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize, Debug)]
pub struct IssueCommentForm {
    pub comment: String,
}

#[authorized]
pub async fn comment(
    user: Option<Oauth2User>,
    Path((owner, repository, issue_number)): Path<(String, String, i32)>,
    State(state): State<AppState>,
    Extension(db): Extension<PgPool>,
    Form(input): Form<IssueCommentForm>,
) -> AppResult<Redirect> {
    let create_comment = CreateIssueCommentCommand {
        owner: &owner,
        repository: &repository,
        author_id: user.id,
        issue_number,
        content: &input.comment,
    };

    create_comment.execute(&state.instance).await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/issues/{issue_number}"
    )))
}

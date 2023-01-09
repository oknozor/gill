use crate::domain::issue::comment::CreateIssueCommentCommand;
use crate::error::AppError;
use crate::get_connected_user;
use crate::oauth::Oauth2User;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::Form;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct IssueCommentForm {
    pub comment: String,
}

pub async fn comment(
    user: Option<Oauth2User>,
    Path((owner, repository, issue_number)): Path<(String, String, i32)>,
    State(state): State<AppState>,
    Form(input): Form<IssueCommentForm>,
) -> Result<Redirect, AppError> {
    let db = state.instance.database();
    let Some(user) = get_connected_user(db, user).await else {
        return Err(AppError::Unauthorized);
    };

    let create_comment = CreateIssueCommentCommand {
        owner: &owner,
        repository: &repository,
        author_id: user.id,
        issue_number,
        content: input.comment,
    };

    create_comment.execute(&state.instance).await?;

    Ok(Redirect::to(&format!(
        "/{owner}/{repository}/issues/{issue_number}"
    )))
}

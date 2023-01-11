use crate::domain::id::ActivityPubId;
use crate::domain::issue::Issue;
use crate::domain::user::User;
use crate::error::AppError;


use gill_db::repository::branch::Branch;
use gill_db::repository::Repository as RepositoryEntity;

use sqlx::PgPool;

use crate::domain::issue::digest::IssueDigest;
use crate::domain::pull_request::PullRequest;
use url::{ParseError, Url};

pub mod branch;
pub mod create;
pub mod digest;
pub mod stats;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    pub id: i32,
    pub activity_pub_id: ActivityPubId<Repository>,
    pub name: String,
    pub summary: Option<String>,
    pub private: bool,
    pub inbox_url: Url,
    pub outbox_url: Url,
    pub followers_url: Url,
    pub attributed_to: ActivityPubId<User>,
    pub clone_uri: Url,
    pub public_key: String,
    pub private_key: Option<String>,
    pub published: chrono::NaiveDateTime,
    pub ticket_tracked_by: ActivityPubId<Repository>,
    pub send_patches_to: Url,
    pub domain: String,
    pub item_count: i32,
    pub is_local: bool,
}

impl TryFrom<RepositoryEntity> for Repository {
    type Error = ParseError;

    fn try_from(repository: RepositoryEntity) -> Result<Self, ParseError> {
        Ok(Self {
            id: repository.id,
            activity_pub_id: ActivityPubId::try_from(repository.activity_pub_id)?,
            name: repository.name,
            summary: repository.summary,
            private: repository.private,
            inbox_url: Url::parse(&repository.inbox_url)?,
            outbox_url: Url::parse(&repository.outbox_url)?,
            followers_url: Url::parse(&repository.followers_url)?,
            attributed_to: ActivityPubId::try_from(repository.attributed_to)?,
            clone_uri: Url::parse(&repository.clone_uri)?,
            public_key: repository.public_key,
            private_key: repository.private_key,
            published: Default::default(),
            ticket_tracked_by: ActivityPubId::try_from(repository.ticket_tracked_by)?,
            send_patches_to: Url::parse(&repository.send_patches_to)?,
            domain: repository.domain,
            item_count: repository.item_count,
            is_local: repository.is_local,
        })
    }
}

impl From<&Repository> for RepositoryEntity {
    fn from(val: &Repository) -> Self {
        RepositoryEntity {
            id: val.id,
            activity_pub_id: val.activity_pub_id.to_string(),
            name: val.name.to_string(),
            summary: val.summary.as_ref().map(|sumary| sumary.to_string()),
            private: val.private,
            inbox_url: val.inbox_url.to_string(),
            outbox_url: val.outbox_url.to_string(),
            followers_url: val.followers_url.to_string(),
            attributed_to: val.attributed_to.to_string(),
            clone_uri: val.clone_uri.to_string(),
            public_key: val.public_key.to_string(),
            private_key: val.private_key.as_ref().map(|pk| pk.to_string()),
            published: val.published,
            ticket_tracked_by: val.ticket_tracked_by.to_string(),
            send_patches_to: val.send_patches_to.to_string(),
            domain: val.domain.to_string(),
            item_count: val.item_count,
            is_local: val.is_local,
        }
    }
}

impl Repository {
    pub async fn by_namespace(
        owner: &str,
        name: &str,
        db: &PgPool,
    ) -> Result<Repository, AppError> {
        let repository = RepositoryEntity::by_namespace(owner, name, db).await?;
        Repository::try_from(repository).map_err(Into::into)
    }

    pub async fn by_activity_pub_id(activity_pub_id: &str, db: &PgPool) -> Result<Self, AppError> {
        let entity = RepositoryEntity::by_activity_pub_id(activity_pub_id, db).await?;
        Repository::try_from(entity).map_err(Into::into)
    }

    pub async fn by_activity_pub_id_optional(
        activity_pub_id: &str,
        db: &PgPool,
    ) -> Result<Option<Self>, AppError> {
        let entity = RepositoryEntity::by_activity_pub_id(activity_pub_id, db).await;
        match entity {
            Ok(entity) => {
                let repository = Repository::try_from(entity)?;
                Ok(Some(repository))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(AppError::from(err)),
        }
    }

    pub async fn add_watcher(&self, watcher_id: i32, db: &PgPool) -> Result<(), AppError> {
        let entity: RepositoryEntity = self.into();
        entity.add_watcher(watcher_id, db).await.map_err(Into::into)
    }

    pub async fn add_fork(&self, forked_by: i32, fork: i32, db: &PgPool) -> Result<(), AppError> {
        let entity: RepositoryEntity = self.into();
        entity
            .add_fork(fork, forked_by, db)
            .await
            .map_err(Into::into)
    }

    pub async fn add_star(&self, starred_by: i32, db: &PgPool) -> Result<(), AppError> {
        let entity: RepositoryEntity = self.into();
        entity.add_star(starred_by, db).await.map_err(Into::into)
    }

    pub async fn issue_by_number(&self, number: i32, db: &PgPool) -> Result<Issue, AppError> {
        let entity: RepositoryEntity = self.into();
        let issue = entity.get_issue(number, db).await?;
        Issue::try_from(issue).map_err(Into::into)
    }

    pub async fn get_watchers(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> Result<Vec<User>, AppError> {
        let entity: RepositoryEntity = self.into();
        let watchers = entity.get_watchers(limit, offset, db).await?;
        let watchers = watchers
            .into_iter()
            .map(User::try_from)
            .filter_map(Result::ok)
            .collect();
        Ok(watchers)
    }

    pub async fn get_issue_digest(
        &self,
        number: i32,
        db: &PgPool,
    ) -> Result<IssueDigest, AppError> {
        let entity: RepositoryEntity = self.into();
        let issue = entity.get_issue_digest(number, db).await?;
        Ok(IssueDigest::from(issue))
    }

    pub async fn close_issue(&self, issue_number: i32, db: &PgPool) -> Result<(), AppError> {
        let entity: RepositoryEntity = self.into();
        let issue = entity.get_issue_digest(issue_number, db).await?;
        issue.close(db).await.map_err(Into::into)
    }

    pub async fn get_pull_request(
        &self,
        number: i32,
        db: &PgPool,
    ) -> Result<PullRequest, AppError> {
        let entity: RepositoryEntity = self.into();
        let entity = entity.get_pull_request(number, db).await?;
        Ok(PullRequest::from(entity))
    }

    pub async fn list_pull_requests(&self, db: &PgPool) -> Result<Vec<PullRequest>, AppError> {
        let entity: RepositoryEntity = self.into();
        let entities = entity.list_pull_requests(db).await?;
        Ok(entities
            .into_iter()
            .map(PullRequest::try_from)
            .filter_map(Result::ok)
            .collect())
    }

    pub async fn get_default_branch(&self, db: &PgPool) -> Option<Branch> {
        let entity: RepositoryEntity = self.into();
        entity.get_default_branch(db).await
    }

    pub async fn create_pull_request(
        &self,
        user_id: i32,
        title: &str,
        description: Option<&str>,
        base: &str,
        compare: &str,
        db: &PgPool,
    ) -> Result<(), AppError> {
        let entity: RepositoryEntity = self.into();
        entity
            .create_pull_request(user_id, title, description, base, compare, db)
            .await
            .map_err(Into::into)
    }

    pub async fn list_issues(&self, db: &PgPool) -> Result<Vec<IssueDigest>, AppError> {
        let entity: RepositoryEntity = self.into();
        let entities = entity.list_issues(db).await?;
        Ok(entities.into_iter().map(IssueDigest::from).collect())
    }

    pub async fn list_branches(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> Result<Vec<Branch>, AppError> {
        let repository: RepositoryEntity = self.into();
        let branches = repository.list_branches(limit, offset, db).await?;
        Ok(branches.into_iter().map(Branch::from).collect())
    }
}

use crate::apub::common::GillApubObject;
use crate::apub::repository::star::Star;
use crate::apub::repository::watch::Watch;
use crate::apub::user::follow::Follow;
use crate::domain::id::ActivityPubId;
use crate::domain::repository::digest::RepositoryDigest;
use crate::domain::repository::Repository;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::traits::Actor;
use gill_db::user::User as UserEntity;
use gill_settings::SETTINGS;
use sqlx::PgPool;
use std::str::FromStr;
use url::{ParseError, Url};
use uuid::Uuid;

pub mod create;
pub mod ssh_key;

/// A user living in gill database
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: i32,
    pub activity_pub_id: ActivityPubId<User>,
    pub username: String,
    pub domain: String,
    pub email: Option<String>,
    pub public_key: String,
    pub private_key: Option<String>,
    pub inbox_url: Url,
    pub outbox_url: Url,
    pub followers_url: Url,
    pub is_local: bool,
}

impl TryFrom<UserEntity> for User {
    type Error = ParseError;

    fn try_from(user: UserEntity) -> Result<Self, ParseError> {
        Ok(Self {
            id: user.id,
            activity_pub_id: ActivityPubId::try_from(user.activity_pub_id)?,
            username: user.username,
            domain: user.domain,
            email: user.email,
            public_key: user.public_key,
            private_key: user.private_key,
            inbox_url: Url::parse(&user.inbox_url)?,
            outbox_url: Url::parse(&user.outbox_url)?,
            followers_url: Url::parse(&user.followers_url)?,
            is_local: user.is_local,
        })
    }
}

impl From<&User> for UserEntity {
    fn from(val: &User) -> Self {
        UserEntity {
            id: val.id,
            activity_pub_id: val.activity_pub_id.to_string(),
            username: val.username.to_owned(),
            domain: val.domain.to_owned(),
            email: val.email.clone(),
            public_key: val.public_key.to_owned(),
            private_key: val.private_key.clone(),
            inbox_url: val.inbox_url.to_string(),
            outbox_url: val.outbox_url.to_string(),
            followers_url: val.followers_url.to_string(),
            is_local: val.is_local,
        }
    }
}

impl User {
    pub async fn by_id(id: i32, db: &PgPool) -> Result<User, AppError> {
        let entity = UserEntity::by_id(id, db).await?;
        User::try_from(entity).map_err(Into::into)
    }

    pub async fn by_activity_pub_id(activity_pub_id: &str, db: &PgPool) -> Result<User, AppError> {
        let entity = UserEntity::by_activity_pub_id(activity_pub_id, db).await?;
        User::try_from(entity).map_err(Into::into)
    }

    pub async fn by_activity_pub_id_optional(
        activity_pub_id: &str,
        db: &PgPool,
    ) -> Result<Option<User>, AppError> {
        let entity = UserEntity::by_activity_pub_id(activity_pub_id, db).await;
        match entity {
            Ok(entity) => {
                let repository = User::try_from(entity)?;
                Ok(Some(repository))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(AppError::from(err)),
        }
    }

    pub async fn by_email(email: &str, db: &PgPool) -> Result<User, AppError> {
        let entity = UserEntity::by_email(email, db).await?;
        User::try_from(entity).map_err(Into::into)
    }

    pub async fn by_name(name: &str, db: &PgPool) -> Result<User, AppError> {
        let entity = UserEntity::by_user_name(name, db).await?;
        User::try_from(entity).map_err(Into::into)
    }

    pub async fn add_follower(&self, follower_id: i32, db: &PgPool) -> Result<(), AppError> {
        let entity: UserEntity = self.into();
        entity.add_follower(follower_id, db).await?;
        Ok(())
    }

    pub fn activity_pub_id_from_namespace(user: &str) -> Result<ObjectId<Self>, AppError> {
        let domain = &SETTINGS.domain;
        let scheme = if SETTINGS.debug { "http" } else { "https" };
        let url = Url::from_str(&format!("{scheme}://{domain}/apub/users/{user}"))?;
        Ok(ObjectId::new(url))
    }

    pub async fn follow(&self, other: &User, instance: &InstanceHandle) -> Result<(), AppError> {
        let follower = self.activity_pub_id.clone().into();
        let following = other.activity_pub_id.clone().into();
        let hostname = instance.local_instance().hostname();
        let activity_id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
        let activity_id = Url::parse(&activity_id)?;
        let follow = Follow::new(follower, following, activity_id);

        tracing::debug!(
            "Sending follow activity to user inboc {}",
            other.shared_inbox_or_inbox()
        );

        self.send(
            follow,
            vec![other.shared_inbox_or_inbox()],
            instance.local_instance(),
        )
        .await?;
        Ok(())
    }

    pub async fn watch_repository(
        &self,
        other: &Repository,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let watcher = self.activity_pub_id.clone().into();
        let watching = other.activity_pub_id.clone().into();
        let hostname = instance.local_instance().hostname();
        let activity_id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
        let activity_id = Url::parse(&activity_id)?;
        let watch = Watch::new(watcher, watching, activity_id);

        tracing::debug!(
            "Sending watch activity to repository inbox {}",
            other.shared_inbox_or_inbox()
        );

        self.send(
            watch,
            vec![other.shared_inbox_or_inbox()],
            instance.local_instance(),
        )
        .await?;
        Ok(())
    }

    pub async fn star_repository(
        &self,
        other: &Repository,
        instance: &InstanceHandle,
    ) -> Result<(), AppError> {
        let starred_by = self.activity_pub_id.clone().into();
        let starred = other.activity_pub_id.clone().into();
        let hostname = instance.local_instance().hostname();
        let activity_id = format!("https://{hostname}/activity/{uuid}", uuid = Uuid::new_v4());
        let activity_id = Url::parse(&activity_id)?;
        let star = Star::new(starred_by, starred, activity_id);
        tracing::debug!(
            "Sending star activity to repository inbox {}",
            other.shared_inbox_or_inbox()
        );

        self.send(
            star,
            vec![other.shared_inbox_or_inbox()],
            instance.local_instance(),
        )
        .await?;
        Ok(())
    }

    pub async fn add_ssh_key(
        &self,
        key_name: &str,
        ssh_key: &str,
        key_type: &str,
        db: &PgPool,
    ) -> Result<(), AppError> {
        let entity: UserEntity = self.into();
        entity
            .add_ssh_key(key_name, ssh_key, key_type, db)
            .await
            .map_err(Into::into)
    }

    pub async fn get_followers(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> Result<Vec<User>, AppError> {
        let entity: UserEntity = self.into();
        let follower_entities = entity.get_followers(limit, offset, db).await?;
        let followers = follower_entities
            .into_iter()
            .map(User::try_from)
            .filter_map(Result::ok)
            .collect();

        Ok(followers)
    }

    pub async fn list_repositories(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> Result<Vec<RepositoryDigest>, AppError> {
        let user: UserEntity = self.into();
        let repositories = user.list_repositories(limit, offset, db).await?;
        Ok(repositories
            .into_iter()
            .map(RepositoryDigest::from)
            .collect())
    }

    pub async fn starred_repositories(
        &self,
        limit: i64,
        offset: i64,
        db: &PgPool,
    ) -> Result<Vec<RepositoryDigest>, AppError> {
        let user: UserEntity = self.into();
        let repositories = user.list_starred_repositories(limit, offset, db).await?;
        Ok(repositories
            .into_iter()
            .map(RepositoryDigest::from)
            .collect())
    }

    pub async fn get_local_repository_by_name(
        &self,
        repo_name: &str,
        db: &PgPool,
    ) -> Result<Repository, AppError> {
        let user: UserEntity = self.into();
        let repository = user.get_local_repository_by_name(repo_name, db).await?;
        Repository::try_from(repository).map_err(Into::into)
    }
}

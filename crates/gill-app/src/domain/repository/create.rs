use crate::domain::id::ActivityPubId;
use crate::domain::repository::Repository;
use crate::domain::user::User;
use crate::error::AppError;
use gill_db::repository::create::CreateRepository as CreateRepositoryEntity;
use gill_db::Insert;
use sqlx::PgPool;
use url::Url;

pub struct CreateRepository {
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
    pub ticket_tracked_by: ActivityPubId<Repository>,
    pub send_patches_to: ActivityPubId<Repository>,
    pub domain: String,
    pub is_local: bool,
}

impl From<CreateRepository> for CreateRepositoryEntity {
    fn from(val: CreateRepository) -> Self {
        CreateRepositoryEntity {
            activity_pub_id: val.activity_pub_id.to_string(),
            name: val.name,
            summary: val.summary,
            private: val.private,
            inbox_url: val.inbox_url.to_string(),
            outbox_url: val.outbox_url.to_string(),
            followers_url: val.followers_url.to_string(),
            attributed_to: val.attributed_to.to_string(),
            clone_uri: val.clone_uri.to_string(),
            public_key: val.public_key,
            private_key: val.private_key,
            ticket_tracked_by: val.ticket_tracked_by.to_string(),
            send_patches_to: val.send_patches_to.to_string(),
            domain: val.domain,
            is_local: val.is_local,
        }
    }
}

impl CreateRepository {
    pub async fn save(self, db: &PgPool) -> Result<Repository, AppError> {
        let entity: CreateRepositoryEntity = self.into();
        let repository = entity.insert(db).await?;
        Repository::try_from(repository).map_err(Into::into)
    }
}

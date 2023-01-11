use crate::domain::id::ActivityPubId;
use crate::domain::user::User;
use crate::error::AppError;
use gill_db::user::CreateUser as CreateUserEntity;
use gill_db::Insert;
use sqlx::PgPool;
use url::Url;

#[derive(Debug, Clone)]
pub struct CreateUser {
    pub username: String,
    pub email: Option<String>,
    pub private_key: Option<String>,
    pub public_key: String,
    pub activity_pub_id: ActivityPubId<User>,
    pub outbox_url: Url,
    pub inbox_url: Url,
    pub domain: String,
    pub followers_url: Url,
    pub is_local: bool,
}

impl From<CreateUser> for CreateUserEntity {
    fn from(val: CreateUser) -> Self {
        CreateUserEntity {
            username: val.username,
            email: val.email,
            private_key: val.private_key,
            public_key: val.public_key,
            activity_pub_id: val.activity_pub_id.to_string(),
            outbox_url: val.outbox_url.to_string(),
            inbox_url: val.inbox_url.to_string(),
            domain: val.domain,
            followers_url: val.followers_url.to_string(),
            is_local: val.is_local,
        }
    }
}

impl CreateUser {
    pub async fn save(self, db: &PgPool) -> Result<User, AppError> {
        let entity: CreateUserEntity = self.into();
        let user = entity.insert(db).await?;
        User::try_from(user).map_err(Into::into)
    }
}

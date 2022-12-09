use crate::apub::object::repository::RepositoryWrapper;
use crate::apub::object::user::UserWrapper;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::data::Data;
use activitypub_federation::traits::ActivityHandler;
use activitystreams_kinds::activity::FollowType;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Watch {
    id: Url,
    pub user: ObjectId<UserWrapper>,
    pub repository: ObjectId<RepositoryWrapper>,
    r#type: FollowType,
}

impl Watch {
    pub fn new(
        user: ObjectId<UserWrapper>,
        repository: ObjectId<RepositoryWrapper>,
        id: Url,
    ) -> Watch {
        Watch {
            id,
            user,
            repository,
            r#type: Default::default(),
        }
    }
}

#[async_trait]
impl ActivityHandler for Watch {
    type DataType = InstanceHandle;
    type Error = AppError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.user.inner()
    }

    async fn verify(
        &self,
        _data: &Data<Self::DataType>,
        _request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn receive(
        self,
        data: &Data<Self::DataType>,
        _request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        let user = ObjectId::<UserWrapper>::new(self.user)
            .dereference_local(data)
            .await?;

        let repository = ObjectId::<RepositoryWrapper>::new(self.repository)
            .dereference(data, data.local_instance(), &mut 0)
            .await?;

        repository.add_watcher(user.local_id(), data).await?;
        Ok(())
    }
}

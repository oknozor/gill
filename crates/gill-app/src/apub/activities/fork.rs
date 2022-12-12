use crate::apub::object::repository::RepositoryWrapper;
use crate::apub::object::user::UserWrapper;
use crate::apub::object::GillApubObject;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::data::Data;
use activitypub_federation::traits::ActivityHandler;
use activitystreams_kinds::activity::CreateType;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Fork {
    id: Url,
    pub repository: ObjectId<RepositoryWrapper>,
    pub fork: ObjectId<RepositoryWrapper>,
    pub forked_by: ObjectId<UserWrapper>,
    r#type: CreateType,
}

impl Fork {
    pub fn new(
        forked_by: ObjectId<UserWrapper>,
        repository: ObjectId<RepositoryWrapper>,
        fork: ObjectId<RepositoryWrapper>,
        id: Url,
    ) -> Fork {
        Fork {
            id,
            repository,
            fork,
            forked_by,
            r#type: Default::default(),
        }
    }
}

#[async_trait]
impl ActivityHandler for Fork {
    type DataType = InstanceHandle;
    type Error = AppError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.forked_by.inner()
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
        let user = ObjectId::<UserWrapper>::new(self.forked_by)
            .dereference_local(data)
            .await?;

        let repository = ObjectId::<RepositoryWrapper>::new(self.repository)
            .dereference(data, data.local_instance(), &mut 0)
            .await?;

        let fork = ObjectId::<RepositoryWrapper>::new(self.fork)
            .dereference(data, data.local_instance(), &mut 0)
            .await?;

        repository
            .add_fork(fork.local_id(), user.local_id(), data)
            .await?;
        Ok(())
    }
}

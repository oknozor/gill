use crate::apub::user::UserWrapper;
use crate::error::AppError;
use crate::instance::InstanceHandle;

use crate::apub::GillApubObject;
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::activity::FollowType;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    id: Url,
    pub follower: ObjectId<UserWrapper>,
    pub followed: ObjectId<UserWrapper>,
    r#type: FollowType,
}

impl Follow {
    pub fn new(
        follower: ObjectId<UserWrapper>,
        followed: ObjectId<UserWrapper>,
        id: Url,
    ) -> Follow {
        Follow {
            id,
            follower,
            followed,
            r#type: Default::default(),
        }
    }
}

#[async_trait]
impl ActivityHandler for Follow {
    type DataType = InstanceHandle;
    type Error = AppError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.follower.inner()
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
        let followed = ObjectId::<UserWrapper>::new(self.followed)
            .dereference_local(data)
            .await?;

        let follower = ObjectId::<UserWrapper>::new(self.follower)
            .dereference(data, data.local_instance(), &mut 0)
            .await?;

        followed.add_follower(follower.local_id(), data).await?;
        Ok(())
    }
}

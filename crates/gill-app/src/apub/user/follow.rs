use crate::error::AppError;
use crate::instance::InstanceHandle;

use crate::apub::common::GillApubObject;
use crate::domain::user::User;
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::activity::FollowType;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    id: Url,
    pub follower: ObjectId<User>,
    pub followed: ObjectId<User>,
    r#type: FollowType,
}

impl Follow {
    pub fn new(follower: ObjectId<User>, followed: ObjectId<User>, id: Url) -> Follow {
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

    async fn receive(
        self,
        data: &Data<Self::DataType>,
        _request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        let followed = ObjectId::<User>::new(self.followed)
            .dereference_local(data)
            .await?;

        let follower = ObjectId::<User>::new(self.follower)
            .dereference(data, data.local_instance(), &mut 0)
            .await?;

        followed
            .add_follower(follower.local_id(), data.database())
            .await?;
        Ok(())
    }
}

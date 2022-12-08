use crate::apub::object::user::ApubUser;
use crate::error::AppError;
use crate::instance::InstanceHandle;
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::activity::FollowType;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    id: Url,
    pub actor: ObjectId<ApubUser>,
    pub object: ObjectId<ApubUser>,
    r#type: FollowType,
}

impl Follow {
    pub fn new(actor: ObjectId<ApubUser>, object: ObjectId<ApubUser>, id: Url) -> Follow {
        Follow {
            id,
            actor,
            object,
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
        self.actor.inner()
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
        println!("Got APUB Follow event");
        println!("{:?}", self);
        Ok(())
    }
}

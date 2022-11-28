use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::activity::FollowType;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{instance::InstanceHandle, object::user::ApubUser};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    id: Url,
    pub(crate) actor: ObjectId<ApubUser>,
    pub(crate) object: ObjectId<ApubUser>,
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

#[async_trait::async_trait]
impl ActivityHandler for Follow {
    type DataType = InstanceHandle;
    type Error = crate::error::Error;

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

    // Ignore clippy false positive: https://github.com/rust-lang/rust-clippy/issues/6446
    #[allow(clippy::await_holding_lock)]
    async fn receive(
        self,
        data: &Data<Self::DataType>,
        request_counter: &mut i32,
    ) -> Result<(), Self::Error> {
        //  let local_user = {
        //      let mut users = data.users.lock().unwrap();
        //      let local_user = users.first_mut().unwrap();
        //      local_user.followers.push(self.actor.inner().clone());
        //      local_user.clone()
        //  };
        //
        // Ok(())
        todo!()
    }
}

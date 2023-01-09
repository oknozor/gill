use std::marker::PhantomData;
use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::traits::ApubObject;
use url::Url;
use crate::instance::InstanceHandle;
use serde::{Serialize, Deserialize};

pub mod issue;
pub mod repository;
pub mod ssh_key;

#[derive(Deserialize, Serialize, Debug)]
pub struct ActivityPubId<T: ApubObject> {
    phantom_data: PhantomData<T>,
    inner: Url,
}

impl<T> From<ObjectId<T>> for ActivityPubId<T>
    where T: ApubObject + Send + 'static,
          for<'de2> <T as ApubObject>::ApubType: Deserialize<'de2> {
    fn from(id: ObjectId<T>) -> Self {
        ActivityPubId {
            phantom_data: PhantomData,
            inner: id.into_inner(),
        }
    }
}

pub trait DomainCommand {
    fn execute(&self, instnace: &InstanceHandle);
}

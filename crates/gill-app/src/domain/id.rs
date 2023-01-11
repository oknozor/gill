use activitypub_federation::core::object_id::ObjectId;
use activitypub_federation::traits::ApubObject;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use url::Url;

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone)]
pub struct ActivityPubId<T: ApubObject> {
    phantom_data: PhantomData<T>,
    inner: Url,
}

impl<T> From<ObjectId<T>> for ActivityPubId<T>
where
    T: ApubObject + Send + 'static,
    for<'de2> <T as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
    fn from(id: ObjectId<T>) -> Self {
        ActivityPubId {
            phantom_data: PhantomData,
            inner: id.into_inner(),
        }
    }
}

impl<T> From<Url> for ActivityPubId<T>
where
    T: ApubObject + Send + 'static,
    for<'de2> <T as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
    fn from(url: Url) -> Self {
        ActivityPubId {
            phantom_data: PhantomData,
            inner: url,
        }
    }
}

impl<T> From<ActivityPubId<T>> for ObjectId<T>
where
    T: ApubObject + Send + 'static,
    for<'de2> <T as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
    fn from(val: ActivityPubId<T>) -> Self {
        ObjectId::new(val.inner)
    }
}

impl<T> From<ActivityPubId<T>> for Url
where
    T: ApubObject + Send + 'static,
    for<'de2> <T as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
    fn from(val: ActivityPubId<T>) -> Self {
        val.inner
    }
}

impl<T> TryFrom<String> for ActivityPubId<T>
where
    T: ApubObject + Send + 'static,
    for<'de2> <T as ApubObject>::ApubType: Deserialize<'de2>,
{
    type Error = url::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(ActivityPubId {
            phantom_data: PhantomData,
            inner: Url::parse(&value)?,
        })
    }
}

impl<T> ToString for ActivityPubId<T>
where
    T: ApubObject + Send + 'static,
{
    fn to_string(&self) -> String {
        self.inner.to_string()
    }
}

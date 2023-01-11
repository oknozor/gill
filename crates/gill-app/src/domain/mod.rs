use crate::instance::InstanceHandle;







pub mod id;
pub mod issue;
pub mod pull_request;
pub mod repository;
pub mod user;

pub trait DomainCommand {
    fn execute(&self, instnace: &InstanceHandle);
}

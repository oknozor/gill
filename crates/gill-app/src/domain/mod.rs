use crate::instance::InstanceHandle;

pub mod issue;
pub mod repository;
pub mod ssh_key;

pub trait DomainCommand {
    fn execute(&self, instnace: &InstanceHandle);
}

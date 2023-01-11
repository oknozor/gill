use gill_db::repository::branch::Branch as BranchEntity;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Branch {
    pub name: String,
    pub repository_id: i32,
    pub is_default: bool,
}

impl From<BranchEntity> for Branch {
    fn from(branch: BranchEntity) -> Self {
        Self {
            name: branch.name,
            repository_id: branch.repository_id,
            is_default: branch.is_default,
        }
    }
}

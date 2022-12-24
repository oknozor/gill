use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 30,
            offset: 0,
        }
    }
}

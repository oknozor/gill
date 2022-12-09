use serde::Deserialize;
use std::num::NonZeroI64;

#[derive(Deserialize)]
pub struct Pagination {
    pub page: NonZeroI64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: NonZeroI64::new(1).unwrap(),
        }
    }
}

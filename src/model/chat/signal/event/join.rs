use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Join {
    pub user:       String,
    pub capacity:   u32,
}
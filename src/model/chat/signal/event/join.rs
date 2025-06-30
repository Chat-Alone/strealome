use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Join {
    pub user_id:    i32,
    pub capacity:   usize,
}
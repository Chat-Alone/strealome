use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Leave {
    pub user_id:    i32,
    pub capacity:   usize,
}
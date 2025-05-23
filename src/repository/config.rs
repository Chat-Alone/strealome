use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoConfig {
    pub url:        Option<&'static str>,
    pub username:   Option<&'static str>,
    pub password:   Option<&'static str>,
    pub database:   Option<&'static str>,
    pub schema:     Option<&'static str>,
}
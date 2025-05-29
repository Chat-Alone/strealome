use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum Author {
    User(i32),
    System,
}

impl Author {
    pub fn is_system(&self) -> bool {
        matches!(self, Author::System)
    }
}

impl From<i32> for Author {
    fn from(value: i32) -> Self {
        match value {
            -1 => Author::System,
            v => Author::User(v),
        }
    }
}

impl From<Author> for i32 {
    fn from(value: Author) -> Self {
        match value {
            Author::System => -1,
            Author::User(v) => v,
        }
    }
}

impl Serialize for Author {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        s.serialize_i32(i32::from(*self))
    }
}

impl<'de> Deserialize<'de> for Author {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let v = i32::deserialize(deserializer)?;
        Ok(Author::from(v))
    }
}
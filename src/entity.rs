use std::fmt::Display;

mod client_entity;

use sqlx::sqlite::SqliteRow;

pub(crate) use client_entity::ClientEntity;
pub(crate) use client_entity::ClientRepo;
pub(crate) use client_entity::InClientEntityDto;
pub(crate) use client_entity::OutClientEntityDtb;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) enum Role {
    #[serde(rename(deserialize = "admin", serialize = "admin"))]
    Admin,
    #[serde(rename(deserialize = "user", serialize = "user"))]
    User,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "{}", "admin"),
            Self::User => write!(f, "{}", "user"),
        }
    }
}

impl Default for Role {
    fn default() -> Self {
        Self::User
    }
}

impl From<&str> for Role {
    fn from(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "admin" => Self::Admin,
            "user" => Self::User,
            _ => Self::User,
        }
    }
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

pub(crate) trait FromSqliteRow {
    fn from_row(row: SqliteRow) -> Option<Self>
    where
        Self: Sized;
}

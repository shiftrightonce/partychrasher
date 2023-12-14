use std::fmt::Display;

use actix_web::{
    body::BoxBody, http::header::ContentType, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use sqlx::{Column, Row};

use crate::{
    entity::{FromSqliteRow, Role},
    helper::generate_id,
};

#[derive(Debug, Clone)]
pub(crate) struct ClientEntity {
    internal_id: Option<i64>,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) role: Role,
    api_secret: String,
}

impl Default for ClientEntity {
    fn default() -> Self {
        Self {
            internal_id: None,
            id: "".to_string(),
            name: generate_id(),
            role: Role::User,
            api_secret: "".to_string(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct NotAuthenticated;

impl Display for NotAuthenticated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "client is not authenticated")
    }
}

impl TryFrom<&HttpRequest> for ClientEntity {
    type Error = NotAuthenticated;
    fn try_from(value: &HttpRequest) -> Result<Self, Self::Error> {
        if let Some(client) = value.extensions_mut().get::<Self>() {
            Ok(client.clone())
        } else {
            Err(NotAuthenticated)
        }
    }
}

impl ClientEntity {
    pub(crate) fn new(name: &str, role: Option<Role>) -> Self {
        let mut client = Self {
            name: name.to_string(),
            ..Self::default()
        };

        if let Some(r) = role {
            client.role = r;
        }

        client
    }

    pub(crate) fn default_admin() -> Self {
        Self {
            role: Role::Admin,
            ..Self::default()
        }
    }

    pub(crate) fn default_user() -> Self {
        Self::default()
    }

    pub(crate) fn internal_id(&self) -> Option<i64> {
        self.internal_id
    }

    pub(crate) fn api_token(&self) -> String {
        if !self.id.is_empty() && !self.api_secret.is_empty() {
            format!("{}-{}", self.id, self.api_secret)
        } else {
            String::new()
        }
    }

    pub(crate) fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }

    pub(crate) fn is_user(&self) -> bool {
        match self.role {
            Role::Admin | Role::User => true,
        }
    }
}

impl FromSqliteRow for ClientEntity {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> Option<Self> {
        let mut entity = Self::default();
        for column in row.columns() {
            match column.name() {
                "internal_id" => entity.internal_id = row.get(column.name()),
                "id" => entity.id = row.get(column.name()),
                "name" => entity.name = row.get(column.name()),
                "api_secret" => entity.api_secret = row.get(column.name()),
                "role" => entity.role = row.get::<String, &str>(column.name()).into(),
                _ => panic!("new field added to the clients table"),
            }
        }

        if entity.internal_id.is_some() {
            Some(entity)
        } else {
            None
        }
    }
}

impl Responder for ClientEntity {
    type Body = BoxBody;
    fn respond_to(self, _req: &HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let body = OutClientEntityDto::from(self);
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(serde_json::to_string(&body).unwrap())
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InClientEntityDto {
    pub(crate) name: Option<String>,
    pub(crate) role: Option<Role>,
}

impl Default for InClientEntityDto {
    fn default() -> Self {
        Self {
            name: None,
            role: Some(Role::User),
        }
    }
}

impl From<ClientEntity> for InClientEntityDto {
    fn from(value: ClientEntity) -> Self {
        Self {
            name: if value.name.is_empty() {
                None
            } else {
                Some(value.name)
            },
            role: Some(value.role),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutClientEntityDto {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) role: Role,
}

impl From<ClientEntity> for OutClientEntityDto {
    fn from(value: ClientEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            role: value.role,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct OutApiTokenDto {
    pub(crate) id: String,
    pub(crate) token: String,
    pub(crate) role: Role,
}

impl From<ClientEntity> for OutApiTokenDto {
    fn from(value: ClientEntity) -> Self {
        Self {
            token: value.api_token(),
            id: value.id,
            role: value.role,
        }
    }
}

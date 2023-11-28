use std::sync::Arc;

use sqlx::{sqlite::SqliteRow, Column, Row};
use ulid::Ulid;

use crate::{
    db::{DbConnection, DbManager},
    helper::generate_id,
};

use super::{FromSqliteRow, Role};

#[derive(Debug)]
pub(crate) struct ClientEntity {
    internal_id: Option<i64>,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) role: Role,
    pub(crate) api_secret: String,
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

impl ClientEntity {
    pub(crate) fn new(name: &str, role: Option<Role>) -> Self {
        let mut client = Self::default();

        client.name = name.to_string();
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

    fn internal_id(&self) -> Option<i64> {
        self.internal_id
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
                _ => (),
            }
        }

        if entity.internal_id.is_some() {
            Some(entity)
        } else {
            None
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct InClientEntityDto {
    pub(crate) name: Option<String>,
    pub(crate) role: Option<Role>,
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
pub(crate) struct OutClientEntityDtb {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) role: Role,
}

impl From<ClientEntity> for OutClientEntityDtb {
    fn from(value: ClientEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            role: value.role,
        }
    }
}

pub(crate) struct ClientRepo {
    pool: DbConnection,
}

impl ClientRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) async fn setup_table(&self) -> Option<ClientEntity> {
        let sql = r#" CREATE TABLE IF NOT EXISTS "clients" (
	"internal_id"	INTEGER,
	"id"	TEXT NOT NULL,
	"name"	TEXT,
	"role"	TEXT,
	"api_secret"	TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);
  "#;

        if sqlx::query(sql).execute(self.pool()).await.is_ok() && !self.has_admin().await {
            return self.create(ClientEntity::default_admin().into()).await;
        }
        None
    }

    pub(crate) async fn create(&self, client: InClientEntityDto) -> Option<ClientEntity> {
        let api_secret = sha256::digest(Ulid::new().to_string());
        match sqlx::query(
            r#"INSERT INTO "clients"("id","name","api_secret","role") VALUES (?,?,?,?);"#,
        )
        .bind(generate_id())
        .bind(client.name.unwrap_or(generate_id()))
        .bind(api_secret)
        .bind(client.role.unwrap_or_default().to_string())
        .execute(self.pool())
        .await
        {
            Ok(result) => self.find_by_internal_id(result.last_insert_rowid()).await,
            _ => None,
        }
    }

    pub(crate) async fn update(&self, id: i64, client: InClientEntityDto) -> Option<ClientEntity> {
        todo!()
    }

    pub(crate) async fn reset_secret(&self, id: i64) -> Option<ClientEntity> {
        todo!()
    }

    pub(crate) async fn find_by_internal_id(&self, id: i64) -> Option<ClientEntity> {
        match sqlx::query(r#"SELECT * FROM "clients" WHERE "internal_id" = ? ;"#)
            .bind(id)
            .map(|row: SqliteRow| ClientEntity::from_row(row))
            .fetch_one(self.pool())
            .await
        {
            Ok(row) => row,
            _ => None,
        }
    }

    pub(crate) async fn count_by_role(&self, role: Role) -> i64 {
        match sqlx::query(r#"SELECT COUNT("internal_id") as "total" FROM "clients" WHERE role = ?"#)
            .bind(role.to_string())
            .map(|row: SqliteRow| row.get::<i64, &str>("total"))
            .fetch_one(self.pool())
            .await
        {
            Ok(total) => total,
            _ => 0,
        }
    }

    pub(crate) async fn has_admin(&self) -> bool {
        self.count_by_role(Role::Admin).await > 0
    }

    fn pool(&self) -> &DbConnection {
        &self.pool
    }
}

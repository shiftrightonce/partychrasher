use futures::stream::TryStreamExt;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use ulid::Ulid;

use crate::{
    db::{DbConnection, Paginator, PaginatorDirection},
    entity::{FromSqliteRow, Role},
    helper::generate_id,
};

use super::{ClientEntity, InClientEntityDto};

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
	"id"	TEXT NOT NULL UNIQUE,
	"name"	TEXT UNIQUE,
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
        let api_secret = self.generate_token();
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

    pub(crate) async fn update(&self, id: &str, client: InClientEntityDto) -> Option<ClientEntity> {
        let mut sql = "UPDATE \"clients\" SET ".to_string();

        if client.name.is_some() {
            sql = format!("{} \"name\" = ? ", sql);
        }

        if client.role.is_some() {
            sql = format!("{}, \"role\" = ? ", sql);
        }

        sql = format!("{} WHERE \"id\" = ?", sql);

        let mut query = sqlx::query(&sql);
        if let Some(name) = &client.name {
            query = query.bind(name);
        }
        if let Some(role) = &client.role {
            query = query.bind(role.to_string());
        }
        query = query.bind(id);

        if query.execute(self.pool()).await.is_ok() {
            return self.find_by_id(id).await;
        }
        None
    }

    pub(crate) async fn delete(&self, id: &str) -> Option<ClientEntity> {
        if let Some(client) = self.find_by_id(id).await {
            if sqlx::query(r#"DELETE FROM "clients" WHERE "id" = ? "#)
                .bind(id)
                .execute(self.pool())
                .await
                .is_ok()
            {
                return Some(client);
            }
        }

        None
    }

    pub(crate) async fn reset_secret(&self, id: &str) -> Option<ClientEntity> {
        let token = self.generate_token();
        if sqlx::query(r#"UPDATE "clients" SET "api_secret" = ? WHERE "id" = ? "#)
            .bind(token)
            .bind(id)
            .execute(self.pool())
            .await
            .is_ok()
        {
            return self.find_by_id(id).await;
        }
        None
    }

    pub(crate) async fn paginate(&self, paginator: &mut Paginator) -> Vec<ClientEntity> {
        let params = vec![paginator.last_value.clone(), paginator.limit.to_string()];
        let mut rows = Vec::new();
        let sql = match &paginator.direction {
            PaginatorDirection::Next => {
                "select * from clients where id > ?  order by id asc limit ?"
            }
            PaginatorDirection::Previous => {
                "select * from clients where id < ?  order by id asc limit ?"
            }
        };

        let mut query = sqlx::query(sql);

        for a_param in params {
            query = query.bind(a_param);
        }

        let mut result_stream = query
            .map(|row: SqliteRow| ClientEntity::from_row(row))
            .fetch(self.pool());

        while let Ok(Some(Some(result))) = result_stream.try_next().await {
            paginator.last_value = result.id.clone();
            rows.push(result)
        }

        rows
    }

    pub(crate) async fn find_by_internal_id(&self, id: i64) -> Option<ClientEntity> {
        if let Ok(row) = sqlx::query(r#"SELECT * FROM "clients" WHERE "internal_id" = ? ;"#)
            .bind(id)
            .map(|row: SqliteRow| ClientEntity::from_row(row))
            .fetch_one(self.pool())
            .await
        {
            return row;
        }
        None
    }

    pub(crate) async fn find_by_id(&self, id: &str) -> Option<ClientEntity> {
        if let Ok(row) = sqlx::query(r#"SELECT * FROM "clients" WHERE "id" = ?;"#)
            .bind(id)
            .map(|row: SqliteRow| ClientEntity::from_row(row))
            .fetch_one(self.pool())
            .await
        {
            return row;
        }
        None
    }

    pub(crate) async fn find_by_name(&self, name: &str) -> Option<ClientEntity> {
        if let Ok(row) = sqlx::query(r#"SELECT * FROM "clients" WHERE "name" = ?;"#)
            .bind(name)
            .map(|row: SqliteRow| ClientEntity::from_row(row))
            .fetch_one(self.pool())
            .await
        {
            return row;
        }
        None
    }

    pub(crate) async fn find_by_api_token(&self, token: &str) -> Option<ClientEntity> {
        let pieces = token.split('-').collect::<Vec<&str>>();
        if pieces.len() == 2 {
            if let Ok(row) =
                sqlx::query(r#"SELECT * FROM "clients" WHERE "id" = ? AND "api_secret" = ? "#)
                    .bind(pieces[0])
                    .bind(pieces[1])
                    .map(|row: SqliteRow| ClientEntity::from_row(row))
                    .fetch_one(self.pool())
                    .await
            {
                return row;
            }
        }

        None
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

    fn generate_token(&self) -> String {
        sha256::digest(Ulid::new().to_string())
    }
}

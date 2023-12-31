use futures::stream::TryStreamExt;
use rand::Rng;
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

    pub(crate) async fn setup_table(&self) -> bool {
        let sql = r#" CREATE TABLE IF NOT EXISTS "clients" (
	"internal_id"	INTEGER,
	"id"	TEXT NOT NULL UNIQUE,
	"name"	TEXT UNIQUE,
    "login_token" TEXT,
	"role"	TEXT,
	"api_secret"	TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);
"#;

        sqlx::query(sql).execute(self.pool()).await.is_ok()
    }

    pub(crate) async fn create(&self, client: InClientEntityDto) -> Option<ClientEntity> {
        self.do_insert(client, Some(self.generate_token()), None)
            .await
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

    pub(crate) async fn find_by_login_token(&self, token: &str) -> Option<ClientEntity> {
        if let Ok(row) = sqlx::query(r#"SELECT * FROM "clients" WHERE "login_token" = ?;"#)
            .bind(token)
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

    pub(crate) async fn do_insert(
        &self,
        client: InClientEntityDto,
        token: Option<String>,
        id: Option<String>,
    ) -> Option<ClientEntity> {
        let api_secret = token.unwrap_or(self.generate_token());
        let entity_id = id.unwrap_or(generate_id());
        if sqlx::query(
            r#"INSERT INTO "clients" ("id","name","api_secret","role") VALUES (?,?,?,?);"#,
        )
        .bind(&entity_id)
        .bind(client.name.unwrap_or(generate_id()))
        .bind(api_secret)
        .bind(client.role.unwrap_or_default().to_string())
        .execute(self.pool())
        .await
        .is_ok()
        {
            return self.set_login_token(&entity_id).await;
        }
        None
    }

    pub(crate) async fn create_default_admin(&self) -> Option<ClientEntity> {
        if let Ok(id) = std::env::var("PARTY_ADMIN_ID") {
            let result = self.find_by_id(&id).await;
            if result.is_some() {
                return result;
            }
        }

        let mut token = Some(self.generate_token());
        let mut id = Some(generate_id());
        if let Ok(existing) = std::env::var("PARTY_ADMIN_TOKEN") {
            if existing != "admin_token" {
                let mut pieces = existing.split('-');
                id = Some(pieces.next().unwrap().to_string());
                token = Some(pieces.last().unwrap().to_string());
            }
        }
        let client = self
            .do_insert(ClientEntity::default_admin().into(), token, id)
            .await;

        if client.is_some() {
            if let Ok(mut content) = tokio::fs::read_to_string("./.env").await {
                content = content
                    .replace("admin_token", &client.as_ref().unwrap().api_token())
                    .replace("admin_id", &client.as_ref().unwrap().id);
                _ = tokio::fs::write("./.env", content).await;
            }
        }

        client
    }

    pub(crate) async fn create_default_client(&self) -> Option<ClientEntity> {
        if let Ok(id) = std::env::var("PARTY_CLIENT_ID") {
            let result = self.find_by_id(&id).await;
            if result.is_some() {
                return result;
            }
        }

        let mut token = Some(self.generate_token());
        let mut id = Some(generate_id());
        if let Ok(existing) = std::env::var("PARTY_CLIENT_TOKEN") {
            if existing != "client_token" {
                let mut pieces = existing.split('-');
                id = Some(pieces.next().unwrap().to_string());
                token = Some(pieces.last().unwrap().to_string());
            }
        }
        let client = self
            .do_insert(ClientEntity::default_client().into(), token, id)
            .await;

        if client.is_some() {
            if let Ok(mut content) = tokio::fs::read_to_string("./.env").await {
                content = content
                    .replace("client_token", &client.as_ref().unwrap().api_token())
                    .replace("client_id", &client.as_ref().unwrap().id);
                _ = tokio::fs::write("./.env", content).await;
            }
        }

        client
    }

    pub(crate) async fn set_login_token(&self, client_id: &str) -> Option<ClientEntity> {
        loop {
            let token = self.generate_login_token();
            if let Ok(total) = sqlx::query(
                "SELECT COUNT(internal_id) as total FROM clients WHERE login_token = ? ",
            )
            .bind(&token)
            .map(|row: SqliteRow| row.get::<i64, &str>("total"))
            .fetch_one(self.pool())
            .await
            {
                if total == 0
                    && sqlx::query("UPDATE clients SET login_token = ? WHERE id = ?")
                        .bind(&token)
                        .bind(client_id)
                        .execute(self.pool())
                        .await
                        .is_ok()
                {
                    return self.find_by_id(client_id).await;
                }
            }
        }
    }

    pub(crate) async fn random_client_by_role(&self, role: Role) -> Option<ClientEntity> {
        if let Ok(client) = sqlx::query("SELECT * FROM clients WHERE role = ? ORDER BY RANDOM()")
            .bind(role.to_string())
            .map(ClientEntity::from_row)
            .fetch_one(self.pool())
            .await
        {
            return client;
        }

        None
    }

    fn pool(&self) -> &DbConnection {
        &self.pool
    }

    fn generate_token(&self) -> String {
        sha256::digest(Ulid::new().to_string())
    }

    fn generate_login_token(&self) -> String {
        let mut rng = rand::thread_rng();
        let alph_rang = 'A'..='Z';
        let num_rang = 0..=9;
        let token = format!(
            "{}-{}{}-{}{}{}",
            rng.gen_range(alph_rang.clone()),
            rng.gen_range(alph_rang.clone()),
            rng.gen_range(num_rang.clone()),
            rng.gen_range(num_rang.clone()),
            rng.gen_range(num_rang.clone()),
            rng.gen_range(alph_rang.clone()),
        );

        token.to_uppercase()
    }
}

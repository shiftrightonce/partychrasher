use futures::stream::TryStreamExt;
use sqlx::SqlitePool;
use sqlx::{sqlite::SqliteRow, types::chrono, Column, Pool, Row, Sqlite};
use ulid::Ulid;

use crate::entity::FromSqliteRow;
use crate::{
    entity::{ClientEntity, Role},
    helper::generate_id,
};

pub(crate) async fn setup_clients_table(pool: &SqlitePool) {
    let sql = r#"
   CREATE TABLE IF NOT EXISTS "clients" (
	"internal_id"	INTEGER,
	"id"	TEXT NOT NULL,
	"name"	TEXT,
	"role"	TEXT,
	"api_secret"	TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);
  "#;

    let _ = sqlx::query(sql).execute(pool).await;
}

pub(crate) async fn count_clients_by_role(pool: &SqlitePool, role: Role) -> i64 {
    match sqlx::query(r#"SELECT COUNT("internal_id") as "total" FROM "clients" WHERE role = ?"#)
        .bind(role.to_string())
        .map(|row: SqliteRow| row.get::<i64, &str>("total"))
        .fetch_one(pool)
        .await
    {
        Ok(total) => total,
        _ => 0,
    }
}

/// Insert new row in the "clients" table
pub(crate) async fn create_new_client(
    pool: &SqlitePool,
    client: ClientEntity,
) -> Option<ClientEntity> {
    let api_secret = sha256::digest(Ulid::new().to_string());

    match sqlx::query(r#"INSERT INTO "clients"("id","name","api_secret","role") VALUES (?,?,?,?);"#)
        .bind(generate_id())
        .bind(client.name)
        .bind(api_secret)
        .bind(client.role.to_string())
        .execute(pool)
        .await
    {
        Ok(result) => {
            if let Some(row) =
                find_record_by_internal_id(pool, result.last_insert_rowid(), "clients").await
            {
                return ClientEntity::from_row(row);
            }
            None
        }
        _ => None,
    }
}

pub(crate) async fn find_record_by_internal_id(
    pool: &SqlitePool,
    id: i64,
    table: &str,
) -> Option<SqliteRow> {
    let sql = format!(r#"SELECT * FROM "{}" WHERE "internal_id" = ? ;"#, table);

    sqlx::query(&sql).bind(id).fetch_one(pool).await.ok()
}

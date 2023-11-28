use std::{str::FromStr, sync::Arc, time::Duration};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};

use crate::{
    config::Config,
    entity::{ClientEntity, ClientRepo, FromSqliteRow, Role},
    sql::{
        count_clients_by_role, create_new_client, find_record_by_internal_id, setup_clients_table,
    },
};

pub(crate) type DbConnection = SqlitePool;

pub(crate) async fn setup_db_connection(config: &Config) -> Arc<DbManager> {
    let option = SqliteConnectOptions::from_str(&config.db_path())
        .unwrap()
        .foreign_keys(true)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(30));

    match SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(option)
        .await
    {
        Ok(pool) => Arc::new(DbManager::new(pool).await),
        Err(e) => {
            panic!("could not setup db: {:?}", e)
        }
    }
}

pub(crate) struct DbManager {
    pool: SqlitePool,
}

impl DbManager {
    async fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) fn client_repo(&self) -> ClientRepo {
        ClientRepo::new(self.pool.clone())
    }

    pub(crate) async fn setup_db(&self) {
        if let Some(client) = self.client_repo().setup_table().await {
            println!("-------- Default Admin created --------");
            println!("Admin ID          : {}", &client.id);
            println!("Admin API Secret  : {}", &client.api_secret);
        }
    }

    pub(crate) async fn insert_client(&self, client: ClientEntity) -> Option<ClientEntity> {
        create_new_client(&self.pool, client).await
    }

    pub(crate) async fn count_clients_by_role(&self, role: Role) -> i64 {
        count_clients_by_role(&self.pool, role).await
    }

    pub(crate) async fn has_default_admin(&self) -> bool {
        self.count_clients_by_role(Role::Admin).await > 0
    }
}

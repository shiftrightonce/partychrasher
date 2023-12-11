use crate::db::DbConnection;

pub(crate) struct TrackRepo {
    pool: DbConnection,
}

impl TrackRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) {
        let sql = r#"CREATE TABLE "tracks" (
	"internal_id"	INTEGER,
	"id"	TEXT,
	"title"	TEXT,
	"metadata"	TEXT,
	PRIMARY KEY("internal_id" AUTOINCREMENT)
);"#;

        _ = sqlx::query(sql).execute(self.pool()).await;
    }
}

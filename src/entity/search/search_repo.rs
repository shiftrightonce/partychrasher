use crate::{db::DbConnection, entity::FromSqliteRow, helper::generate_id};
use futures::stream::TryStreamExt;

use super::{InSearchHitEntityDto, SearchEntity, SearchHitEntity};

pub(crate) struct SearchRepo {
    pool: DbConnection,
}

impl SearchRepo {
    pub(crate) fn new(pool: DbConnection) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DbConnection {
        &self.pool
    }

    pub(crate) async fn setup_table(&self) {
        let search_main_table = r#"
        CREATE TABLE IF NOT EXISTS "search_terms" (
          "internal_id" INTEGER,
          "term" TEXT NOT NULL UNIQUE,
	        PRIMARY KEY("internal_id" AUTOINCREMENT)
        )
      "#;

        let hits_table = r#"
        CREATE TABLE IF NOT EXISTS "search_hits" (
          "internal_id" INTEGER,
          "id" TEXT NOT NULL UNIQUE,
          "entity" TEXT NOT NULL,
          "entity_id" TEXT NOT NULL,
          "metadata" TEXT,
	        PRIMARY KEY("internal_id" AUTOINCREMENT),
          UNIQUE("entity", "entity_id")
        )
      "#;
        let pivot_table = r#"
        CREATE TABLE IF NOT EXISTS "search_pivot" (
          "search_id" INTEGER,
          "hit_id" TEXT NOT NULL UNIQUE,
          UNIQUE("search_id", "hit_id")
        )
      "#;

        _ = sqlx::query(search_main_table).execute(self.pool()).await;
        if let Err(e) = sqlx::query(hits_table).execute(self.pool()).await {
            dbg!(e);
        }
        _ = sqlx::query(pivot_table).execute(self.pool()).await;
    }

    pub(crate) async fn create(&self, entity: InSearchHitEntityDto) {
        let mut search_terms = Vec::new();
        let params = "?,".repeat(entity.keywords.len());
        let params = params.trim_end_matches(',');

        for a_term in &entity.keywords {
            _ = sqlx::query(r#"INSERT INTO "search_terms" ("term") values (?)"#)
                .bind(a_term)
                .execute(self.pool())
                .await;
        }

        let sql = format!(
            "SELECT * FROM \"search_terms\" WHERE \"term\" IN ({}) ",
            params
        );
        let mut query = sqlx::query(&sql);
        for entry in &entity.keywords {
            query = query.bind(entry);
        }
        if let Ok(results) = query
            .map(SearchEntity::from_row)
            .fetch_all(self.pool())
            .await
        {
            results.into_iter().for_each(|r| {
                if let Some(record) = r {
                    search_terms.push(record);
                }
            })
        }

        // insert hit
        let hit_id = generate_id();
        if  sqlx::query(r#"INSERT OR IGNORE INTO "search_hits" ("id", "entity", "entity_id", "metadata") values(?, ?, ?, ?) "#).bind(&hit_id)
    .bind(&entity.entity)
    .bind(&entity.entity_id)
    .bind(&entity.metadata_to_string())
    .execute(self.pool())
    .await.is_ok() {
        for a_search in &search_terms {
          if let Err(e) = sqlx::query(r#"INSERT OR IGNORE INTO "search_pivot" ("search_id", "hit_id") values (?, ?) "#)
            .bind(a_search.internal_id)
            .bind(&hit_id)
            .execute(self.pool())
            .await {
               println!("search pivot: {:?}", e);
            }

        }
    }
    }

    pub(crate) async fn update(&self, entity: InSearchHitEntityDto) {
        // for now use the "create" method.
        // in the future, we may want to do something like flush the
        // existing entries for this entity
        self.create(entity).await;
    }

    pub(crate) async fn delete(&self, entity: InSearchHitEntityDto) {
        _ = sqlx::query("DELETE FROM search_hits WHERE entity = ? AND entity_id = ?")
            .bind(&entity.entity)
            .bind(&entity.entity_id)
            .execute(self.pool())
            .await
    }

    pub(crate) async fn search(&self, keyword: &str) -> Vec<SearchHitEntity> {
        let mut results = Vec::new();
        let mut result_stream = sqlx::query(
          r#"SELECT search_hits.internal_id, search_hits.id, search_hits.entity, search_hits.entity_id, search_hits.metadata FROM search_pivot 
          LEFT JOIN search_hits on search_hits.id = search_pivot.hit_id LEFT JOIN search_terms on search_terms.internal_id = search_pivot.search_id WHERE 
          search_terms.term like ? LIMIT 20"#)
        .bind(format!("%{}%", keyword))
        .map(SearchHitEntity::from_row)
        .fetch(self.pool());

        while let Ok(Some(Some(row))) = result_stream.try_next().await {
            results.push(row)
        }

        results
    }
}

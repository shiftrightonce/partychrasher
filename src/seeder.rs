use crate::{db::DbManager, entity::client::InClientEntityDto};

async fn seed_clients(db_manager: &DbManager, total: u64) {
    let client_repo = db_manager.client_repo();
    for _ in 0..total {
        client_repo
            .create(if rand::random() {
                InClientEntityDto {
                    name: None,
                    role: Some(crate::entity::Role::Admin),
                }
            } else {
                InClientEntityDto::default()
            })
            .await;
    }
}

pub(crate) async fn run_seeders(db_manager: &DbManager, total: u64) {
    seed_clients(db_manager, total).await
}

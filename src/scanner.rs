pub(crate) async fn scan(path: String) {
    println!("we are about to scan this path: {:?}", path);
    match tokio::fs::read_dir(path).await {
        Ok(mut entries) => {
            while let Ok(Some(an_entry)) = entries.next_entry().await {
                if an_entry.metadata().await.unwrap().is_dir() {
                    println!("read directory: {:?} ", an_entry.path())
                } else {
                    // println!("read file: {:?} ", an_entry.path())
                }
            }
        }
        Err(e) => println!("could not read director: {:?}", e),
    }
}

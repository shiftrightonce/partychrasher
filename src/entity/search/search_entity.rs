pub(crate) struct SearchHitEntity {
    entity: String,
    id: String,
    metadata: serde_json::Value,
}
pub(crate) struct SearchEntity {
    pub(crate) keyword: String,
    pub(crate) hits: String,
}

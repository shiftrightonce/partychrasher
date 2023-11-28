use ulid::Ulid;

pub(crate) fn generate_id() -> String {
    Ulid::new().to_string().to_ascii_lowercase()
}

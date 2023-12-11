use base64::Engine;
use ulid::Ulid;

pub(crate) fn generate_id() -> String {
    Ulid::new().to_string().to_ascii_lowercase()
}

pub(crate) fn base64_encode(subject: &str) -> String {
    base64::display::Base64Display::new(
        subject.as_bytes(),
        &base64::engine::general_purpose::STANDARD,
    )
    .to_string()
}

pub(crate) fn base64_decode(subject: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::STANDARD
        .decode(subject)
        .ok()
}

pub(crate) fn base64_decode_to_string(subject: &str) -> Option<String> {
    if let Some(result) = base64_decode(subject) {
        return String::from_utf8(result).ok();
    }
    None
}

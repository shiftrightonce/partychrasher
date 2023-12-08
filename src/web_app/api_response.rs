#![allow(dead_code)]

#[derive(Debug, serde::Serialize)]
pub(crate) struct ApiResponse<D: serde::Serialize> {
    data: Option<D>,
    success: bool,
    message: Option<String>,
}

impl Default for ApiResponse<serde_json::Value> {
    fn default() -> Self {
        Self {
            data: None,
            success: true,
            message: None,
        }
    }
}

impl<D: serde::Serialize> ApiResponse<D> {
    pub(crate) fn success(data: D) -> Self {
        Self {
            data: Some(data),
            success: true,
            message: None,
        }
    }

    pub(crate) fn error(message: &str) -> Self {
        Self {
            data: None,
            success: false,
            message: Some(message.to_string()),
        }
    }

    pub(crate) fn to_json(self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

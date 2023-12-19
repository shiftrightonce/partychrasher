#![allow(dead_code)]

use actix_web::HttpResponse;

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

    pub(crate) fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }

    pub(crate) fn success_response(data: D) -> HttpResponse {
        HttpResponse::Ok().json(Self::success(data))
    }

    pub(crate) fn not_found_response(message: Option<&str>) -> HttpResponse {
        HttpResponse::NotFound().json(Self::error(message.unwrap_or("resource not found")))
    }

    pub(crate) fn into_response(data: Option<D>) -> HttpResponse {
        if let Some(d) = data {
            Self::success_response(d)
        } else {
            Self::not_found_response(None)
        }
    }
}

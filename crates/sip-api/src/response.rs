use axum::Json;
use serde::Serialize;

pub fn success<T: Serialize>(data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "data": data,
    }))
}

pub fn success_paginated<T: Serialize>(data: T, total: i64, offset: i64, limit: i64) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "data": data,
        "meta": {
            "total": total,
            "offset": offset,
            "limit": limit,
        }
    }))
}

pub fn error(code: &str, message: &str) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "error",
        "error": {
            "code": code,
            "message": message,
        }
    }))
}

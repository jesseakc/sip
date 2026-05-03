use axum::Json;
use serde::Serialize;

pub fn success<T: Serialize>(data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "data": data,
    }))
}

pub fn success_paginated<T: Serialize>(
    data: T,
    total: i64,
    offset: i64,
    limit: i64,
) -> Json<serde_json::Value> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestItem {
        id: i32,
        name: String,
    }

    #[test]
    fn test_success_shape() {
        let json = success(TestItem { id: 1, name: "test".into() });
        assert_eq!(json["status"], "ok");
        assert_eq!(json["data"]["id"], 1);
    }

    #[test]
    fn test_success_paginated_shape() {
        let items = vec![TestItem { id: 1, name: "a".into() }, TestItem { id: 2, name: "b".into() }];
        let json = success_paginated(items, 100, 0, 20);
        assert_eq!(json["status"], "ok");
        assert_eq!(json["data"][0]["id"], 1);
        assert_eq!(json["meta"]["total"], 100);
        assert_eq!(json["meta"]["offset"], 0);
        assert_eq!(json["meta"]["limit"], 20);
    }

    #[test]
    fn test_error_shape() {
        let json = error("AUTH_FAILED", "Invalid credentials");
        assert_eq!(json["status"], "error");
        assert_eq!(json["error"]["code"], "AUTH_FAILED");
    }
}

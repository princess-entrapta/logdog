use serde_json::json;

#[test]
fn test_add() {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("http://localhost:8000/api/health")
        .send()
        .unwrap();

    let status = resp.status();

    assert_eq!(status, 200);

    let resp = client
        .get("http://localhost:8000/api/metric")
        .send()
        .unwrap();

    let status = resp.status();
    let body: serde_json::Value = serde_json::from_str(resp.text().unwrap().as_str()).unwrap();

    assert_eq!(status, 200);
    assert!(body.is_array());

    let send_body = json!({
    "columns": [{"name": "test_col", "query": "logdata", "metric_agg": "max"}],
    "filter": {"name": "test_view", "query": "true"}})
    .to_string();
    let resp = client
        .post("http://localhost:8000/api/view")
        .header("Content-Type", "application/json")
        .body(send_body)
        .send()
        .unwrap();
    let status = resp.status();
    let body = resp.text().unwrap();

    assert_eq!(status, 201);
    let body: serde_json::Value = serde_json::from_str(body.as_str()).unwrap();

    assert!(body.is_object());

    let send_body = json!({"start": chrono::DateTime::from_timestamp(1711302824, 0),
    "end": chrono::DateTime::from_timestamp(1711302888, 0),
    "metric_name": "test_col",
    "view_name": "test_view"})
    .to_string();
    let resp = client
        .post("http://localhost:8000/api/get/metric")
        .header("Content-Type", "application/json")
        .body(send_body)
        .send()
        .unwrap();
    let status = resp.status();
    let body = resp.text().unwrap();

    assert_eq!(status, 200);
    let body: serde_json::Value = serde_json::from_str(body.as_str()).unwrap();

    assert!(body.is_array());

    let resp = client
        .delete("http://localhost:8000/api/view/test_view")
        .send()
        .unwrap();

    let status = resp.status();

    assert_eq!(status, 200);
}

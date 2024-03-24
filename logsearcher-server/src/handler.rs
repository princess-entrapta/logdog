use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use serde_json::json;
use std::iter::zip;

use crate::errors::AppError;
use crate::{
    model::{LogQuery, MetricQuery, ViewQuery},
    AppState,
};

pub fn app() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health_checker_handler))
        .route("/api/density", post(density_handler))
        .route("/api/logs", post(logs_handler))
        .route("/api/listviews", get(list_views))
        .route("/api/view", post(create_view_handler))
        .route("/api/view/:view_name", delete(delete_view_handler))
        .route("/api/metric", get(list_metrics))
        .route("/api/get/metric", post(post_get_metric))
}

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Log viewer utility";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    axum::Json(json_response)
}

pub async fn density_handler(
    State(data): State<AppState>,
    density_query: Json<LogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let logs = data
        .db
        .get_density(
            density_query.start.naive_utc(),
            density_query.end.naive_utc(),
            &density_query.table,
        )
        .await?;
    Ok(axum::Json(logs))
}

pub async fn logs_handler(
    State(data): State<AppState>,
    log_query: Json<LogQuery>,
) -> Result<impl IntoResponse, AppError> {
    Ok(axum::Json(
        data.db
            .get_logs(
                log_query.start.naive_utc(),
                log_query.end.naive_utc(),
                log_query.offset,
                log_query.table.to_owned(),
            )
            .await?,
    ))
}

pub async fn delete_view_handler(
    State(data): State<AppState>,
    Path(view_name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    data.db.delete_view(view_name).await?;
    Ok(StatusCode::OK)
}

pub async fn create_view_handler(
    State(data): State<AppState>,
    log_query: Json<ViewQuery>,
) -> Result<impl IntoResponse, AppError> {
    let filter_name = log_query.filter.name.to_owned();
    let filter_query = log_query.filter.query.to_owned();
    let (names, queries) = log_query
        .0
        .columns
        .into_iter()
        .map(|c| (c.name, (c.query, c.metric_agg)))
        .unzip();
    let filter_name = if filter_name.len() == 0 {
        "logs".to_owned()
    } else {
        filter_name
    };
    data.db
        .upsert_columns_and_filters(&names, &queries, &filter_name, &filter_query)
        .await?;

    data.db.create_mat_views(filter_name, filter_query).await?;
    Ok((StatusCode::CREATED, "{}".to_string()))
}

pub async fn list_metrics(State(data): State<AppState>) -> Result<impl IntoResponse, AppError> {
    Ok(axum::Json(data.db.get_col_names().await?))
}

pub async fn post_get_metric(
    State(data): State<AppState>,
    Json(metric_query): Json<MetricQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (col_query, metric_agg) = data
        .db
        .get_metric_query_agg(metric_query.metric_name)
        .await?;
    let filter_query = data.db.get_filter(metric_query.view_name).await?;
    Ok(axum::Json(
        data.db
            .get_filters(
                metric_query.start,
                metric_query.end,
                metric_agg,
                col_query,
                filter_query,
            )
            .await?,
    ))
}

pub async fn list_views(State(data): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let rows = data.db.list_filters().await?;
    Ok(axum::Json(
        rows.into_iter()
            .map(|(name, aggs, metrics)| {
                let mut val = serde_json::Map::new();
                val.insert("name".to_owned(), name.into());
                val.insert(
                    "cols".to_owned(),
                    zip(aggs, metrics)
                        .into_iter()
                        .map(|(agg, metric)| json!({"metric": metric, "agg":agg}))
                        .collect(),
                );
                val.into()
            })
            .collect::<Vec<serde_json::Value>>(),
    ))
}

#[cfg(test)]
mod tests {

    use crate::repository::Repository;

    use super::{app, AppState};
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[sqlx::test]
    async fn test_health(pool: sqlx::PgPool) {
        let app = app().with_state(AppState {
            db: Repository { pool: pool },
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .method("GET")
                    .body("".to_owned())
                    .unwrap(),
            )
            .await;
        assert_eq!(resp.expect("Should not fail").status(), 200);
    }

    #[sqlx::test]
    async fn test_list_metric(pool: sqlx::PgPool) {
        let app = app().with_state(AppState {
            db: Repository { pool: pool },
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/metric")
                    .method("GET")
                    .body("".to_owned())
                    .unwrap(),
            )
            .await
            .expect("Request should not fail");
        assert_eq!(resp.status(), 200);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"[\"Data\"]");
    }
}

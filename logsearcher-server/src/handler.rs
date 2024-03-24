use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use std::iter::zip;

use crate::errors::AppError;
use crate::{
    model::{LogQuery, MetricQuery, ViewQuery},
    AppState,
};

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

pub async fn view_handler(
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

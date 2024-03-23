use std::sync::Arc;

use axum::{routing::get, routing::post, Router};

use crate::{
    handler::{
        density_handler, health_checker_handler, list_metrics, list_views, logs_handler,
        post_get_metric, view_handler,
    },
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/density", post(density_handler))
        .route("/api/logs", post(logs_handler))
        .route("/api/listviews", get(list_views))
        .route("/api/createview", post(view_handler))
        .route("/api/metric", get(list_metrics))
        .route("/api/get/metric", post(post_get_metric))
        .with_state(app_state)
}

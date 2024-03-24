mod config;
mod errors;
mod handler;
mod model;
mod repository;

use crate::config::Config;
use crate::repository::Repository;
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{delete, get, post},
    Router,
};
use dotenv::dotenv;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    db: Repository,
}
use crate::handler::{
    create_view_handler, delete_view_handler, density_handler, health_checker_handler,
    list_metrics, list_views, logs_handler, post_get_metric,
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt().with_target(false).json().init();
    let config = Config::new();
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/health", get(health_checker_handler))
        .route("/api/density", post(density_handler))
        .route("/api/logs", post(logs_handler))
        .route("/api/listviews", get(list_views))
        .route("/api/view", post(create_view_handler))
        .route("/api/view/:view_name", delete(delete_view_handler))
        .route("/api/metric", get(list_metrics))
        .route("/api/get/metric", post(post_get_metric))
        .with_state(AppState {
            db: Repository::connect(config.pg_url.as_str()).await,
        })
        .layer(cors);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

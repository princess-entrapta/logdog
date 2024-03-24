mod config;
mod errors;
mod handler;
mod model;
mod repository;

use crate::config::Config;
use crate::repository::Repository;
use axum::extract::{MatchedPath, Request};
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::get,
    routing::post,
    Router,
};
use dotenv::dotenv;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    db: Repository,
}
use crate::handler::{
    density_handler, health_checker_handler, list_metrics, list_views, logs_handler,
    post_get_metric, view_handler,
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let _ = tracing_subscriber::fmt().json().finish();
    let config = Config::new();
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/density", post(density_handler))
        .route("/api/logs", post(logs_handler))
        .route("/api/listviews", get(list_views))
        .route("/api/createview", post(view_handler))
        .route("/api/metric", get(list_metrics))
        .route("/api/get/metric", post(post_get_metric))
        .with_state(AppState {
            db: Repository::connect(config.pg_url.as_str()).await,
        })
        .layer(
            TraceLayer::new_for_http()
                // Create our own span for the request and include the matched path. The matched
                // path is useful for figuring out which handler the request was routed to.
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();

                    // axum automatically adds this extension.
                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());

                    tracing::debug_span!("request", %method, %uri, matched_path)
                })
                // By default `TraceLayer` will log 5xx responses but we're doing our specific
                // logging of errors so disable that
                .on_failure(()),
        )
        .layer(cors);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod config;
mod errors;
mod handler;
mod model;
mod repository;

use crate::config::Config;
use crate::repository::Repository;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use dotenv::dotenv;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: Repository,
}
use crate::handler::app;

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

    let app = app()
        .with_state(AppState {
            db: Repository::connect(config.pg_url.as_str()).await,
        })
        .layer(cors);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

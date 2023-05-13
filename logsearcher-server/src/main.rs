mod handler;
mod model;
mod route;

use std::sync::Arc;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use deadpool_postgres::*;
use tokio_postgres::NoTls;

use dotenv::dotenv;
use route::create_router;
use tower_http::cors::CorsLayer;

pub struct AppState {
    db: deadpool_postgres::Pool,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_owned());
    cfg.user = Some("postgres".to_owned());
    cfg.dbname = Some("postgres".to_owned());
    cfg.password = Some("test".to_owned());
    let pool = cfg.create_pool(None, NoTls).unwrap();

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:8000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router(Arc::new(AppState { db: pool.clone() })).layer(cors);

    println!("Server started successfully");

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

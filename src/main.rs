use axum::{
    routing::{get, post},
    Router,
};
use axum_sessions::{async_session::MemoryStore, SessionLayer};
use diesel::{pg::PgConnection, r2d2::ConnectionManager};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::CONFIG;
use crate::database::*;
use crate::handlers::auth::login;
use crate::handlers::health::get_health_endpoint;
use crate::handlers::user::{create_user_endpoint, get_user_endpoint, get_users_endpoint};

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate validator_derive;

pub mod auth;
pub mod config;
pub mod database;
pub mod errors;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod pagination;
pub mod schema;
pub mod validate;

const SESSION_COOKIE_NAME: &str = "rust_axum_session";

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Pulling from computer env instead of env file
    tracing_subscriber::fmt::init();

    let config = CONFIG.clone();

    // Session setup
    let session_layer = SessionLayer::new(MemoryStore::new(), config.session_key.as_bytes())
        .with_cookie_name(SESSION_COOKIE_NAME);

    // CORS setup
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        // allow requests from any origin
        .allow_origin(Any);

    // DB connection setup
    let manager = ConnectionManager::<PgConnection>::new(config.database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool");
    let db_conn = pool;

    let auth_routes = Router::new()
        .route("/users", post(create_user_endpoint).get(get_users_endpoint))
        .route("/users/:id", get(get_user_endpoint))
        .route_layer(session_layer.clone());

    let open_routes = Router::new().route("/", get(get_health_endpoint));

    let login_routes = Router::new()
        .route("/auth/login", post(login))
        .route_layer(session_layer);

    let app = Router::new()
        .merge(auth_routes)
        .merge(open_routes)
        .merge(login_routes)
        .with_state(db_conn)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // TODO: make this match config value
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

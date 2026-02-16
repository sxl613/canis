mod media;
mod templates;
use askama::Template;
use axum::extract::{Path as AxumPath, Query, State};
use axum::middleware;
use axum::response::IntoResponse;
use axum::{Router, http::StatusCode, response::Html, routing::get};
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf};
use templates::{IndexTemplate, WatchTemplate};
use tower_http::services::ServeDir;

#[derive(Deserialize)]
struct ListParams {
    #[serde(default = "default_page")]
    page: u32,

    #[serde(default = "default_page_size")]
    page_size: u32,

    #[serde(default)]
    sort: SortField,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    50
}

#[derive(Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
enum SortField {
    Name,
    Size,
    LastModified,
    Created,
}

impl SortField {
    pub fn is_name(&self) -> bool {
        matches!(self, SortField::Name)
    }
    pub fn is_size(&self) -> bool {
        matches!(self, SortField::Size)
    }
    pub fn is_last_modified(&self) -> bool {
        matches!(self, SortField::LastModified)
    }
}

impl Default for SortField {
    fn default() -> SortField {
        SortField::LastModified
    }
}

#[derive(Clone)]
struct AppState {
    media_path: PathBuf,
    auth_cookie_name: Option<String>,
    auth_cookie_value: Option<String>,
}

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Read config from environment
    let addr = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let bind_addr = format!("{}:{}", addr, port);
    let media_path = std::env::var("MEDIA_PATH").unwrap_or_else(|_| "./media".to_string());
    let auth_cookie_value = std::env::var("AUTH_COOKIE_VALUE").ok();
    let auth_cookie_name = std::env::var("AUTH_COOKIE_NAME").ok();

    // Create application state
    let state = AppState {
        media_path: PathBuf::from(&media_path),
        auth_cookie_value: auth_cookie_value,
        auth_cookie_name: auth_cookie_name,
    };

    // Build our application with routes
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/v/{filename}", get(watch_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        // Serve static files from media directory
        .nest_service("/media", ServeDir::new(media_path))
        .with_state(state);

    // Parse the bind address
    let addr: SocketAddr = bind_addr.parse().expect("Invalid BIND_ADDRESS");

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server error");
}

async fn index_handler(
    State(state): State<AppState>,
    Query(query): Query<ListParams>,
) -> Result<Html<String>, (StatusCode, String)> {
    let files = media::list_media_files(&state.media_path, &query);
    IndexTemplate { files, query }
        .render()
        .map(Html)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn watch_handler(
    AxumPath(filename): AxumPath<String>,
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    let video = match media::find_media_file(&state.media_path, &filename) {
        Some(f) => f,
        None => return Err((StatusCode::NOT_FOUND, "File not found".to_string())),
    };
    WatchTemplate { video }
        .render()
        .map(Html)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn auth_middleware(
    State(state): State<AppState>,
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    match (&state.auth_cookie_name, &state.auth_cookie_value) {
        (Some(name), Some(pwd)) if !name.trim().is_empty() && !pwd.trim().is_empty() => {
            if let Some(cookie_header) = request.headers().get("cookie") {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for cookie in cookie_str.split(";") {
                        let cookie = cookie.trim();
                        if let Some((key, value)) = cookie.split_once("=") {
                            if key == name && value == pwd {
                                return next.run(request).await;
                            }
                        }
                    }
                }
            }
            return StatusCode::UNAUTHORIZED.into_response();
        }
        _ => {
            return next.run(request).await;
        }
    }
}

mod media;
mod templates;
use askama::Template;
use axum::extract::{Query, State};
use axum::{Router, http::StatusCode, response::Html, routing::get};
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf};
use templates::IndexTemplate;
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

    // Create application state
    let state = AppState {
        media_path: PathBuf::from(&media_path),
    };

    // Build our application with routes
    let app = Router::new()
        .route("/", get(index_handler))
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

// async fn play_video(Path(filename): Path<String>, State(state): State<AppState>) -> Response {
//     // Construct full path securely
//     let file_path = state.media_dir.join(&filename);

//     // Security check: Ensure the file is inside the media_dir (prevents ../ attacks)
//     if !file_path.starts_with(&state.media_dir) {
//         return StatusCode::FORBIDDEN.into_response();
//     }

//     // Serve the file with automatic mime guessing and Range support (seeking)
//     match ServeFile::new(file_path).try_into_response() {
//         Ok(res) => res.into_response(),
//         Err(err) => StatusCode::NOT_FOUND.into_response(),
//     }
// }

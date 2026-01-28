mod media;
mod templates;
use askama::Template;
use axum::{Router, extract::State, http::StatusCode, response::Html, routing::get};
use templates::IndexTemplate;
use std::{net::SocketAddr, path::PathBuf};
use tower_http::services::ServeDir;

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
) -> Result<Html<String>, (StatusCode, String)> {
    let files = media::list_media_files(&state.media_path);
    IndexTemplate { files }
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

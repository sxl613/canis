use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Read config from environment
    let bind_addr = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let media_path = std::env::var("MEDIA_PATH").unwrap_or_else(|_| "./media".to_string());

    // Build our application with routes
    let app = Router::new()
        .route("/", get(|| async { "0K" }))
        // Serve static files from media directory
        .nest_service("/media", ServeDir::new(media_path));

    // Parse the bind address
    let addr: SocketAddr = bind_addr.parse().expect("Invalid BIND_ADDRESS");

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app).await.expect("Server error");
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

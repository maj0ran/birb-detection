use axum::{http::StatusCode, routing::get, Json, Router};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use trmnl::{
    render_html_to_png, DeviceInfo, DisplayResponse, LogEntry, LogResponse, RenderConfig,
    SetupResponse,
};

/// Application state
struct AppState {
    /// Base URL for images
    base_url: String,
    /// Directory to store images
    image_dir: PathBuf,
    /// Last generated filename
    last_filename: RwLock<Option<String>>,
    /// Render configuration
    render_config: RenderConfig,
}

/// GET /api/setup - Device registration
async fn setup(device: DeviceInfo) -> Json<SetupResponse> {
    println!("Device {} requesting setup", device.mac_address);

    Json(SetupResponse::new(
        format!("trmnl-{}", device.short_id()),
        "https://example.com/welcome.png",
        "Welcome to BYOS!",
    ))
}

/// GET /api/display - Main display endpoint
async fn display(device: DeviceInfo) -> Json<DisplayResponse> {
    println!(
        "Device {} requesting display (battery: {:?}%)",
        device.mac_address,
        device.battery_percentage()
    );

    // In a real implementation, you would:
    // 1. Generate HTML content based on your data
    // 2. Render to PNG
    // 3. Save the PNG and return its URL

    // Use timestamp in filename for cache busting
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let response = Json(
        DisplayResponse::new(
            "http://192.168.178.54:3000/assets/snail.png",
            format!("{}", timestamp),
        )
        .with_refresh_rate(60),
    );

    println!("response: {:?}", response);

    response
}

/// POST /api/log - Device telemetry
async fn log(device: DeviceInfo, Json(entry): Json<LogEntry>) -> Json<LogResponse> {
    println!(
        "Log from {}: {:?} (battery: {:?}V)",
        device.mac_address,
        entry.log_message,
        entry
            .device_status_stamp
            .as_ref()
            .and_then(|s| s.battery_voltage)
    );

    Json(LogResponse::ok())
}

pub async fn run() {
    println!("Starting TRMNL BYOS server on http://localhost:3000");
    println!();
    println!("Endpoints:");
    println!("  GET  /api/setup   - Device registration");
    println!("  GET  /api/display - Get display image");
    println!("  POST /api/log     - Device telemetry");
    println!();
    println!("Test with:");
    println!("  curl -H 'ID: test-device' http://localhost:3000/api/display");

    let app = Router::new()
        .route("/api/setup", get(setup))
        .route("/api/display", get(display))
        .route("/api/log", axum::routing::post(log))
        .nest_service("/assets", ServeDir::new("assets"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub async fn generate_image() -> Result<(), (StatusCode, String)> {
    let state = Arc::new(AppState {
        base_url: "http://localhost:3000".to_string(),
        image_dir: PathBuf::from("/tmp/trmnl-images"),
        last_filename: RwLock::new(None),
        render_config: RenderConfig::default(),
    });

    let html = fs::read_to_string("encyclopedia/Turdus merula/index.html")
        .unwrap_or(("Failed to read HTML file".to_string()));

    // Render to PNG
    let png_data = render_html_to_png(&html, &state.render_config)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Save to file
    let filename = "foo.png".to_string();
    let image_path = state.image_dir.join(&filename);

    tokio::fs::create_dir_all(&state.image_dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tokio::fs::write(&image_path, &png_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Update last filename
    *state.last_filename.write().await = Some(filename.clone());

    let image_url = format!("{}/images/{}", state.base_url, filename);

    Ok(())
}

use my_fintech_app::{
    config, 
    routes::auth_routes::{auth_routes, AppState},
    handlers
};
use axum::routing::{get, post};
use axum::Router;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    tracing::info!("🚀 Starting Fintech Application...");

    // Load configuration
    let config = config::Config::from_env()?;
    tracing::info!("✅ Configuration loaded");

    // Connect to database
    let pool = config::create_db_pool(&config.database_url).await?;
    tracing::info!("✅ Database connected");

    // Create app state
    let state = AppState {
        pool,
        jwt_secret: config.jwt_secret.clone(),
    };

    // 1. API Routes (already have state attached)
    let api_routes = auth_routes(state.clone());

    // 2. Web Routes (need state attached)
    let web_routes = Router::new()
        .route("/", get(handlers::web::login_page))
        .route("/login", get(handlers::web::login_page))
        .route("/login", post(handlers::web::login_submit))
        .route("/register", get(handlers::web::register_page))
        .route("/register", post(handlers::web::register_submit))
        .route("/dashboard", get(handlers::web::dashboard_page))
        .route("/dashboard/transactions", get(handlers::web::transactions_page))
        .route("/dashboard/deposit", get(handlers::web::deposit_page))
        .route("/dashboard/deposit", post(handlers::web::deposit_submit))
        .route("/dashboard/withdraw", get(handlers::web::withdraw_page))
        .route("/dashboard/withdraw", post(handlers::web::withdraw_submit))
        .route("/dashboard/transfer", get(handlers::web::transfer_page))
        .route("/dashboard/transfer", post(handlers::web::transfer_submit))
        .route("/logout", post(handlers::web::logout))
        .with_state(state);

    // 3. Merge everything - API routes under /api, web routes at root
    let app = Router::new()
        .nest("/api", api_routes)  // API routes: /api/register, /api/login, etc.
        .merge(web_routes)          // Web routes: /login, /register, /dashboard
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(TraceLayer::new_for_http());

    // Start the server
    let addr = config.server_address();
    tracing::info!("🌐 Server listening on http://{}", addr);
    tracing::info!("📝 Available endpoints:");
    tracing::info!("   API:");
    tracing::info!("     POST http://{}/api/register", addr);
    tracing::info!("     POST http://{}/api/login", addr);
    tracing::info!("     GET  http://{}/api/me (protected)", addr);
    tracing::info!("     GET  http://{}/api/wallet (protected)", addr);
    tracing::info!("   Web:");
    tracing::info!("     GET  http://{}/ (login page)", addr);
    tracing::info!("     GET  http://{}/dashboard (dashboard)", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

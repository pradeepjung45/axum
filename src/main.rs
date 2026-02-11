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
        jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        rate_limiter: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
    };

    // Create web routes with state
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
        .with_state(state.clone());

    // Build our application with routes
    let app = Router::new()
        .nest("/api", auth_routes(state.clone()))
        .merge(web_routes)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            my_fintech_app::middleware::rate_limit::rate_limit_middleware,
        ))
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
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}

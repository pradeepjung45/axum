use axum::{routing::{get, post}, Router};
use crate::handlers::{auth, user, wallet};
use sqlx::PgPool;

// ============================================================================
// APP STATE
// ============================================================================
// Combined state that holds both database pool and config
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

// ============================================================================
// AUTH ROUTES  
// ============================================================================

/// Create the authentication routes
pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        // Public routes (no authentication required)
        .route("/register", post(auth::register_handler))
        .route("/login", post(auth::login_handler))
        // Protected routes (authentication required)
        .route("/me", get(user::get_me))
        .route("/wallet", get(wallet::get_wallet))
        .route("/wallet/deposit", post(wallet::deposit))
        .route("/wallet/withdraw", post(wallet::withdraw))
        .route("/wallet/transfer", post(wallet::transfer))
        .route("/transactions", get(wallet::get_history))
        .with_state(state)
}

// ============================================================================
// UNDERSTANDING AXUM ROUTING
// ============================================================================
/*
Axum routing is declarative - you describe what you want.

Basic routing:
```rust
Router::new()
    .route("/path", get(handler))     // GET /path
    .route("/path", post(handler))    // POST /path
    .route("/path", put(handler))     // PUT /path
    .route("/path", delete(handler))  // DELETE /path
```

Nested routes:
```rust
let app = Router::new()
    .nest("/auth", auth_routes())     // /auth/register, /auth/login
    .nest("/wallet", wallet_routes()) // /wallet/balance, /wallet/deposit
```

State sharing:
```rust
Router::new()
    .route("/endpoint", post(handler))
    .with_state(pool)    // All handlers can access pool via State<PgPool>
    .with_state(config)  // All handlers can access config via State<Config>
```

The final URL will be:
- Base: http://localhost:3000
- Nested: /auth
- Route: /register
- Full: http://localhost:3000/auth/register
*/

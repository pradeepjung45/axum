use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use time::Duration;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use crate::middleware::auth::AuthUser;
use crate::routes::auth_routes::AppState;
use crate::domain::models::{UserResponse, WalletResponse, TransactionResponse};
use crate::repository::user_repo;
use crate::services::wallet_service;

// ============================================================================
// TEMPLATES
// ============================================================================

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    user: UserResponse,
    wallet: WalletResponse,
    transactions: Vec<TransactionResponse>,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// Serve the login page
pub async fn login_page() -> impl IntoResponse {
    LoginTemplate
}

/// Serve the register page
pub async fn register_page() -> impl IntoResponse {
    RegisterTemplate
}

/// Serve the dashboard (protected)
pub async fn dashboard_page(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
) ->  Result<impl IntoResponse, crate::error::AppError> {
    // 1. Get User
    let user = user_repo::find_user_by_id(&state.pool, user_id).await
        .map(UserResponse::from)?;

    // 2. Get Wallet
    let wallet = user_repo::get_wallet_by_user_id(&state.pool, user_id).await
        .map(WalletResponse::from)?;

    // 3. Get Recent Transactions (Limit 5 for overview)
    // Note: strict typing might need us to limit in query or slice here
    let transactions_raw = wallet_service::get_history(&state.pool, user_id).await?;
    let transactions: Vec<TransactionResponse> = transactions_raw
        .into_iter()
        .take(5)
        .map(TransactionResponse::from)
        .collect();

    let template = DashboardTemplate {
        user,
        wallet,
        transactions,
    };

    Ok(template)
}

#[derive(Template)]
#[template(path = "deposit.html")]
struct DepositTemplate;

/// Serve the deposit page
pub async fn deposit_page() -> impl IntoResponse {
    DepositTemplate
}

/// Handle deposit form submission
pub async fn deposit_submit(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Form(req): Form<crate::domain::models::DepositRequest>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    use axum::response::AppendHeaders;

    // Call the service
    wallet_service::deposit(&state.pool, user_id, req.amount).await?;

    // Return success message and redirect
    Ok((
        AppendHeaders([("HX-Redirect", "/dashboard".to_string())]),
        "Deposit successful! Redirecting..."
    ))
}

#[derive(Template)]
#[template(path = "withdraw.html")]
struct WithdrawTemplate;

/// Serve the withdraw page
pub async fn withdraw_page() -> impl IntoResponse {
    WithdrawTemplate
}

/// Handle withdraw form submission
pub async fn withdraw_submit(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Form(req): Form<crate::domain::models::WithdrawRequest>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    use axum::response::AppendHeaders;

    // Call the service
    wallet_service::withdraw(&state.pool, user_id, req.amount).await?;

    // Return success message and redirect
    Ok((
        AppendHeaders([("HX-Redirect", "/dashboard".to_string())]),
        "Withdrawal successful! Redirecting..."
    ))
}

#[derive(Template)]
#[template(path = "transactions.html")]
struct TransactionsTemplate {
    transactions: Vec<TransactionResponse>,
}

/// Serve the transactions page (full history)
pub async fn transactions_page(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    // Get ALL transactions
    let transactions_raw = wallet_service::get_history(&state.pool, user_id).await?;
    
    let transactions: Vec<TransactionResponse> = transactions_raw
        .into_iter()
        .map(TransactionResponse::from)
        .collect();

    let template = TransactionsTemplate {
        transactions,
    };

    Ok(template)
}

#[derive(Template)]
#[template(path = "transfer.html")]
struct TransferTemplate;

/// Serve the transfer page
pub async fn transfer_page() -> impl IntoResponse {
    TransferTemplate
}

/// Handle transfer form submission
pub async fn transfer_submit(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Form(req): Form<crate::domain::models::TransferRequest>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    use axum::response::AppendHeaders;

    tracing::info!("📥 Transfer request received: {:?}", req);

    // Call the service
    wallet_service::transfer(
        &state.pool,
        &state.email_service,
        &state.notification_service,
        user_id,
        &req.recipient_email,
        req.amount
    ).await?;

    // Return success message and redirect
    Ok((
        AppendHeaders([("HX-Redirect", "/dashboard".to_string())]),
        "Transfer successful! Redirecting..."
    ))
}

/// Handle web form registration (form-encoded, not JSON)
pub async fn register_submit(
    State(state): State<AppState>,
    Form(req): Form<crate::domain::models::CreateUserRequest>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    use axum::response::AppendHeaders;
    
    // Call the service
    let response = crate::services::auth_service::register(
        &state.pool,
        &req.email,
        &req.password,
        &req.full_name,
        &state.jwt_secret,
    )
    .await?;
    
    // Build cookie header
    let cookie_value = format!(
        "auth_token={}; Path=/; HttpOnly; SameSite=Lax",
        response.token
    );
    
    // Return with Set-Cookie and HX-Redirect headers
    Ok((
        AppendHeaders([
            ("Set-Cookie", cookie_value),
            ("HX-Redirect", "/dashboard".to_string()),
        ]),
        "Registration successful! Redirecting..."
    ))
}

/// Handle web form login (form-encoded, not JSON)
pub async fn login_submit(
    State(state): State<AppState>,
    Form(req): Form<crate::domain::models::LoginRequest>,
) -> Result<impl IntoResponse, crate::error::AppError> {
    use axum::response::AppendHeaders;
    
    // Call the service
    let response = crate::services::auth_service::login(
        &state.pool,
        &req.email,
        &req.password,
        &state.jwt_secret,
    )
    .await?;

    // Build cookie header
    let cookie_value = format!(
        "auth_token={}; Path=/; HttpOnly; SameSite=Lax",
        response.token
    );
    
    // Return with Set-Cookie and HX-Redirect headers
    Ok((
        AppendHeaders([
            ("Set-Cookie", cookie_value),
            ("HX-Redirect", "/dashboard".to_string()),
        ]),
        "Login successful! Redirecting..."
    ))
}

/// Handle logout (clear cookie)
pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build(("auth_token", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(0))
        .build();
    
    (jar.add(cookie), Redirect::to("/login"))
}

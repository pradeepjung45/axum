use axum::{extract::State, http::StatusCode, Json};
use crate::domain::models::{DepositRequest, WalletResponse, WithdrawRequest};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::repository::user_repo;
use crate::routes::auth_routes::AppState;
use crate::services::wallet_service;

// ============================================================================
// WALLET HANDLERS
// ============================================================================

/// Get the authenticated user's wallet
pub async fn get_wallet(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<WalletResponse>, AppError> {
    let wallet = user_repo::get_wallet_by_user_id(&state.pool, user_id).await?;
    Ok(Json(WalletResponse::from(wallet)))
}

/// Deposit money into the authenticated user's wallet
pub async fn deposit(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<DepositRequest>,
) -> Result<Json<WalletResponse>, AppError> {
    let wallet = wallet_service::deposit(&state.pool, user_id, req.amount).await?;
    Ok(Json(WalletResponse::from(wallet)))
}

/// Withdraw money from the authenticated user's wallet
///
/// HTTP Endpoint: POST /wallet/withdraw
/// 
/// Headers:
/// Authorization: Bearer <token>
///
/// Request Body:
/// ```json
/// {
///   "amount": "50.00"
/// }
/// ```
///
/// Success Response (200 OK):
/// ```json
/// {
///   "id": "...",
///   "balance": "50.50",
///   "currency": "USD"
/// }
/// ```
///
/// Error Responses:
/// - 400 Bad Request: Amount <= 0
/// - 422 Unprocessable Entity: Insufficient balance
pub async fn withdraw(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WalletResponse>, AppError> {
    let wallet = wallet_service::withdraw(&state.pool, user_id, req.amount).await?;
    Ok(Json(WalletResponse::from(wallet)))
}

/// Transfer money to another user
///
/// HTTP Endpoint: POST /wallet/transfer
/// 
/// Headers:
/// Authorization: Bearer <token>
///
/// Request Body:
/// ```json
/// {
///   "recipient_email": "bob@example.com",
///   "amount": "25.00"
/// }
/// ```
///
/// Success Response (200 OK):
/// ```json
/// {
///   "id": "...",
///   "balance": "25.50",
///   "currency": "USD"
/// }
/// ```
pub async fn transfer(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
    Json(req): Json<crate::domain::models::TransferRequest>,
) -> Result<Json<WalletResponse>, AppError> {
    let wallet = wallet_service::transfer(&state.pool, user_id, &req.recipient_email, req.amount).await?;
    Ok(Json(WalletResponse::from(wallet)))
}

/// Get transaction history
///
/// HTTP Endpoint: GET /transactions
/// 
/// Headers:
/// Authorization: Bearer <token>
///
/// Success Response (200 OK):
/// ```json
/// [
///   {
///     "id": "...",
///     "transaction_type": "DEPOSIT",
///     "amount": "100.00",
///     "status": "COMPLETED",
///     "created_at": "..."
///   }
/// ]
/// ```
pub async fn get_history(
    AuthUser(user_id): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::domain::models::TransactionResponse>>, AppError> {
    let transactions = wallet_service::get_history(&state.pool, user_id).await?;
    
    // Convert to response DTOs
    let response: Vec<crate::domain::models::TransactionResponse> = transactions
        .into_iter()
        .map(crate::domain::models::TransactionResponse::from)
        .collect();
        
    Ok(Json(response))
}

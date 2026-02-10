use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

// ============================================================================
// CENTRALIZED ERROR HANDLING
// ============================================================================
// This enum represents ALL possible errors in our application.
// 
// Why do we need this?
// - Instead of panicking (crashing), we handle errors gracefully
// - We can send proper HTTP status codes to clients
// - We get consistent error messages across the entire app
//
// The #[derive(Error)] comes from the 'thiserror' crate and automatically
// implements the Error trait for us.

#[derive(Debug, Error)]
pub enum AppError {
    // ========================================================================
    // DATABASE ERRORS
    // ========================================================================
    
    /// When a database query fails (connection issues, syntax errors, etc.)
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    // ========================================================================
    // AUTHENTICATION & AUTHORIZATION ERRORS
    // ========================================================================
    
    /// When user provides wrong email/password
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    /// When JWT token is missing or invalid
    #[error("Invalid or missing authentication token")]
    InvalidToken,
    
    /// When user tries to access something they don't own
    #[error("Unauthorized access")]
    Unauthorized,
    
    // ========================================================================
    // VALIDATION ERRORS
    // ========================================================================
    
    /// When user input is invalid (e.g., negative amount, invalid email)
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// When a user tries to register with an email that already exists
    #[error("User with this email already exists")]
    UserAlreadyExists,
    
    /// When we try to find a user/wallet/transaction that doesn't exist
    #[error("{0} not found")]
    NotFound(String),
    
    // ========================================================================
    // BUSINESS LOGIC ERRORS
    // ========================================================================
    
    /// When user tries to withdraw more money than they have
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    /// When a transaction fails for business reasons
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    // ========================================================================
    // GENERAL ERRORS
    // ========================================================================
    
    /// For any other unexpected errors
    #[error("Internal server error: {0}")]
    InternalError(String),
}

// ============================================================================
// CONVERT AppError TO HTTP RESPONSE
// ============================================================================
// This is the magic that makes Axum work with our errors!
// 
// When a handler returns Result<T, AppError>, Axum automatically calls this
// function to convert the error into an HTTP response.
//
// We return:
// - Appropriate HTTP status code (404, 401, 500, etc.)
// - JSON error message for the client

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Determine the HTTP status code based on the error type
        let status_code = match &self {
            // 400 Bad Request - Client sent invalid data
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            
            // 401 Unauthorized - Authentication failed
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::InvalidToken => StatusCode::UNAUTHORIZED,
            
            // 403 Forbidden - User doesn't have permission
            AppError::Unauthorized => StatusCode::FORBIDDEN,
            
            // 404 Not Found - Resource doesn't exist
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            
            // 409 Conflict - Resource already exists
            AppError::UserAlreadyExists => StatusCode::CONFLICT,
            
            // 422 Unprocessable Entity - Business logic error
            AppError::InsufficientBalance => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::TransactionFailed(_) => StatusCode::UNPROCESSABLE_ENTITY,
            
            // 500 Internal Server Error - Something went wrong on our end
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // Create a JSON response with error details
        let error_message = self.to_string();
        
        let body = Json(json!({
            "error": error_message,
            "status": status_code.as_u16(),
        }));

        // Return the response with status code and JSON body
        (status_code, body).into_response()
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

impl AppError {
    /// Helper to create a NotFound error with a custom message
    pub fn not_found(resource: &str) -> Self {
        AppError::NotFound(resource.to_string())
    }
    
    /// Helper to create a ValidationError with a custom message
    pub fn validation(message: &str) -> Self {
        AppError::ValidationError(message.to_string())
    }
    
    /// Helper to create an InternalError with a custom message
    pub fn internal(message: &str) -> Self {
        AppError::InternalError(message.to_string())
    }
}

// ============================================================================
// USAGE EXAMPLES (commented out, just for reference)
// ============================================================================

/*
// Example 1: In a handler function
async fn get_user(user_id: Uuid) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| AppError::not_found("User"))?; // Converts sqlx::Error to AppError
    
    Ok(Json(user))
}

// Example 2: Validation
fn validate_amount(amount: Decimal) -> Result<(), AppError> {
    if amount <= Decimal::ZERO {
        return Err(AppError::validation("Amount must be positive"));
    }
    Ok(())
}

// Example 3: Business logic
fn check_balance(wallet: &Wallet, amount: Decimal) -> Result<(), AppError> {
    if wallet.balance < amount {
        return Err(AppError::InsufficientBalance);
    }
    Ok(())
}
*/

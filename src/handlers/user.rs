use axum::{extract::State, Json};
use crate::domain::models::UserResponse;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::repository::user_repo;
use crate::routes::auth_routes::AppState;

// ============================================================================
// USER HANDLERS
// ============================================================================

/// Get the authenticated user's profile
///
/// This is a protected endpoint that requires a valid JWT token.
///
/// HTTP Endpoint: GET /me
/// 
/// Headers:
/// Authorization: Bearer <token>
///
/// Success Response (200 OK):
/// ```json
/// {
///   "id": "...",
///   "email": "alice@example.com",
///   "full_name": "Alice Smith",
///   "created_at": "2024-..."
/// }
/// ```
pub async fn get_me(
    AuthUser(user_id): AuthUser,  // ← Automatic JWT validation!
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, AppError> {
    // If we get here, the user is authenticated!
    // The AuthUser extractor already validated the token.
    
    let user = user_repo::find_user_by_id(&state.pool, user_id).await?;
    
    Ok(Json(UserResponse::from(user)))
}

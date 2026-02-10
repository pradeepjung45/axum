use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::request::Parts,
};
use crate::error::AppError;
use crate::routes::auth_routes::AppState;
use crate::utils::jwt::validate_token;
use uuid::Uuid;

// ============================================================================
// AUTHENTICATED USER EXTRACTOR
// ============================================================================

/// Extractor for authenticated users
///
/// This extracts and validates the JWT token from the Authorization header.
/// If the token is valid, it returns the user's UUID.
pub struct AuthUser(pub Uuid);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. Try to get token from Authorization header
        let token = if let Some(auth_header) = parts.headers.get("Authorization") {
            let auth_str = auth_header.to_str().map_err(|_| AppError::InvalidToken)?;
            if auth_str.starts_with("Bearer ") {
                Some(auth_str[7..].to_string())
            } else {
                None
            }
        } else {
            None
        };

        // 2. If no header, try to parse from Cookie header
        let token = if let Some(t) = token {
            t
        } else {
            // Parse Cookie header manually
            if let Some(cookie_header) = parts.headers.get("Cookie") {
                let cookie_str = cookie_header.to_str().map_err(|_| AppError::InvalidToken)?;
                
                // Parse cookies (format: "name1=value1; name2=value2")
                let auth_token = cookie_str
                    .split(';')
                    .map(|s| s.trim())
                    .find_map(|cookie| {
                        let mut parts = cookie.split('=');
                        let name = parts.next()?;
                        let value = parts.next()?;
                        if name == "auth_token" {
                            Some(value.to_string())
                        } else {
                            None
                        }
                    });
                
                auth_token.ok_or(AppError::InvalidToken)?
            } else {
                return Err(AppError::InvalidToken);
            }
        };

        // 3. Validate the token
        let claims = validate_token(&token, &state.jwt_secret)?;

        // 4. Get user ID from claims
        let user_id = claims.user_id()?;

        Ok(AuthUser(user_id))
    }
}

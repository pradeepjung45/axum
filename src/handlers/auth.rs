use axum::{extract::State, http::StatusCode, Json};
use crate::domain::models::{CreateUserRequest, LoginRequest, LoginResponse};
use crate::error::AppError;
use crate::routes::auth_routes::AppState;
use crate::services::auth_service;

/// Register a new user
pub async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), AppError> {
    let response = auth_service::register(
        &state.pool,
        &req.email,
        &req.password,
        &req.full_name,
        &state.jwt_secret,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Login an existing user
pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let response = auth_service::login(
        &state.pool,
        &req.email,
        &req.password,
        &state.jwt_secret,
    )
    .await?;

    Ok(Json(response))
}

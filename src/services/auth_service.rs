use crate::domain::models::{LoginResponse, UserResponse};
use crate::error::AppError;
use crate::repository::user_repo;
use crate::utils::jwt::{generate_token, hash_password, verify_password};
use sqlx::PgPool;

// ============================================================================
// AUTH SERVICE
// ============================================================================
// The service layer contains BUSINESS LOGIC.
//
// What's the difference between Service and Repository?
// - Repository: "How to talk to the database" (SQL queries)
// - Service: "What the app should do" (business rules, orchestration)
//
// Example:
// - Repository: create_user(), create_wallet()
// - Service: register() - calls both, generates token, enforces rules

/// Register a new user
///
/// This orchestrates the entire registration process:
/// 1. Hash the password (security)
/// 2. Create user in database
/// 3. Create wallet for user (every user gets a wallet)
/// 4. Generate JWT token
/// 5. Return user info and token
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `email` - User's email
/// * `password` - Plain text password (will be hashed)
/// * `full_name` - User's full name
/// * `jwt_secret` - Secret key for signing JWT tokens
///
/// # Returns
/// LoginResponse with token and user info (without password hash)
///
/// # Errors
/// - `AppError::UserAlreadyExists` if email is already registered
/// - `AppError::ValidationError` if input is invalid
/// - `AppError::DatabaseError` for database issues
///
/// # Example
/// ```
/// let response = register(
///     &pool,
///     "user@example.com",
///     "mypassword123",
///     "John Doe",
///     &config.jwt_secret
/// ).await?;
///
/// // Returns:
/// // LoginResponse {
/// //     token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
/// //     user: UserResponse { id, email, full_name, created_at }
/// // }
/// ```
pub async fn register(
    pool: &PgPool,
    email: &str,
    password: &str,
    full_name: &str,
    jwt_secret: &str,
) -> Result<LoginResponse, AppError> {
    // ========================================================================
    // STEP 1: Validate input
    // ========================================================================
    
    // Check email is not empty
    if email.trim().is_empty() {
        return Err(AppError::validation("Email cannot be empty"));
    }
    
    // Check password length (at least 8 characters)
    if password.len() < 8 {
        return Err(AppError::validation("Password must be at least 8 characters"));
    }
    
    // Check full name is not empty
    if full_name.trim().is_empty() {
        return Err(AppError::validation("Full name cannot be empty"));
    }
    
    // ========================================================================
    // STEP 2: Hash the password
    // ========================================================================
    // NEVER store plain passwords!
    let password_hash = hash_password(password)?;
    
    // ========================================================================
    // STEP 3: Create user in database
    // ========================================================================
    // This will error if email already exists (unique constraint)
    let user = user_repo::create_user(pool, email, &password_hash, full_name).await?;
    
    // ========================================================================
    // STEP 4: Create wallet for user
    // ========================================================================
    // Every user gets a wallet with $0.00 balance
    let _wallet = user_repo::create_wallet(pool, user.id).await?;
    
    // ========================================================================
    // STEP 5: Generate JWT token
    // ========================================================================
    // Token expires in 24 hours
    let token = generate_token(user.id, jwt_secret)?;
    
    // ========================================================================
    // STEP 6: Return response
    // ========================================================================
    // Convert User to UserResponse (removes password_hash for security)
    let user_response = UserResponse::from(user);
    
    Ok(LoginResponse {
        token,
        user: user_response,
    })
}

/// Login an existing user
///
/// This handles the login process:
/// 1. Find user by email
/// 2. Verify password
/// 3. Generate JWT token
/// 4. Return user info and token
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `email` - User's email
/// * `password` - Plain text password
/// * `jwt_secret` - Secret key for signing JWT tokens
///
/// # Returns
/// LoginResponse with token and user info
///
/// # Errors
/// - `AppError::InvalidCredentials` if email or password is wrong
/// - `AppError::DatabaseError` for database issues
///
/// # Example
/// ```
/// let response = login(
///     &pool,
///     "user@example.com",
///     "mypassword123",
///     &config.jwt_secret
/// ).await?;
///
/// // Returns same format as register()
/// ```
pub async fn login(
    pool: &PgPool,
    email: &str,
    password: &str,
    jwt_secret: &str,
) -> Result<LoginResponse, AppError> {
    // ========================================================================
    // STEP 1: Find user by email
    // ========================================================================
    // If user doesn't exist, this returns AppError::NotFound
    // We convert it to InvalidCredentials for security
    // (don't reveal whether email exists or not)
    let user = user_repo::find_user_by_email(pool, email)
        .await
        .map_err(|_| AppError::InvalidCredentials)?;
    
    // ========================================================================
    // STEP 2: Verify password
    // ========================================================================
    // Compare the provided password with the stored hash
    // If wrong, returns AppError::InvalidCredentials
    verify_password(password, &user.password_hash)?;
    
    // ========================================================================
    // STEP 3: Generate JWT token
    // ========================================================================
    let token = generate_token(user.id, jwt_secret)?;
    
    // ========================================================================
    // STEP 4: Return response
    // ========================================================================
    let user_response = UserResponse::from(user);
    
    Ok(LoginResponse {
        token,
        user: user_response,
    })
}

// ============================================================================
// WHY WE DON'T REVEAL IF EMAIL EXISTS
// ============================================================================
/*
In the login function, we convert NotFound to InvalidCredentials.

Bad approach (reveals too much):
- "Email not found" → Attacker knows email doesn't exist
- "Wrong password" → Attacker knows email exists, can brute-force password

Good approach (our approach):
- "Invalid credentials" → Attacker doesn't know if email or password is wrong

This prevents email enumeration attacks where attackers try to find valid emails.
*/

// ============================================================================
// USAGE EXAMPLE (commented out)
// ============================================================================
/*
// In a handler:
async fn register_handler(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let response = auth_service::register(
        &pool,
        &req.email,
        &req.password,
        &req.full_name,
        &config.jwt_secret,
    ).await?;
    
    Ok(Json(response))
}
*/

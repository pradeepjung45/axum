use crate::error::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// JWT CLAIMS
// ============================================================================
// Claims are the data we store inside a JWT token.
//
// What is a JWT?
// JWT (JSON Web Token) is like a secure ID card. When a user logs in:
// 1. We create a token with their user_id and expiration time
// 2. We sign it with our secret key (so it can't be faked)
// 3. We send it to the user
// 4. User includes it in future requests to prove who they are
//
// Why JWT?
// - Stateless: We don't need to store sessions in a database
// - Secure: Signed with a secret, can't be tampered with
// - Self-contained: Contains all the info we need (user_id, expiration)

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject - the user ID this token belongs to
    pub sub: String,  // "sub" is a standard JWT field meaning "subject"
    
    /// Expiration time (Unix timestamp)
    pub exp: usize,   // "exp" is a standard JWT field for expiration
    
    /// Issued at (Unix timestamp)
    pub iat: usize,   // "iat" is a standard JWT field for "issued at"
}

impl Claims {
    /// Create new claims for a user
    ///
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `expiration_hours` - How many hours until the token expires
    ///
    /// # Returns
    /// Claims with user_id and expiration time set
    pub fn new(user_id: Uuid, expiration_hours: i64) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(expiration_hours);
        
        Claims {
            sub: user_id.to_string(),
            exp: expiration.timestamp() as usize,
            iat: now.timestamp() as usize,
        }
    }
    
    /// Get the user ID from claims
    pub fn user_id(&self) -> Result<Uuid, AppError> {
        Uuid::parse_str(&self.sub)
            .map_err(|_| AppError::InvalidToken)
    }
}

// ============================================================================
// JWT TOKEN FUNCTIONS
// ============================================================================

/// Generate a JWT token for a user
///
/// This creates a signed token that the user can use for authentication.
///
/// # Arguments
/// * `user_id` - The user's UUID
/// * `secret` - The JWT secret key from config
///
/// # Returns
/// A signed JWT token string
///
/// # Example
/// ```
/// let token = generate_token(user_id, &config.jwt_secret)?;
/// // Returns something like: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
/// ```
pub fn generate_token(user_id: Uuid, secret: &str) -> Result<String, AppError> {
    // Create claims with 24 hour expiration
    let claims = Claims::new(user_id, 24);
    
    // Encode the token with our secret
    let token = encode(
        &Header::default(),                    // Use default header (HS256 algorithm)
        &claims,                               // Our claims data
        &EncodingKey::from_secret(secret.as_bytes()), // Our secret key
    )
    .map_err(|e| AppError::internal(&format!("Failed to generate token: {}", e)))?;
    
    Ok(token)
}

/// Validate a JWT token and extract the claims
///
/// This checks if a token is valid (not expired, properly signed) and returns the claims.
///
/// # Arguments
/// * `token` - The JWT token string
/// * `secret` - The JWT secret key from config
///
/// # Returns
/// The claims if valid, or an error if invalid/expired
///
/// # Example
/// ```
/// let claims = validate_token(&token, &config.jwt_secret)?;
/// let user_id = claims.user_id()?;
/// ```
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    // Decode and validate the token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(), // Uses default validation (checks expiration, signature)
    )
    .map_err(|e| {
        // Different error messages based on what went wrong
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                AppError::InvalidToken // Token expired
            }
            _ => AppError::InvalidToken // Invalid signature or malformed token
        }
    })?;
    
    Ok(token_data.claims)
}

// ============================================================================
// PASSWORD HASHING
// ============================================================================
// We NEVER store plain passwords in the database!
// Instead, we hash them using Argon2 (a secure hashing algorithm).
//
// How it works:
// 1. User registers with password "mypassword123"
// 2. We hash it: "$argon2id$v=19$m=19456,t=2,p=1$..."
// 3. We store the hash in the database
// 4. When user logs in, we hash their input and compare to stored hash
//
// Why Argon2?
// - Slow on purpose (makes brute-force attacks impractical)
// - Includes a random "salt" (so same password = different hash each time)
// - Winner of the Password Hashing Competition

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash a password using Argon2
///
/// # Arguments
/// * `password` - The plain text password
///
/// # Returns
/// A hashed password string safe to store in the database
///
/// # Example
/// ```
/// let hash = hash_password("mypassword123")?;
/// // Returns: "$argon2id$v=19$m=19456,t=2,p=1$..."
/// ```
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng); // Generate random salt
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::internal(&format!("Failed to hash password: {}", e)))?
        .to_string();
    
    Ok(password_hash)
}

/// Verify a password against a hash
///
/// # Arguments
/// * `password` - The plain text password to check
/// * `hash` - The stored password hash from the database
///
/// # Returns
/// Ok(()) if password matches, Err if it doesn't
///
/// # Example
/// ```
/// verify_password("mypassword123", &user.password_hash)?;
/// // Returns Ok(()) if correct, Err(AppError::InvalidCredentials) if wrong
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<(), AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::internal(&format!("Invalid password hash: {}", e)))?;
    
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::InvalidCredentials) // Wrong password
}

// ============================================================================
// USAGE EXAMPLES (commented out)
// ============================================================================

/*
// Example 1: User Registration
async fn register_user(email: &str, password: &str) -> Result<User, AppError> {
    // Hash the password before storing
    let password_hash = hash_password(password)?;
    
    // Store user with hashed password
    let user = create_user_in_db(email, &password_hash).await?;
    
    Ok(user)
}

// Example 2: User Login
async fn login_user(email: &str, password: &str) -> Result<String, AppError> {
    // Get user from database
    let user = get_user_by_email(email).await?;
    
    // Verify password
    verify_password(password, &user.password_hash)?;
    
    // Generate JWT token
    let token = generate_token(user.id, &config.jwt_secret)?;
    
    Ok(token)
}

// Example 3: Protected Route
async fn get_user_profile(token: &str) -> Result<User, AppError> {
    // Validate token
    let claims = validate_token(token, &config.jwt_secret)?;
    
    // Get user ID from claims
    let user_id = claims.user_id()?;
    
    // Fetch user from database
    let user = get_user_by_id(user_id).await?;
    
    Ok(user)
}
*/

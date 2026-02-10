use crate::domain::models::{User, Wallet};
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// USER REPOSITORY
// ============================================================================

/// Create a new user in the database
pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    full_name: &str,
) -> Result<User, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, full_name)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, full_name, 
                  created_at as "created_at!", 
                  updated_at as "updated_at!"
        "#,
        email,
        password_hash,
        full_name
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.is_unique_violation() {
                return AppError::UserAlreadyExists;
            }
        }
        AppError::DatabaseError(e)
    })?;

    Ok(user)
}

/// Find a user by email
pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<User, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, full_name, 
               created_at as "created_at!", 
               updated_at as "updated_at!"
        FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::not_found("User"),
        _ => AppError::DatabaseError(e),
    })?;

    Ok(user)
}

/// Find a user by ID
pub async fn find_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, full_name, 
               created_at as "created_at!", 
               updated_at as "updated_at!"
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::not_found("User"),
        _ => AppError::DatabaseError(e),
    })?;

    Ok(user)
}

// ============================================================================
// WALLET REPOSITORY
// ============================================================================

/// Create a wallet for a user
pub async fn create_wallet(pool: &PgPool, user_id: Uuid) -> Result<Wallet, AppError> {
    let wallet = sqlx::query_as!(
        Wallet,
        r#"
        INSERT INTO wallets (user_id, balance, currency)
        VALUES ($1, 0.00, 'USD')
        RETURNING id, user_id, 
                  balance as "balance!", 
                  currency, 
                  created_at as "created_at!", 
                  updated_at as "updated_at!"
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::DatabaseError)?;

    Ok(wallet)
}

/// Get a user's wallet
pub async fn get_wallet_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Wallet, AppError> {
    let wallet = sqlx::query_as!(
        Wallet,
        r#"
        SELECT id, user_id, 
               balance as "balance!", 
               currency, 
               created_at as "created_at!", 
               updated_at as "updated_at!"
        FROM wallets
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::not_found("Wallet"),
        _ => AppError::DatabaseError(e),
    })?;

    Ok(wallet)
}

/// Update wallet balance
pub async fn update_wallet_balance(
    pool: &PgPool,
    wallet_id: Uuid,
    new_balance: rust_decimal::Decimal,
) -> Result<Wallet, AppError> {
    let wallet = sqlx::query_as!(
        Wallet,
        r#"
        UPDATE wallets
        SET balance = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, user_id, 
                  balance as "balance!", 
                  currency, 
                  created_at as "created_at!", 
                  updated_at as "updated_at!"
        "#,
        new_balance,
        wallet_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::not_found("Wallet"),
        _ => AppError::DatabaseError(e),
    })?;

    Ok(wallet)
}

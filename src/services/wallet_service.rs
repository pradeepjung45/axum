use crate::error::AppError;
use crate::repository::user_repo;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// WALLET SERVICE
// ============================================================================
// Business logic for wallet operations

/// Deposit money into a wallet
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - The user's UUID
/// * `amount` - Amount to deposit (must be positive)
///
/// # Returns
/// The updated wallet with new balance
pub async fn deposit(
    pool: &PgPool,
    user_id: Uuid,
    amount: Decimal,
) -> Result<crate::domain::models::Wallet, AppError> {
    // 1. Validate amount
    if amount <= Decimal::ZERO {
        return Err(AppError::validation("Deposit amount must be greater than 0"));
    }

    // 2. Start transaction
    let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

    // 3. Get current wallet (locking row)
    let wallet = sqlx::query_as!(
        crate::domain::models::Wallet,
        r#"
        SELECT id, user_id, balance as "balance!", currency, created_at as "created_at!", updated_at as "updated_at!"
        FROM wallets
        WHERE user_id = $1
        FOR UPDATE
        "#,
        user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::not_found("Wallet"),
        _ => AppError::DatabaseError(e),
    })?;

    // 4. Calculate new balance
    let new_balance = wallet.balance + amount;

    // 5. Update wallet
    let updated_wallet = sqlx::query_as!(
        crate::domain::models::Wallet,
        r#"
        UPDATE wallets
        SET balance = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, user_id, balance as "balance!", currency, created_at as "created_at!", updated_at as "updated_at!"
        "#,
        new_balance,
        wallet.id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::DatabaseError)?;

    // 6. Record Transaction
    sqlx::query!(
        r#"
        INSERT INTO transactions (wallet_id, transaction_type, amount, description, status)
        VALUES ($1, 'DEPOSIT', $2, 'Deposit funds', 'COMPLETED')
        "#,
        wallet.id,
        amount
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::DatabaseError)?;

    // 7. Commit
    tx.commit().await.map_err(AppError::DatabaseError)?;

    Ok(updated_wallet)
}

/// Withdraw money from a wallet
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - The user's UUID
/// * `amount` - Amount to withdraw (must be positive and <= balance)
///
/// # Returns
/// The updated wallet with new balance
pub async fn withdraw(
    pool: &PgPool,
    user_id: Uuid,
    amount: Decimal,
) -> Result<crate::domain::models::Wallet, AppError> {
    // 1. Validate amount
    if amount <= Decimal::ZERO {
        return Err(AppError::validation("Withdrawal amount must be greater than 0"));
    }

    // 2. Start transaction
    let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

    // 3. Get current wallet (locking row)
    let wallet = sqlx::query_as!(
        crate::domain::models::Wallet,
        r#"
        SELECT id, user_id, balance as "balance!", currency, created_at as "created_at!", updated_at as "updated_at!"
        FROM wallets
        WHERE user_id = $1
        FOR UPDATE
        "#,
        user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::not_found("Wallet"),
        _ => AppError::DatabaseError(e),
    })?;

    // 4. Check balance
    if wallet.balance < amount {
        return Err(AppError::InsufficientBalance);
    }

    // 5. Calculate new balance
    let new_balance = wallet.balance - amount;

    // 6. Update wallet
    let updated_wallet = sqlx::query_as!(
        crate::domain::models::Wallet,
        r#"
        UPDATE wallets
        SET balance = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, user_id, balance as "balance!", currency, created_at as "created_at!", updated_at as "updated_at!"
        "#,
        new_balance,
        wallet.id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::DatabaseError)?;

    // 7. Record Transaction
    sqlx::query!(
        r#"
        INSERT INTO transactions (wallet_id, transaction_type, amount, description, status)
        VALUES ($1, 'WITHDRAWAL', $2, 'Withdraw funds', 'COMPLETED')
        "#,
        wallet.id,
        amount
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::DatabaseError)?;

    // 8. Commit
    tx.commit().await.map_err(AppError::DatabaseError)?;

    Ok(updated_wallet)
}

/// Transfer money to another user
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `sender_id` - The sender's UUID
/// * `recipient_email` - The recipient's email address
/// * `amount` - Amount to transfer (must be positive and <= balance)
///
/// # Returns
/// The updated sender's wallet
pub async fn transfer(
    pool: &PgPool,
    email_service: &crate::services::email_service::EmailService,
    sender_id: Uuid,
    recipient_email: &str,
    amount: Decimal,
) -> Result<crate::domain::models::Wallet, AppError> {
    // 1. Validate amount
    if amount <= Decimal::ZERO {
        return Err(AppError::validation("Transfer amount must be greater than 0"));
    }

    // 2. Start a database transaction (Atomic Operation)
    let mut tx = pool.begin().await.map_err(AppError::DatabaseError)?;

    // 3. Get sender's wallet (FOR UPDATE to lock the row)
    let sender_wallet = sqlx::query_as!(
        crate::domain::models::Wallet,
        r#"
        SELECT id, user_id, balance as "balance!", currency, created_at as "created_at!", updated_at as "updated_at!"
        FROM wallets
        WHERE user_id = $1
        FOR UPDATE
        "#,
        sender_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::not_found("Sender wallet"),
        _ => AppError::DatabaseError(e),
    })?;

    // 4. Check balance
    if sender_wallet.balance < amount {
        return Err(AppError::InsufficientBalance);
    }

    // 5. Get recipient user and wallet
    let recipient_user = sqlx::query!(
        r#"SELECT id FROM users WHERE email = $1"#,
        recipient_email
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::validation("Recipient not found"),
        _ => AppError::DatabaseError(e),
    })?;

    if recipient_user.id == sender_id {
        return Err(AppError::validation("Cannot transfer money to yourself"));
    }

    let recipient_wallet = sqlx::query!(
        r#"
        SELECT id FROM wallets WHERE user_id = $1 FOR UPDATE
        "#,
        recipient_user.id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::not_found("Recipient wallet"),
        _ => AppError::DatabaseError(e),
    })?;

    // 6. Deduct from sender
    let new_sender_balance = sender_wallet.balance - amount;
    let updated_sender_wallet = sqlx::query_as!(
        crate::domain::models::Wallet,
        r#"
        UPDATE wallets
        SET balance = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING id, user_id, balance as "balance!", currency, created_at as "created_at!", updated_at as "updated_at!"
        "#,
        new_sender_balance,
        sender_wallet.id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::DatabaseError)?;

    // Record Sender Transaction (Debit)
    sqlx::query!(
        r#"
        INSERT INTO transactions (wallet_id, transaction_type, amount, description, status)
        VALUES ($1, 'TRANSFER', $2, 'Transfer sent', 'COMPLETED')
        "#,
        sender_wallet.id,
        amount
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::DatabaseError)?;

    // 7. Add to recipient
    sqlx::query!(
        r#"
        UPDATE wallets
        SET balance = balance + $1, updated_at = NOW()
        WHERE id = $2
        "#,
        amount,
        recipient_wallet.id
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::DatabaseError)?;

    // Record Recipient Transaction (Credit)
    sqlx::query!(
        r#"
        INSERT INTO transactions (wallet_id, transaction_type, amount, description, status)
        VALUES ($1, 'TRANSFER', $2, 'Transfer received', 'COMPLETED')
        "#,
        recipient_wallet.id,
        amount
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::DatabaseError)?;

    // 8. Commit transaction
    // 8. Commit transaction
    tx.commit().await.map_err(AppError::DatabaseError)?;

    // 9. Send Email Notification (Async)
    let email_service = email_service.clone();
    let recipient_email = recipient_email.to_string();
    tokio::spawn(async move {
        email_service.send_transfer_success(&recipient_email, amount).await;
    });

    Ok(updated_sender_wallet)
}

/// Get transaction history for a user
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `user_id` - The user's UUID
///
/// # Returns
/// List of transactions
pub async fn get_history(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<crate::domain::models::Transaction>, AppError> {
    // We first need to get the wallet_id for the user
    let wallet = user_repo::get_wallet_by_user_id(pool, user_id).await?;
    
    let transactions = sqlx::query_as!(
        crate::domain::models::Transaction,
        r#"
        SELECT id, wallet_id, transaction_type, amount, description, status as "status!", created_at as "created_at!"
        FROM transactions
        WHERE wallet_id = $1
        ORDER BY created_at DESC
        "#,
        wallet.id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::DatabaseError)?;

    Ok(transactions)
}

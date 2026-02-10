# Step 2: Error Handling - Explained

## What Is Error Handling?

Error handling is how we deal with things that go wrong in our application. Instead of crashing, we want to:
1. Catch the error
2. Understand what went wrong
3. Send a helpful message to the user
4. Return the appropriate HTTP status code

## The AppError Enum

An **enum** in Rust is like a list of possible variants. Our `AppError` enum lists all possible errors:

```rust
pub enum AppError {
    DatabaseError(sqlx::Error),
    InvalidCredentials,
    InvalidToken,
    // ... and more
}
```

Think of it like a menu of errors - each variant is a different type of problem.

## Error Categories

### 1. **Database Errors**
- `DatabaseError` - When database queries fail
- Example: Connection lost, table doesn't exist, SQL syntax error

### 2. **Authentication Errors**
- `InvalidCredentials` - Wrong email/password
- `InvalidToken` - JWT token is missing or invalid
- `Unauthorized` - User trying to access something they don't own

### 3. **Validation Errors**
- `ValidationError` - Invalid input (negative amount, bad email format)
- `UserAlreadyExists` - Email already registered
- `NotFound` - Resource doesn't exist (user, wallet, transaction)

### 4. **Business Logic Errors**
- `InsufficientBalance` - Not enough money in wallet
- `TransactionFailed` - Transaction couldn't complete

### 5. **General Errors**
- `InternalError` - Unexpected errors

## HTTP Status Codes

Each error maps to an HTTP status code:

| Error Type | Status Code | Meaning |
|------------|-------------|---------|
| `ValidationError` | 400 | Bad Request - Client sent invalid data |
| `InvalidCredentials` | 401 | Unauthorized - Authentication failed |
| `Unauthorized` | 403 | Forbidden - No permission |
| `NotFound` | 404 | Not Found - Resource doesn't exist |
| `UserAlreadyExists` | 409 | Conflict - Resource already exists |
| `InsufficientBalance` | 422 | Unprocessable Entity - Business rule violated |
| `DatabaseError` | 500 | Internal Server Error - Our fault |

## The Magic: `IntoResponse`

This trait converts our `AppError` into an HTTP response:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 1. Pick the right status code
        // 2. Create JSON error message
        // 3. Return HTTP response
    }
}
```

**Why is this useful?**
- Handlers can return `Result<T, AppError>`
- Axum automatically converts errors to HTTP responses
- Consistent error format across all endpoints

## Usage Examples

### Example 1: Database Query
```rust
async fn get_user(id: Uuid) -> Result<User, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_one(&pool)
        .await
        .map_err(|_| AppError::not_found("User"))?;
    
    Ok(user)
}
```

If the user doesn't exist, this returns a 404 with:
```json
{
  "error": "User not found",
  "status": 404
}
```

### Example 2: Validation
```rust
fn validate_amount(amount: Decimal) -> Result<(), AppError> {
    if amount <= Decimal::ZERO {
        return Err(AppError::validation("Amount must be positive"));
    }
    Ok(())
}
```

If amount is negative, this returns a 400 with:
```json
{
  "error": "Validation error: Amount must be positive",
  "status": 400
}
```

### Example 3: Business Logic
```rust
fn withdraw(wallet: &mut Wallet, amount: Decimal) -> Result<(), AppError> {
    if wallet.balance < amount {
        return Err(AppError::InsufficientBalance);
    }
    wallet.balance -= amount;
    Ok(())
}
```

If balance is too low, this returns a 422 with:
```json
{
  "error": "Insufficient balance",
  "status": 422
}
```

## Key Rust Concepts

### The `?` Operator
```rust
let user = get_user(id).await?;
```

This is shorthand for:
```rust
let user = match get_user(id).await {
    Ok(user) => user,
    Err(e) => return Err(e),
};
```

It automatically returns the error if something goes wrong.

### The `#[from]` Attribute
```rust
#[error("Database error: {0}")]
DatabaseError(#[from] sqlx::Error),
```

This automatically converts `sqlx::Error` into `AppError::DatabaseError`. So you can do:
```rust
let user = query.fetch_one(&pool).await?; // sqlx::Error auto-converts to AppError
```

## Next Steps

Now that we have error handling, we can:
1. Implement the **Config** module (load environment variables)
2. Build the **Repository** layer (database operations with proper error handling)
3. Create **JWT utilities** (with error handling for invalid tokens)

# Step 5: Repository Layer - Explained

## What is the Repository Layer?

The repository layer is the **data access layer** - it's responsible for ALL database operations.

Think of it as a **librarian**:
- You ask the librarian for a book (data)
- The librarian knows where everything is stored (database)
- You don't need to know the library's organization system (SQL)

## Why Separate Repository from Business Logic?

**Without Repository Pattern:**
```rust
// Business logic mixed with SQL - messy!
async fn register_user(email: &str, password: &str) -> Result<User, AppError> {
    let hash = hash_password(password)?;
    let user = sqlx::query!("INSERT INTO users...").fetch_one().await?;
    let wallet = sqlx::query!("INSERT INTO wallets...").fetch_one().await?;
    // SQL everywhere!
}
```

**With Repository Pattern:**
```rust
// Clean separation!
async fn register_user(email: &str, password: &str) -> Result<User, AppError> {
    let hash = hash_password(password)?;
    let user = user_repo::create_user(&pool, email, &hash, name).await?;
    let wallet = user_repo::create_wallet(&pool, user.id).await?;
    // No SQL here, just function calls!
}
```

**Benefits:**
- Business logic is clean and readable
- All SQL in one place
- Easy to test (can mock the repository)
- Easy to switch databases later

## The Functions We Created

### User Operations

#### 1. `create_user()`
Creates a new user in the database.

```rust
pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    full_name: &str,
) -> Result<User, AppError>
```

**What it does:**
1. Inserts user into `users` table
2. Returns the created user with auto-generated ID and timestamps
3. Handles duplicate email error

**SQL:**
```sql
INSERT INTO users (email, password_hash, full_name)
VALUES ($1, $2, $3)
RETURNING id, email, password_hash, full_name, created_at, updated_at
```

#### 2. `find_user_by_email()`
Finds a user by their email address.

```rust
pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<User, AppError>
```

**Used for:** Login (need to verify password)

**SQL:**
```sql
SELECT id, email, password_hash, full_name, created_at, updated_at
FROM users
WHERE email = $1
```

#### 3. `find_user_by_id()`
Finds a user by their UUID.

```rust
pub async fn find_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<User, AppError>
```

**Used for:** Getting user profile, validating user exists

### Wallet Operations

#### 4. `create_wallet()`
Creates a wallet for a user with initial balance of $0.00.

```rust
pub async fn create_wallet(pool: &PgPool, user_id: Uuid) -> Result<Wallet, AppError>
```

**When used:** Automatically when a user registers

#### 5. `get_wallet_by_user_id()`
Gets a user's wallet.

```rust
pub async fn get_wallet_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Wallet, AppError>
```

**Used for:** Checking balance, making transactions

#### 6. `update_wallet_balance()`
Updates a wallet's balance.

```rust
pub async fn update_wallet_balance(
    pool: &PgPool,
    wallet_id: Uuid,
    new_balance: Decimal,
) -> Result<Wallet, AppError>
```

**Used for:** After deposits, withdrawals, transfers

## Understanding SQLx Queries

### The `query_as!` Macro

```rust
sqlx::query_as!(
    User,              // 1. Map results to this struct
    r#"                // 2. SQL query (raw string)
    SELECT id, email, password_hash, full_name, created_at, updated_at
    FROM users
    WHERE email = $1   // 3. $1 = first parameter
    "#,
    email              // 4. Value for $1
)
.fetch_one(pool)       // 5. Get exactly one row
.await                 // 6. Wait for completion
```

### Key Features

**1. Compile-Time Checking**
SQLx connects to your database at compile time and verifies:
- Table exists
- Columns exist
- Types match

If you typo a column name, it won't compile!

**2. SQL Injection Protection**
Parameters (`$1`, `$2`) are **safely escaped**:
```rust
// Safe - uses parameters
query_as!(..., "WHERE email = $1", email)

// NEVER do this - vulnerable to SQL injection!
query_as!(..., format!("WHERE email = '{}'", email))
```

**3. Automatic Mapping**
Database row automatically converts to Rust struct:
```
Database Row:        Rust Struct:
id: UUID        →    User {
email: VARCHAR  →        id: Uuid,
...             →        email: String,
                         ...
                     }
```

### Fetch Methods

```rust
.fetch_one(pool)      // Returns exactly 1 row, errors if 0 or >1
.fetch_optional(pool) // Returns Option<T>, None if no rows
.fetch_all(pool)      // Returns Vec<T>, all matching rows
```

## Error Handling

### Unique Constraint Violation
```rust
.map_err(|e| {
    if let sqlx::Error::Database(db_err) = &e {
        if db_err.is_unique_violation() {
            return AppError::UserAlreadyExists;
        }
    }
    AppError::DatabaseError(e)
})
```

**What this does:**
- Checks if error is a unique constraint violation (duplicate email)
- Returns `AppError::UserAlreadyExists` (409 Conflict)
- Otherwise returns generic `AppError::DatabaseError` (500 Internal Server Error)

### Row Not Found
```rust
.map_err(|e| match e {
    sqlx::Error::RowNotFound => AppError::not_found("User"),
    _ => AppError::DatabaseError(e),
})
```

**What this does:**
- If no rows found, returns `AppError::NotFound` (404)
- Otherwise returns `AppError::DatabaseError` (500)

## How This Will Be Used

In the **Service Layer** (next step):

```rust
// Registration
pub async fn register(pool: &PgPool, email: &str, password: &str, name: &str) 
    -> Result<(User, String), AppError> 
{
    // 1. Hash password
    let hash = hash_password(password)?;
    
    // 2. Create user (uses repository)
    let user = user_repo::create_user(pool, email, &hash, name).await?;
    
    // 3. Create wallet (uses repository)
    let wallet = user_repo::create_wallet(pool, user.id).await?;
    
    // 4. Generate token
    let token = generate_token(user.id, secret)?;
    
    Ok((user, token))
}
```

Clean and readable - no SQL in sight!

## Next Steps

Now we can implement:
1. **Auth Service** - Registration and login logic using these repository functions
2. **Transaction Service** - Deposit, withdrawal, transfer logic

Ready to continue?

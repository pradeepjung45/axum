# Step 1: Domain Models - Explained

## What Are Domain Models?

Domain models are **Rust structs** that represent the core data in our application. They match our database tables and define what information we store and work with.

Think of them as **blueprints** for our data.

## The Three Main Models

### 1. **User** 
Represents a person using our app.

**Fields:**
- `id` - Unique identifier (like a social security number for the user)
- `email` - User's email address (must be unique)
- `password_hash` - Encrypted password (we NEVER store plain passwords!)
- `full_name` - User's name
- `created_at` / `updated_at` - Timestamps

**Why separate `CreateUserRequest` from `User`?**
- When someone registers, they only provide: email, password, name
- The system generates: id, password_hash (from password), timestamps
- This separation keeps things clean and secure

### 2. **Wallet**
Represents a user's digital wallet (like a bank account).

**Fields:**
- `id` - Unique identifier for the wallet
- `user_id` - Links to the user who owns this wallet
- `balance` - How much money is in the wallet
- `currency` - Type of currency (USD, EUR, etc.)

**Why use `rust_decimal::Decimal` for money?**
- Regular floats (`f64`) have precision errors: `0.1 + 0.2 = 0.30000000000000004`
- With money, we need EXACT precision
- `Decimal` type ensures `$0.10 + $0.20 = $0.30` exactly

### 3. **Transaction**
Represents a money movement (deposit, withdrawal, transfer).

**Fields:**
- `id` - Unique identifier
- `wallet_id` - Which wallet this affects
- `transaction_type` - What kind: DEPOSIT, WITHDRAWAL, or TRANSFER
- `amount` - How much money
- `description` - Optional note (e.g., "Coffee purchase")
- `status` - PENDING, COMPLETED, or FAILED

## Request vs Response Structs

### Request Structs (What we receive from users)
- `CreateUserRequest` - Data to create a new user
- `LoginRequest` - Email + password for login
- `CreateTransactionRequest` - Data to create a transaction

### Response Structs (What we send back to users)
- `UserResponse` - User info WITHOUT password hash (security!)
- `WalletResponse` - Wallet info
- `TransactionResponse` - Transaction details
- `LoginResponse` - Token + user info after successful login

## Key Rust Concepts Used

### Attributes (the `#[...]` things)
- `#[derive(Debug)]` - Allows us to print the struct for debugging
- `#[derive(Serialize, Deserialize)]` - Converts to/from JSON
- `#[derive(FromRow)]` - Converts from database rows to structs

### `impl From<User> for UserResponse`
This is a **conversion function**. It takes a `User` and creates a `UserResponse` from it, automatically removing sensitive data like `password_hash`.

```rust
let user: User = get_user_from_db();
let response: UserResponse = user.into(); // Converts safely
```

## Next Steps

Now that we have our data structures defined, we can:
1. Create error handling (to handle things that go wrong)
2. Build the repository layer (to save/load this data from the database)
3. Implement authentication (login/register using these models)

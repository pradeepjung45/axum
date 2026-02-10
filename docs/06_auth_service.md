# Step 6: Auth Service - Explained

## What is the Service Layer?

The service layer contains **business logic** - the "brain" of your application.

### Layer Comparison

| Layer | Responsibility | Example |
|-------|---------------|---------|
| **Repository** | Database operations | `create_user()` - INSERT INTO users |
| **Service** | Business logic | `register()` - hash password, create user, create wallet, generate token |
| **Handler** | HTTP handling | Parse JSON, call service, return response |

## The Two Functions

### 1. `register()` - User Registration

**What it does:**
Orchestrates the entire registration process in 6 steps.

```rust
pub async fn register(
    pool: &PgPool,
    email: &str,
    password: &str,
    full_name: &str,
    jwt_secret: &str,
) -> Result<LoginResponse, AppError>
```

**Step-by-step breakdown:**

#### Step 1: Validate Input
```rust
if email.trim().is_empty() {
    return Err(AppError::validation("Email cannot be empty"));
}
if password.len() < 8 {
    return Err(AppError::validation("Password must be at least 8 characters"));
}
```

**Why validate here?**
- Catch errors early (before database operations)
- Consistent validation rules
- Better error messages for users

#### Step 2: Hash Password
```rust
let password_hash = hash_password(password)?;
```

**Security:** Never store plain passwords!

#### Step 3: Create User
```rust
let user = user_repo::create_user(pool, email, &password_hash, full_name).await?;
```

**What happens:**
- Inserts user into database
- Returns error if email already exists

#### Step 4: Create Wallet
```rust
let _wallet = user_repo::create_wallet(pool, user.id).await?;
```

**Business rule:** Every user automatically gets a wallet with $0.00 balance.

#### Step 5: Generate Token
```rust
let token = generate_token(user.id, jwt_secret)?;
```

**Why immediately?** User is logged in right after registration (better UX).

#### Step 6: Return Response
```rust
let user_response = UserResponse::from(user);
Ok(LoginResponse { token, user: user_response })
```

**Security:** Convert `User` to `UserResponse` to remove `password_hash`.

### 2. `login()` - User Login

**What it does:**
Authenticates a user and generates a token.

```rust
pub async fn login(
    pool: &PgPool,
    email: &str,
    password: &str,
    jwt_secret: &str,
) -> Result<LoginResponse, AppError>
```

**Step-by-step breakdown:**

#### Step 1: Find User
```rust
let user = user_repo::find_user_by_email(pool, email)
    .await
    .map_err(|_| AppError::InvalidCredentials)?;
```

**Security note:** We convert `NotFound` to `InvalidCredentials`.

**Why?**
- Don't reveal if email exists or not
- Prevents email enumeration attacks

#### Step 2: Verify Password
```rust
verify_password(password, &user.password_hash)?;
```

**What happens:**
- Hashes the provided password
- Compares to stored hash
- Returns error if they don't match

#### Step 3: Generate Token
```rust
let token = generate_token(user.id, jwt_secret)?;
```

Token expires in 24 hours.

#### Step 4: Return Response
```rust
Ok(LoginResponse { token, user: user_response })
```

Same format as registration response.

## Security Best Practices

### 1. Email Enumeration Prevention

**Bad approach:**
```rust
// Login attempt for "user@example.com"
if user not found:
    return "Email not found"  // ❌ Reveals email doesn't exist
if password wrong:
    return "Wrong password"   // ❌ Reveals email exists
```

**Good approach (ours):**
```rust
// Login attempt for "user@example.com"
if user not found OR password wrong:
    return "Invalid credentials"  // ✅ Doesn't reveal which is wrong
```

**Why this matters:**
Attackers can't use your login to discover valid email addresses.

### 2. Password Requirements

We enforce minimum 8 characters:
```rust
if password.len() < 8 {
    return Err(AppError::validation("Password must be at least 8 characters"));
}
```

**Could add more:**
- Require uppercase, lowercase, numbers, symbols
- Check against common password lists
- Implement password strength meter

### 3. Input Validation

We validate before database operations:
```rust
if email.trim().is_empty() {
    return Err(AppError::validation("Email cannot be empty"));
}
```

**Benefits:**
- Faster (no database call for invalid input)
- Better error messages
- Prevents database errors

## Complete Flow Examples

### Registration Flow
```
User submits:
  email: "alice@example.com"
  password: "mypassword123"
  full_name: "Alice Smith"
         ↓
Service validates input
         ↓
Service hashes password
  → "$argon2id$v=19$m=19456..."
         ↓
Repository creates user in DB
  → User { id: UUID, email, password_hash, ... }
         ↓
Repository creates wallet
  → Wallet { id: UUID, user_id, balance: 0.00, ... }
         ↓
Service generates JWT token
  → "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
         ↓
Returns to user:
  {
    "token": "eyJhbGci...",
    "user": {
      "id": "...",
      "email": "alice@example.com",
      "full_name": "Alice Smith",
      "created_at": "2024-..."
    }
  }
```

### Login Flow
```
User submits:
  email: "alice@example.com"
  password: "mypassword123"
         ↓
Repository finds user by email
  → User { id, email, password_hash, ... }
         ↓
Service verifies password
  hash("mypassword123") == stored hash?
  → Yes ✓
         ↓
Service generates JWT token
  → "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
         ↓
Returns same format as registration
```

## Error Scenarios

### Registration Errors

| Scenario | Error | HTTP Status |
|----------|-------|-------------|
| Email already exists | `UserAlreadyExists` | 409 Conflict |
| Password too short | `ValidationError` | 400 Bad Request |
| Empty email | `ValidationError` | 400 Bad Request |
| Database down | `DatabaseError` | 500 Internal Server Error |

### Login Errors

| Scenario | Error | HTTP Status |
|----------|-------|-------------|
| Email doesn't exist | `InvalidCredentials` | 401 Unauthorized |
| Wrong password | `InvalidCredentials` | 401 Unauthorized |
| Database down | `DatabaseError` | 500 Internal Server Error |

## Next Steps

Now we can implement:
1. **Handlers** - HTTP layer that calls these service functions
2. **Routes** - Define API endpoints (`POST /auth/register`, `POST /auth/login`)
3. **Main.rs** - Wire everything together and start the server

Then we can test in Postman!

Ready to continue?

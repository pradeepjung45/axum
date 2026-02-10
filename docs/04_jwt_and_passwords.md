# Step 4: JWT & Password Utilities - Explained

## What is JWT (JSON Web Token)?

Think of JWT as a **secure ID card** for your application.

### Real-World Analogy

Imagine a nightclub:
1. You show your ID at the door (login with email/password)
2. Bouncer gives you a wristband with your name and expiration time (JWT token)
3. You show the wristband to enter different areas (use token for API requests)
4. Wristband expires at closing time (token expires after 24 hours)

### How JWT Works

```
User logs in → Server creates JWT → User stores token → User sends token with requests
```

**JWT Structure:**
```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c
```

It has 3 parts (separated by `.`):
1. **Header** - Algorithm info
2. **Payload** - Claims (user_id, expiration)
3. **Signature** - Proves it's authentic

## The Claims Struct

```rust
pub struct Claims {
    pub sub: String,   // Subject (user ID)
    pub exp: usize,    // Expiration timestamp
    pub iat: usize,    // Issued at timestamp
}
```

**Standard JWT fields:**
- `sub` (subject) - Who the token is for (user ID)
- `exp` (expiration) - When the token expires
- `iat` (issued at) - When the token was created

## Token Generation

```rust
pub fn generate_token(user_id: Uuid, secret: &str) -> Result<String, AppError>
```

**What happens:**
1. Create claims with user_id and 24-hour expiration
2. Sign with our secret key
3. Return the token string

**Example:**
```rust
let token = generate_token(user.id, "my-secret-key")?;
// Returns: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## Token Validation

```rust
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, AppError>
```

**What happens:**
1. Decode the token
2. Verify signature (is it signed with our secret?)
3. Check expiration (is it still valid?)
4. Return claims if everything is OK

**Example:**
```rust
let claims = validate_token(&token, "my-secret-key")?;
let user_id = claims.user_id()?;
// Now we know which user this is!
```

## Password Hashing with Argon2

### Why Hash Passwords?

**NEVER store plain passwords!** If your database is hacked, all passwords are exposed.

**Instead:**
- Store a **hash** (one-way transformation)
- Can't reverse a hash to get the original password
- To verify: hash the input and compare to stored hash

### How Argon2 Works

```
Plain password: "mypassword123"
         ↓
     Argon2 hash
         ↓
Hash: "$argon2id$v=19$m=19456,t=2,p=1$randomsalt$hashedvalue"
```

**Key features:**
- **Slow** - Takes ~100ms to hash (makes brute-force attacks impractical)
- **Salt** - Random data added to each hash (same password = different hash)
- **Secure** - Winner of Password Hashing Competition

### Hashing a Password

```rust
pub fn hash_password(password: &str) -> Result<String, AppError>
```

**Example:**
```rust
let hash = hash_password("mypassword123")?;
// Returns: "$argon2id$v=19$m=19456,t=2,p=1$..."

// Store this hash in the database, NOT the plain password!
```

### Verifying a Password

```rust
pub fn verify_password(password: &str, hash: &str) -> Result<(), AppError>
```

**Example:**
```rust
// User tries to login with "mypassword123"
verify_password("mypassword123", &user.password_hash)?;
// Returns Ok(()) if correct, Err if wrong
```

## Complete Authentication Flow

### Registration
```rust
// 1. User submits email + password
let email = "user@example.com";
let password = "mypassword123";

// 2. Hash the password
let password_hash = hash_password(password)?;

// 3. Store user in database with hashed password
let user = create_user(email, password_hash).await?;

// 4. Generate JWT token
let token = generate_token(user.id, &config.jwt_secret)?;

// 5. Return token to user
```

### Login
```rust
// 1. User submits email + password
let email = "user@example.com";
let password = "mypassword123";

// 2. Get user from database
let user = get_user_by_email(email).await?;

// 3. Verify password
verify_password(password, &user.password_hash)?;

// 4. Generate JWT token
let token = generate_token(user.id, &config.jwt_secret)?;

// 5. Return token to user
```

### Protected Request
```rust
// 1. User sends token in request header
// Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

// 2. Validate token
let claims = validate_token(&token, &config.jwt_secret)?;

// 3. Get user ID from claims
let user_id = claims.user_id()?;

// 4. User is authenticated! Proceed with request
```

## Security Best Practices

### JWT Secret
- **Must be long** (at least 32 characters)
- **Must be random** (not a dictionary word)
- **Must be secret** (never commit to Git)
- **Stored in .env** (not in code)

### Token Expiration
- We use **24 hours** (good balance)
- Too short = users login too often (annoying)
- Too long = security risk if token is stolen

### Password Requirements
- Argon2 is slow (~100ms) - this is intentional!
- Makes brute-force attacks impractical
- Attacker can only try ~10 passwords per second

## Key Rust Concepts

### `Result<T, E>`
All functions return `Result` because they can fail:
```rust
pub fn generate_token(...) -> Result<String, AppError>
```

### Timestamps
```rust
let now = Utc::now();
let exp = now + Duration::hours(24);
exp.timestamp() // Converts to Unix timestamp (seconds since 1970)
```

### Error Mapping
```rust
.map_err(|_| AppError::InvalidCredentials)
```
Converts any error into our `AppError::InvalidCredentials`.

## Next Steps

Now we can:
1. **Build the Repository layer** - Database operations (create user, get user)
2. **Implement Auth Service** - Registration and login logic using these utilities
3. **Create Auth Middleware** - Automatically validate tokens on protected routes

Which would you like to tackle next?

# Step 3: Configuration Module - Explained

## What Is Configuration?

Configuration is how we load settings into our application. Instead of hardcoding values like database passwords in our code, we store them in environment variables.

**Why?**
- **Security**: Passwords and secrets stay out of version control (Git)
- **Flexibility**: Easy to change settings without recompiling
- **Environment-specific**: Different settings for development vs production

## The Config Struct

```rust
pub struct Config {
    pub database_url: String,    // Where to find the database
    pub jwt_secret: String,       // Secret for signing tokens
    pub server_host: String,      // Server IP address
    pub server_port: u16,         // Server port number
}
```

This struct holds all our app's configuration in one place.

## Loading from Environment Variables

### The `.env` File

Remember our `.env` file? It looks like this:

```
DATABASE_URL=postgresql://fintech_user:fintech_password@localhost:5433/fintech_db
JWT_SECRET=your-secret-key-change-this-in-production-min-32-chars
```

### How It Works

1. **dotenvy** reads the `.env` file
2. It loads the variables into the environment
3. We use `env::var()` to read them

```rust
let database_url = env::var("DATABASE_URL")
    .map_err(|_| AppError::internal("DATABASE_URL must be set"))?;
```

**What does this do?**
- Tries to read `DATABASE_URL` from environment
- If it doesn't exist, returns an error
- The `?` operator propagates the error up

## Required vs Optional Variables

### Required (will error if missing)
- `DATABASE_URL` - Can't connect to database without it
- `JWT_SECRET` - Can't create tokens without it

### Optional (have defaults)
- `SERVER_HOST` - Defaults to `"0.0.0.0"` (listen on all interfaces)
- `SERVER_PORT` - Defaults to `3000`

```rust
let server_host = env::var("SERVER_HOST")
    .unwrap_or_else(|_| "0.0.0.0".to_string());
```

This says: "Try to read SERVER_HOST, but if it's not set, use '0.0.0.0' instead."

## Security Validation

We validate that `JWT_SECRET` is at least 32 characters:

```rust
if jwt_secret.len() < 32 {
    return Err(AppError::internal(
        "JWT_SECRET must be at least 32 characters long"
    ));
}
```

**Why?** Short secrets are easy to crack. A 32+ character secret is much more secure.

## Database Connection Pool

### What Is a Connection Pool?

Imagine a restaurant:
- **Without pool**: Every customer waits for a new table to be built (slow!)
- **With pool**: Tables are ready and reused (fast!)

A connection pool keeps database connections ready to use.

```rust
pub async fn create_db_pool(database_url: &str) -> Result<PgPool, AppError> {
    PgPoolOptions::new()
        .max_connections(5)  // Keep 5 connections ready
        .connect(database_url)
        .await
        .map_err(|e| AppError::internal(&format!("Failed to connect: {}", e)))
}
```

### Why 5 Connections?

- For a small app, 5 is plenty
- Each connection uses memory
- PostgreSQL has a limit on total connections
- You can increase this for production

## How This Will Be Used

In `main.rs`, we'll do:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load config from .env
    let config = Config::from_env()?;
    
    // 2. Create database pool
    let db_pool = create_db_pool(&config.database_url).await?;
    
    // 3. Pass db_pool to all our handlers
    // 4. Start the server
    
    Ok(())
}
```

The `db_pool` will be shared across all requests using Axum's state management.

## Key Rust Concepts

### `Result<T, E>`
Every function that can fail returns a `Result`:
- `Ok(value)` - Success
- `Err(error)` - Failure

### The `?` Operator
```rust
let config = Config::from_env()?;
```

This is shorthand for:
```rust
let config = match Config::from_env() {
    Ok(c) => c,
    Err(e) => return Err(e),
};
```

### `async` Functions
```rust
pub async fn create_db_pool(...) -> Result<PgPool, AppError>
```

- `async` means this function does I/O (network, disk)
- Must be `await`ed: `let pool = create_db_pool(url).await?;`
- Allows other code to run while waiting

## Next Steps

Now we can:
1. **Implement JWT utilities** - Create and validate authentication tokens
2. **Build the repository layer** - Database operations using our pool
3. **Create services** - Business logic that uses config and database

Which would you like to tackle next?

# Step 7: Handlers & Routes - Explained

## What Are Handlers and Routes?

**Handlers** = Functions that handle HTTP requests  
**Routes** = URL paths that map to handlers

Think of it like a restaurant:
- **Routes** = Menu (what you can order)
- **Handlers** = Kitchen staff (who prepares your order)
- **Service** = Recipes (how to make the food)

## The Three Layers

| Layer | Responsibility | Example |
|-------|---------------|---------|
| **Handler** | HTTP handling | Parse JSON, extract state, return response |
| **Service** | Business logic | Validate, hash password, create user |
| **Repository** | Database | Execute SQL queries |

## Auth Handlers

### 1. `register_handler()`

**HTTP Endpoint:** `POST /auth/register`

```rust
pub async fn register_handler(
    State(pool): State<PgPool>,           // Extract database pool
    State(config): State<Config>,         // Extract config
    Json(req): Json<CreateUserRequest>,   // Parse JSON body
) -> Result<(StatusCode, Json<LoginResponse>), AppError>
```

**What it does:**
1. Extracts database pool from shared state
2. Extracts config from shared state
3. Parses JSON request body into `CreateUserRequest`
4. Calls `auth_service::register()`
5. Returns `201 Created` with `LoginResponse`

**Request example:**
```json
POST /auth/register
Content-Type: application/json

{
  "email": "alice@example.com",
  "password": "mypassword123",
  "full_name": "Alice Smith"
}
```

**Response example:**
```json
HTTP/1.1 201 Created
Content-Type: application/json

{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "alice@example.com",
    "full_name": "Alice Smith",
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

### 2. `login_handler()`

**HTTP Endpoint:** `POST /auth/login`

```rust
pub async fn login_handler(
    State(pool): State<PgPool>,
    State(config): State<Config>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError>
```

**What it does:**
1. Extracts state (same as register)
2. Parses JSON body into `LoginRequest`
3. Calls `auth_service::login()`
4. Returns `200 OK` with `LoginResponse`

**Request example:**
```json
POST /auth/login
Content-Type: application/json

{
  "email": "alice@example.com",
  "password": "mypassword123"
}
```

**Response example:**
```json
HTTP/1.1 200 OK
Content-Type: application/json

{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": { ... }
}
```

## Axum Extractors

Extractors are how Axum gets data from HTTP requests.

### Common Extractors

#### `State<T>` - Shared Application State
```rust
State(pool): State<PgPool>
```
Extracts shared state (database pool, config, etc.)

#### `Json<T>` - Parse JSON Body
```rust
Json(req): Json<CreateUserRequest>
```
Automatically parses JSON request body into a struct.  
Returns `400 Bad Request` if JSON is invalid.

#### `Path<T>` - URL Parameters
```rust
Path(user_id): Path<Uuid>
```
For routes like `/users/:id`

#### `Query<T>` - Query String
```rust
Query(params): Query<SearchParams>
```
For URLs like `/search?q=rust&page=1`

### How Extractors Work

```rust
pub async fn handler(
    State(pool): State<PgPool>,      // 1. Extract pool from state
    Json(req): Json<LoginRequest>,   // 2. Parse JSON body
) -> Result<Json<LoginResponse>, AppError> {
    // 3. Use the extracted data
    let response = auth_service::login(&pool, &req.email, &req.password).await?;
    Ok(Json(response))
}
```

**Axum automatically:**
- Extracts state
- Parses JSON (or returns 400 if invalid)
- Calls your handler
- Converts return value to HTTP response

## Routes

Routes map URLs to handlers.

### `auth_routes()`

```rust
pub fn auth_routes(pool: PgPool, config: Config) -> Router {
    Router::new()
        .route("/register", post(auth::register_handler))
        .route("/login", post(auth::login_handler))
        .with_state(pool.clone())
        .with_state(config)
}
```

**What this creates:**
- `POST /register` → `register_handler`
- `POST /login` → `login_handler`

### HTTP Methods

```rust
.route("/path", get(handler))     // GET
.route("/path", post(handler))    // POST
.route("/path", put(handler))     // PUT
.route("/path", delete(handler))  // DELETE
```

### Nested Routes

In `main.rs`, we'll nest these routes:
```rust
let app = Router::new()
    .nest("/auth", auth_routes(pool.clone(), config.clone()));
```

**Result:**
- `POST /auth/register`
- `POST /auth/login`

## Return Types

Handlers can return different types:

### 1. Just JSON (200 OK)
```rust
Result<Json<T>, AppError>
```
Returns `200 OK` with JSON body.

### 2. Custom Status Code
```rust
Result<(StatusCode, Json<T>), AppError>
```
Returns custom status (e.g., `201 Created`) with JSON body.

### 3. No Body
```rust
Result<StatusCode, AppError>
```
Returns just a status code (e.g., `204 No Content`).

## Error Handling

The `?` operator automatically converts errors to HTTP responses:

```rust
let response = auth_service::register(...).await?;
//                                              ↑
//                If this returns Err(AppError::UserAlreadyExists),
//                Axum converts it to:
//                HTTP 409 Conflict
//                { "error": "User with this email already exists", "status": 409 }
```

This works because we implemented `IntoResponse` for `AppError` in `error.rs`.

## Complete Request Flow

```
Client sends:
  POST /auth/register
  { "email": "alice@example.com", "password": "...", "full_name": "Alice" }
         ↓
Axum routing:
  Matches route "/auth/register" → register_handler
         ↓
Extractors:
  State(pool) → Extracts PgPool
  State(config) → Extracts Config
  Json(req) → Parses JSON into CreateUserRequest
         ↓
Handler:
  Calls auth_service::register(pool, email, password, name, secret)
         ↓
Service:
  Validates, hashes password, creates user, creates wallet, generates token
         ↓
Handler:
  Returns (StatusCode::CREATED, Json(LoginResponse))
         ↓
Axum:
  Converts to HTTP response
         ↓
Client receives:
  HTTP 201 Created
  { "token": "...", "user": { ... } }
```

## Next Steps

Now we need to create `main.rs` to:
1. Load configuration
2. Create database pool
3. Set up routes
4. Start the server on port 3000

Then we can test in Postman!

Ready to finish this?

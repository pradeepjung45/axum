# Step 8: Auth Middleware - Explained

## What is Middleware?

Middleware is code that runs **before** your handler.

Think of it like airport security:
- **Without middleware**: Every flight attendant checks your ID
- **With middleware**: Security checks once at the gate

## Our Auth Middleware: `AuthUser`

We created an **Axum extractor** that automatically validates JWT tokens.

### Code:
```rust
pub struct AuthUser(pub Uuid);
```

This is a **wrapper** around a user ID. When you use it in a handler, Axum automatically:
1. Extracts the `Authorization` header
2. Validates the JWT token
3. Returns the user ID

## How to Use It

### Protected Handler:
```rust
async fn get_me(
    AuthUser(user_id): AuthUser,  // ← Automatic JWT validation!
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, AppError> {
    // If we get here, user is authenticated!
    let user = user_repo::find_user_by_id(&state.pool, user_id).await?;
    Ok(Json(UserResponse::from(user)))
}
```

### What Happens:

**Client Request:**
```
GET /me
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Behind the Scenes:**
1. Axum sees `AuthUser` in handler signature
2. Calls `AuthUser::from_request_parts()`
3. Extracts `Authorization: Bearer <token>`
4. Validates token with JWT secret
5. Returns user ID from token
6. Handler runs with authenticated user_id

**If token is invalid:**
- Returns `401 Unauthorized`
- Handler never runs

## Error Cases

| Scenario | Error | Response |
|----------|-------|----------|
| No Authorization header | InvalidToken | 401 Unauthorized |
| Wrong format (not "Bearer ...") | InvalidToken | 401 Unauthorized |
| Invalid signature | InvalidToken | 401 Unauthorized |
| Expired token | InvalidToken | 401 Unauthorized |
| Malformed user ID | InvalidToken | 401 Unauthorized |

## The New Protected Endpoint

**GET** `/me` - Get your profile

**Headers:**
```
Authorization: Bearer <your-token>
```

**Response (200 OK):**
```json
{
  "id": "a8b29e5e-ba10-4e65-9d7a-bf296ad7322f",
  "email": "alice@example.com",
  "full_name": "Alice Smith",
  "created_at": "2026-02-02T22:09:11.208673Z"
}
```

**No token? (401 Unauthorized):**
```json
{
  "error": "Invalid or missing authentication token",
  "status": 401
}
```

## Routes Overview

**Public routes** (no token needed):
- `POST /register` - Create account
- `POST /login` - Get token

**Protected routes** (token required):
- `GET /me` - Get your profile

## How It's Different from Manual Validation

**Without middleware** (manual validation):
```rust
async fn get_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<UserResponse>, AppError> {
    // Manual token extraction
    let auth_header = headers.get("Authorization")...;
    let token = auth_header.strip_prefix("Bearer ")...;
    let claims = validate_token(token, &state.jwt_secret)?;
    let user_id = claims.user_id()?;
    
    // Finally, the actual logic
    let user = user_repo::find_user_by_id(&state.pool, user_id).await?;
    Ok(Json(UserResponse::from(user)))
}
```

**With middleware** (automatic):
```rust
async fn get_me(
    AuthUser(user_id): AuthUser,  // All validation happens here!
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, AppError> {
    // Just the actual logic
    let user = user_repo::find_user_by_id(&state.pool, user_id).await?;
    Ok(Json(UserResponse::from(user)))
}
```

Much cleaner! 🎉

## Testing

1. **Restart your server** (Ctrl+C, then `cargo run`)

2. **Get a token** (login or register):
```
POST /login
{
  "email": "alice@example.com",
  "password": "mypassword123"
}
```

3. **Test protected route** with token:
```
GET /me
Authorization: Bearer <your-token>
```

4. **Test without token** (should fail):
```
GET /me
(no Authorization header)
```

## Next Steps

Now you can protect **any route** by just adding `AuthUser`:
```rust
async fn get_wallet(
    AuthUser(user_id): AuthUser,  // Protected!
    State(state): State<AppState>,
) -> Result<Json<WalletResponse>, AppError> {
    let wallet = get_wallet_by_user_id(&state.pool, user_id).await?;
    Ok(Json(WalletResponse::from(wallet)))
}
```

Ready to test it?

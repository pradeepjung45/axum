# My Fintech App

A small fintech application built with Axum framework, implementing CRUD operations with JWT authentication.

## Project Structure

```
/my-fintech-app
├── Cargo.toml              # Dependencies and project configuration
├── .env                    # Environment variables (DB_URL, JWT_SECRET)
├── .gitignore              # Git ignore rules
├── migrations/             # SQLx or Diesel migrations
├── src/
│   ├── main.rs             # App entry point & router initialization
│   ├── lib.rs              # Library entry point
│   ├── config.rs           # Config loading (Env vars)
│   ├── error.rs            # Centralized error handling (AppError enum)
│   ├── routes/             # Route definitions (aggregates handlers)
│   │   └── mod.rs
│   ├── handlers/           # HTTP logic (Extractors, status codes)
│   │   └── mod.rs
│   ├── services/           # Business logic (The "Brain" of your app)
│   │   └── mod.rs
│   ├── domain/             # Core logic & Entities (Database-agnostic)
│   │   └── mod.rs
│   ├── repository/         # Data Access Layer (SQL queries)
│   │   └── mod.rs
│   ├── middleware/         # JWT Auth, Logging, CORS
│   │   └── mod.rs
│   └── utils/              # Hashing, JWT generation, validation
│       └── mod.rs
└── tests/                  # Integration tests
```

## Tech Stack

- **Framework**: Axum 0.7
- **Runtime**: Tokio
- **Database**: PostgreSQL with SQLx
- **Authentication**: JWT (jsonwebtoken)
- **Password Hashing**: Argon2
- **Validation**: Validator

## Setup

1. Update `.env` with your database credentials
2. Run `cargo build` to install dependencies
3. Set up database migrations (coming soon)
4. Run `cargo run` to start the server

## Next Steps

- Implement domain models
- Set up error handling
- Create JWT utilities
- Build authentication system
- Implement wallet/transaction features

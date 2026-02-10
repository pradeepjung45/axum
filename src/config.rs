use crate::error::AppError;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

// ============================================================================
// CONFIGURATION STRUCT
// ============================================================================
// This holds all the configuration values our app needs.
//
// Why do we need this?
// - To keep sensitive data (passwords, secrets) out of our code
// - To easily change settings between development and production
// - To have one central place for all configuration

#[derive(Debug, Clone)]
pub struct Config {
    /// PostgreSQL database connection URL
    /// Example: "postgresql://user:password@localhost:5433/fintech_db"
    pub database_url: String,
    
    /// Secret key for signing JWT tokens
    /// This MUST be kept secret! Anyone with this can create fake tokens.
    /// In production, use a long random string (at least 32 characters)
    pub jwt_secret: String,
    
    /// Server host address (e.g., "0.0.0.0" or "127.0.0.1")
    pub server_host: String,
    
    /// Server port (e.g., 3000)
    pub server_port: u16,
}

impl Config {
    /// Load configuration from environment variables
    /// 
    /// This reads from the .env file (thanks to dotenvy) and environment variables.
    /// 
    /// Returns an error if any required variable is missing.
    pub fn from_env() -> Result<Self, AppError> {
        // Load .env file into environment variables
        // This is safe to call even if .env doesn't exist
        dotenvy::dotenv().ok();
        
        // Read DATABASE_URL (required)
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::internal("DATABASE_URL must be set"))?;
        
        // Read JWT_SECRET (required)
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| AppError::internal("JWT_SECRET must be set"))?;
        
        // Validate JWT_SECRET length (should be at least 32 characters for security)
        if jwt_secret.len() < 32 {
            return Err(AppError::internal(
                "JWT_SECRET must be at least 32 characters long"
            ));
        }
        
        // Read SERVER_HOST (optional, defaults to "0.0.0.0")
        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());
        
        // Read SERVER_PORT (optional, defaults to 3000)
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|_| AppError::internal("SERVER_PORT must be a valid port number"))?;
        
        Ok(Config {
            database_url,
            jwt_secret,
            server_host,
            server_port,
        })
    }
    
    /// Get the full server address (host:port)
    /// Example: "0.0.0.0:3000"
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

// ============================================================================
// DATABASE CONNECTION POOL
// ============================================================================
// A connection pool manages multiple database connections efficiently.
//
// Why use a pool?
// - Creating a new connection for every request is slow
// - A pool reuses connections, making our app much faster
// - It limits the number of connections to avoid overwhelming the database

/// Create a database connection pool
///
/// This establishes connections to PostgreSQL and keeps them ready for use.
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string
///
/// # Returns
/// A connection pool that can be shared across the application
pub async fn create_db_pool(database_url: &str) -> Result<PgPool, AppError> {
    PgPoolOptions::new()
        .max_connections(5)  // Maximum number of connections in the pool
        .connect(database_url)
        .await
        .map_err(|e| {
            AppError::internal(&format!("Failed to connect to database: {}", e))
        })
}

// ============================================================================
// USAGE EXAMPLE (commented out)
// ============================================================================

/*
// In main.rs, you would use it like this:

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load configuration
    let config = Config::from_env()?;
    
    // 2. Create database pool
    let db_pool = create_db_pool(&config.database_url).await?;
    
    // 3. Start server
    println!("Server running on {}", config.server_address());
    
    Ok(())
}
*/

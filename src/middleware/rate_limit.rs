use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{net::SocketAddr, time::{Duration, Instant}};
use crate::routes::auth_routes::AppState;

const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60); // 1 minute
const MAX_REQUESTS: u32 = 20; // Max 20 requests per minute

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: axum::extract::Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let ip = addr.ip();
    
    // Check Rate Limit
    let allowed = {
        // LOCK THE MUTEX
        // This block ensures only one thread can update the map at a time
        let mut limiter = state.rate_limiter.lock().unwrap();

        let (count, reset_time) = limiter.entry(ip).or_insert((0, Instant::now()));

        if reset_time.elapsed() > RATE_LIMIT_WINDOW {
            // Window expired, reset counter
            *count = 1;
            *reset_time = Instant::now();
            true
        } else {
            // Window active, increment count
            if *count < MAX_REQUESTS {
                *count += 1;
                true
            } else {
                // Limit exceeded!
                false
            }
        }
    }; // Mutex is UNLOCKED here automatically when 'limiter' goes out of scope

    if allowed {
        Ok(next.run(req).await)
    } else {
        Err((
            StatusCode::TOO_MANY_REQUESTS,
            format!("Rate limit exceeded! Ongoing abuse detected from {}", ip),
        ))
    }
}

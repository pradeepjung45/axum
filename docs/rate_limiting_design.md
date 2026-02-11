# Rate Limiting Design
## 1. What is Rate Limiting?
Rate limiting is a strategy for limiting network traffic. It puts a cap on how often someone can repeat an action within a certain timeframe – for example, trying to log in to an account.

## 2. Which Algorithm are we using?
We will implement the **Fixed Window Counter** (a simplified version of Token Bucket).

**How it works:**
- We track the number of requests per **IP Address**.
- We define a **Window** (e.g., 1 minute).
- We define a **Limit** (e.g., 10 requests).
- If an IP makes > 10 requests in the current minute, we block them with `429 Too Many Requests`.

## 3. Where are we making changes?

We need to touch 3 main areas. Here is exactly what you can tell your senior:

### A. The State (Where we store the counts)
**File:** `src/routes/auth_routes.rs` (inside `AppState`)

We need a place to store "IP Address -> (Count, Timestamp)".
Since `AppState` is shared across all threads, we need to wrap this data in:
`Arc<Mutex<HashMap<IpAddr, (u32, Instant)>>>`

- **Arc**: Allows multiple requests to *own* a reference to this map.
- **Mutex**: Ensures only *one* request can update the count for an IP at a time (preventing race conditions).

### B. The Middleware (The Logic)
**File:** `src/middleware/rate_limit.rs` (New File)

We will create a new middleware function that:
1.  Extracts the user's IP address.
2.  Locks the Mutex to get access to the HashMap.
3.  Checks if the IP is new or existing.
    - If **New**: Add to map with count = 1.
    - If **Existing**:
        - Has the window expired? -> Reset count to 1.
        - Is count < Limit? -> Increment count.
        - Is count >= Limit? -> **REJECT** request.

### C. The Application Entry (Connecting it)
**File:** `src/main.rs`

We simply initialize the empty HashMap when the app starts and register the middleware layer so it runs for every request.

## 4. Why this approach?
- **In-Memory**: It's fast (no database calls).
- **Thread-Safe**: Uses Rust's `Mutex` to safely handle thousands of concurrent requests.
- **Simple**: Easy to understand and debug compared to Redis-based distributed rate limiters.

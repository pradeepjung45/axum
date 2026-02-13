# 🧠 Technical FAQ: The "How It Works"
## 1. Safety: "How does SQLx prevent SQL Injection?"
**The Question:** "You said SQLx is safe. But if I pass a user input string into a query, isn't that dangerous?"

**The Answer:**
"SQLx uses **Prepared Statements** (parameterization). It separates the *SQL Code* from the *Data*.
When we run:
```rust
sqlx::query("SELECT * FROM users WHERE email = $1")
    .bind(email) // <--- The Data
    .fetch_one(&pool)
```
The database receives the query template first: `SELECT * FROM users WHERE email = $1`. It compiles and optimizes this plan.
THEN, it sends the data separately. The database treats the input strictly as a *literal value*, never as executable code. Even if a user inputs `'; DROP TABLE users; --`, it is treated as a weird email string, not a command."

---

## 2. Speed: "Why is Axum + Tokio fast?"
**The Question:** "Rust is fast, but what makes this async stack special?"

**The Answer:**
"It's about **Non-Blocking I/O**.
- **Tokio** (the runtime) uses an Event Loop. Instead of dedicating 1 OS thread per user (which consumes ~2MB RAM each), it uses a small pool of threads (equal to CPU cores) to handle thousands of tasks.
- When `await` is called (e.g., waiting for DB), the thread *doesn't sleep*. It switches to handle another incoming request immediately.
- **Axum** is a thin layer over `hyper` (the fastest HTTP library in Rust). It compiles down to highly optimized state machines with zero overhead for abstractions."

---

## 3. Reliability: "How do Transactions ensure money is never lost?"
**The Question:** "What if the server crashes halfway through a transfer?"

**The Answer:**
"We use **ACID Transactions** provided by Postgres.
In `wallet_service.rs`, we do:
```rust
let mut tx = pool.begin().await?; // START TRANSACTION
// 1. Deduct from Alice
// 2. Add to Bob
tx.commit().await?; // COMMIT
```
This is **Atomic**.
- If Step 1 succeeds but Step 2 fails (or server crashes), the database performs a **ROLLBACK**.
- Alice's money comes back. The state reverts as if nothing ever happened.
- The `Tx` (Transaction) object in Rust ensures that we are holding a lock on those rows, preventing race conditions (Isolation)."

---

## 4. Security: "Explain the security layers."
**The Question:** "Walk me through how you secured this."

**The Answer:**
1.  **Passwords (Argon2)**: "We don't store passwords. We store a *hash* using Argon2 via the `argon2` crate. It's memory-hard, making it resistant to GPU brute-force attacks."
2.  **Authentication (JWT)**: "We issue a signed JSON Web Token. It contains the User ID and Expiration. The server signs it with a `JWT_SECRET`. If a user tampers with the token, the signature verification fails immediately."
3.  **Rate Limiting**: "We track IP addresses in an `Arc<Mutex<HashMap>>`. If an IP exceeds 20 requests/minute, we return 429. This prevents brute-force login attempts."

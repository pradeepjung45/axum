# 🎓 Mentor Presentation Guide
## "My Fintech App: Architecture & Implementation"

This document is your script. Use it to explain **WHAT** we built, **WHY** we made specific choices, and **HOW** it works.

---

## 1. The Tech Stack & "The Downgrade"
**Mentor Question:** "Why did you use these specific versions?"

**Your Answer:**
"We built this using **Axum 0.7**, which is the latest and greatest web framework for Rust.
However, we faced a compatibility issue with `axum-extra` (used for cookies). The latest `axum-extra` required a newer `axum` version than what was stable, causing a **dependency conflict**.
**The Fix:** We pinned `axum-extra` to version `0.9.0` to match `axum 0.7.5`. This ensured all our cookie handling logic worked without breaking the build."

---

## 2. The Architecture (3-Layer Pattern)
**Mentor Question:** "How is your code organized?"

**Your Answer:**
"I followed the industry-standard **Service-Repository Pattern** to keep code clean and testable."

### 🏛️ The 3 Layers:
1.  **Handler Layer (`src/handlers/`)**:
    *   **Role:** The "Front Desk". Accepts HTTP requests (JSON or Form data).
    *   **What it does:** extracting data, validating inputs, calling the Service.
    *   **Example:** `web::login_submit` takes the form, calls `auth_service::login`, sets the cookie, and redirects.
2.  **Service Layer (`src/services/`)**:
    *   **Role:** The "Brain". valid business logic.
    *   **What it does:** Hashes passwords, calculates balances, checks for sufficient funds.
    *   **Example:** `wallet_service::transfer` checks if you have \$50 before letting you send it.
3.  **Repository Layer (`src/repository/`)**:
    *   **Role:** The "Storage". talks to the Database.
    *   **What it does:** Executes raw SQL queries using `sqlx`.
    *   **Example:** `user_repo::create_user` runs `INSERT INTO users...`.

---

## 3. The Data Flow (A "Day in the Life" of a Request)
**Mentor Question:** "Trace a Transfer request for me."

**Your Answer:**
"Let's follow a \$10 transfer from Alice to Bob:"

1.  **Request**: Browser sends `POST /dashboard/transfer` with `amount=10`.
2.  **Middleware (`rate_limit.rs`)**:
    *   Intercepts the request.
    *   checks **IP Address** against our `Arc<Mutex<HashMap>>`.
    *   If allowed, passes it on.
3.  **Router (`main.rs`)**: Matches path `/dashboard/transfer` to handler `transfer_submit`.
4.  **Handler (`web.rs`)**:
    *   Extracts `AuthUser` (validates JWT cookie).
    *   Extracts `Form` data.
    *   Calls `wallet_service::transfer()`.
5.  **Service (`wallet_service.rs`)**:
    *   Starts a **DB Transaction** (essential for money!).
    *   Deducts \$10 from Alice.
    *   Adds \$10 to Bob.
    *   Records the transaction record.
    *   Commits the transaction.
6.  **Response**: Handler returns `HX-Redirect` to send user back to dashboard.

---

## 4. Advanced Concepts (Rust Specifics)
**Mentor Question:** "Show me something advanced."

**Your Answer:**
"I implemented **Rate Limiting** using Rust's concurrency primitives:"

*   **`Arc` (Atomic Reference Counting)**: Allows our `AppState` (and the rate limiter) to be shared across thousands of concurrent web requests safely.
*   **`Mutex` (Mutual Exclusion)**: Protects our rate limit counter. It ensures that if two requests hit at the exact same nanosecond, they don't corrupt the data. We lock, update, and unlock.

---

## 5. Frontend Integration
**Mentor Question:** "How does the frontend talk to the backend?"

**Your Answer:**
"We used **HTMX** for a modern, SPA-like feel without writing complex JavaScript."
- **Forms**: Instead of standard submission, we use `hx-post="/login"`.
- **Updates**: The server returns partial HTML or an `HX-Redirect` header.
- **Why**: It keeps our logic in Rust (the backend) rather than splitting it between Rust and React/Vue.

---

## 🔍 Summary Checklist
- [x] **Safe**: SQLx prevents SQL injection.
- [x] **Fast**: Axum + Tokio handles async requests.
- [x] **Reliable**: DB Transactions ensure money is never lost.
- [x] **Secure**: Rate Limiting + JWT Auth + Argon2 Password Hashing.


# Freelancia Backend (Rust + Axum)

Freelancia is a modern freelance marketplace platform. This is the backend API built using **Rust** and **Axum**, focusing on speed, safety, and scalability.

---

## 🚧 Current Progress

### ✅ Project Setup (May 14)
- Initialized `freelancia_backend` using Cargo
- Installed core dependencies:
  - `axum`, `tokio`, `serde`, `serde_json`, `tower`, `tracing`, `dotenvy`, `tower-http`
- Created modular structure:
  - `routes/`, `handlers/`, `models/`, `config/`

### ✅ Basic Routes
- `GET /health` – Health check
- `GET /users` – Fetch sample users
- `POST /users` – Accept JSON payload

### ✅ Middleware
- Implemented request logging via `tower_http::trace::TraceLayer`

---

### ✅ Authentication (May 15)
- Integrated JWT-based auth system
- Created `auth_middleware.rs` to extract and validate JWTs
- Added protected route: `GET /profile`
- Middleware checks and injects `AuthUser` from token
- `.env` used for managing secret keys securely

---
## 🚀 Project Progress (May 16)

### ✅ Backend Setup & Core Features Implemented

- **Auth System (Axum)**
  - `/signup` and `/login` endpoints.
  - JWT generation with token-based authentication.
  - Middleware for protected routes (`auth_required`, `client_only`, `freelancer_only`).
  - **Hybrid Authentication Support**:
    - Email/password login implemented.
    - Wallet login structure added (signature verification TODO).
- **User Model Enhancements**
  - New fields: `wallet_address`, `verified_wallet`, `role` (client/freelancer).

- **Job System (Basic)**
  - Job schema/table added to DB.
  - `POST /job`: Create job (Client only).
  - `GET /job`: View all jobs (Freelancer only).

  ### 🗓️ May 17 Progress

- Implemented secure wallet verification using EIP-712:
  - `/auth/wallet/request-nonce` and `/auth/wallet/verify` endpoints.
  - Prevents duplicate wallet verification attempts.

- Added hybrid authentication:
  - Supports both traditional email/password and wallet-based login.
  - Role-specific validation during signup.

- Added middleware for role-based access control:
  - Routes like `/profile/verified` restricted to verified wallet roles.

- Introduced `AppError::BadRequest` to handle invalid logic clearly.

- Added inline documentation to explain logic and edge cases in:
  - `auth.rs`, `job.rs`, `middleware.rs`, and `utils.rs`.

- Ensured code readability and future maintainability.

### ✅ May 18, 2025 – Daily Update

- Implemented advanced job filtering via `GET /api/jobs`
  - FTS5 full-text search on title/description
  - Single-skill filtering with dynamic sorting
  - JWT auth middleware protected

- Added secure logout functionality:
  - `POST /api/logout` with JWT blacklisting
  - Introduced `blacklisted_tokens` table
  - Optimized queries using index

- Fixed key issues:
  - Claims.exp conversion to i64 for SQLx encoding
  - Middleware borrow resolution using owned JWTs
  - Verified filtering and auth logic via Postman

> Stack: Rust, Axum, SQLx 0.7.x, SQLite FTS5, JWT, Postman

### ✅ May 19, 2025 – Daily Update

- Implemented full job proposal system:
  - Created `proposals` table with fields:
    - `job_id` (foreign key to `jobs`)
    - `freelancer_id` (foreign key to `users`)
    - `cover_letter`, `bid_amount`, `status`, `created_at`
  - Used SQLx with SQLite for schema setup and migrations

- Developed Proposal API Endpoints:
  - `POST /api/proposals`: freelancers can submit proposals
  - `GET /api/proposals/job/:job_id`: clients view proposals for their own jobs
  - `PATCH /api/proposals/:id`: clients accept/reject proposals
  - `GET /api/proposals/me`: freelancers view and filter their submitted proposals

- Added role-based access control:
  - Only authenticated freelancers can submit proposals
  - Only job owners (clients) can view/modify their job’s proposals

- Refactored and documented:
  - Added detailed comments in proposal handlers for clarity
  - Verified proposal logic and edge cases using Postman

> **Stack**: Rust, Axum, SQLx 0.7.x, SQLite, JWT, Postman


## 🔧 Build and Compile
```bash
cargo build
```

## 🔧 Run the Server

```bash
cargo run --bin freelancia backend
```

## 🔧 Run the test

```bash
cargo test
```


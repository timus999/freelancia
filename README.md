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

## 🔧 Run the Server

```bash
cargo run

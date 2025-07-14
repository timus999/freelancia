
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

## 🔗 Smart Contract Integration (Anchor + Solana)

We’ve integrated a basic [Anchor](https://book.anchor-lang.com/) smart contract as part of Freelancia’s decentralized escrow and job management system.

### 🧱 Anchor Program Overview

The initial Anchor program is deployed and tested locally. It will be responsible for:

- Escrow contract logic (job payment locking)
- Verifiable proposal acceptance between client and freelancer
- Ensuring funds are released only upon agreement or milestone approval

# 🔐 Escrow Smart Contract (Anchor Program)

This Anchor-based Solana smart contract implements a **secure and trustless escrow system** for native SOL transfers between a _client (maker)_ and a _freelancer (taker)_, with optional arbitration and automatic resolution.

---

## 📦 Overview

The escrow contract facilitates a decentralized workflow with the following phases:
### 1. 🚀 Escrow Creation (`create_escrow`)

**Role:** Maker (client)  
**Purpose:** Initialize a new escrow by locking funds securely into a program-controlled vault.

#### ✅ Preconditions:
- Deadline and auto-release timestamps must be valid and increasing
- Maker must sign the transaction
- Vault PDA is derived and funded accordingly

#### 🔄 State Changes:
- Creates and initializes the escrow account with all metadata
- Creates a vault PDA account holding the locked funds (rent-exempt)
- Funds equal to `amount` + rent are transferred into the vault
- Escrow status set to `Active`

#### 🧾 Arguments:
- `escrow_id: u64` — Unique identifier for the escrow
- `amount: u64` — Total funds locked (in lamports)
- `deadline: i64` — Timestamp by which work must be completed
- `auto_release_at: i64` — Timestamp after which funds auto-release to taker if no dispute
- `spec_hash: [u8; 32]` — Hash of the job specification or contract details
- `arbiter: Option<Pubkey>` — Optional arbiter public key for dispute resolution

#### 📦 Accounts:
| Name         | Type        | Required | Description                       |
|--------------|-------------|----------|----------------------------------|
| maker        | `Signer`    | ✅       | Client funding and creating escrow |
| taker        | `AccountInfo` | ✅     | Freelancer assigned to the escrow |
| escrow       | `Account`   | ✅       | Newly created escrow account       |
| vault        | `AccountInfo` | ✅     | PDA account holding locked funds   |
| system_program | `Program`  | ✅       | System program for account creation and transfers |

---

### 2. 📤 Work Submission (`submit_work`)

**Role:** Taker (freelancer)  
**Purpose:** Submit proof of completed work by recording a deliverable hash on-chain.

#### ✅ Preconditions:
- Escrow must be in `Active` state
- Only the `taker` can submit work
- Deliverable hash is a valid 32-byte hash of the submitted work

#### 🔄 State Changes:
- Updates escrow status to `Submitted`
- Stores the `deliverable_hash` on-chain for maker's review

#### 🧾 Arguments:
- `deliverable_hash: [u8; 32]` — Hash representing the completed deliverable content

#### 📦 Accounts:
| Name   | Type        | Required | Description                      |
|--------|-------------|----------|---------------------------------|
| taker  | `Signer`    | ✅       | Freelancer submitting work       |
| escrow | `Account`   | ✅       | Escrow account being updated     |

---

### 3. ✅ Work Approval (`approve_work`)

**Role:** Maker (client)  
**Purpose:** Release escrowed funds to the taker upon approval of submitted work.

#### ✅ Preconditions:
- Escrow must be in `Submitted` state
- Only the `maker` can approve
- Sufficient funds available in the vault

#### 🔄 State Changes:
- Transfers full remaining funds from vault to taker
- Updates escrow status to `Completed`
- Records completion timestamp

#### 🧾 Arguments: _None_

#### 📦 Accounts:
| Name           | Type         | Required | Description                     |
|----------------|--------------|----------|---------------------------------|
| maker          | `Signer`     | ✅       | Client approving and releasing funds |
| taker          | `AccountInfo`| ✅       | Freelancer receiving funds       |
| escrow         | `Account`    | ✅       | Escrow account to update         |
| vault          | `AccountInfo`| ✅       | PDA vault holding the funds      |
| system_program | `Program`    | ✅       | System program for transfers     |

---

### 🔁 4. `request_revision`

**Role:** Maker (client)  
**Purpose:** Reject the submitted deliverables and revert the escrow back to an active state for revision.

#### ✅ Preconditions:
- Escrow must be in `Submitted` state
- Only the `maker` (client) can request a revision

#### 🔄 State Changes:
- Escrow status changes back to `Active`
- Increments `revision_requests` by 1

#### 🧾 Arguments: _None_

#### 📦 Accounts:
| Name   | Type         | Required | Description                      |
|--------|--------------|----------|----------------------------------|
| maker  | `Signer`     | ✅       | Client requesting revision       |
| escrow | `Account`    | ✅       | Escrow account to modify         |

---

### ⚖️ 5. `raise_dispute`

**Role:** Maker (client) or Taker (freelancer)  
**Purpose:** Escalate the escrow to a dispute state by providing hashed evidence (e.g., an IPFS hash of a document).

#### ✅ Preconditions:
- Escrow must be in `Active` or `Submitted` state
- Caller must be either `maker` or `taker`

#### 🔄 State Changes:
- Escrow status becomes `Disputed`
- `dispute_evidence_uri_hash` is recorded

#### 🧾 Arguments:
- `evidence_uri_hash: [u8; 32]` — A 32-byte hash (typically SHA-256) representing off-chain dispute evidence

#### 📦 Accounts:
| Name    | Type      | Required | Description                          |
|---------|-----------|----------|--------------------------------------|
| caller  | `Signer`  | ✅       | Must be either the maker or taker    |
| escrow  | `Account` | ✅       | Escrow account to dispute            |

---

### 👩‍⚖️ 6. `arbiter_resolve`

**Role:** Arbiter  
**Purpose:** Allows the assigned arbiter to resolve a dispute by splitting remaining funds between the taker (freelancer) and maker (client).

#### ✅ Preconditions:
- Escrow must be in `Disputed` state
- Arbiter must match the one specified during `create_escrow`
- Combined amount must not exceed vault balance
- At least one of the amounts must be > 0

#### 🔄 State Changes:
- Transfers specified lamports from the vault to each party
- Updates `amount_released` and `amount_refunded`
- Escrow status becomes `Completed`
- Sets `completed_at` timestamp

#### 🧾 Arguments:
- `taker_amount: u64` — Amount (in lamports) to release to taker
- `maker_amount: u64` — Amount (in lamports) to refund to maker

#### 📦 Accounts:
| Name            | Type       | Required | Description                                |
|-----------------|------------|----------|--------------------------------------------|
| arbiter         | `Signer`   | ✅       | Arbiter assigned in the escrow             |
| maker           | `AccountInfo` | ✅   | Recipient of refunded amount (if any)      |
| taker           | `AccountInfo` | ✅   | Recipient of released amount (if any)      |
| escrow          | `Account`  | ✅       | Escrow to resolve                          |
| vault           | `AccountInfo` | ✅   | PDA vault holding SOL                      |
| system_program  | `Program`  | ✅       | System Program (for transfers)             |

---

---

### ❌ 7. `cancel_before_start`

**Role:** Maker (client)  
**Purpose:** Cancel the escrow before the taker has submitted any work, and refund all locked funds back to the maker.

#### ✅ Preconditions:
- Escrow must be in `Active` state
- `amount_released` must be 0
- Caller must be the `maker`
- Vault must hold funds

#### 🔄 State Changes:
- Transfers all locked funds from vault back to the maker
- Escrow status is set to `Cancelled`
- Updates `amount_refunded`

#### 🧾 Arguments: _None_

#### 📦 Accounts:
| Name            | Type          | Required | Description                             |
|-----------------|---------------|----------|-----------------------------------------|
| maker           | `Signer`      | ✅       | Creator of the escrow                   |
| escrow          | `Account`     | ✅       | The escrow account to cancel            |
| vault           | `AccountInfo` | ✅       | PDA vault holding locked funds          |
| system_program  | `Program`     | ✅       | System program to perform transfers     |

---

### ⏱️ 8. `claim_timeout`

**Role:** Conditional – Maker or Taker  
**Purpose:** Allows either party to claim funds if the other fails to act within allowed timeframes.

---

#### 🧭 Scenario A: Maker Claims Refund (No Work Submitted)

- **Condition**: `escrow.status == Active && now > deadline`
- **Caller**: Must be `maker`
- **Effect**:
  - Vault funds are refunded to maker
  - `status → Cancelled`

---

#### 🧭 Scenario B: Taker Claims Payment (Work Submitted, Maker Silent)

- **Condition**: `escrow.status == Submitted && now > auto_release_at`
- **Caller**: Must be `taker`
- **Effect**:
  - Vault funds are released to taker
  - `status → Completed`
  - `completed_at` is set

---

#### ✅ Preconditions:
- Escrow must be in valid state (`Active` or `Submitted`)
- Timestamp conditions must be met (deadline or auto_release_at)
- Caller must match the required role (`maker` or `taker`)
- Vault must contain funds

#### 🔄 State Changes:
- Transfers full unreleased/refundable amount from vault
- Updates either `amount_refunded` or `amount_released`
- Sets status to `Cancelled` or `Completed`

#### 🧾 Arguments: _None_

#### 📦 Accounts:
| Name            | Type          | Required | Description                                      |
|-----------------|---------------|----------|--------------------------------------------------|
| claimant        | `Signer`      | ✅       | Either the `maker` or `taker`                    |
| escrow          | `Account`     | ✅       | Escrow being resolved                            |
| vault           | `AccountInfo` | ✅       | PDA vault containing locked funds                |
| system_program  | `Program`     | ✅       | System program to transfer SOL from the vault    |

---



### 📦 Program Details

- **Language**: Rust
- **Framework**: Anchor
- **Solana Cluster**: Localhost / Devnet
- **Program Name**: `escrow`

### ⚙️ How to Run the Program Locally

```bash
anchor build
anchor test
```

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


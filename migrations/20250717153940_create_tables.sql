-- Add migration script here

--users
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT UNIQUE,
    password TEXT, -- Hashed, nullable for Web3 users
    wallet_address TEXT UNIQUE,
    role TEXT NOT NULL CHECK ( role IN ('freelancer', 'client')),
    wallet_user BOOLEAN NOT NULL DEFAULT FALSE,
    verified_wallet BOOLEAN NOT NULL DEFAULT FALSE,
    admin BOOLEAN NOT NULL DEFAULT FALSE
);

-- Jobs
CREATE TABLE IF NOT EXISTS jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    skills TEXT NOT NULL, -- comma-separated or JSON if needed
    budget INTEGER NOT NULL,
    location TEXT NOT NULL,
    job_type TEXT NOT NULL,
    job_ipfs_hash TEXT NOT NULL,
    posted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deadline TEXT NOT NULL,
    client_id INTEGER NOT NULL,
    category TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('open','closed','submitted', 'completed')),
    FOREIGN KEY(client_id) REFERENCES users(id)
);

-- Bids
-- CREATE TABLE IF NOT EXISTS bids (
--     id INTEGER PRIMARY KEY AUTOINCREMENT,
--     job_id INTEGER NOT NULL,
--     freelancer_id TEXT NOT NULL,
--     timeline TEXT NOT NULL,
--     budget INTEGER NOT NULL,
--     message TEXT NOT NULL,
--     FOREIGN KEY(job_id) REFERENCES jobs(id),
--     FOREIGN KEY(freelancer_id) REFERENCES users(id)
-- );

-- Profiles
CREATE TABLE IF NOT EXISTS profiles (
    user_id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('freelancer', 'client')),
    bio TEXT,
    skills TEXT, -- comma-separated
    certifications TEXT,
    work_history TEXT,
    profile_ipfs_hash TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

-- CREATE TABLE IF NOT EXISTS reviews (
--     id INTEGER PRIMARY KEY AUTOINCREMENT,
--     job_id INTEGER NOT NULL,
--     reviewer_id INTEGER NOT NULL,
--     rating INTEGER NOT NULL CHECK(rating BETWEEN 1 AND 5),
--     review TEXT,
--     review_ipfs_hash TEXT NOT NULL,
--     FOREIGN KEY (job_id) REFERENCES jobs(id),
--     FOREIGN KEY (reviewer_id) REFERENCES users(id)
-- );

--nonces table
CREATE TABLE IF NOT EXISTS nonces (
    wallet_address TEXT NOT NULL,
    nonce TEXT NOT NULL,
    created_at TEXT NOT NULL, -- ISO 8601
    expires_at TEXT NOT NULL, -- ISO 8601
    PRIMARY KEY (wallet_address, nonce)
);


-- FTS5 virtual table to index title and description
CREATE VIRTUAL TABLE jobs_fts USING fts5(
    title,
    description,
    job_id UNINDEXED,
    tokenize = 'porter unicode61'
);

CREATE TRIGGER jobs_insert AFTER INSERT ON jobs
BEGIN
    INSERT INTO jobs_fts (job_id, title, description)
    VALUES (new.id, new.title, new.description);
END;

CREATE TRIGGER jobs_update AFTER UPDATE ON jobs
BEGIN
    UPDATE jobs_fts
    SET title = new.title,
        description = new.description
    WHERE job_id = new.id;
END;

CREATE TRIGGER jobs_delete AFTER DELETE ON jobs
BEGIN
    DELETE FROM jobs_fts WHERE job_id = old.id;
END;

--blacklisted_token table
CREATE TABLE blacklisted_tokens (
    token TEXT PRIMARY KEY,
    expires_at INTEGER NOT NULL
);
CREATE INDEX idx_blacklisted_tokens_expires_at ON blacklisted_tokens(expires_at);

--Proposals table
CREATE TABLE IF NOT EXISTS proposals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id INTEGER NOT NULL,
    freelancer_id INTEGER NOT NULL,
    cover_letter TEXT NOT NULL,
    bid_amount REAL NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('submitted', 'accepted', 'rejected')),
    created_at INTEGER NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    FOREIGN KEY (freelancer_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS job_applications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    job_id INTEGER NOT NULL,
    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    approved BOOLEAN DEFAULT 0,
    approved_at TEXT,
    freelancer_wallet TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE,
    UNIQUE(user_id, job_id) -- A user can only apply once to a job
);

CREATE TABLE IF NOT EXISTS saved_jobs (
    user_id INTEGER NOT NULL,
    job_id INTEGER NOT NULL,
    saved_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, job_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (job_id) REFERENCES jobs(id) ON DELETE CASCADE
);


CREATE TABLE IF NOT EXISTS job_deliverables (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    application_id INTEGER NOT NULL UNIQUE, -- Links to approved application
    ipfs_hash TEXT NOT NULL,
    submitted BOOLEAN NOT NULL DEFAULT 0,
    submitted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    disputed BOOLEAN DEFAULT 0,
    disputed_at TEXT,
    timeout_claimed BOOLEAN DEFAULT 0,
    timeout_claimed_at TEXT,
    review_requested BOOLEAN DEFAULT 0,
    review_requested_at TEXT,
    cancelled BOOLEAN DEFAULT 0,
    cancelled_at TEXT,
    resolved TEXT DEFAULT 0,
    arbiter_id INTEGER REFERENCES users(id),
    FOREIGN KEY (application_id) REFERENCES job_applications(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,                 -- the recipient (job creator)
    message TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    type TEXT DEFAULT 'generic',
    job_id INTEGER,
    actor_id INTEGER,
    escrow_pda TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE job_user_interactions (
    user_id     INTEGER NOT NULL,
    job_id      INTEGER NOT NULL,
    applied     BOOLEAN DEFAULT FALSE,
    saved       BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (user_id, job_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (job_id)  REFERENCES jobs(id)
);

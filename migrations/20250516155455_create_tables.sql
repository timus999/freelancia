
--users
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    password TEXT, -- Hashed, nullable for Web3 users
    wallet_address TEXT UNIQUE,
    role TEXT NOT NULL,
    verified_wallet BOOLEAN NOT NULL DEFAULT FALSE
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
    FOREIGN KEY(client_id) REFERENCES users(id)
);

-- Bids
CREATE TABLE IF NOT EXISTS bids (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id INTEGER NOT NULL,
    freelancer_id TEXT NOT NULL,
    timeline TEXT NOT NULL,
    budget INTEGER NOT NULL,
    message TEXT NOT NULL,
    FOREIGN KEY(job_id) REFERENCES jobs(id),
    FOREIGN KEY(freelancer_id) REFERENCES users(id)
);

-- Profiles
CREATE TABLE IF NOT EXISTS profiles (
    user_id TEXT PRIMARY KEY,
    username TEXT,
    bio TEXT,
    skills TEXT NOT NULL,
    certifications TEXT,
    work_history TEXT,
    profile_ipfs_hash TEXT NOT NULL,
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


FTS5 virtual table to index title and description
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

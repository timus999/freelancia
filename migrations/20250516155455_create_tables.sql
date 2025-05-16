-- Add migration script here
-- Add migration script here
-- Add migration script here
-- migrations/20250516120000_create_jobs_bids_profiles.sql

-- Users (already exist, but included here for FK reference)
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    password TEXT, -- Hashed, nullable for Web3 users
    wallet_address TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL CHECK(role IN ('freelancer', 'client')),
    verified_wallet BOOLEAN DEFAULT FALSE
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
    client_id TEXT NOT NULL,
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

CREATE TABLE IF NOT EXISTS reviews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id INTEGER NOT NULL,
    reviewer_id INTEGER NOT NULL,
    rating INTEGER NOT NULL CHECK(rating BETWEEN 1 AND 5),
    review TEXT,
    review_ipfs_hash TEXT NOT NULL,
    FOREIGN KEY (job_id) REFERENCES jobs(id),
    FOREIGN KEY (reviewer_id) REFERENCES users(id)
);

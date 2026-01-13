-- Initial database schema for Profile Pulse
-- Migration: 20250113_001_initial_schema

-- Contacts table - stores primary contact information
CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    organization TEXT,
    title TEXT,
    photo_url TEXT,
    photo_blob BLOB,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_contacts_name ON contacts(name);
CREATE INDEX IF NOT EXISTS idx_contacts_email ON contacts(email) WHERE email IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_contacts_updated ON contacts(updated_at DESC);

-- Social profiles table - stores social media profile information
CREATE TABLE IF NOT EXISTS social_profiles (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    username TEXT NOT NULL,
    url TEXT NOT NULL,
    profile_pic_url TEXT,
    verified BOOLEAN DEFAULT 0 NOT NULL,
    confidence_score REAL,
    discovered_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_social_profiles_contact ON social_profiles(contact_id);
CREATE INDEX IF NOT EXISTS idx_social_profiles_platform ON social_profiles(platform);
CREATE UNIQUE INDEX IF NOT EXISTS idx_social_profiles_unique ON social_profiles(contact_id, platform, username);

-- Custom fields table - stores additional VCF fields and custom data
CREATE TABLE IF NOT EXISTS custom_fields (
    contact_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (contact_id, key),
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_custom_fields_contact ON custom_fields(contact_id);

-- Fetch queue table - manages profile picture and data fetching operations
CREATE TABLE IF NOT EXISTS fetch_queue (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    username TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'in_progress', 'success', 'failed', 'skipped')),
    priority INTEGER DEFAULT 0 NOT NULL,
    retry_count INTEGER DEFAULT 0 NOT NULL,
    last_attempt INTEGER,
    error_message TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_fetch_queue_status ON fetch_queue(status);
CREATE INDEX IF NOT EXISTS idx_fetch_queue_priority ON fetch_queue(priority DESC, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_fetch_queue_contact ON fetch_queue(contact_id);

-- Cache table - stores HTTP responses and profile picture data
CREATE TABLE IF NOT EXISTS fetch_cache (
    key TEXT PRIMARY KEY NOT NULL,
    data BLOB,
    content_type TEXT,
    cached_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    hit_count INTEGER DEFAULT 0 NOT NULL,
    last_accessed INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_fetch_cache_expires ON fetch_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_fetch_cache_accessed ON fetch_cache(last_accessed);

-- Rate limits table - tracks API rate limit usage per platform
CREATE TABLE IF NOT EXISTS rate_limits (
    platform TEXT PRIMARY KEY NOT NULL,
    requests_made INTEGER DEFAULT 0 NOT NULL,
    window_start INTEGER NOT NULL,
    last_request INTEGER,
    daily_quota INTEGER NOT NULL,
    hourly_quota INTEGER NOT NULL
);

-- Settings table - stores application configuration
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Insert default settings
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES
    ('app_version', '0.1.0', strftime('%s', 'now')),
    ('db_version', '1', strftime('%s', 'now')),
    ('cache_ttl_days', '7', strftime('%s', 'now')),
    ('rate_limit_enabled', 'true', strftime('%s', 'now'));
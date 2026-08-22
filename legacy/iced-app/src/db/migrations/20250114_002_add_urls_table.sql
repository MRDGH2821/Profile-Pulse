-- Migration: 20250114_002_add_urls_table
-- Add contact_urls table and convert social_profiles to profile_cache

-- Create contact_urls table to store all URLs (including social media)
CREATE TABLE IF NOT EXISTS contact_urls (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    url TEXT NOT NULL,
    label TEXT,  -- "GitHub", "LinkedIn", "Personal", "Blog", "Work", etc.
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_contact_urls_contact ON contact_urls(contact_id);
CREATE INDEX IF NOT EXISTS idx_contact_urls_label ON contact_urls(label) WHERE label IS NOT NULL;

-- Rename social_profiles to profile_cache (stores fetched profile data)
ALTER TABLE social_profiles RENAME TO profile_cache;

-- Add url_id column to reference the source URL
ALTER TABLE profile_cache ADD COLUMN url_id TEXT REFERENCES contact_urls(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_profile_cache_url ON profile_cache(url_id) WHERE url_id IS NOT NULL;

-- Update db_version in settings
UPDATE settings SET value = '2', updated_at = strftime('%s', 'now') WHERE key = 'db_version';
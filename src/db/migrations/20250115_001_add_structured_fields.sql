-- Migration: Add structured email, phone, address, and date tables
-- Created: 2025-01-15
-- Description: Adds support for multiple emails, phones, addresses, and dates with labels

-- Contact emails with labels
CREATE TABLE IF NOT EXISTS contact_emails (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    email TEXT NOT NULL,
    label TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX idx_contact_emails_contact_id ON contact_emails(contact_id);
CREATE INDEX idx_contact_emails_label ON contact_emails(label);

-- Contact phones with labels
CREATE TABLE IF NOT EXISTS contact_phones (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    phone TEXT NOT NULL,
    label TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX idx_contact_phones_contact_id ON contact_phones(contact_id);
CREATE INDEX idx_contact_phones_label ON contact_phones(label);

-- Contact addresses with labels
CREATE TABLE IF NOT EXISTS contact_addresses (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    street TEXT,
    city TEXT,
    state TEXT,
    postal_code TEXT,
    country TEXT,
    label TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX idx_contact_addresses_contact_id ON contact_addresses(contact_id);
CREATE INDEX idx_contact_addresses_label ON contact_addresses(label);

-- Contact significant dates with labels
CREATE TABLE IF NOT EXISTS contact_dates (
    id TEXT PRIMARY KEY NOT NULL,
    contact_id TEXT NOT NULL,
    date TEXT NOT NULL, -- Stored as ISO 8601 date (YYYY-MM-DD)
    label TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE INDEX idx_contact_dates_contact_id ON contact_dates(contact_id);
CREATE INDEX idx_contact_dates_label ON contact_dates(label);

-- Migrate existing email and phone data to new tables (if exists)
-- This preserves backward compatibility with the deprecated email/phone columns
INSERT INTO contact_emails (id, contact_id, email, label, created_at, updated_at)
SELECT 
    lower(hex(randomblob(16))),
    id,
    email,
    'Home',
    created_at,
    updated_at
FROM contacts
WHERE email IS NOT NULL AND email != '';

INSERT INTO contact_phones (id, contact_id, phone, label, created_at, updated_at)
SELECT 
    lower(hex(randomblob(16))),
    id,
    phone,
    'Mobile',
    created_at,
    updated_at
FROM contacts
WHERE phone IS NOT NULL AND phone != '';
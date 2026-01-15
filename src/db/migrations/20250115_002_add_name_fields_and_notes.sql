-- Migration: Add structured name fields and notes
-- Created: 2025-01-15
-- Description: Adds support for name components (prefix, first, middle, last, suffix) and notes field

-- Add structured name fields
ALTER TABLE contacts ADD COLUMN name_prefix TEXT;
ALTER TABLE contacts ADD COLUMN first_name TEXT;
ALTER TABLE contacts ADD COLUMN middle_name TEXT;
ALTER TABLE contacts ADD COLUMN last_name TEXT;
ALTER TABLE contacts ADD COLUMN name_suffix TEXT;
ALTER TABLE contacts ADD COLUMN nickname TEXT;

-- Add notes field
ALTER TABLE contacts ADD COLUMN notes TEXT;

-- Add department field
ALTER TABLE contacts ADD COLUMN department TEXT;

-- Create index on first and last name for better search performance
CREATE INDEX IF NOT EXISTS idx_contacts_first_name ON contacts(first_name);
CREATE INDEX IF NOT EXISTS idx_contacts_last_name ON contacts(last_name);
CREATE INDEX IF NOT EXISTS idx_contacts_nickname ON contacts(nickname);
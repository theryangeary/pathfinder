-- Add completion tracking fields to games table
ALTER TABLE games ADD COLUMN completed BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE games ADD COLUMN completed_at TIMESTAMP;
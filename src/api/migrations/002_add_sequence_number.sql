-- Add sequence_number column to games table
-- The first game should have sequence number 1, incrementing by 1 for each subsequent day

ALTER TABLE games ADD COLUMN sequence_number INTEGER NOT NULL DEFAULT 0;

-- Update existing games to have proper sequence numbers based on their date
-- Assuming games are created in chronological order
UPDATE games 
SET sequence_number = (
    SELECT COUNT(*) 
    FROM games g2 
    WHERE g2.date <= games.date
);

-- Add unique constraint to ensure no duplicate sequence numbers
CREATE UNIQUE INDEX idx_games_sequence_number ON games(sequence_number);
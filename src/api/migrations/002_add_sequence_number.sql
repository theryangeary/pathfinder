-- Add sequence_number column to games table
-- The first game should have sequence number 1, incrementing by 1 for each subsequent day

-- Add column if it doesn't exist (PostgreSQL supports IF NOT EXISTS for ALTER TABLE)
ALTER TABLE games ADD COLUMN IF NOT EXISTS sequence_number INTEGER;

-- Update existing games to have proper sequence numbers based on their date
-- Assuming games are created in chronological order
UPDATE games 
SET sequence_number = (
    SELECT ROW_NUMBER() OVER (ORDER BY date)
    FROM games g2 
    WHERE g2.id = games.id
)
WHERE sequence_number IS NULL;

-- Make the column NOT NULL after updating existing records
ALTER TABLE games ALTER COLUMN sequence_number SET NOT NULL;

-- Add unique constraint to ensure no duplicate sequence numbers
CREATE UNIQUE INDEX IF NOT EXISTS idx_games_sequence_number ON games(sequence_number);
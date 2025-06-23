-- Remove path and path_constraint_set columns from game_answers table
-- Add unique constraint on (game_id, word) to ensure one row per word per game

-- First, remove duplicate rows keeping only the first occurrence of each (game_id, word) pair
DELETE FROM game_answers 
WHERE id NOT IN (
    SELECT DISTINCT ON (game_id, word) id 
    FROM game_answers 
    ORDER BY game_id, word, created_at
);

-- Drop the columns we no longer need (conditionally in case they were already dropped)
ALTER TABLE game_answers DROP COLUMN IF EXISTS path;
ALTER TABLE game_answers DROP COLUMN IF EXISTS path_constraint_set;

-- Add unique constraint on (game_id, word)
CREATE UNIQUE INDEX IF NOT EXISTS idx_game_answers_unique_game_word ON game_answers(game_id, word);

-- Drop the old non-unique index since we now have a unique one
DROP INDEX IF EXISTS idx_game_answers_game_word;
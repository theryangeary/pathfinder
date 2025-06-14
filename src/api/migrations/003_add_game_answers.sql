-- Add game_answers table to store valid answers for each game
-- Each row represents one possible path to form a word on a specific game board

CREATE TABLE IF NOT EXISTS game_answers (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL,
    word TEXT NOT NULL,
    path TEXT NOT NULL, -- JSON array representing the path coordinates
    path_constraint_set TEXT NOT NULL, -- JSON PathConstraintSet object
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_game_answers_game_id ON game_answers(game_id);
CREATE INDEX IF NOT EXISTS idx_game_answers_word ON game_answers(word);
CREATE INDEX IF NOT EXISTS idx_game_answers_game_word ON game_answers(game_id, word);

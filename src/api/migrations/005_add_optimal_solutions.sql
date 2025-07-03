-- Add optimal_solutions table to store the optimal solution for each game
-- Contains the best possible words and their scores for each daily puzzle

CREATE TABLE IF NOT EXISTS optimal_solutions (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL,
    words_and_scores TEXT NOT NULL,  -- JSON blob containing optimal words and their scores
    total_score INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    UNIQUE(game_id)  -- One optimal solution per game
);

-- Index for performance
CREATE INDEX IF NOT EXISTS idx_optimal_solutions_game_id ON optimal_solutions(game_id);
CREATE INDEX IF NOT EXISTS idx_optimal_solutions_total_score ON optimal_solutions(total_score);
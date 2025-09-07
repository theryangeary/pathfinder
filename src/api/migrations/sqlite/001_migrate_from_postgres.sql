-- SQLite database schema for word game backend
-- Note: SQLite doesn't support IF NOT EXISTS for all operations, so this assumes a fresh database

PRAGMA foreign_keys = ON;

-- Users table - tracks browser sessions via cookies
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    cookie_token TEXT UNIQUE NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    last_seen TEXT DEFAULT (datetime('now'))
);

-- Games table - stores daily puzzles and historical games
CREATE TABLE games (
    id TEXT PRIMARY KEY,
    date TEXT UNIQUE NOT NULL, -- YYYY-MM-DD format
    board_data TEXT NOT NULL,  -- JSON serialized board data
    threshold_score INTEGER NOT NULL,
    sequence_number INTEGER NOT NULL UNIQUE,
    completed INTEGER NOT NULL DEFAULT 0, -- SQLite uses INTEGER for boolean
    completed_at TEXT, -- SQLite uses TEXT for timestamps
    created_at TEXT DEFAULT (datetime('now'))
);

-- Game entries table - user solutions for specific games
CREATE TABLE game_entries (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    game_id TEXT NOT NULL,
    answers_data TEXT NOT NULL, -- JSON serialized answers array
    total_score INTEGER NOT NULL,
    completed INTEGER DEFAULT 0, -- SQLite uses INTEGER for boolean
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    UNIQUE(user_id, game_id)  -- One entry per user per game
);

-- Game answers table - stores valid answers for each game
CREATE TABLE game_answers (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL,
    word TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    UNIQUE(game_id, word) -- One row per word per game
);

-- Optimal solutions table - stores the optimal solution for each game
CREATE TABLE optimal_solutions (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL UNIQUE, -- One optimal solution per game
    words_and_scores TEXT NOT NULL,  -- JSON blob containing optimal words and their scores
    total_score INTEGER NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX idx_games_date ON games(date);
CREATE INDEX idx_games_sequence_number ON games(sequence_number);
CREATE INDEX idx_game_entries_user_id ON game_entries(user_id);
CREATE INDEX idx_game_entries_game_id ON game_entries(game_id);
CREATE INDEX idx_game_entries_total_score ON game_entries(total_score);
CREATE INDEX idx_users_cookie_token ON users(cookie_token);
CREATE INDEX idx_game_answers_game_id ON game_answers(game_id);
CREATE INDEX idx_game_answers_word ON game_answers(word);
CREATE INDEX idx_optimal_solutions_game_id ON optimal_solutions(game_id);
CREATE INDEX idx_optimal_solutions_total_score ON optimal_solutions(total_score);

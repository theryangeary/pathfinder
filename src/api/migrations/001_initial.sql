-- Initial database schema for word game backend

-- Users table - tracks browser sessions via cookies
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    cookie_token TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_seen TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Games table - stores daily puzzles and historical games
CREATE TABLE games (
    id TEXT PRIMARY KEY,
    date TEXT UNIQUE NOT NULL, -- YYYY-MM-DD format
    board_data TEXT NOT NULL,  -- JSON serialized board data
    threshold_score INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Game entries table - user solutions for specific games
CREATE TABLE game_entries (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    game_id TEXT NOT NULL,
    answers_data TEXT NOT NULL, -- JSON serialized answers array
    total_score INTEGER NOT NULL,
    completed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    UNIQUE(user_id, game_id)  -- One entry per user per game
);

-- Indexes for performance
CREATE INDEX idx_games_date ON games(date);
CREATE INDEX idx_game_entries_user_id ON game_entries(user_id);
CREATE INDEX idx_game_entries_game_id ON game_entries(game_id);
CREATE INDEX idx_game_entries_total_score ON game_entries(total_score);
CREATE INDEX idx_users_cookie_token ON users(cookie_token);
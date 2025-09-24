-- drop id from game_answers table, use composite key instead
CREATE TABLE game_answers2( 
    game_id TEXT NOT NULL,
    word TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
    PRIMARY KEY (game_id, word)
);
INSERT INTO game_answers2 (game_id, word, created_at)
   SELECT game_id, word, created_at FROM game_answers;
DROP TABLE game_answers;

CREATE INDEX idx_game_answers2_game_id ON game_answers2(game_id);

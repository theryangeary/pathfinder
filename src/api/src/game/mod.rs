pub mod trie;
pub mod scoring;
pub mod directions;
pub mod board;
pub mod conversion;

pub use trie::Trie;
pub use scoring::Scorer;
pub use board::Board;
pub use conversion::*;

// BoardGenerator for game generation
pub struct BoardGenerator {
    letter_frequencies: std::collections::HashMap<char, f64>,
}

impl BoardGenerator {
    pub fn new() -> Self {
        let mut letter_frequencies = std::collections::HashMap::new();
        letter_frequencies.insert('a', 0.078);
        letter_frequencies.insert('b', 0.02);
        letter_frequencies.insert('c', 0.04);
        letter_frequencies.insert('d', 0.038);
        letter_frequencies.insert('e', 0.11);
        letter_frequencies.insert('f', 0.014);
        letter_frequencies.insert('g', 0.03);
        letter_frequencies.insert('h', 0.023);
        letter_frequencies.insert('i', 0.086);
        letter_frequencies.insert('j', 0.0021);
        letter_frequencies.insert('k', 0.0097);
        letter_frequencies.insert('l', 0.053);
        letter_frequencies.insert('m', 0.027);
        letter_frequencies.insert('n', 0.072);
        letter_frequencies.insert('o', 0.061);
        letter_frequencies.insert('p', 0.028);
        letter_frequencies.insert('q', 0.0019);
        letter_frequencies.insert('r', 0.073);
        letter_frequencies.insert('s', 0.087);
        letter_frequencies.insert('t', 0.067);
        letter_frequencies.insert('u', 0.033);
        letter_frequencies.insert('v', 0.01);
        letter_frequencies.insert('w', 0.0091);
        letter_frequencies.insert('x', 0.0027);
        letter_frequencies.insert('y', 0.016);
        letter_frequencies.insert('z', 0.0044);

        Self { letter_frequencies }
    }

    pub fn generate_board<R: rand::Rng>(&self, rng: &mut R) -> Board {
        use rand::seq::SliceRandom;
        use rand::distributions::{Distribution, WeightedIndex};

        // Create weighted distribution for letter selection
        let letters: Vec<char> = self.letter_frequencies.keys().cloned().collect();
        let weights: Vec<f64> = letters
            .iter()
            .map(|&c| self.letter_frequencies[&c])
            .collect();

        // Generate 4x4 board
        let mut board = Board::new();
        
        for row in 0..4 {
            for col in 0..4 {
                // Choose random letter based on frequency
                let letter = self.weighted_choice(&letters, &weights, rng);
                let points = crate::game::scoring::points_for_letter(letter, &self.letter_frequencies);
                
                board.set_tile(row, col, letter, points, false);
            }
        }

        // Set wildcard tiles at positions (1,1) and (2,2) - non-adjacent interior tiles
        board.set_tile(1, 1, '*', 0, true);
        board.set_tile(2, 2, '*', 0, true);

        board
    }

    fn weighted_choice<R: rand::Rng>(
        &self,
        letters: &[char],
        weights: &[f64],
        rng: &mut R,
    ) -> char {
        use rand::distributions::{Distribution, WeightedIndex};
        
        let dist = WeightedIndex::new(weights).unwrap();
        letters[dist.sample(rng)]
    }
}

use std::path::{PathBuf, Path};
use std::collections::HashMap;
use anyhow::Result;

/// Main game engine that combines all the game logic components
#[derive(Clone)]
pub struct GameEngine {
    word_trie: Trie,
    scorer: Scorer,
}

impl GameEngine {
    pub async fn new<P: AsRef<Path>>(wordlist_path: P) -> Result<Self> {
        let word_trie = Trie::from_file(wordlist_path.as_ref().to_path_buf())?;
        Ok(Self {
            word_trie,
            scorer: Scorer::new(),
        })
    }

    pub fn validate_word(&self, word: &str) -> bool {
        self.word_trie.search(word)
    }

    pub fn score_word(&self, word: &str) -> u32 {
        self.scorer.score(word)
    }

    pub fn find_word_paths(&self, board: &Board, word: &str) -> board::answer::Answer {
        board.paths_for(word)
    }

    pub fn validate_answer(&self, board: &Board, word: &str) -> Result<board::answer::Answer, String> {
        // First check if the word is in our dictionary
        if !self.validate_word(word) {
            return Err(format!("Word '{}' not found in dictionary", word));
        }

        // Find all possible paths for this word on the board
        let answer = self.find_word_paths(board, word);
        
        if answer.paths.is_empty() {
            return Err(format!("Word '{}' cannot be formed on this board", word));
        }

        Ok(answer)
    }

    pub fn validate_answer_with_constraints(
        &self, 
        board: &Board, 
        word: &str, 
        existing_answers: &[board::answer::Answer]
    ) -> Result<board::answer::Answer, String> {
        let answer = self.validate_answer(board, word)?;
        
        // Check wildcard constraint consistency
        let mut all_answers = existing_answers.iter().collect::<Vec<_>>();
        all_answers.push(&answer);
        
        // Wildcard constraint validation moved to application layer
        
        Ok(answer)
    }

    pub async fn validate_word_with_constraints(
        &self,
        board: &Board,
        word: &str,
        previous_constraints: &HashMap<String, char>,
    ) -> Result<Option<board::answer::Answer>> {
        // First check if the word is in our dictionary
        if !self.validate_word(word) {
            return Ok(None);
        }

        // Find all possible paths for this word on the board
        let answer = self.find_word_paths(board, word);
        
        if answer.paths.is_empty() {
            return Ok(None);
        }

        // Apply constraint filtering based on existing constraints
        let filtered_answer = answer.filter_paths_by_constraints(previous_constraints);
        
        if filtered_answer.paths.is_empty() {
            return Ok(None);
        }

        Ok(Some(filtered_answer))
    }

    pub async fn find_all_valid_words(&self, board: &Board) -> Result<Vec<board::answer::Answer>> {
        let mut valid_answers = Vec::new();
        
        // Generate all possible words from the board using DFS
        let mut found_words = std::collections::HashSet::new();
        
        // Start from each position on the board
        for row in 0..4 {
            for col in 0..4 {
                let mut visited = std::collections::HashSet::new();
                self.find_words_from_position(board, row, col, String::new(), &mut visited, &mut found_words);
            }
        }
        
        // Validate found words against our dictionary and create answers
        for word in found_words {
            if word.len() >= 3 && self.validate_word(&word) {
                if let Ok(answer) = self.validate_answer(board, &word) {
                    if !answer.paths.is_empty() {
                        valid_answers.push(answer);
                    }
                }
            }
        }
        
        Ok(valid_answers)
    }

    fn find_words_from_position(
        &self,
        board: &Board,
        row: usize,
        col: usize,
        current_word: String,
        visited: &mut std::collections::HashSet<(usize, usize)>,
        found_words: &mut std::collections::HashSet<String>,
    ) {
        // Don't exceed reasonable word length
        if current_word.len() > 16 {
            return;
        }

        // Mark current position as visited
        visited.insert((row, col));
        
        // Get current tile
        let tile = board.get_tile(row, col);
        
        if tile.is_wildcard {
            // For wildcards, try all possible letters
            for letter in 'a'..='z' {
                let mut new_word = current_word.clone();
                new_word.push(letter);
                
                // Early termination: if this prefix can't lead to any valid words, skip
                if !self.word_trie.has_prefix(&new_word) {
                    continue;
                }
                
                // If word is long enough and valid, add it to found words
                if new_word.len() >= 3 && self.word_trie.search(&new_word) {
                    found_words.insert(new_word.clone());
                }
                
                // Explore adjacent positions with this wildcard letter choice
                self.explore_adjacent_positions(board, row, col, new_word, visited, found_words);
            }
        } else {
            // For regular tiles, add the letter
            let mut new_word = current_word;
            new_word.push_str(&tile.letter);
            
            // Early termination: if this prefix can't lead to any valid words, stop
            if !self.word_trie.has_prefix(&new_word) {
                visited.remove(&(row, col));
                return;
            }
            
            // If word is long enough and valid, add it to found words
            if new_word.len() >= 3 && self.word_trie.search(&new_word) {
                found_words.insert(new_word.clone());
            }
            
            // Explore adjacent positions
            self.explore_adjacent_positions(board, row, col, new_word, visited, found_words);
        }
        
        // Unmark position for other paths
        visited.remove(&(row, col));
    }

    fn explore_adjacent_positions(
        &self,
        board: &Board,
        row: usize,
        col: usize,
        current_word: String,
        visited: &mut std::collections::HashSet<(usize, usize)>,
        found_words: &mut std::collections::HashSet<String>,
    ) {
        let directions = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1),
        ];
        
        for (dr, dc) in directions {
            let new_row = row as i32 + dr;
            let new_col = col as i32 + dc;
            
            if new_row >= 0 && new_row < 4 && new_col >= 0 && new_col < 4 {
                let new_row = new_row as usize;
                let new_col = new_col as usize;
                
                if !visited.contains(&(new_row, new_col)) {
                    self.find_words_from_position(board, new_row, new_col, current_word.clone(), visited, found_words);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_wordlist() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "cat").unwrap();
        writeln!(temp_file, "dog").unwrap();
        writeln!(temp_file, "test").unwrap();
        writeln!(temp_file, "word").unwrap();
        writeln!(temp_file, "game").unwrap();
        writeln!(temp_file, "path").unwrap();
        writeln!(temp_file, "tile").unwrap();
        writeln!(temp_file, "board").unwrap();
        temp_file.flush().unwrap();
        temp_file
    }

    fn create_test_board() -> Board {
        let mut board = Board::new();
        
        // Create a simple board for testing
        // c a t e
        // o m p l
        // * s e *  (wildcards at 2,0 and 2,3)
        // r n d g
        
        board.set_tile(0, 0, 'c', 2, false);
        board.set_tile(0, 1, 'a', 1, false);
        board.set_tile(0, 2, 't', 1, false);
        board.set_tile(0, 3, 'e', 1, false);
        
        board.set_tile(1, 0, 'o', 1, false);
        board.set_tile(1, 1, 'm', 2, false);
        board.set_tile(1, 2, 'p', 2, false);
        board.set_tile(1, 3, 'l', 1, false);
        
        board.set_tile(2, 0, '*', 0, true);  // wildcard
        board.set_tile(2, 1, 's', 1, false);
        board.set_tile(2, 2, 'e', 1, false);
        board.set_tile(2, 3, '*', 0, true);  // wildcard
        
        board.set_tile(3, 0, 'r', 1, false);
        board.set_tile(3, 1, 'n', 1, false);
        board.set_tile(3, 2, 'd', 1, false);
        board.set_tile(3, 3, 'g', 2, false);
        
        board
    }

    #[test]
    fn test_board_generator_new() {
        let generator = BoardGenerator::new();
        
        // Check that all expected letters are in the frequency map
        assert_eq!(generator.letter_frequencies.len(), 26);
        assert!(generator.letter_frequencies.contains_key(&'a'));
        assert!(generator.letter_frequencies.contains_key(&'z'));
        
        // Check specific frequency values
        assert_eq!(generator.letter_frequencies[&'e'], 0.11);
        assert_eq!(generator.letter_frequencies[&'q'], 0.0019);
    }

    #[test]
    fn test_board_generator_generate_board() {
        let generator = BoardGenerator::new();
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        
        let board = generator.generate_board(&mut rng);
        
        // Check that wildcards are placed correctly
        assert!(board.get_tile(1, 1).is_wildcard);
        assert!(board.get_tile(2, 2).is_wildcard);
        assert_eq!(board.get_tile(1, 1).letter, "*");
        assert_eq!(board.get_tile(2, 2).letter, "*");
        assert_eq!(board.get_tile(1, 1).points, 0);
        assert_eq!(board.get_tile(2, 2).points, 0);
        
        // Check that other tiles are not wildcards
        assert!(!board.get_tile(0, 0).is_wildcard);
        assert!(!board.get_tile(3, 3).is_wildcard);
        
        // Check that regular tiles have valid letters and points
        for row in 0..4 {
            for col in 0..4 {
                let tile = board.get_tile(row, col);
                if !tile.is_wildcard {
                    assert!(tile.letter.chars().next().unwrap().is_ascii_lowercase());
                    assert!(tile.points > 0);
                }
            }
        }
    }

    #[test]
    fn test_board_generator_weighted_choice() {
        let generator = BoardGenerator::new();
        let letters = vec!['a', 'e', 'z'];
        let weights = vec![0.078, 0.11, 0.0044]; // e is most frequent
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        
        // Run multiple times to check distribution
        let mut e_count = 0;
        let total = 1000;
        
        for _ in 0..total {
            let choice = generator.weighted_choice(&letters, &weights, &mut rng);
            if choice == 'e' {
                e_count += 1;
            }
        }
        
        // E should be chosen more often than other letters (roughly proportional to weight)
        // This is probabilistic, so we use a reasonable range
        assert!(e_count > total / 10); // Should be much more than 1/3
    }

    #[tokio::test]
    async fn test_game_engine_new() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await;
        
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        
        // Test that the engine was initialized properly
        assert!(engine.validate_word("cat"));
        assert!(engine.validate_word("dog"));
        assert!(!engine.validate_word("nonexistent"));
    }

    #[tokio::test]
    async fn test_game_engine_validate_word() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        
        // Test valid words
        assert!(engine.validate_word("cat"));
        assert!(engine.validate_word("dog"));
        assert!(engine.validate_word("test"));
        
        // Test invalid words
        assert!(!engine.validate_word("xyz"));
        assert!(!engine.validate_word(""));
        assert!(!engine.validate_word("notinlist"));
    }

    #[tokio::test]
    async fn test_game_engine_score_word() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        
        let cat_score = engine.score_word("cat");
        let dog_score = engine.score_word("dog");
        
        // Both should be positive scores
        assert!(cat_score > 0);
        assert!(dog_score > 0);
        
        // Empty string should score 0
        assert_eq!(engine.score_word(""), 0);
    }

    #[tokio::test]
    async fn test_game_engine_find_word_paths() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        let board = create_test_board();
        
        // Test finding paths for "cat" (should exist: c-a-t at positions (0,0)-(0,1)-(0,2))
        let answer = engine.find_word_paths(&board, "cat");
        assert!(!answer.paths.is_empty());
        assert_eq!(answer.word, "cat");
        
        // Test finding paths for non-existent word formation
        let answer = engine.find_word_paths(&board, "xyz");
        assert!(answer.paths.is_empty());
    }

    #[tokio::test]
    async fn test_game_engine_validate_answer_valid() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        let board = create_test_board();
        
        // Test valid answer
        let result = engine.validate_answer(&board, "cat");
        assert!(result.is_ok());
        
        let answer = result.unwrap();
        assert_eq!(answer.word, "cat");
        assert!(!answer.paths.is_empty());
    }

    #[tokio::test]
    async fn test_game_engine_validate_answer_invalid_word() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        let board = create_test_board();
        
        // Test invalid word (not in dictionary)
        let result = engine.validate_answer(&board, "xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in dictionary"));
    }

    #[tokio::test]
    async fn test_game_engine_validate_answer_no_path() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        let board = create_test_board();
        
        // Test word that exists in dictionary but can't be formed on board
        let result = engine.validate_answer(&board, "game");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be formed on this board"));
    }

    #[tokio::test]
    async fn test_game_engine_validate_answer_with_constraints() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        let board = create_test_board();
        
        // Test with empty constraints
        let existing_answers = vec![];
        let result = engine.validate_answer_with_constraints(&board, "cat", &existing_answers);
        assert!(result.is_ok());
        
        let answer = result.unwrap();
        assert_eq!(answer.word, "cat");
    }

    #[tokio::test]
    async fn test_game_engine_validate_word_with_constraints_valid() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        let board = create_test_board();
        
        // Test with empty constraints
        let constraints = HashMap::new();
        let result = engine.validate_word_with_constraints(&board, "cat", &constraints).await;
        assert!(result.is_ok());
        
        let answer = result.unwrap();
        assert!(answer.is_some());
        assert_eq!(answer.unwrap().word, "cat");
    }

    #[tokio::test]
    async fn test_game_engine_validate_word_with_constraints_invalid_word() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        let board = create_test_board();
        
        // Test invalid word
        let constraints = HashMap::new();
        let result = engine.validate_word_with_constraints(&board, "xyz", &constraints).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_game_engine_validate_word_with_constraints_no_path() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        let board = create_test_board();
        
        // Test word that can't be formed on board
        let constraints = HashMap::new();
        let result = engine.validate_word_with_constraints(&board, "game", &constraints).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_find_all_valid_words() {
        let temp_file = create_test_wordlist();
        let engine = GameEngine::new(temp_file.path()).await.unwrap();
        let board = create_test_board();
        
        let result = engine.find_all_valid_words(&board).await;
        assert!(result.is_ok());
        
        let valid_answers = result.unwrap();
        
        // Should find at least "cat" since we know it exists on the board
        let cat_found = valid_answers.iter().any(|answer| answer.word == "cat");
        assert!(cat_found);
        
        // All found words should be at least 3 characters long
        for answer in &valid_answers {
            assert!(answer.word.len() >= 3);
            assert!(!answer.paths.is_empty());
        }
    }

    #[test]
    fn test_find_words_from_position_length_limit() {
        // This is harder to test without access to private methods, so we'll test behavior indirectly
        // through find_all_valid_words ensuring it doesn't hang with very long word generation
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp_file = create_test_wordlist();
            let engine = GameEngine::new(temp_file.path()).await.unwrap();
            let board = create_test_board();
            
            // This should complete quickly without hanging
            let start = std::time::Instant::now();
            let result = engine.find_all_valid_words(&board).await;
            let duration = start.elapsed();
            
            assert!(result.is_ok());
            // Should complete in reasonable time (much less than 1 second for this simple board)
            assert!(duration < std::time::Duration::from_secs(1));
        });
    }

    #[test]
    fn test_explore_adjacent_positions_bounds() {
        // Test that adjacent position exploration respects board bounds
        // This is tested indirectly through the find_all_valid_words functionality
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp_file = create_test_wordlist();
            let engine = GameEngine::new(temp_file.path()).await.unwrap();
            let board = create_test_board();
            
            // Should complete without panicking (testing bounds checking)
            let result = engine.find_all_valid_words(&board).await;
            assert!(result.is_ok());
        });
    }
}
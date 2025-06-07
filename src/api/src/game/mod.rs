pub mod trie;
pub mod scoring;
pub mod directions;
pub mod board;
pub mod utils;
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
        
        utils::GameUtils::validate_wildcard_consistency(&all_answers)?;
        
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
                
                // If word is long enough, add it to found words
                if new_word.len() >= 3 {
                    found_words.insert(new_word.clone());
                }
                
                // Explore adjacent positions with this wildcard letter choice
                self.explore_adjacent_positions(board, row, col, new_word, visited, found_words);
            }
        } else {
            // For regular tiles, add the letter
            let mut new_word = current_word;
            new_word.push_str(&tile.letter);
            
            // If word is long enough, add it to found words
            if new_word.len() >= 3 {
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
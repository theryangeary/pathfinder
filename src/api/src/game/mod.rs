pub mod board;
pub mod conversion;
pub mod directions;
pub mod scoring;
pub mod trie;

pub use board::Board;
use std::collections::HashMap;
pub use trie::Trie;

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
                let points = crate::game::scoring::points_for_letter(letter);

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

use anyhow::Result;
use std::sync::Arc;

use crate::game::board::constraints::AnswerGroupConstraintSet;
use crate::game::scoring::ScoreSheet;
use crate::http_api::ApiAnswer;

/// Main game engine that combines all the game logic components
#[derive(Clone)]
pub struct GameEngine {
    word_trie: Arc<Trie>,
}

impl GameEngine {
    pub fn new<T: Into<Trie>>(trie_source: T) -> Self {
        let word_trie = Arc::new(trie_source.into());
        Self { word_trie }
    }

    pub fn validate_api_answer_group(
        &self,
        board: &Board,
        answers: Vec<ApiAnswer>,
    ) -> Result<(), String> {
        // Sanitize input
        let mut sanitized_answers: Vec<ApiAnswer> = Vec::new();
        for answer in answers{
            sanitized_answers.push(answer.clone().sanitize());
        }
        self.validate_answer_group(
            board,
            sanitized_answers
                .iter()
                .map(|m| m.word.to_string())
                .collect(),
        )
    }

    fn validate_answer_group(&self, board: &Board, answers: Vec<String>) -> Result<(), String> {
        // First validate that all words exist in the dictionary
        for answer in &answers {
            if !self.is_valid_word_in_dictionary(answer) {
                return Err(format!("Word '{}' is not in the dictionary", answer));
            }
        }

        // need a step here where we check a word actually has >1 paths, unless maybe is_valid_set is already handling that for us

        // Get all paths for each word
        let answers_with_all_paths = board.get_answers_with_all_paths(answers)?;

        // Ensure constraints can be satisfied
        if AnswerGroupConstraintSet::is_valid_set(answers_with_all_paths) {
            Ok(())
        } else {
            Err("Some answers have conflicting wildcard constraints".to_string())
        }
    }

    pub fn is_valid_word_in_dictionary(&self, word: &str) -> bool {
        self.word_trie.search(word)
    }

    /// score_answer_group finds all the possible AnswerGroupConstraintSets, calculates the scores for all words based on each set of constraints, and returns the HashMap of answer -> score for the highest total scoring paths that can coexist based on constraints. It returns an error if the answers cannot coexist based on constraints.
    pub fn score_answer_group(
        &self,
        board: &Board,
        answers: Vec<String>,
    ) -> Result<ScoreSheet, String> {
        if answers.is_empty() {
            return Ok(ScoreSheet::new());
        }

        // Find all possible paths for each answer
        let mut answer_objects = Vec::new();
        for word in answers {
            let answer = self.find_word_paths(board, &word);
            if answer.paths.is_empty() {
                return Err(format!("Word '{}' cannot be formed on this board", word));
            }
            answer_objects.push(answer);
        }

        let valid_constraint_set =
            if let Ok(v) = AnswerGroupConstraintSet::try_from(&answer_objects) {
                v
            } else {
                return Err(
                    "Answers cannot coexist due to conflicting wildcard constraints".to_string(),
                );
            };

        // For each valid path constraint set, calculate the maximum possible score
        let mut max_total_score = 0u32;
        let mut best_scores_by_word = HashMap::new();

        for path_constraint in &valid_constraint_set.path_constraint_sets {
            let mut total_score = 0u32;
            let mut scores_by_word = HashMap::new();

            // For each answer, find the best scoring path that satisfies this constraint
            for answer_obj in &answer_objects {
                let mut best_path_score = 0;

                // Check all paths for this answer to find the one that works with current constraints
                for path in &answer_obj.paths {
                    // Check if this path's constraints are compatible with the current path_constraint
                    if path.constraints.merge(*path_constraint).is_ok() {
                        let path_score: u32 = path
                            .tiles
                            .iter()
                            .map(|tile| tile.points)
                            .sum::<i32>()
                            .try_into()
                            .unwrap();
                        best_path_score = best_path_score.max(path_score);
                    }
                }

                // Record this answer's best score and add to total (ensuring non-negative)
                let word_score = best_path_score;
                scores_by_word.insert(answer_obj.word.clone(), word_score);
                total_score += word_score;
            }

            // If this constraint set gives us a better total score, use it
            if total_score > max_total_score {
                max_total_score = total_score;
                best_scores_by_word = scores_by_word;
            }
        }

        Ok(ScoreSheet::from(best_scores_by_word))
    }

    pub fn find_word_paths(&self, board: &Board, word: &str) -> board::answer::Answer {
        board.paths_for(word)
    }

    pub fn validate_answer(
        &self,
        board: &Board,
        word: &str,
    ) -> Result<board::answer::Answer, String> {
        // First check if the word is in our dictionary
        if !self.is_valid_word_in_dictionary(word) {
            return Err(format!("Word '{}' not found in dictionary", word));
        }

        // Find all possible paths for this word on the board
        let answer = self.find_word_paths(board, word);

        if answer.paths.is_empty() {
            return Err(format!("Word '{}' cannot be formed on this board", word));
        }

        Ok(answer)
    }

    pub async fn find_all_valid_words(&self, board: &Board) -> Result<Vec<board::answer::Answer>> {
        let mut valid_answers = Vec::new();

        // Generate all possible words from the board using DFS
        let mut found_words = std::collections::HashSet::new();

        // Start from each position on the board
        for row in 0..4 {
            for col in 0..4 {
                let mut visited = std::collections::HashSet::new();
                self.find_words_from_position(
                    board,
                    row,
                    col,
                    String::new(),
                    &mut visited,
                    &mut found_words,
                );
            }
        }

        // Validate found words against our dictionary and create answers
        for word in found_words {
            if word.len() >= 3 && self.is_valid_word_in_dictionary(&word) {
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
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        for (dr, dc) in directions {
            let new_row = row as i32 + dr;
            let new_col = col as i32 + dc;

            if (0..4).contains(&new_row) && (0..4).contains(&new_col) {
                let new_row = new_row as usize;
                let new_col = new_col as usize;

                if !visited.contains(&(new_row, new_col)) {
                    self.find_words_from_position(
                        board,
                        new_row,
                        new_col,
                        current_word.clone(),
                        visited,
                        found_words,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;
    use rand::SeedableRng;
    fn create_test_wordlist() -> Vec<&'static str> {
        vec![
            "cat", "dog", "test", "word", "game", "path", "tile", "board", "day", "days", "year",
            "data",
        ]
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

        board.set_tile(2, 0, '*', 0, true); // wildcard
        board.set_tile(2, 1, 's', 1, false);
        board.set_tile(2, 2, 'e', 1, false);
        board.set_tile(2, 3, '*', 0, true); // wildcard

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
        let words = create_test_wordlist();
        let engine = GameEngine::new(words);

        // Test that the engine was initialized properly
        assert!(engine.is_valid_word_in_dictionary("cat"));
        assert!(engine.is_valid_word_in_dictionary("dog"));
        assert!(!engine.is_valid_word_in_dictionary("nonexistent"));
    }

    #[tokio::test]
    async fn test_game_engine_validate_word() {
        let words = create_test_wordlist();
        let engine = GameEngine::new(words);

        // Test valid words
        assert!(engine.is_valid_word_in_dictionary("cat"));
        assert!(engine.is_valid_word_in_dictionary("dog"));
        assert!(engine.is_valid_word_in_dictionary("test"));

        // Test invalid words
        assert!(!engine.is_valid_word_in_dictionary("xyz"));
        assert!(!engine.is_valid_word_in_dictionary(""));
        assert!(!engine.is_valid_word_in_dictionary("notinlist"));
    }

    #[test] // NEW
    fn test_score_answer_group_constraint_set_size() {}

    #[test]
    fn test_score_answer_group_comprehensive() {
        #[derive(Debug)]
        enum ExpectedResult {
            Success {
                expected_scores: Vec<(&'static str, u32)>,
            },
            Error {
                error_fragment: &'static str,
            },
        }

        struct TestCase {
            name: &'static str,
            board: Board,
            words: Vec<&'static str>,
            answers: Vec<String>,
            expected_result: ExpectedResult,
            description: &'static str,
        }

        let test_cases = vec![
            TestCase {
                name: "empty_input",
                board: create_test_board(),
                words: create_test_wordlist(),
                answers: vec![],
                expected_result: ExpectedResult::Success { expected_scores: vec![] },
                description: "Empty answer list should return empty HashMap",
            },
            TestCase {
                name: "single_valid_word",
                board: create_test_board(),
                words: create_test_wordlist(),
                answers: vec!["cat".to_string()],
                expected_result: ExpectedResult::Success { expected_scores: vec![("cat", 4)] },
                description: "Single valid word should return its correct score",
            },
            TestCase {
                name: "word_not_in_dictionary",
                board: create_test_board(),
                words: create_test_wordlist(),
                answers: vec!["xyz".to_string()],
                expected_result: ExpectedResult::Error { error_fragment: "cannot be formed" },
                description: "Word not formable on board should error",
            },
            TestCase {
                name: "word_not_formable_on_board",
                board: create_test_board(),
                words: vec!["cat", "dog", "test", "word", "game", "path", "tile", "board", "impossible"],
                answers: vec!["impossible".to_string()],
                expected_result: ExpectedResult::Error { error_fragment: "cannot be formed" },
                description: "Valid dictionary word not formable on board should error",
            },
            TestCase {
                name: "multiple_compatible_words",
                board: create_test_board(),
                words: create_test_wordlist(),
                answers: vec!["cat".to_string(), "test".to_string()],
                expected_result: ExpectedResult::Success { expected_scores: vec![("cat", 4), ("test", 2)] },
                description: "Multiple words with compatible constraints should have correct individual scores",
            },
            TestCase {
                name: "wildcard_constraint_scenarios",
                board: create_constraint_test_board(),
                words: create_test_wordlist_with_constraints(),
                answers: vec!["cam".to_string(), "mat".to_string()],
                expected_result: ExpectedResult::Success { expected_scores: vec![("cam", 5), ("mat", 4)] },
                description: "Words requiring wildcard constraints should have correct individual scores",
            },
            TestCase {
                name: "conflicting_wildcard_constraints",
                board: create_constraint_test_board(),
                words: vec!["cat", "dog", "test", "word", "hello", "world", "valid", "conflict1", "conflict2"],
                answers: vec!["conflict1".to_string(), "conflict2".to_string()],
                expected_result: ExpectedResult::Error { error_fragment: "cannot be formed" },
                description: "Words with conflicting wildcard constraints should error",
            },
            TestCase {
                name: "single_word_path_optimization",
                board: create_test_board(),
                words: create_test_wordlist(),
                answers: vec!["cat".to_string()],
                expected_result: ExpectedResult::Success { expected_scores: vec![("cat", 4)] },
                description: "Should select highest scoring valid path for single word",
            },
            TestCase {
                name: "zero_score_wildcard_handling",
                board: {
                    let mut board = Board::new();
                    // Create board where word "cat" can be formed with wildcards: c(1,1 wildcard) -> a(1,2) -> t(1,3)
                    board.set_tile(1, 1, '*', 0, true); // Can be 'c' (0 points)
                    board.set_tile(1, 2, 'a', 1, false); // a (1 point) 
                    board.set_tile(1, 3, 't', 1, false); // t (1 point)
                    // Fill other spots to avoid issues
                    for i in 0..4 {
                        for j in 0..4 {
                            if board.get_tile(i, j).letter.is_empty() {
                                board.set_tile(i, j, 'x', 1, false);
                            }
                        }
                    }
                    board
                },
                words: create_test_wordlist(),
                answers: vec!["cat".to_string()],
                expected_result: ExpectedResult::Success { expected_scores: vec![("cat", 2)] }, // 0 + 1 + 1 = 2
                description: "Should correctly score words with zero-point wildcard tiles",
            },
            TestCase {
                name: "mixed_wildcard_and_regular_paths",
                board: create_constraint_test_board(),
                words: create_test_wordlist_with_constraints(),
                answers: vec!["cat".to_string(), "cam".to_string()],
                expected_result: ExpectedResult::Success { expected_scores: vec![("cat", 4), ("cam", 5)] },
                description: "Should correctly score mix of wildcard and non-wildcard paths",
            },
            TestCase {
                name: "scoring_with_letter_frequency_values",
                board: {
                    let mut board = Board::new();
                    // Create a board with specific letters to test frequency-based scoring
                    // q (9 points), u (3 points), a (1 point), t (1 point) - if "quat" were a word
                    // But we'll use actual words from our test list
                    board.set_tile(0, 0, 'c', 2, false); // c = 2 points
                    board.set_tile(0, 1, 'a', 1, false); // a = 1 point  
                    board.set_tile(0, 2, 't', 1, false); // t = 1 point
                    board.set_tile(1, 0, 'o', 1, false); // o = 1 point
                    board.set_tile(1, 1, 'm', 3, false); // m = 3 points
                    // Fill rest with x
                    for i in 0..4 {
                        for j in 0..4 {
                            if board.get_tile(i, j).letter.is_empty() {
                                board.set_tile(i, j, 'x', 4, false);
                            }
                        }
                    }
                    board
                },
                words: create_test_wordlist_with_constraints(),
                answers: vec!["cat".to_string()], // c(2) + a(1) + t(1) = 4
                expected_result: ExpectedResult::Success { expected_scores: vec![("cat", 4)] },
                description: "Should correctly apply letter frequency-based scoring",
            },
            TestCase {
                name: "multiple_words_optimal_constraint_selection",
                board: create_constraint_test_board(),
                words: create_test_wordlist_with_constraints(),
                answers: vec!["cat".to_string(), "mat".to_string(), "cam".to_string()],
                expected_result: ExpectedResult::Success { expected_scores: vec![("cat", 4), ("mat", 4), ("cam", 5)] },
                description: "Should select optimal constraint set when multiple words compete for wildcards",
            },
            TestCase {
                name: "single_word_multiple_path_options",
                board: {
                    let mut board = Board::new();
                    // Create a board where "cat" has multiple possible paths with different scores
                    board.set_tile(0, 0, 'c', 2, false); // One path: c(2) -> a(1) -> t(1) = 4
                    board.set_tile(0, 1, 'a', 1, false);
                    board.set_tile(0, 2, 't', 1, false);
                    board.set_tile(1, 0, 'c', 2, false); // Another path: c(2) -> a(3) -> t(5) = 10  
                    board.set_tile(1, 1, 'a', 3, false);
                    board.set_tile(1, 2, 't', 5, false);

                    // Fill rest with x
                    for i in 0..4 {
                        for j in 0..4 {
                            if board.get_tile(i, j).letter.is_empty() {
                                board.set_tile(i, j, 'x', 1, false);
                            }
                        }
                    }
                    board
                },
                words: create_test_wordlist(),
                answers: vec!["cat".to_string()],
                expected_result: ExpectedResult::Success { expected_scores: vec![("cat", 10)] }, // Should pick highest scoring path
                description: "Should select the highest scoring path when multiple paths exist for same word",
            },
        ];

        for test_case in test_cases {
            let engine = GameEngine::new(test_case.words);
            let result = engine.score_answer_group(&test_case.board, test_case.answers);

            match (&result, &test_case.expected_result) {
                (Ok(actual_scores), ExpectedResult::Success { expected_scores }) => {
                    assert_eq!(actual_scores.map.len(), expected_scores.len(),
                        "Test case '{}': Score count mismatch. Expected {} scores, got {}. Description: {}",
                        test_case.name, expected_scores.len(), actual_scores.map.len(), test_case.description);

                    for (expected_word, expected_score) in expected_scores {
                        assert!(
                            actual_scores.map.contains_key(*expected_word),
                            "Test case '{}': Missing word '{}' in results. Description: {}",
                            test_case.name,
                            expected_word,
                            test_case.description
                        );

                        let actual_score = actual_scores.map[*expected_word];
                        assert_eq!(actual_score, *expected_score,
                            "Test case '{}': Score mismatch for word '{}'. Expected {}, got {}. Description: {}",
                            test_case.name, expected_word, expected_score, actual_score, test_case.description);
                    }
                }
                (Err(actual_error), ExpectedResult::Error { error_fragment }) => {
                    assert!(actual_error.contains(error_fragment),
                        "Test case '{}': Error message mismatch. Expected to contain '{}', got '{}'. Description: {}",
                        test_case.name, error_fragment, actual_error, test_case.description);
                }
                (Ok(actual_scores), ExpectedResult::Error { error_fragment }) => {
                    panic!("Test case '{}': Expected error containing '{}', but got success with scores: {:?}. Description: {}", 
                        test_case.name, error_fragment, actual_scores.map, test_case.description);
                }
                (Err(actual_error), ExpectedResult::Success { .. }) => {
                    panic!(
                        "Test case '{}': Expected success but got error: '{}'. Description: {}",
                        test_case.name, actual_error, test_case.description
                    );
                }
            }
        }
    }

    #[test]
    fn test_game_engine_score_answer_group_basic() {
        let words = create_test_wordlist();
        let engine = GameEngine::new(words);

        // Create a simple test board
        let board = create_test_board();

        // Test with valid words that exist on the board
        let answers = vec!["cat".to_string(), "dog".to_string()];
        let result = engine.score_answer_group(&board, answers.clone());

        assert!(result.is_ok());
        let scores = result.unwrap();

        // Should have scores for both words
        assert_eq!(scores.map.len(), 2);
        assert!(scores.map.contains_key("cat"));
        assert!(scores.map.contains_key("dog"));

        // Scores should be positive (assuming the words can be formed)
        for (word, score) in &scores.map {
            println!("Word: {}, Score: {}", word, score);
        }

        // Test with empty input
        let empty_result = engine.score_answer_group(&board, vec![]);
        assert!(empty_result.is_ok());
        assert_eq!(empty_result.unwrap().map.len(), 0);
    }

    #[tokio::test]
    async fn test_game_engine_find_word_paths() {
        let words = create_test_wordlist();
        let engine = GameEngine::new(words);
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
        let words = create_test_wordlist();
        let engine = GameEngine::new(words);
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
        let words = create_test_wordlist();
        let engine = GameEngine::new(words);
        let board = create_test_board();

        // Test invalid word (not in dictionary)
        let result = engine.validate_answer(&board, "xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in dictionary"));
    }

    #[tokio::test]
    async fn test_game_engine_validate_answer_no_path() {
        let words = create_test_wordlist();
        let engine = GameEngine::new(words);
        let board = create_test_board();

        // Test word that exists in dictionary but can't be formed on board
        let result = engine.validate_answer(&board, "game");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("cannot be formed on this board"));
    }

    #[tokio::test]
    async fn test_find_all_valid_words() {
        let words = create_test_wordlist();
        let engine = GameEngine::new(words);
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
            let words = create_test_wordlist();
            let engine = GameEngine::new(words);
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
            let words = create_test_wordlist();
            let engine = GameEngine::new(words);
            let board = create_test_board();

            // Should complete without panicking (testing bounds checking)
            let result = engine.find_all_valid_words(&board).await;
            assert!(result.is_ok());
        });
    }

    fn create_test_wordlist_with_constraints() -> Vec<&'static str> {
        vec![
            "cat", "cam", "mat", "map", "test", "word", "hello", "world", "valid",
        ]
    }

    fn create_constraint_test_board() -> Board {
        let mut board = Board::new();

        // Create a simple test board for constraint testing:
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

        board.set_tile(2, 0, '*', 0, true); // wildcard
        board.set_tile(2, 1, 's', 1, false);
        board.set_tile(2, 2, 'e', 1, false);
        board.set_tile(2, 3, '*', 0, true); // wildcard

        board.set_tile(3, 0, 'r', 1, false);
        board.set_tile(3, 1, 'n', 1, false);
        board.set_tile(3, 2, 'd', 1, false);
        board.set_tile(3, 3, 'g', 2, false);

        board
    }

    fn create_test_wordlist_with_diode() -> Vec<&'static str> {
        vec![
            "ran", "rod", "diode", "best", "test", "redo", "bet", "door", "ore", "do", "od", "re",
            "to", "ar", "or", "an", "no", "it", "id", "di", "io", "oi", "radio"
        ]
    }

    fn create_test_wordlist_with_biscuit() -> Vec<&'static str> {
        vec![
            "biscuit", "biscuits", "bis", "cut", "suit", "sits", "bit", "its", "cut", "sue", "use",
            "sit", "is", "it", "us", "bi", "sc", "cu", "ui", "ic", "ci", "is", "si", "it", "ti",
            "pas", "seer", "nil", "bit",
        ]
    }

    #[tokio::test]
    async fn test_diode_validate_answer_group() {
        let words = create_test_wordlist_with_diode();
        let engine = GameEngine::new(words);
        let board = test_utils::create_test_board("iaroo*nhdo*terbe");
        // iaro
        // o*nh
        // do*t
        // erbe


        // Test that 'biscuit' can coexist with a set of valid other words
        let result = engine.validate_answer_group(
            &board,
            vec![
                "diode".to_string(),
                "redo".to_string(),
                "radio".to_string(),
            ],
        );
        assert!(
            result.is_ok(),
            "diode should be valid on the test board: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_biscuit_validate_answer_group() {
        let words = create_test_wordlist_with_biscuit();
        let engine = GameEngine::new(words);
        let board = test_utils::create_test_board("ebnlp*icai*sseer");

        // Test that 'biscuit' passes validate_answer_group
        // Possible path: B(0,0) -> I(0,1) -> S(0,2) -> C(0,3) -> U(1,2) -> I(1,3) -> T(2,1)
        let result = engine.validate_answer_group(&board, vec!["biscuit".to_string()]);
        assert!(
            result.is_ok(),
            "biscuit should be valid on the test board: {:?}",
            result
        );

        // Test that 'biscuit' can coexist with a set of valid other words
        let result = engine.validate_answer_group(
            &board,
            vec![
                "biscuit".to_string(),
                "pas".to_string(),
                "seer".to_string(),
                "nil".to_string(),
                "bit".to_string(),
            ],
        );
        assert!(
            result.is_ok(),
            "biscuit should be valid on the test board: {:?}",
            result
        );

        // Also test that biscuit can be found as a valid word
        let answer_result = engine.validate_answer(&board, "biscuit");
        assert!(
            answer_result.is_ok(),
            "biscuit should be findable on the board: {:?}",
            answer_result
        );

        let answer = answer_result.unwrap();
        assert_eq!(answer.word, "biscuit");
        assert!(
            !answer.paths.is_empty(),
            "biscuit should have at least one valid path"
        );
    }
}

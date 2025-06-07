use std::collections::HashMap;

const FREQUENCY_A: f32 = 0.078;
const FREQUENCY_B: f32 = 0.02;
const FREQUENCY_C: f32 = 0.04;
const FREQUENCY_D: f32 = 0.038;
const FREQUENCY_E: f32 = 0.11;
const FREQUENCY_F: f32 = 0.014;
const FREQUENCY_G: f32 = 0.03;
const FREQUENCY_H: f32 = 0.023;
const FREQUENCY_I: f32 = 0.086;
const FREQUENCY_J: f32 = 0.0021;
const FREQUENCY_K: f32 = 0.0097;
const FREQUENCY_L: f32 = 0.053;
const FREQUENCY_M: f32 = 0.027;
const FREQUENCY_N: f32 = 0.072;
const FREQUENCY_O: f32 = 0.061;
const FREQUENCY_P: f32 = 0.028;
const FREQUENCY_Q: f32 = 0.0019;
const FREQUENCY_R: f32 = 0.073;
const FREQUENCY_S: f32 = 0.087;
const FREQUENCY_T: f32 = 0.067;
const FREQUENCY_U: f32 = 0.033;
const FREQUENCY_V: f32 = 0.01;
const FREQUENCY_W: f32 = 0.0091;
const FREQUENCY_X: f32 = 0.0027;
const FREQUENCY_Y: f32 = 0.016;
const FREQUENCY_Z: f32 = 0.0044;

#[derive(Clone)]
pub struct Scorer {
    letter_points: HashMap<char, u32>,
}

impl Scorer {
    pub fn new() -> Self {
        Self {
            letter_points: Scorer::generate_point_values(),
        }
    }

    fn generate_letter_frequency_map() -> HashMap<char, f32> {
        let mut letter_frequencies = HashMap::new();
        letter_frequencies.insert('a', FREQUENCY_A);
        letter_frequencies.insert('b', FREQUENCY_B);
        letter_frequencies.insert('c', FREQUENCY_C);
        letter_frequencies.insert('d', FREQUENCY_D);
        letter_frequencies.insert('e', FREQUENCY_E);
        letter_frequencies.insert('f', FREQUENCY_F);
        letter_frequencies.insert('g', FREQUENCY_G);
        letter_frequencies.insert('h', FREQUENCY_H);
        letter_frequencies.insert('i', FREQUENCY_I);
        letter_frequencies.insert('j', FREQUENCY_J);
        letter_frequencies.insert('k', FREQUENCY_K);
        letter_frequencies.insert('l', FREQUENCY_L);
        letter_frequencies.insert('m', FREQUENCY_M);
        letter_frequencies.insert('n', FREQUENCY_N);
        letter_frequencies.insert('o', FREQUENCY_O);
        letter_frequencies.insert('p', FREQUENCY_P);
        letter_frequencies.insert('q', FREQUENCY_Q);
        letter_frequencies.insert('r', FREQUENCY_R);
        letter_frequencies.insert('s', FREQUENCY_S);
        letter_frequencies.insert('t', FREQUENCY_T);
        letter_frequencies.insert('u', FREQUENCY_U);
        letter_frequencies.insert('v', FREQUENCY_V);
        letter_frequencies.insert('w', FREQUENCY_W);
        letter_frequencies.insert('x', FREQUENCY_X);
        letter_frequencies.insert('y', FREQUENCY_Y);
        letter_frequencies.insert('z', FREQUENCY_Z);
        return letter_frequencies;
    }

    fn generate_point_values() -> HashMap<char, u32> {
        let mut letter_points = HashMap::new();
        let frequencies = Scorer::generate_letter_frequency_map();
        let e = frequencies[&'e'];
        for (letter, freq) in frequencies.into_iter() {
            letter_points.insert(letter, ((e / freq).log2().floor() as u32) + 1);
        }
        return letter_points;
    }

    pub fn score(&self, word: &str) -> u32 {
        word.chars().fold(0, |acc, e| acc + self.letter_points[&e])
    }

    pub fn get_letter_points(&self, letter: char) -> u32 {
        *self.letter_points.get(&letter).unwrap_or(&0)
    }
}

pub fn points_for_letter(letter: char, letter_frequencies: &std::collections::HashMap<char, f64>) -> i32 {
    let e_freq = letter_frequencies.get(&'e').unwrap_or(&0.11);
    let letter_freq = letter_frequencies.get(&letter.to_ascii_lowercase()).unwrap_or(&0.01);
    
    ((e_freq / letter_freq).log2().floor() as i32) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorer_new() {
        let scorer = Scorer::new();
        assert!(scorer.letter_points.len() == 26);
        assert!(scorer.letter_points.contains_key(&'e'));
        assert!(scorer.letter_points.contains_key(&'z'));
    }

    #[test]
    fn test_generate_letter_frequency_map() {
        let frequencies = Scorer::generate_letter_frequency_map();
        assert_eq!(frequencies.len(), 26);
        assert_eq!(frequencies[&'e'], FREQUENCY_E);
        assert_eq!(frequencies[&'q'], FREQUENCY_Q);
        assert_eq!(frequencies[&'z'], FREQUENCY_Z);
    }

    #[test]
    fn test_generate_point_values() {
        let point_values = Scorer::generate_point_values();
        assert_eq!(point_values.len(), 26);
        
        // E should have the lowest point value (1) since it's most frequent
        assert_eq!(point_values[&'e'], 1);
        
        // Q and Z should have high point values since they're rare
        assert!(point_values[&'q'] > point_values[&'e']);
        assert!(point_values[&'z'] > point_values[&'e']);
        
        // Verify specific calculations for known frequencies
        let e_freq = FREQUENCY_E;
        let a_freq = FREQUENCY_A;
        let expected_a_points = ((e_freq / a_freq).log2().floor() as u32) + 1;
        assert_eq!(point_values[&'a'], expected_a_points);
    }

    #[test]
    fn test_score_single_letter_words() {
        let scorer = Scorer::new();
        let e_score = scorer.score("e");
        let q_score = scorer.score("q");
        
        assert_eq!(e_score, 1); // E should be worth 1 point
        assert!(q_score > e_score); // Q should be worth more than E
    }

    #[test]
    fn test_score_multi_letter_words() {
        let scorer = Scorer::new();
        let cat_score = scorer.score("cat");
        let quiz_score = scorer.score("quiz");
        
        // Quiz should score higher due to Q and Z being rare letters
        assert!(quiz_score > cat_score);
        
        // Verify accumulation: score should equal sum of individual letter scores
        let c_score = scorer.get_letter_points('c');
        let a_score = scorer.get_letter_points('a');
        let t_score = scorer.get_letter_points('t');
        assert_eq!(cat_score, c_score + a_score + t_score);
    }

    #[test]
    fn test_score_empty_string() {
        let scorer = Scorer::new();
        assert_eq!(scorer.score(""), 0);
    }

    #[test]
    fn test_get_letter_points_valid_letters() {
        let scorer = Scorer::new();
        
        // Test specific known values
        assert_eq!(scorer.get_letter_points('e'), 1);
        assert!(scorer.get_letter_points('q') > 1);
        assert!(scorer.get_letter_points('z') > 1);
        
        // Test consistency with scoring
        let word = "test";
        let manual_score: u32 = word.chars()
            .map(|c| scorer.get_letter_points(c))
            .sum();
        assert_eq!(scorer.score(word), manual_score);
    }

    #[test]
    fn test_get_letter_points_invalid_letter() {
        let scorer = Scorer::new();
        
        // Non-alphabetic characters should return 0
        assert_eq!(scorer.get_letter_points('1'), 0);
        assert_eq!(scorer.get_letter_points('!'), 0);
        assert_eq!(scorer.get_letter_points(' '), 0);
    }

    #[test]
    fn test_get_letter_points_case_sensitivity() {
        let scorer = Scorer::new();
        
        // Currently only lowercase letters are in the map
        assert!(scorer.get_letter_points('a') > 0);
        assert_eq!(scorer.get_letter_points('A'), 0); // Uppercase not in map
    }

    #[test]
    fn test_points_for_letter_function() {
        let mut frequencies = HashMap::new();
        frequencies.insert('e', 0.11);
        frequencies.insert('a', 0.078);
        frequencies.insert('q', 0.0019);
        
        let e_points = points_for_letter('e', &frequencies);
        let a_points = points_for_letter('a', &frequencies);
        let q_points = points_for_letter('q', &frequencies);
        
        assert_eq!(e_points, 1); // E should be 1 point (e_freq / e_freq = 1, log2(1) = 0, floor(0) + 1 = 1)
        assert!(a_points >= e_points); // A is less frequent than E, so should be worth same or more
        assert!(q_points > a_points); // Q much less frequent than A
        
        // Verify the calculation for 'a': log2(0.11/0.078) + 1
        let expected_a = ((0.11f64 / 0.078f64).log2().floor() as i32) + 1;
        assert_eq!(a_points, expected_a);
    }

    #[test]
    fn test_points_for_letter_missing_e_frequency() {
        let mut frequencies = HashMap::new();
        frequencies.insert('a', 0.078);
        
        let a_points = points_for_letter('a', &frequencies);
        
        // Should use default E frequency of 0.11
        let expected = ((0.11f64 / 0.078f64).log2().floor() as i32) + 1;
        assert_eq!(a_points, expected);
    }

    #[test]
    fn test_points_for_letter_missing_letter_frequency() {
        let mut frequencies = HashMap::new();
        frequencies.insert('e', 0.11);
        
        let unknown_points = points_for_letter('x', &frequencies);
        
        // Should use default letter frequency of 0.01
        let expected = ((0.11f64 / 0.01f64).log2().floor() as i32) + 1;
        assert_eq!(unknown_points, expected);
    }

    #[test]
    fn test_points_for_letter_case_insensitive() {
        let mut frequencies = HashMap::new();
        frequencies.insert('e', 0.11);
        frequencies.insert('a', 0.078);
        
        let lowercase_points = points_for_letter('a', &frequencies);
        let uppercase_points = points_for_letter('A', &frequencies);
        
        assert_eq!(lowercase_points, uppercase_points);
    }

    #[test]
    fn test_frequency_constants() {
        // Verify that the frequency constants are reasonable
        assert!(FREQUENCY_E > 0.0 && FREQUENCY_E < 1.0);
        assert!(FREQUENCY_Q > 0.0 && FREQUENCY_Q < 1.0);
        
        // E should be the most frequent letter
        assert!(FREQUENCY_E > FREQUENCY_A);
        assert!(FREQUENCY_E > FREQUENCY_Q);
        assert!(FREQUENCY_E > FREQUENCY_Z);
        
        // Q and Z should be among the least frequent
        assert!(FREQUENCY_Q < FREQUENCY_A);
        assert!(FREQUENCY_Z < FREQUENCY_A);
    }
}
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

pub fn points_for_letter(letter: char, letter_frequencies: &std::collections::HashMap<char, f64>) -> i32 {
    let e_freq = letter_frequencies.get(&'e').unwrap_or(&0.11);
    let letter_freq = letter_frequencies.get(&letter.to_ascii_lowercase()).unwrap_or(&0.01);
    
    ((e_freq / letter_freq).log2().floor() as i32) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

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

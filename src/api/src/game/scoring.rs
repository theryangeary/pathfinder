use std::collections::HashMap;
use phf::phf_map;

static LETTER_FREQUENCIES: phf::Map<char, f64> = phf_map! {
    'a' => 0.078,
    'b' => 0.02,
    'c' => 0.04,
    'd' => 0.038,
    'e' => 0.11,
    'f' => 0.014,
    'g' => 0.03,
    'h' => 0.023,
    'i' => 0.086,
    'j' => 0.0021,
    'k' => 0.0097,
    'l' => 0.053,
    'm' => 0.027,
    'n' => 0.072,
    'o' => 0.061,
    'p' => 0.028,
    'q' => 0.0019,
    'r' => 0.073,
    's' => 0.087,
    't' => 0.067,
    'u' => 0.033,
    'v' => 0.01,
    'w' => 0.0091,
    'x' => 0.0027,
    'y' => 0.016,
    'z' => 0.0044,
};

pub fn points_for_letter(letter: char) -> i32 {
    let e_freq = LETTER_FREQUENCIES.get(&'e').unwrap_or(&0.11);
    let letter_freq = LETTER_FREQUENCIES.get(&letter.to_ascii_lowercase()).unwrap_or(&0.01);
    
    ((e_freq / letter_freq).log2().floor() as i32) + 1
}

pub struct ScoreSheet{
    pub map: HashMap<String, u32>
}

impl ScoreSheet {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }
}

impl From<HashMap<String, u32>> for ScoreSheet {
    fn from(map: HashMap<String, u32>) -> Self {
        Self {map}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_points_for_letter_function() {
        let e_points = points_for_letter('e');
        let a_points = points_for_letter('a');
        let q_points = points_for_letter('q');
        
        assert_eq!(e_points, 1); // E should be 1 point (e_freq / e_freq = 1, log2(1) = 0, floor(0) + 1 = 1)
        assert!(a_points >= e_points); // A is less frequent than E, so should be worth same or more
        assert!(q_points > a_points); // Q much less frequent than A
        
        // Verify the calculation for 'a': log2(0.11/0.078) + 1
        let expected_a = ((0.11f64 / 0.078f64).log2().floor() as i32) + 1;
        assert_eq!(a_points, expected_a);
    }

    #[test]
    fn test_points_for_letter_uses_static_frequencies() {
        let a_points = points_for_letter('a');
        
        // Should use static frequency map
        let expected = ((0.11f64 / 0.078f64).log2().floor() as i32) + 1;
        assert_eq!(a_points, expected);
    }

    #[test]
    fn test_points_for_letter_unknown_letter_fallback() {
        // Test with a character not in our frequency map
        let unknown_points = points_for_letter('🙂');
        
        // Should use default letter frequency of 0.01
        let expected = ((0.11f64 / 0.01f64).log2().floor() as i32) + 1;
        assert_eq!(unknown_points, expected);
    }

    #[test]
    fn test_points_for_letter_case_insensitive() {
        let lowercase_points = points_for_letter('a');
        let uppercase_points = points_for_letter('A');
        
        assert_eq!(lowercase_points, uppercase_points);
    }

    #[test]
    fn test_letter_frequencies() {
        // Verify that the frequency map contains reasonable values
        let e_freq = LETTER_FREQUENCIES.get(&'e').unwrap();
        let q_freq = LETTER_FREQUENCIES.get(&'q').unwrap();
        let a_freq = LETTER_FREQUENCIES.get(&'a').unwrap();
        let z_freq = LETTER_FREQUENCIES.get(&'z').unwrap();
        
        assert!(*e_freq > 0.0 && *e_freq < 1.0);
        assert!(*q_freq > 0.0 && *q_freq < 1.0);
        
        // E should be the most frequent letter
        assert!(*e_freq > *a_freq);
        assert!(*e_freq > *q_freq);
        assert!(*e_freq > *z_freq);
        
        // Q and Z should be among the least frequent
        assert!(*q_freq < *a_freq);
        assert!(*z_freq < *a_freq);
    }
}

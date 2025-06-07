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
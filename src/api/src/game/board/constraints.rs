
use std::collections::{HashMap, HashSet};

use crate::game::board::constraints;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UnsatisfiableConstraint;

// Constraint represents the constraint imposed upon a single wildcard tile.
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    // Unconstrainted means that the wildcard tile is unused and therefore could represent any letter
    Unconstrainted,
    // Decided means that the wildcard must be a single, specific letter and cannot be any other letter
    Decided(char),
    // AnyOf means that the wildcard must be one of more than one letters
    AnyOf(Vec<char>),
}

impl Constraint {
    fn merge(&self, other: Constraint) -> Result<Constraint, UnsatisfiableConstraint> {
        match self {
            Constraint::Unconstrainted => Ok(other),
            Constraint::Decided(c) => match other {
                Constraint::Unconstrainted => Ok(Constraint::Decided(*c)),
                Constraint::Decided(d) => {
                    if *c == d {
                        Ok(Constraint::Decided(*c))
                    } else {
                        Err(UnsatisfiableConstraint)
                    }
                }
                Constraint::AnyOf(options) => {
                    Constraint::merge_decided_with_any_of(*c, options)
                }
            },
            Constraint::AnyOf(options) => match other {
                Constraint::Unconstrainted => Ok(Constraint::AnyOf(options.clone())),
                Constraint::Decided(decided) => Constraint::merge_decided_with_any_of(decided, options.clone()),
                Constraint::AnyOf(options2) => {
                    let h1:HashSet<char> = options.iter().cloned().collect();
                    let h2:HashSet<char> = options2.iter().cloned().collect();
                    let v:Vec<char> = h1.intersection(&h2).cloned().collect();
                    if v.len() == 0 {
                        return Err(UnsatisfiableConstraint);
                    } else if v.len() == 1 {
                        return Ok(Constraint::Decided(v[0]));
                    } else {
                        return Ok(Constraint::AnyOf(v))
                    }
                },
            },
        }
    }

    fn merge_decided_with_any_of(
        decided: char,
        any_of_vec: Vec<char>,
    ) -> Result<Constraint, UnsatisfiableConstraint> {
        for c in any_of_vec {
            if decided == c {
                return Ok(Constraint::Decided(decided));
            }
        }
        Err(UnsatisfiableConstraint)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constraints(pub HashMap<String, char>);
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintsSet(pub HashMap<String, Constraint>);

impl ConstraintsSet {
    pub fn new() -> Self {
        ConstraintsSet(HashMap::new())
    }

    pub fn has_collision_with(&self, other: &ConstraintsSet) -> bool {
        for (key, other_constraint) in &other.0 {
            if let Some(self_constraint) = self.0.get(key) {
                if self_constraint.merge(other_constraint.clone()).is_err() {
                    return true;
                }
            }
        }
        false
    }

    pub fn intersection(&self, other: &ConstraintsSet) -> Result<ConstraintsSet, UnsatisfiableConstraint> {
        let mut result = self.0.clone();
        
        for (key, other_constraint) in &other.0 {
            if let Some(self_constraint) = self.0.get(key) {
                // Both sets have this key, merge the constraints
                let merged_constraint = self_constraint.merge(other_constraint.clone())?;
                result.insert(key.clone(), merged_constraint);
            } else {
                // Only other set has this key, add it to result
                result.insert(key.clone(), other_constraint.clone());
            }
        }
        
        Ok(ConstraintsSet(result))
    }
}

impl Constraints {
    pub fn new() -> Self {
        Constraints(HashMap::new())
    }

    pub fn has_collision_with(&self, other: &Constraints) -> bool {
        let mut merged = self.0.clone();
        for (k, v) in (&other.0).into_iter() {
            let previous = merged.insert(k.to_string(), *v);
            if let Some(p) = previous {
                if p != *v {
                    return true;
                }
            }
        }
        return false;
    }

    // returns None if the intersection of constraints is invalid
    pub fn intersection(&self, other: &Constraints) -> Option<Constraints> {
        let mut merged = self.0.clone();
        for (k, v) in other.0.iter() {
            let previous = merged.insert(k.to_string(), *v);
            if let Some(p) = previous {
                if p != *v {
                    return None;
                }
            }
        }
        return Some(Constraints(merged));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const X: &str = "1_1";
    const Y: &str = "2_2";

    fn constraints_from(xi: Option<char>, yi: Option<char>) -> Constraints {
        let mut c = HashMap::new();
        if let Some(x) = xi {
            c.insert(X.into(), x);
        }
        if let Some(y) = yi {
            c.insert(Y.into(), y);
        }
        return Constraints(c);
    }

    #[test]
    fn test_has_collision_with() {
        let ab = constraints_from(Some('a'), Some('b'));
        let ac = constraints_from(Some('a'), Some('c'));

        assert!(ab.has_collision_with(&ac));
    }

    #[test]
    fn test_constraint_merge() {
        struct TestCase {
            name: &'static str,
            c1: Constraint,
            c2: Constraint,
            expected: Result<Constraint, UnsatisfiableConstraint>,
        }

        let test_cases = vec![
            // === Unconstrainted + X cases ===
            TestCase {
                name: "Unconstrainted + Unconstrainted",
                c1: Constraint::Unconstrainted,
                c2: Constraint::Unconstrainted,
                expected: Ok(Constraint::Unconstrainted),
            },
            TestCase {
                name: "Unconstrainted + Decided",
                c1: Constraint::Unconstrainted,
                c2: Constraint::Decided('a'),
                expected: Ok(Constraint::Decided('a')),
            },
            TestCase {
                name: "Unconstrainted + AnyOf (single)",
                c1: Constraint::Unconstrainted,
                c2: Constraint::AnyOf(vec!['a']),
                expected: Ok(Constraint::AnyOf(vec!['a'])),
            },
            TestCase {
                name: "Unconstrainted + AnyOf (multiple)",
                c1: Constraint::Unconstrainted,
                c2: Constraint::AnyOf(vec!['a', 'b', 'c']),
                expected: Ok(Constraint::AnyOf(vec!['a', 'b', 'c'])),
            },
            TestCase {
                name: "Unconstrainted + AnyOf (empty)",
                c1: Constraint::Unconstrainted,
                c2: Constraint::AnyOf(vec![]),
                expected: Ok(Constraint::AnyOf(vec![])),
            },

            // === Decided + X cases ===
            TestCase {
                name: "Decided + Unconstrainted",
                c1: Constraint::Decided('a'),
                c2: Constraint::Unconstrainted,
                expected: Ok(Constraint::Decided('a')),
            },
            TestCase {
                name: "Decided + Decided (same)",
                c1: Constraint::Decided('a'),
                c2: Constraint::Decided('a'),
                expected: Ok(Constraint::Decided('a')),
            },
            TestCase {
                name: "Decided + Decided (different)",
                c1: Constraint::Decided('a'),
                c2: Constraint::Decided('b'),
                expected: Err(UnsatisfiableConstraint),
            },
            TestCase {
                name: "Decided + AnyOf (contained)",
                c1: Constraint::Decided('a'),
                c2: Constraint::AnyOf(vec!['a', 'b', 'c']),
                expected: Ok(Constraint::Decided('a')),
            },
            TestCase {
                name: "Decided + AnyOf (not contained)",
                c1: Constraint::Decided('d'),
                c2: Constraint::AnyOf(vec!['a', 'b', 'c']),
                expected: Err(UnsatisfiableConstraint),
            },
            TestCase {
                name: "Decided + AnyOf (single, matching)",
                c1: Constraint::Decided('a'),
                c2: Constraint::AnyOf(vec!['a']),
                expected: Ok(Constraint::Decided('a')),
            },
            TestCase {
                name: "Decided + AnyOf (single, not matching)",
                c1: Constraint::Decided('a'),
                c2: Constraint::AnyOf(vec!['b']),
                expected: Err(UnsatisfiableConstraint),
            },
            TestCase {
                // N.B. this case is never expected to happen
                name: "Decided + AnyOf (empty)",
                c1: Constraint::Decided('a'),
                c2: Constraint::AnyOf(vec![]),
                expected: Err(UnsatisfiableConstraint),
            },

            // === AnyOf + X cases ===
            TestCase {
                name: "AnyOf + Unconstrainted",
                c1: Constraint::AnyOf(vec!['a', 'b']),
                c2: Constraint::Unconstrainted,
                expected: Ok(Constraint::AnyOf(vec!['a', 'b'])),
            },
            TestCase {
                name: "AnyOf + Decided (contained)",
                c1: Constraint::AnyOf(vec!['a', 'b', 'c']),
                c2: Constraint::Decided('b'),
                expected: Ok(Constraint::Decided('b')),
            },
            TestCase {
                name: "AnyOf + Decided (not contained)",
                c1: Constraint::AnyOf(vec!['a', 'b', 'c']),
                c2: Constraint::Decided('d'),
                expected: Err(UnsatisfiableConstraint),
            },
            TestCase {
                // N.B. this case is never expected to happen
                name: "AnyOf (empty) + Decided",
                c1: Constraint::AnyOf(vec![]),
                c2: Constraint::Decided('a'),
                expected: Err(UnsatisfiableConstraint),
            },

            // === AnyOf + AnyOf cases ===
            TestCase {
                name: "AnyOf + AnyOf (full overlap)",
                c1: Constraint::AnyOf(vec!['a', 'b']),
                c2: Constraint::AnyOf(vec!['a', 'b']),
                expected: Ok(Constraint::AnyOf(vec!['a', 'b'])),
            },
            TestCase {
                name: "AnyOf + AnyOf (partial overlap - multiple)",
                c1: Constraint::AnyOf(vec!['a', 'b', 'c']),
                c2: Constraint::AnyOf(vec!['b', 'c', 'd']),
                expected: Ok(Constraint::AnyOf(vec!['b', 'c'])),
            },
            TestCase {
                name: "AnyOf + AnyOf (partial overlap - single)",
                c1: Constraint::AnyOf(vec!['a', 'b']),
                c2: Constraint::AnyOf(vec!['b', 'c']),
                expected: Ok(Constraint::Decided('b')),
            },
            TestCase {
                name: "AnyOf + AnyOf (no overlap)",
                c1: Constraint::AnyOf(vec!['a', 'b']),
                c2: Constraint::AnyOf(vec!['c', 'd']),
                expected: Err(UnsatisfiableConstraint),
            },
            TestCase {
                name: "AnyOf + AnyOf (one empty)",
                c1: Constraint::AnyOf(vec!['a', 'b']),
                c2: Constraint::AnyOf(vec![]),
                expected: Err(UnsatisfiableConstraint),
            },
            TestCase {
                name: "AnyOf + AnyOf (both empty)",
                c1: Constraint::AnyOf(vec![]),
                c2: Constraint::AnyOf(vec![]),
                expected: Err(UnsatisfiableConstraint),
            },
            TestCase {
                name: "AnyOf (single) + AnyOf (single, same)",
                c1: Constraint::AnyOf(vec!['a']),
                c2: Constraint::AnyOf(vec!['a']),
                expected: Ok(Constraint::Decided('a')),
            },
            TestCase {
                name: "AnyOf (single) + AnyOf (single, different)",
                c1: Constraint::AnyOf(vec!['a']),
                c2: Constraint::AnyOf(vec!['b']),
                expected: Err(UnsatisfiableConstraint),
            },
            TestCase {
                name: "AnyOf (single) + AnyOf (multiple, contained)",
                c1: Constraint::AnyOf(vec!['a']),
                c2: Constraint::AnyOf(vec!['a', 'b', 'c']),
                expected: Ok(Constraint::Decided('a')),
            },
            TestCase {
                name: "AnyOf (single) + AnyOf (multiple, not contained)",
                c1: Constraint::AnyOf(vec!['a']),
                c2: Constraint::AnyOf(vec!['b', 'c', 'd']),
                expected: Err(UnsatisfiableConstraint),
            },

            // === Edge cases with duplicate characters ===
            TestCase {
                name: "AnyOf + AnyOf (with duplicates)",
                c1: Constraint::AnyOf(vec!['a', 'b', 'a']),
                c2: Constraint::AnyOf(vec!['b', 'c', 'b']),
                expected: Ok(Constraint::Decided('b')),
            },
            TestCase {
                name: "Decided + AnyOf (with duplicates, contained)",
                c1: Constraint::Decided('a'),
                c2: Constraint::AnyOf(vec!['a', 'b', 'a']),
                expected: Ok(Constraint::Decided('a')),
            },
        ];

        for test_case in test_cases {
            let result = test_case.c1.merge(test_case.c2);
            match (&result, &test_case.expected) {
                (Ok(Constraint::AnyOf(actual)), Ok(Constraint::AnyOf(expected))) => {
                    // For AnyOf constraints, we need to compare sets since order doesn't matter
                    let actual_set: std::collections::HashSet<char> = actual.iter().cloned().collect();
                    let expected_set: std::collections::HashSet<char> = expected.iter().cloned().collect();
                    assert_eq!(actual_set, expected_set, "Failed test case: {}", test_case.name);
                }
                _ => {
                    assert_eq!(result, test_case.expected, "Failed test case: {}", test_case.name);
                }
            }
        }
    }

    #[test]
    fn test_merge_decided_with_any_of() {
        // Test the helper function directly
        assert_eq!(
            Constraint::merge_decided_with_any_of('a', vec!['a', 'b', 'c']),
            Ok(Constraint::Decided('a'))
        );
        
        assert_eq!(
            Constraint::merge_decided_with_any_of('d', vec!['a', 'b', 'c']),
            Err(UnsatisfiableConstraint)
        );
        
        assert_eq!(
            Constraint::merge_decided_with_any_of('a', vec![]),
            Err(UnsatisfiableConstraint)
        );
        
        assert_eq!(
            Constraint::merge_decided_with_any_of('a', vec!['a']),
            Ok(Constraint::Decided('a'))
        );
        
        // Test with duplicates
        assert_eq!(
            Constraint::merge_decided_with_any_of('a', vec!['a', 'b', 'a']),
            Ok(Constraint::Decided('a'))
        );
    }

    struct ConstraintsSetTestCase {
        name: &'static str,
        set1: ConstraintsSet,
        set2: ConstraintsSet,
        expected_collision: bool,
    }

    fn constraints_set_from(pairs: Vec<(&str, Constraint)>) -> ConstraintsSet {
        let mut map = HashMap::new();
        for (key, constraint) in pairs {
            map.insert(key.to_string(), constraint);
        }
        ConstraintsSet(map)
    }

    fn create_constraints_set_test_cases() -> Vec<ConstraintsSetTestCase> {
        vec![
            // === Empty sets ===
            ConstraintsSetTestCase {
                name: "Empty sets",
                set1: constraints_set_from(vec![]),
                set2: constraints_set_from(vec![]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Empty vs non-empty",
                set1: constraints_set_from(vec![]),
                set2: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Non-empty vs empty",
                set1: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                set2: constraints_set_from(vec![]),
                expected_collision: false,
            },

            // === Non-overlapping keys ===
            ConstraintsSetTestCase {
                name: "Different keys - no collision",
                set1: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                set2: constraints_set_from(vec![("w2", Constraint::Decided('b'))]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Multiple different keys - no collision",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y']))
                ]),
                set2: constraints_set_from(vec![
                    ("w3", Constraint::Decided('b')),
                    ("w4", Constraint::Unconstrainted)
                ]),
                expected_collision: false,
            },

            // === Same keys, compatible constraints ===
            ConstraintsSetTestCase {
                name: "Unconstrainted + Unconstrainted",
                set1: constraints_set_from(vec![("w1", Constraint::Unconstrainted)]),
                set2: constraints_set_from(vec![("w1", Constraint::Unconstrainted)]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Unconstrainted + Decided",
                set1: constraints_set_from(vec![("w1", Constraint::Unconstrainted)]),
                set2: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Decided + Unconstrainted",
                set1: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                set2: constraints_set_from(vec![("w1", Constraint::Unconstrainted)]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Decided + Decided (same)",
                set1: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                set2: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Unconstrainted + AnyOf",
                set1: constraints_set_from(vec![("w1", Constraint::Unconstrainted)]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b']))]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "AnyOf + Unconstrainted",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b']))]),
                set2: constraints_set_from(vec![("w1", Constraint::Unconstrainted)]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Decided + AnyOf (contained)",
                set1: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b', 'c']))]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "AnyOf + Decided (contained)",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b', 'c']))]),
                set2: constraints_set_from(vec![("w1", Constraint::Decided('b'))]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "AnyOf + AnyOf (overlap)",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b']))]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['b', 'c']))]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "AnyOf + AnyOf (full overlap)",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b']))]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b']))]),
                expected_collision: false,
            },

            // === Same keys, incompatible constraints ===
            ConstraintsSetTestCase {
                name: "Decided + Decided (different)",
                set1: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                set2: constraints_set_from(vec![("w1", Constraint::Decided('b'))]),
                expected_collision: true,
            },
            ConstraintsSetTestCase {
                name: "Decided + AnyOf (not contained)",
                set1: constraints_set_from(vec![("w1", Constraint::Decided('d'))]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b', 'c']))]),
                expected_collision: true,
            },
            ConstraintsSetTestCase {
                name: "AnyOf + Decided (not contained)",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b', 'c']))]),
                set2: constraints_set_from(vec![("w1", Constraint::Decided('d'))]),
                expected_collision: true,
            },
            ConstraintsSetTestCase {
                name: "AnyOf + AnyOf (no overlap)",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b']))]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['c', 'd']))]),
                expected_collision: true,
            },
            ConstraintsSetTestCase {
                name: "Decided + AnyOf (empty)",
                set1: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec![]))]),
                expected_collision: true,
            },
            ConstraintsSetTestCase {
                name: "AnyOf (empty) + Decided",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec![]))]),
                set2: constraints_set_from(vec![("w1", Constraint::Decided('a'))]),
                expected_collision: true,
            },
            ConstraintsSetTestCase {
                name: "AnyOf (empty) + AnyOf (empty)",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec![]))]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec![]))]),
                expected_collision: true,
            },

            // === Multiple keys with mixed scenarios ===
            ConstraintsSetTestCase {
                name: "Mixed keys - one collision",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::Unconstrainted)
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::Decided('b')), // collision here
                    ("w2", Constraint::Decided('x'))  // no collision
                ]),
                expected_collision: true,
            },
            ConstraintsSetTestCase {
                name: "Mixed keys - no collisions",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y']))
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),     // no collision
                    ("w2", Constraint::AnyOf(vec!['x']))  // no collision (subset)
                ]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Partial key overlap - compatible",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y']))
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::Unconstrainted), // no collision
                    ("w3", Constraint::Decided('z'))    // different key
                ]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Partial key overlap - incompatible",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y']))
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::Decided('b')),   // collision here
                    ("w3", Constraint::Decided('z'))    // different key
                ]),
                expected_collision: true,
            },

            // === Complex multi-wildcard scenarios ===
            ConstraintsSetTestCase {
                name: "Multiple wildcards - all compatible",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y', 'z'])),
                    ("w3", Constraint::Unconstrainted)
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::AnyOf(vec!['a', 'b'])),
                    ("w2", Constraint::Decided('y')),
                    ("w3", Constraint::AnyOf(vec!['p', 'q']))
                ]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Multiple wildcards - one incompatible",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y', 'z'])),
                    ("w3", Constraint::Unconstrainted)
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::AnyOf(vec!['a', 'b'])),
                    ("w2", Constraint::Decided('w')),  // collision here
                    ("w3", Constraint::AnyOf(vec!['p', 'q']))
                ]),
                expected_collision: true,
            },

            // === Edge cases ===
            ConstraintsSetTestCase {
                name: "Single char AnyOf collision",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a']))]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['b']))]),
                expected_collision: true,
            },
            ConstraintsSetTestCase {
                name: "Single char AnyOf compatible",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a']))]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a']))]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Duplicate chars in AnyOf",
                set1: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['a', 'b', 'a']))]),
                set2: constraints_set_from(vec![("w1", Constraint::AnyOf(vec!['b', 'c', 'b']))]),
                expected_collision: false, // overlap on 'b'
            },
        ]
    }

    #[test]
    fn test_constraints_set_has_collision_with() {
        let test_cases = create_constraints_set_test_cases();

        for test_case in test_cases {
            let result = test_case.set1.has_collision_with(&test_case.set2);
            assert_eq!(
                result, 
                test_case.expected_collision,
                "Failed test case: {} - expected collision: {}, got: {}",
                test_case.name,
                test_case.expected_collision,
                result
            );
        }
    }

    #[test]
    fn test_constraints_set_intersection() {
        let test_cases = create_constraints_set_test_cases();

        for test_case in test_cases {
            let result = test_case.set1.intersection(&test_case.set2);
            
            if test_case.expected_collision {
                // If there's a collision, intersection should return an error
                assert!(
                    result.is_err(),
                    "Failed test case: {} - expected intersection error but got Ok",
                    test_case.name
                );
            } else {
                // If there's no collision, intersection should succeed
                assert!(
                    result.is_ok(),
                    "Failed test case: {} - expected intersection success but got Err",
                    test_case.name
                );
                
                // Verify the result makes sense by checking it doesn't have collision with either input
                let intersection = result.unwrap();
                assert!(
                    !intersection.has_collision_with(&test_case.set1),
                    "Failed test case: {} - intersection result has collision with set1",
                    test_case.name
                );
                assert!(
                    !intersection.has_collision_with(&test_case.set2),
                    "Failed test case: {} - intersection result has collision with set2",
                    test_case.name
                );
            }
        }
    }
}

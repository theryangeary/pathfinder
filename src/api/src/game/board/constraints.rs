use std::collections::{HashMap, HashSet};

use crate::game::board::answer::Answer;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UnsatisfiableConstraint;

/// PathConstraintSet represents the constraints imposed upon all wildcard tiles on the board for a particular Path
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PathConstraintSet {
    // Unconstrainted means that the wildcard tile is unused and therefore could represent any letter
    Unconstrainted,
    // FirstDecided means that the first wildcard must be a particular letter and cannot be any other letter. The second wildcard is unconstrainted.
    FirstDecided(char),
    // SecondDecided means that the second wildcard must be a particular letter and cannot be any other letter. The first wildcard is unconstrained.
    SecondDecided(char),
    // BothDecided means that both wildcards must respectively be a single, specific letter and cannot be any other letter
    BothDecided(char, char),
}

impl PathConstraintSet {
    pub fn merge(
        &self,
        other: PathConstraintSet,
    ) -> Result<PathConstraintSet, UnsatisfiableConstraint> {
        match self {
            PathConstraintSet::Unconstrainted => Ok(other),
            PathConstraintSet::FirstDecided(first) => match other {
                PathConstraintSet::Unconstrainted => Ok(PathConstraintSet::FirstDecided(*first)),
                PathConstraintSet::FirstDecided(other_first) => {
                    if *first == other_first {
                        Ok(PathConstraintSet::FirstDecided(*first))
                    } else {
                        Err(UnsatisfiableConstraint)
                    }
                }
                PathConstraintSet::SecondDecided(second) => {
                    Ok(PathConstraintSet::BothDecided(*first, second))
                }
                PathConstraintSet::BothDecided(other_first, other_second) => {
                    if *first == other_first {
                        Ok(PathConstraintSet::BothDecided(*first, other_second))
                    } else {
                        Err(UnsatisfiableConstraint)
                    }
                }
            },
            PathConstraintSet::SecondDecided(second) => match other {
                PathConstraintSet::Unconstrainted => Ok(PathConstraintSet::SecondDecided(*second)),
                PathConstraintSet::FirstDecided(first) => {
                    Ok(PathConstraintSet::BothDecided(first, *second))
                }
                PathConstraintSet::SecondDecided(other_second) => {
                    if *second == other_second {
                        Ok(PathConstraintSet::SecondDecided(*second))
                    } else {
                        Err(UnsatisfiableConstraint)
                    }
                }
                PathConstraintSet::BothDecided(other_first, other_second) => {
                    if *second == other_second {
                        Ok(PathConstraintSet::BothDecided(other_first, *second))
                    } else {
                        Err(UnsatisfiableConstraint)
                    }
                }
            },
            PathConstraintSet::BothDecided(first, second) => match other {
                PathConstraintSet::Unconstrainted => {
                    Ok(PathConstraintSet::BothDecided(*first, *second))
                }
                PathConstraintSet::FirstDecided(other_first) => {
                    if *first == other_first {
                        Ok(PathConstraintSet::BothDecided(*first, *second))
                    } else {
                        Err(UnsatisfiableConstraint)
                    }
                }
                PathConstraintSet::SecondDecided(other_second) => {
                    if *second == other_second {
                        Ok(PathConstraintSet::BothDecided(*first, *second))
                    } else {
                        Err(UnsatisfiableConstraint)
                    }
                }
                PathConstraintSet::BothDecided(other_first, other_second) => {
                    if *first == other_first && *second == other_second {
                        Ok(PathConstraintSet::BothDecided(*first, *second))
                    } else {
                        Err(UnsatisfiableConstraint)
                    }
                }
            },
        }
    }
}

/// AnswerGroupConstraintSet represents the constraints which, if one of them is satisfied, allows a set of words to exist on the board
#[derive(Debug, Clone, PartialEq)]
pub struct AnswerGroupConstraintSet {
    pub path_constraint_sets: Vec<PathConstraintSet>,
}

impl From<Vec<PathConstraintSet>> for AnswerGroupConstraintSet {
    fn from(path_constraint_sets: Vec<PathConstraintSet>) -> Self {
        Self {
            path_constraint_sets,
        }
    }
}

impl AnswerGroupConstraintSet {
    /// intersection iterates through the path_constraint_sets from self and other, nested, and finds any PathConstraintSets which can validly merge
    pub fn intersection(
        &self,
        other: AnswerGroupConstraintSet,
    ) -> Result<AnswerGroupConstraintSet, UnsatisfiableConstraint> {
        let mut result_sets = Vec::new();

        for self_constraint in &self.path_constraint_sets {
            for other_constraint in &other.path_constraint_sets {
                if let Ok(merged) = self_constraint.merge(*other_constraint) {
                    result_sets.push(merged);
                }
            }
        }

        if result_sets.is_empty() {
            Err(UnsatisfiableConstraint)
        } else {
            Ok(AnswerGroupConstraintSet {
                path_constraint_sets: result_sets,
            })
        }
    }

    /// merge_all iterates through the AnswerGroupConstraintSets and finds the cummulative intersection of all of them
    fn merge_all(sets: Vec<Self>) -> Result<AnswerGroupConstraintSet, UnsatisfiableConstraint> {
        let mut cummulative_answer_group_constraints = None;
        for set in sets {
            cummulative_answer_group_constraints = match cummulative_answer_group_constraints {
                None => Some(set),
                Some(existing_constraints_set) => Some(existing_constraints_set.intersection(set)?),
            };
        };
        cummulative_answer_group_constraints.ok_or(UnsatisfiableConstraint)
    }

    pub fn is_valid_set(answers: Vec<Answer>) -> bool {
        let contraint_sets = answers.iter().map(|m| m.constraints_set.clone()).collect();
        AnswerGroupConstraintSet::merge_all(contraint_sets).is_ok()
    }
}

/// Constraint represents the constraint imposed upon a single wildcard tile.
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
                Constraint::AnyOf(options) => Constraint::merge_decided_with_any_of(*c, options),
            },
            Constraint::AnyOf(options) => match other {
                Constraint::Unconstrainted => Ok(Constraint::AnyOf(options.clone())),
                Constraint::Decided(decided) => {
                    Constraint::merge_decided_with_any_of(decided, options.clone())
                }
                Constraint::AnyOf(options2) => {
                    let h1: HashSet<char> = options.iter().cloned().collect();
                    let h2: HashSet<char> = options2.iter().cloned().collect();
                    let v: Vec<char> = h1.intersection(&h2).cloned().collect();
                    if v.len() == 0 {
                        return Err(UnsatisfiableConstraint);
                    } else if v.len() == 1 {
                        return Ok(Constraint::Decided(v[0]));
                    } else {
                        return Ok(Constraint::AnyOf(v));
                    }
                }
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

    pub fn intersection(
        &self,
        other: &ConstraintsSet,
    ) -> Result<ConstraintsSet, UnsatisfiableConstraint> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const X: &str = "1_1";
    const Y: &str = "2_2";

    fn constraints_from(xi: Option<char>, yi: Option<char>) -> ConstraintsSet {
        let mut c = HashMap::new();
        if let Some(x) = xi {
            c.insert(X.into(), Constraint::Decided(x));
        }
        if let Some(y) = yi {
            c.insert(Y.into(), Constraint::Decided(y));
        }
        return ConstraintsSet(c);
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
                    let actual_set: std::collections::HashSet<char> =
                        actual.iter().cloned().collect();
                    let expected_set: std::collections::HashSet<char> =
                        expected.iter().cloned().collect();
                    assert_eq!(
                        actual_set, expected_set,
                        "Failed test case: {}",
                        test_case.name
                    );
                }
                _ => {
                    assert_eq!(
                        result, test_case.expected,
                        "Failed test case: {}",
                        test_case.name
                    );
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
                    ("w2", Constraint::AnyOf(vec!['x', 'y'])),
                ]),
                set2: constraints_set_from(vec![
                    ("w3", Constraint::Decided('b')),
                    ("w4", Constraint::Unconstrainted),
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
                    ("w2", Constraint::Unconstrainted),
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::Decided('b')), // collision here
                    ("w2", Constraint::Decided('x')), // no collision
                ]),
                expected_collision: true,
            },
            ConstraintsSetTestCase {
                name: "Mixed keys - no collisions",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y'])),
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),     // no collision
                    ("w2", Constraint::AnyOf(vec!['x'])), // no collision (subset)
                ]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Partial key overlap - compatible",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y'])),
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::Unconstrainted), // no collision
                    ("w3", Constraint::Decided('z')),   // different key
                ]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Partial key overlap - incompatible",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y'])),
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::Decided('b')), // collision here
                    ("w3", Constraint::Decided('z')), // different key
                ]),
                expected_collision: true,
            },
            // === Complex multi-wildcard scenarios ===
            ConstraintsSetTestCase {
                name: "Multiple wildcards - all compatible",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y', 'z'])),
                    ("w3", Constraint::Unconstrainted),
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::AnyOf(vec!['a', 'b'])),
                    ("w2", Constraint::Decided('y')),
                    ("w3", Constraint::AnyOf(vec!['p', 'q'])),
                ]),
                expected_collision: false,
            },
            ConstraintsSetTestCase {
                name: "Multiple wildcards - one incompatible",
                set1: constraints_set_from(vec![
                    ("w1", Constraint::Decided('a')),
                    ("w2", Constraint::AnyOf(vec!['x', 'y', 'z'])),
                    ("w3", Constraint::Unconstrainted),
                ]),
                set2: constraints_set_from(vec![
                    ("w1", Constraint::AnyOf(vec!['a', 'b'])),
                    ("w2", Constraint::Decided('w')), // collision here
                    ("w3", Constraint::AnyOf(vec!['p', 'q'])),
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
                result, test_case.expected_collision,
                "Failed test case: {} - expected collision: {}, got: {}",
                test_case.name, test_case.expected_collision, result
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

    struct PathConstraintSetTestCase {
        name: &'static str,
        pcs1: PathConstraintSet,
        pcs2: PathConstraintSet,
        expected: Result<PathConstraintSet, UnsatisfiableConstraint>,
    }

    fn create_path_constraint_set_test_cases() -> Vec<PathConstraintSetTestCase> {
        vec![
            // === Unconstrainted + X cases ===
            PathConstraintSetTestCase {
                name: "Unconstrainted + Unconstrainted",
                pcs1: PathConstraintSet::Unconstrainted,
                pcs2: PathConstraintSet::Unconstrainted,
                expected: Ok(PathConstraintSet::Unconstrainted),
            },
            PathConstraintSetTestCase {
                name: "Unconstrainted + FirstDecided",
                pcs1: PathConstraintSet::Unconstrainted,
                pcs2: PathConstraintSet::FirstDecided('a'),
                expected: Ok(PathConstraintSet::FirstDecided('a')),
            },
            PathConstraintSetTestCase {
                name: "Unconstrainted + SecondDecided",
                pcs1: PathConstraintSet::Unconstrainted,
                pcs2: PathConstraintSet::SecondDecided('b'),
                expected: Ok(PathConstraintSet::SecondDecided('b')),
            },
            PathConstraintSetTestCase {
                name: "Unconstrainted + BothDecided",
                pcs1: PathConstraintSet::Unconstrainted,
                pcs2: PathConstraintSet::BothDecided('a', 'b'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'b')),
            },
            // === FirstDecided + X cases ===
            PathConstraintSetTestCase {
                name: "FirstDecided + Unconstrainted",
                pcs1: PathConstraintSet::FirstDecided('a'),
                pcs2: PathConstraintSet::Unconstrainted,
                expected: Ok(PathConstraintSet::FirstDecided('a')),
            },
            PathConstraintSetTestCase {
                name: "FirstDecided + FirstDecided (same)",
                pcs1: PathConstraintSet::FirstDecided('a'),
                pcs2: PathConstraintSet::FirstDecided('a'),
                expected: Ok(PathConstraintSet::FirstDecided('a')),
            },
            PathConstraintSetTestCase {
                name: "FirstDecided + FirstDecided (different)",
                pcs1: PathConstraintSet::FirstDecided('a'),
                pcs2: PathConstraintSet::FirstDecided('b'),
                expected: Err(UnsatisfiableConstraint),
            },
            PathConstraintSetTestCase {
                name: "FirstDecided + SecondDecided",
                pcs1: PathConstraintSet::FirstDecided('a'),
                pcs2: PathConstraintSet::SecondDecided('b'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'b')),
            },
            PathConstraintSetTestCase {
                name: "FirstDecided + BothDecided (compatible)",
                pcs1: PathConstraintSet::FirstDecided('a'),
                pcs2: PathConstraintSet::BothDecided('a', 'b'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'b')),
            },
            PathConstraintSetTestCase {
                name: "FirstDecided + BothDecided (incompatible)",
                pcs1: PathConstraintSet::FirstDecided('x'),
                pcs2: PathConstraintSet::BothDecided('a', 'b'),
                expected: Err(UnsatisfiableConstraint),
            },
            // === SecondDecided + X cases ===
            PathConstraintSetTestCase {
                name: "SecondDecided + Unconstrainted",
                pcs1: PathConstraintSet::SecondDecided('b'),
                pcs2: PathConstraintSet::Unconstrainted,
                expected: Ok(PathConstraintSet::SecondDecided('b')),
            },
            PathConstraintSetTestCase {
                name: "SecondDecided + FirstDecided",
                pcs1: PathConstraintSet::SecondDecided('b'),
                pcs2: PathConstraintSet::FirstDecided('a'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'b')),
            },
            PathConstraintSetTestCase {
                name: "SecondDecided + SecondDecided (same)",
                pcs1: PathConstraintSet::SecondDecided('b'),
                pcs2: PathConstraintSet::SecondDecided('b'),
                expected: Ok(PathConstraintSet::SecondDecided('b')),
            },
            PathConstraintSetTestCase {
                name: "SecondDecided + SecondDecided (different)",
                pcs1: PathConstraintSet::SecondDecided('b'),
                pcs2: PathConstraintSet::SecondDecided('c'),
                expected: Err(UnsatisfiableConstraint),
            },
            PathConstraintSetTestCase {
                name: "SecondDecided + BothDecided (compatible)",
                pcs1: PathConstraintSet::SecondDecided('b'),
                pcs2: PathConstraintSet::BothDecided('a', 'b'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'b')),
            },
            PathConstraintSetTestCase {
                name: "SecondDecided + BothDecided (incompatible)",
                pcs1: PathConstraintSet::SecondDecided('x'),
                pcs2: PathConstraintSet::BothDecided('a', 'b'),
                expected: Err(UnsatisfiableConstraint),
            },
            // === BothDecided + X cases ===
            PathConstraintSetTestCase {
                name: "BothDecided + Unconstrainted",
                pcs1: PathConstraintSet::BothDecided('a', 'b'),
                pcs2: PathConstraintSet::Unconstrainted,
                expected: Ok(PathConstraintSet::BothDecided('a', 'b')),
            },
            PathConstraintSetTestCase {
                name: "BothDecided + FirstDecided (compatible)",
                pcs1: PathConstraintSet::BothDecided('a', 'b'),
                pcs2: PathConstraintSet::FirstDecided('a'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'b')),
            },
            PathConstraintSetTestCase {
                name: "BothDecided + FirstDecided (incompatible)",
                pcs1: PathConstraintSet::BothDecided('a', 'b'),
                pcs2: PathConstraintSet::FirstDecided('x'),
                expected: Err(UnsatisfiableConstraint),
            },
            PathConstraintSetTestCase {
                name: "BothDecided + SecondDecided (compatible)",
                pcs1: PathConstraintSet::BothDecided('a', 'b'),
                pcs2: PathConstraintSet::SecondDecided('b'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'b')),
            },
            PathConstraintSetTestCase {
                name: "BothDecided + SecondDecided (incompatible)",
                pcs1: PathConstraintSet::BothDecided('a', 'b'),
                pcs2: PathConstraintSet::SecondDecided('x'),
                expected: Err(UnsatisfiableConstraint),
            },
            PathConstraintSetTestCase {
                name: "BothDecided + BothDecided (same)",
                pcs1: PathConstraintSet::BothDecided('a', 'b'),
                pcs2: PathConstraintSet::BothDecided('a', 'b'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'b')),
            },
            PathConstraintSetTestCase {
                name: "BothDecided + BothDecided (first different)",
                pcs1: PathConstraintSet::BothDecided('a', 'b'),
                pcs2: PathConstraintSet::BothDecided('x', 'b'),
                expected: Err(UnsatisfiableConstraint),
            },
            PathConstraintSetTestCase {
                name: "BothDecided + BothDecided (second different)",
                pcs1: PathConstraintSet::BothDecided('a', 'b'),
                pcs2: PathConstraintSet::BothDecided('a', 'x'),
                expected: Err(UnsatisfiableConstraint),
            },
            PathConstraintSetTestCase {
                name: "BothDecided + BothDecided (both different)",
                pcs1: PathConstraintSet::BothDecided('a', 'b'),
                pcs2: PathConstraintSet::BothDecided('x', 'y'),
                expected: Err(UnsatisfiableConstraint),
            },
            // === Edge cases with same letters ===
            PathConstraintSetTestCase {
                name: "FirstDecided + SecondDecided (same letter)",
                pcs1: PathConstraintSet::FirstDecided('a'),
                pcs2: PathConstraintSet::SecondDecided('a'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'a')),
            },
            PathConstraintSetTestCase {
                name: "SecondDecided + FirstDecided (same letter)",
                pcs1: PathConstraintSet::SecondDecided('a'),
                pcs2: PathConstraintSet::FirstDecided('a'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'a')),
            },
            PathConstraintSetTestCase {
                name: "BothDecided same letter both positions",
                pcs1: PathConstraintSet::BothDecided('a', 'a'),
                pcs2: PathConstraintSet::FirstDecided('a'),
                expected: Ok(PathConstraintSet::BothDecided('a', 'a')),
            },
            PathConstraintSetTestCase {
                name: "FirstDecided same as BothDecided same letter",
                pcs1: PathConstraintSet::FirstDecided('z'),
                pcs2: PathConstraintSet::BothDecided('z', 'z'),
                expected: Ok(PathConstraintSet::BothDecided('z', 'z')),
            },
            PathConstraintSetTestCase {
                name: "SecondDecided same as BothDecided same letter",
                pcs1: PathConstraintSet::SecondDecided('z'),
                pcs2: PathConstraintSet::BothDecided('z', 'z'),
                expected: Ok(PathConstraintSet::BothDecided('z', 'z')),
            },
            // === Additional comprehensive coverage ===
            PathConstraintSetTestCase {
                name: "FirstDecided + BothDecided (first matches, different letters)",
                pcs1: PathConstraintSet::FirstDecided('x'),
                pcs2: PathConstraintSet::BothDecided('x', 'y'),
                expected: Ok(PathConstraintSet::BothDecided('x', 'y')),
            },
            PathConstraintSetTestCase {
                name: "SecondDecided + BothDecided (second matches, different letters)",
                pcs1: PathConstraintSet::SecondDecided('y'),
                pcs2: PathConstraintSet::BothDecided('x', 'y'),
                expected: Ok(PathConstraintSet::BothDecided('x', 'y')),
            },
            PathConstraintSetTestCase {
                name: "BothDecided + FirstDecided (first matches, same letters)",
                pcs1: PathConstraintSet::BothDecided('m', 'm'),
                pcs2: PathConstraintSet::FirstDecided('m'),
                expected: Ok(PathConstraintSet::BothDecided('m', 'm')),
            },
            PathConstraintSetTestCase {
                name: "BothDecided + SecondDecided (second matches, same letters)",
                pcs1: PathConstraintSet::BothDecided('n', 'n'),
                pcs2: PathConstraintSet::SecondDecided('n'),
                expected: Ok(PathConstraintSet::BothDecided('n', 'n')),
            },
            // === Symmetry tests ===
            PathConstraintSetTestCase {
                name: "Symmetry: FirstDecided('p') + SecondDecided('q')",
                pcs1: PathConstraintSet::FirstDecided('p'),
                pcs2: PathConstraintSet::SecondDecided('q'),
                expected: Ok(PathConstraintSet::BothDecided('p', 'q')),
            },
            PathConstraintSetTestCase {
                name: "Symmetry: SecondDecided('q') + FirstDecided('p')",
                pcs1: PathConstraintSet::SecondDecided('q'),
                pcs2: PathConstraintSet::FirstDecided('p'),
                expected: Ok(PathConstraintSet::BothDecided('p', 'q')),
            },
            PathConstraintSetTestCase {
                name: "Symmetry: BothDecided('r', 's') + Unconstrainted",
                pcs1: PathConstraintSet::BothDecided('r', 's'),
                pcs2: PathConstraintSet::Unconstrainted,
                expected: Ok(PathConstraintSet::BothDecided('r', 's')),
            },
            PathConstraintSetTestCase {
                name: "Symmetry: Unconstrainted + BothDecided('r', 's')",
                pcs1: PathConstraintSet::Unconstrainted,
                pcs2: PathConstraintSet::BothDecided('r', 's'),
                expected: Ok(PathConstraintSet::BothDecided('r', 's')),
            },
        ]
    }

    #[test]
    fn test_path_constraint_set_merge() {
        let test_cases = create_path_constraint_set_test_cases();

        for test_case in test_cases {
            let result = test_case.pcs1.merge(test_case.pcs2);
            assert_eq!(
                result, test_case.expected,
                "Failed test case: {}",
                test_case.name
            );
        }
    }

    struct AnswerGroupConstraintSetTestCase {
        name: &'static str,
        set1: AnswerGroupConstraintSet,
        set2: AnswerGroupConstraintSet,
        expected_error: bool,
        expected_result_count: Option<usize>,
        expected_result_set: AnswerGroupConstraintSet,
    }

    fn answer_group_from(constraints: Vec<PathConstraintSet>) -> AnswerGroupConstraintSet {
        AnswerGroupConstraintSet {
            path_constraint_sets: constraints,
        }
    }

    fn create_answer_group_constraint_set_test_cases() -> Vec<AnswerGroupConstraintSetTestCase> {
        vec![
            // === Empty sets ===
            AnswerGroupConstraintSetTestCase {
                name: "Empty sets",
                set1: answer_group_from(vec![]),
                set2: answer_group_from(vec![]),
                expected_error: true,
                expected_result_count: None,
                expected_result_set: answer_group_from(vec![]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Empty vs non-empty",
                set1: answer_group_from(vec![]),
                set2: answer_group_from(vec![PathConstraintSet::Unconstrainted]),
                expected_error: true,
                expected_result_count: None,
                expected_result_set: answer_group_from(vec![]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Non-empty vs empty",
                set1: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                set2: answer_group_from(vec![]),
                expected_error: true,
                expected_result_count: None,
                expected_result_set: answer_group_from(vec![]),
            },
            // === Single constraint sets ===
            AnswerGroupConstraintSetTestCase {
                name: "Single Unconstrainted + Single Unconstrainted",
                set1: answer_group_from(vec![PathConstraintSet::Unconstrainted]),
                set2: answer_group_from(vec![PathConstraintSet::Unconstrainted]),
                expected_error: false,
                expected_result_count: Some(1),
                expected_result_set: answer_group_from(vec![PathConstraintSet::Unconstrainted]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Single Unconstrainted + Single FirstDecided",
                set1: answer_group_from(vec![PathConstraintSet::Unconstrainted]),
                set2: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                expected_error: false,
                expected_result_count: Some(1),
                expected_result_set: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Single FirstDecided + Single SecondDecided",
                set1: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                set2: answer_group_from(vec![PathConstraintSet::SecondDecided('b')]),
                expected_error: false,
                expected_result_count: Some(1),
                expected_result_set: answer_group_from(vec![PathConstraintSet::BothDecided(
                    'a', 'b',
                )]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Single FirstDecided + Single FirstDecided (same)",
                set1: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                set2: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                expected_error: false,
                expected_result_count: Some(1),
                expected_result_set: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Single FirstDecided + Single FirstDecided (different)",
                set1: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                set2: answer_group_from(vec![PathConstraintSet::FirstDecided('b')]),
                expected_error: true,
                expected_result_count: None,
                expected_result_set: answer_group_from(vec![]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Single BothDecided + Single FirstDecided (compatible)",
                set1: answer_group_from(vec![PathConstraintSet::BothDecided('a', 'b')]),
                set2: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                expected_error: false,
                expected_result_count: Some(1),
                expected_result_set: answer_group_from(vec![PathConstraintSet::BothDecided(
                    'a', 'b',
                )]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Single BothDecided + Single FirstDecided (incompatible)",
                set1: answer_group_from(vec![PathConstraintSet::BothDecided('a', 'b')]),
                set2: answer_group_from(vec![PathConstraintSet::FirstDecided('c')]),
                expected_error: true,
                expected_result_count: None,
                expected_result_set: answer_group_from(vec![]),
            },
            // === Multiple constraint sets ===
            AnswerGroupConstraintSetTestCase {
                name: "Multiple compatible constraints (2x2 = 4)",
                set1: answer_group_from(vec![
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::FirstDecided('a'),
                ]),
                set2: answer_group_from(vec![
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::SecondDecided('b'),
                ]),
                expected_error: false,
                expected_result_count: Some(4), // All 4 combinations should work
                expected_result_set: answer_group_from(vec![
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::SecondDecided('b'),
                    PathConstraintSet::FirstDecided('a'),
                    PathConstraintSet::BothDecided('a', 'b'),
                ]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Multiple mixed constraints (2x2 = 3 valid)",
                set1: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('a'),
                    PathConstraintSet::SecondDecided('b'),
                ]),
                set2: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('a'), // Works with first
                    PathConstraintSet::FirstDecided('c'), // Doesn't work with first, but works with second
                ]),
                expected_error: false,
                expected_result_count: Some(3), // FirstDecided('a')+FirstDecided('a'), SecondDecided('b')+FirstDecided('a'), SecondDecided('b')+FirstDecided('c')
                expected_result_set: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('a'),
                    PathConstraintSet::BothDecided('a', 'b'),
                    PathConstraintSet::BothDecided('c', 'b'),
                ]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Multiple incompatible constraints",
                set1: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('a'),
                    PathConstraintSet::FirstDecided('b'),
                ]),
                set2: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('c'),
                    PathConstraintSet::FirstDecided('d'),
                ]),
                expected_error: true,
                expected_result_count: None,
                expected_result_set: answer_group_from(vec![]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Single vs multiple (partial compatibility)",
                set1: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                set2: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('a'), // Compatible
                    PathConstraintSet::FirstDecided('b'), // Incompatible
                    PathConstraintSet::Unconstrainted,    // Compatible
                ]),
                expected_error: false,
                expected_result_count: Some(2), // 2 out of 3 combinations work
                expected_result_set: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('a'),
                    PathConstraintSet::FirstDecided('a'),
                ]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Complex BothDecided scenarios",
                set1: answer_group_from(vec![
                    PathConstraintSet::BothDecided('a', 'b'),
                    PathConstraintSet::FirstDecided('x'),
                ]),
                set2: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('a'),  // Works with BothDecided
                    PathConstraintSet::SecondDecided('b'), // Works with BothDecided
                    PathConstraintSet::FirstDecided('x'), // Works with FirstDecided('x'), also combines with FirstDecided('x') for SecondDecided
                ]),
                expected_error: false,
                expected_result_count: Some(4), // 4 valid combinations
                expected_result_set: answer_group_from(vec![
                    PathConstraintSet::BothDecided('a', 'b'),
                    PathConstraintSet::BothDecided('a', 'b'),
                    PathConstraintSet::BothDecided('x', 'b'),
                    PathConstraintSet::FirstDecided('x'),
                ]),
            },
            // === Edge cases ===
            AnswerGroupConstraintSetTestCase {
                name: "Large set with many Unconstrainted",
                set1: answer_group_from(vec![
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::Unconstrainted,
                ]),
                set2: answer_group_from(vec![
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::FirstDecided('a'),
                ]),
                expected_error: false,
                expected_result_count: Some(6), // 3 * 2 = 6 combinations
                expected_result_set: answer_group_from(vec![
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::FirstDecided('a'),
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::FirstDecided('a'),
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::FirstDecided('a'),
                ]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Duplicate constraints in same set",
                set1: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('a'),
                    PathConstraintSet::FirstDecided('a'), // Duplicate
                ]),
                set2: answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                expected_error: false,
                expected_result_count: Some(2), // Both duplicates create valid merges
                expected_result_set: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('a'),
                    PathConstraintSet::FirstDecided('a'),
                ]),
            },
            AnswerGroupConstraintSetTestCase {
                name: "Mix of same letters different positions",
                set1: answer_group_from(vec![
                    PathConstraintSet::FirstDecided('z'),
                    PathConstraintSet::SecondDecided('z'),
                ]),
                set2: answer_group_from(vec![PathConstraintSet::BothDecided('z', 'z')]),
                expected_error: false,
                expected_result_count: Some(2), // Both should work with BothDecided('z', 'z')
                expected_result_set: answer_group_from(vec![
                    PathConstraintSet::BothDecided('z', 'z'),
                    PathConstraintSet::BothDecided('z', 'z'),
                ]),
            },
        ]
    }

    #[test]
    fn test_answer_group_constraint_set_intersection() {
        let test_cases = create_answer_group_constraint_set_test_cases();

        for test_case in test_cases {
            let result = test_case.set1.intersection(test_case.set2.clone());

            if test_case.expected_error {
                // If we expect an error, intersection should return UnsatisfiableConstraint
                assert!(
                    result.is_err(),
                    "Failed test case: {} - expected intersection error but got Ok({:?})",
                    test_case.name,
                    result.unwrap().path_constraint_sets
                );
            } else {
                // If we don't expect an error, intersection should succeed
                assert!(
                    result.is_ok(),
                    "Failed test case: {} - expected intersection success but got Err",
                    test_case.name
                );

                let intersection = result.unwrap();

                // Check the expected result count if specified
                if let Some(expected_count) = test_case.expected_result_count {
                    assert_eq!(
                        intersection.path_constraint_sets.len(),
                        expected_count,
                        "Failed test case: {} - expected {} results but got {}",
                        test_case.name,
                        expected_count,
                        intersection.path_constraint_sets.len()
                    );
                }

                // Check the expected result set - order doesn't matter, so we compare as sets
                use std::collections::HashSet;
                let actual_set: HashSet<PathConstraintSet> =
                    intersection.path_constraint_sets.iter().cloned().collect();
                let expected_set: HashSet<PathConstraintSet> = test_case
                    .expected_result_set
                    .path_constraint_sets
                    .iter()
                    .cloned()
                    .collect();
                assert_eq!(
                    actual_set, expected_set,
                    "Failed test case: {} - expected result set {:?} but got {:?}",
                    test_case.name, expected_set, actual_set
                );

                // Verify all results are valid by checking they don't produce empty intersections
                // when intersected with the original sets
                for constraint in &intersection.path_constraint_sets {
                    // Each result should be derivable from merging constraints from both sets
                    let mut found_origin = false;
                    for set1_constraint in &test_case.set1.path_constraint_sets {
                        for set2_constraint in &test_case.set2.path_constraint_sets {
                            if let Ok(merged) = set1_constraint.merge(*set2_constraint) {
                                if merged == *constraint {
                                    found_origin = true;
                                    break;
                                }
                            }
                        }
                        if found_origin {
                            break;
                        }
                    }
                    assert!(
                        found_origin,
                        "Failed test case: {} - result constraint {:?} not derivable from input sets",
                        test_case.name,
                        constraint
                    );
                }
            }
        }
    }

    // Test cases for AnswerGroupConstraintSet::merge_all
    struct MergeAllTestCase {
        name: &'static str,
        input_sets: Vec<AnswerGroupConstraintSet>,
        expected_error: bool,
        expected_result: Option<AnswerGroupConstraintSet>,
    }

    fn create_merge_all_test_cases() -> Vec<MergeAllTestCase> {
        vec![
            // === Empty input cases ===
            MergeAllTestCase {
                name: "Empty input vector",
                input_sets: vec![],
                expected_error: true,
                expected_result: None,
            },
            // === Single set cases ===
            MergeAllTestCase {
                name: "Single empty set",
                input_sets: vec![answer_group_from(vec![])],
                expected_error: false,
                expected_result: Some(answer_group_from(vec![])),
            },
            MergeAllTestCase {
                name: "Single non-empty set",
                input_sets: vec![answer_group_from(vec![
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::FirstDecided('a'),
                ])],
                expected_error: false,
                expected_result: Some(answer_group_from(vec![
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::FirstDecided('a'),
                ])),
            },
            // === Two set cases ===
            MergeAllTestCase {
                name: "Two compatible sets",
                input_sets: vec![
                    answer_group_from(vec![
                        PathConstraintSet::Unconstrainted,
                        PathConstraintSet::FirstDecided('a'),
                    ]),
                    answer_group_from(vec![
                        PathConstraintSet::SecondDecided('b'),
                        PathConstraintSet::Unconstrainted,
                    ])
                ],
                expected_error: false,
                expected_result: Some(answer_group_from(vec![
                    PathConstraintSet::SecondDecided('b'),
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::BothDecided('a', 'b'),
                    PathConstraintSet::FirstDecided('a'),
                ])),
            },
            MergeAllTestCase {
                name: "Two incompatible sets",
                input_sets: vec![
                    answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                    answer_group_from(vec![PathConstraintSet::FirstDecided('b')]),
                ],
                expected_error: true,
                expected_result: None,
            },
            MergeAllTestCase {
                name: "Two partially compatible sets",
                input_sets: vec![
                    answer_group_from(vec![
                        PathConstraintSet::FirstDecided('a'),
                        PathConstraintSet::Unconstrainted,
                    ]),
                    answer_group_from(vec![
                        PathConstraintSet::FirstDecided('b'),
                        PathConstraintSet::FirstDecided('a'),
                    ])
                ],
                expected_error: false,
                expected_result: Some(answer_group_from(vec![
                    PathConstraintSet::FirstDecided('a'),
                    PathConstraintSet::FirstDecided('b'),
                    PathConstraintSet::FirstDecided('a'),
                ])),
            },
            // === Three set cases ===
            MergeAllTestCase {
                name: "Three compatible sets",
                input_sets: vec![
                    answer_group_from(vec![PathConstraintSet::Unconstrainted]),
                    answer_group_from(vec![PathConstraintSet::FirstDecided('x')]),
                    answer_group_from(vec![PathConstraintSet::SecondDecided('y')]),
                ],
                expected_error: false,
                expected_result: Some(answer_group_from(vec![
                    PathConstraintSet::BothDecided('x', 'y'),
                ])),
            },
            MergeAllTestCase {
                name: "Three sets - first two compatible, third incompatible",
                input_sets: vec![
                    answer_group_from(vec![PathConstraintSet::FirstDecided('a')]),
                    answer_group_from(vec![PathConstraintSet::SecondDecided('b')]),
                    answer_group_from(vec![PathConstraintSet::FirstDecided('c')]),
                ],
                expected_error: true,
                expected_result: None,
            },
            MergeAllTestCase {
                name: "Three sets - complex intersection",
                input_sets: vec![
                    answer_group_from(vec![
                        PathConstraintSet::Unconstrainted,
                        PathConstraintSet::FirstDecided('a'),
                    ]),
                    answer_group_from(vec![
                        PathConstraintSet::SecondDecided('b'),
                        PathConstraintSet::FirstDecided('a'),
                    ]),
                    answer_group_from(vec![
                        PathConstraintSet::FirstDecided('a'),
                        PathConstraintSet::BothDecided('a', 'b'),
                    ])
                ],
                expected_error: false,
                expected_result: Some(answer_group_from(vec![
                    PathConstraintSet::BothDecided('a', 'b'),
                    PathConstraintSet::FirstDecided('a'),
                ])),
            },
            // === Edge cases ===
            MergeAllTestCase {
                name: "Many sets with gradual constraint tightening",
                input_sets: vec![
                    answer_group_from(vec![PathConstraintSet::Unconstrainted]),
                    answer_group_from(vec![
                        PathConstraintSet::FirstDecided('z'),
                        PathConstraintSet::Unconstrainted,
                    ]),
                    answer_group_from(vec![
                        PathConstraintSet::SecondDecided('w'),
                        PathConstraintSet::FirstDecided('z'),
                    ]),
                    answer_group_from(vec![PathConstraintSet::BothDecided('z', 'w')]),
                ],
                expected_error: false,
                expected_result: Some(answer_group_from(vec![
                    PathConstraintSet::BothDecided('z', 'w'),
                ])),
            },
            MergeAllTestCase {
                name: "Sets with duplicate constraints",
                input_sets: vec![
                    answer_group_from(vec![
                        PathConstraintSet::FirstDecided('p'),
                        PathConstraintSet::FirstDecided('p'),
                    ]),
                    answer_group_from(vec![PathConstraintSet::FirstDecided('p')]),
                ],
                expected_error: false,
                expected_result: Some(answer_group_from(vec![
                    PathConstraintSet::FirstDecided('p'),
                    PathConstraintSet::FirstDecided('p'),
                ])),
            },
            MergeAllTestCase {
                name: "Large number of compatible sets",
                input_sets: vec![
                    answer_group_from(vec![PathConstraintSet::Unconstrainted]),
                    answer_group_from(vec![PathConstraintSet::Unconstrainted]),
                    answer_group_from(vec![PathConstraintSet::Unconstrainted]),
                    answer_group_from(vec![PathConstraintSet::FirstDecided('m')]),
                    answer_group_from(vec![PathConstraintSet::SecondDecided('n')]),
                ],
                expected_error: false,
                expected_result: Some(answer_group_from(vec![
                    PathConstraintSet::BothDecided('m', 'n'),
                ])),
            },
        ]
    }

    #[test]
    fn test_answer_group_constraint_set_merge_all() {
        let test_cases = create_merge_all_test_cases();

        for test_case in test_cases {
            let result = AnswerGroupConstraintSet::merge_all(test_case.input_sets);

            if test_case.expected_error {
                assert!(
                    result.is_err(),
                    "Failed test case: {} - expected error but got Ok({:?})",
                    test_case.name,
                    result.as_ref().map(|s| &s.path_constraint_sets)
                );
            } else {
                assert!(
                    result.is_ok(),
                    "Failed test case: {} - expected success but got Err",
                    test_case.name
                );

                if let Some(expected_result) = test_case.expected_result {
                    let actual_result = result.unwrap();
                    
                    // Compare as sets since order doesn't matter
                    use std::collections::HashSet;
                    let actual_set: HashSet<PathConstraintSet> = 
                        actual_result.path_constraint_sets.iter().cloned().collect();
                    let expected_set: HashSet<PathConstraintSet> = 
                        expected_result.path_constraint_sets.iter().cloned().collect();
                    
                    assert_eq!(
                        actual_set, expected_set,
                        "Failed test case: {} - expected {:?} but got {:?}",
                        test_case.name, expected_set, actual_set
                    );
                }
            }
        }
    }

    // Test cases for AnswerGroupConstraintSet::is_valid_set
    struct IsValidSetTestCase {
        name: &'static str,
        answers: Vec<Answer>,
        expected_valid: bool,
    }

    fn create_test_answer(word: &str, constraints: Vec<PathConstraintSet>) -> Answer {
        Answer {
            word: word.to_string(),
            paths: vec![], // Paths don't matter for constraint validation
            constraints_set: AnswerGroupConstraintSet {
                path_constraint_sets: constraints,
            },
        }
    }

    fn create_is_valid_set_test_cases() -> Vec<IsValidSetTestCase> {
        vec![
            // === Empty cases ===
            IsValidSetTestCase {
                name: "Empty answer list",
                answers: vec![],
                expected_valid: false,
            },
            // === Single answer cases ===
            IsValidSetTestCase {
                name: "Single answer with empty constraints",
                answers: vec![create_test_answer("word", vec![])],
                expected_valid: true,
            },
            IsValidSetTestCase {
                name: "Single answer with valid constraints",
                answers: vec![create_test_answer("word", vec![
                    PathConstraintSet::Unconstrainted,
                    PathConstraintSet::FirstDecided('a'),
                ])],
                expected_valid: true,
            },
            // === Two answer cases ===
            IsValidSetTestCase {
                name: "Two compatible answers",
                answers: vec![
                    create_test_answer("word1", vec![
                        PathConstraintSet::FirstDecided('a'),
                        PathConstraintSet::Unconstrainted,
                    ]),
                    create_test_answer("word2", vec![
                        PathConstraintSet::SecondDecided('b'),
                        PathConstraintSet::FirstDecided('a'),
                    ]),
                ],
                expected_valid: true,
            },
            IsValidSetTestCase {
                name: "Two incompatible answers",
                answers: vec![
                    create_test_answer("word1", vec![PathConstraintSet::FirstDecided('a')]),
                    create_test_answer("word2", vec![PathConstraintSet::FirstDecided('b')]),
                ],
                expected_valid: false,
            },
            IsValidSetTestCase {
                name: "Two answers with partial compatibility",
                answers: vec![
                    create_test_answer("word1", vec![
                        PathConstraintSet::FirstDecided('x'),
                        PathConstraintSet::SecondDecided('y'),
                    ]),
                    create_test_answer("word2", vec![
                        PathConstraintSet::FirstDecided('z'),
                        PathConstraintSet::BothDecided('x', 'y'),
                    ]),
                ],
                expected_valid: true, // SecondDecided('y') can work with BothDecided('x', 'y')
            },
            // === Multiple answer cases ===
            IsValidSetTestCase {
                name: "Three compatible answers",
                answers: vec![
                    create_test_answer("word1", vec![PathConstraintSet::Unconstrainted]),
                    create_test_answer("word2", vec![PathConstraintSet::FirstDecided('p')]),
                    create_test_answer("word3", vec![PathConstraintSet::SecondDecided('q')]),
                ],
                expected_valid: true,
            },
            IsValidSetTestCase {
                name: "Three answers - third incompatible",
                answers: vec![
                    create_test_answer("word1", vec![PathConstraintSet::FirstDecided('a')]),
                    create_test_answer("word2", vec![PathConstraintSet::SecondDecided('b')]),
                    create_test_answer("word3", vec![PathConstraintSet::FirstDecided('c')]),
                ],
                expected_valid: false,
            },
            IsValidSetTestCase {
                name: "Complex multi-answer scenario - valid",
                answers: vec![
                    create_test_answer("word1", vec![
                        PathConstraintSet::Unconstrainted,
                        PathConstraintSet::FirstDecided('m'),
                    ]),
                    create_test_answer("word2", vec![
                        PathConstraintSet::SecondDecided('n'),
                        PathConstraintSet::FirstDecided('m'),
                    ]),
                    create_test_answer("word3", vec![
                        PathConstraintSet::BothDecided('m', 'n'),
                        PathConstraintSet::FirstDecided('m'),
                        PathConstraintSet::SecondDecided('n'),
                    ]),
                ],
                expected_valid: true,
            },
            IsValidSetTestCase {
                name: "Complex multi-answer scenario - invalid",
                answers: vec![
                    create_test_answer("word1", vec![PathConstraintSet::FirstDecided('a')]),
                    create_test_answer("word2", vec![PathConstraintSet::FirstDecided('b')]),
                    create_test_answer("word3", vec![PathConstraintSet::FirstDecided('c')]),
                    create_test_answer("word4", vec![PathConstraintSet::FirstDecided('d')]),
                ],
                expected_valid: false,
            },
            // === Edge cases ===
            IsValidSetTestCase {
                name: "Many answers with gradual constraint building",
                answers: vec![
                    create_test_answer("step1", vec![PathConstraintSet::Unconstrainted]),
                    create_test_answer("step2", vec![
                        PathConstraintSet::FirstDecided('x'),
                        PathConstraintSet::Unconstrainted,
                    ]),
                    create_test_answer("step3", vec![
                        PathConstraintSet::SecondDecided('y'),
                        PathConstraintSet::FirstDecided('x'),
                    ]),
                    create_test_answer("step4", vec![PathConstraintSet::BothDecided('x', 'y')]),
                ],
                expected_valid: true,
            },
            IsValidSetTestCase {
                name: "Answers with duplicate constraints",
                answers: vec![
                    create_test_answer("dup1", vec![
                        PathConstraintSet::FirstDecided('z'),
                        PathConstraintSet::FirstDecided('z'),
                    ]),
                    create_test_answer("dup2", vec![PathConstraintSet::FirstDecided('z')]),
                ],
                expected_valid: true,
            },
            IsValidSetTestCase {
                name: "Same letter different positions",
                answers: vec![
                    create_test_answer("same1", vec![PathConstraintSet::FirstDecided('w')]),
                    create_test_answer("same2", vec![PathConstraintSet::SecondDecided('w')]),
                    create_test_answer("same3", vec![PathConstraintSet::BothDecided('w', 'w')]),
                ],
                expected_valid: true,
            },
            IsValidSetTestCase {
                name: "Empty constraint sets in answers",
                answers: vec![
                    create_test_answer("empty1", vec![]),
                    create_test_answer("empty2", vec![]),
                ],
                expected_valid: false, // Empty constraint sets intersect to empty set, which fails
            },
            IsValidSetTestCase {
                name: "Mix of empty and non-empty constraint sets",
                answers: vec![
                    create_test_answer("empty", vec![]),
                    create_test_answer("non_empty", vec![PathConstraintSet::Unconstrainted]),
                ],
                expected_valid: false, // Empty set intersected with non-empty set = no valid merges
            },
            // === Real-world-like scenarios ===
            IsValidSetTestCase {
                name: "Realistic word puzzle scenario - valid",
                answers: vec![
                    create_test_answer("CAT", vec![PathConstraintSet::Unconstrainted]),
                    create_test_answer("DOG", vec![
                        PathConstraintSet::FirstDecided('O'),
                        PathConstraintSet::Unconstrainted,
                    ]),
                    create_test_answer("TOP", vec![
                        PathConstraintSet::SecondDecided('P'),
                        PathConstraintSet::FirstDecided('O'),
                    ]),
                    create_test_answer("POT", vec![PathConstraintSet::BothDecided('O', 'P')]),
                ],
                expected_valid: true,
            },
            IsValidSetTestCase {
                name: "Realistic word puzzle scenario - conflicting wildcards",
                answers: vec![
                    create_test_answer("STAR", vec![PathConstraintSet::FirstDecided('S')]),
                    create_test_answer("CART", vec![PathConstraintSet::FirstDecided('C')]),
                    create_test_answer("PART", vec![PathConstraintSet::FirstDecided('P')]),
                ],
                expected_valid: false, // All require different first wildcard letters
            },
        ]
    }

    #[test]
    fn test_answer_group_constraint_set_is_valid_set() {
        let test_cases = create_is_valid_set_test_cases();

        for test_case in test_cases {
            let result = AnswerGroupConstraintSet::is_valid_set(test_case.answers);
            assert_eq!(
                result, test_case.expected_valid,
                "Failed test case: {} - expected {} but got {}",
                test_case.name, test_case.expected_valid, result
            );
        }
    }
}

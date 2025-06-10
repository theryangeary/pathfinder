
use std::collections::{HashMap, HashSet};

use crate::game::board::constraints;

#[derive(Debug, Copy, Clone, PartialEq)]
struct UnsatisfiableConstraint;

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    Unconstrainted,
    Decided(char),
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

    // pub fn has_collision_with(&self, other: &ConstraintsSet) -> bool {
    //     let mut merged = self.0.clone();
    //     for (k, v) in (&other.0).into_iter() {
    //         let previous = merged.insert(k.to_string(), *v);
    //         if let Some(p) = previous {
    //             if p != *v {
    //                 return true;
    //             }
    //         }
    //     }
    //     return false;
    // }

    // // returns None if the intersection of constraints is invalid
    // pub fn intersection(&self, other: &ConstraintsSet) -> Option<ConstraintsSet> {
    //     let mut merged = self.0.clone();
    //     for (k, v) in other.0.iter() {
    //         let previous = merged.insert(k.to_string(), *v);
    //         if let Some(p) = previous {
    //             if p != *v {
    //                 return None;
    //             }
    //         }
    //     }
    //     return Some(ConstraintsSet(merged));
    // }
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
}

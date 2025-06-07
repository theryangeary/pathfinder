use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Constraints(pub HashMap<String, char>);

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
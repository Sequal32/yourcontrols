use std::{collections::HashMap, hash::Hash};

struct DiffChecker<A, B> {
    values: HashMap<A, B>,
}

impl<A, B> DiffChecker<A, B>
where
    A: Eq + Hash,
    B: PartialEq,
{
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: A, value: B) -> bool {
        let did_change = self.values.get(&id).map_or(false, |x| *x != value);

        self.values.insert(id, value);

        did_change
    }

    pub fn clear(&mut self) {
        self.values.clear()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_diff() {
        let mut diff_checker = DiffChecker::new();
        diff_checker.add(0, 1.0);

        assert!(!diff_checker.add(0, 1.0));
        assert!(diff_checker.add(0, 5.0));
    }

    #[test]
    fn test_clear() {
        let mut diff_checker = DiffChecker::new();
        diff_checker.add(0, 1.0);
        diff_checker.clear();
        assert!(!diff_checker.add(0, 1.0));
    }
}

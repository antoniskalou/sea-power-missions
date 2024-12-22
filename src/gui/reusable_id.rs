use std::collections::BTreeSet;

/// A reusable (recyclable) ID.
///
/// ID's are returned in sequence, however if an ID is removed it will be
/// reused when a new ID is requested.
pub struct ReusableId {
    available_ids: BTreeSet<usize>,
    next_id: usize,
}

impl ReusableId {
    pub fn new(start: usize) -> Self {
        ReusableId {
            available_ids: BTreeSet::new(),
            next_id: start,
        }
    }

    /// Fetch the next item in the sequence.
    ///
    /// If a previous ID has been released, it will be reused.
    pub fn next(&mut self) -> usize {
        if let Some(&id) = self.available_ids.first() {
            self.available_ids.remove(&id);
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        }
    }

    /// Release a currently existing ID from use, allowing it to be reused
    /// by `.next()`.
    pub fn release(&mut self, id: usize) {
        if id < self.next_id {
            self.available_ids.insert(id);
        }
    }
}

impl Default for ReusableId {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next() {
        let mut id = ReusableId::new(1);
        assert_eq!(id.next(), 1);
        assert_eq!(id.next(), 2);
        assert_eq!(id.next(), 3);
    }

    #[test]
    fn test_remove() {
        let mut id = ReusableId::new(1);
        assert_eq!(id.next(), 1);
        assert_eq!(id.next(), 2);
        id.release(1);
        assert_eq!(id.next(), 1);
        id.release(2);
        assert_eq!(id.next(), 2);
        assert_eq!(id.next(), 3);
    }

    #[test]
    fn test_remove_ignores_duplicates() {
        let mut id = ReusableId::new(1);
        id.release(1);
        id.release(1);
        assert_eq!(id.next(), 1);
        assert_eq!(id.next(), 2);
    }

    #[test]
    fn test_remove_keeps_order() {
        let mut id = ReusableId::new(1);
        assert_eq!(id.next(), 1);
        assert_eq!(id.next(), 2);
        assert_eq!(id.next(), 3);
        id.release(2);
        id.release(1);
        assert_eq!(id.next(), 1);
    }
}

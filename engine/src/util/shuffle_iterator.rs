// FIXME this is not exactly the same implementation as the one in Tortoise but
// it should work for proof of concept. the implementations must match before
// it can be considered correct

use std::collections::VecDeque;

use crate::util::rng::Rng;

/// A consuming iterator that shuffles an array of elements.
#[derive(Debug)]
pub struct ShuffledOwned<T> {
    remaining: VecDeque<T>,
}

impl<T> ShuffledOwned<T> {
    pub fn new(items: VecDeque<T>) -> Self {
        ShuffledOwned { remaining: items }
    }
}

impl<T> ShuffledOwned<T> {
    pub fn next(&mut self, rng: &mut impl Rng) -> Option<T> {
        if self.remaining.is_empty() {
            return None;
        }

        let next = rng.next_int(self.remaining.len() as i64) as usize;
        self.remaining.swap(0, next);

        self.remaining.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Utility class to simulate an RNG for testing purposes
    #[derive(Debug)]
    struct R {
        remaining: VecDeque<i64>,
    }

    impl Rng for R {
        fn next_int(&mut self, max: i64) -> i64 {
            let next = self.remaining.pop_front().unwrap();
            assert!(0 <= next && next < max);
            next
        }
    }

    #[test]
    fn test_shuffle_owned_iterator() {
        let mut rng = R { remaining: vec![5, 2, 2, 2, 0, 0].into() };
        let items = vec![0, 1, 2, 3, 4, 5].into();
        let mut iter = ShuffledOwned::new(items);

        assert_eq!(iter.next(&mut rng), Some(5));
        assert_eq!(iter.next(&mut rng), Some(3));
        assert_eq!(iter.next(&mut rng), Some(4));
        assert_eq!(iter.next(&mut rng), Some(0));
        assert_eq!(iter.next(&mut rng), Some(2));
        assert_eq!(iter.next(&mut rng), Some(1));
    }
}

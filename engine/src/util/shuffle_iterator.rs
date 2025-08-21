// TODO this is not exactly the same implementation as the one in Tortoise but
// it should work for proof of concept

use std::{collections::VecDeque, mem};

use crate::util::rng::Rng;

/// A borrowing iterator that shuffles a mutable slice. If [`Iterator::next`] is
/// called `n` times, then the first `n` elements of the slice will contain the
/// elements that were returned, in order, while the rest of the elements retain
/// their relative orders.
pub struct ShuffledMut<'a, T, R> {
    remaining: &'a mut [T],
    rng: R,
}

impl<'a, T, R> ShuffledMut<'a, T, R> {
    pub fn new(slice: &'a mut [T], rng: R) -> Self {
        ShuffledMut { remaining: slice, rng }
    }
}

impl<'a, T, R> Iterator for ShuffledMut<'a, T, R>
where
    R: Rng,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let remaining = mem::take(&mut self.remaining);
        if remaining.is_empty() {
            return None;
        }

        let next = self.rng.next_int(remaining.len() as i64) as usize;
        remaining.swap(0, next);

        let (head, tail) = remaining.split_first_mut().expect("should not be empty");

        self.remaining = tail;
        Some(head)
    }
}

/// A consuming iterator that shuffles an array of elements.
#[derive(Debug)]
pub struct ShuffledOwned<T, R> {
    remaining: VecDeque<T>,
    rng: R,
}

impl<T, R> ShuffledOwned<T, R> {
    pub fn new(items: VecDeque<T>, rng: R) -> Self {
        ShuffledOwned { remaining: items, rng }
    }
}

impl<T, R> Iterator for ShuffledOwned<T, R>
where
    R: Rng,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let next = self.rng.next_int(self.remaining.len() as i64) as usize;
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
        let rng = R { remaining: vec![5, 2, 2, 2, 0, 0].into() };
        let items = vec![0, 1, 2, 3, 4, 5].into();
        let mut iter = ShuffledOwned::new(items, rng);

        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
    }

    #[test]
    fn test_shuffle_borrowed_iterator() {
        let rng = R { remaining: vec![5, 2, 2, 2, 0, 0].into() };
        let mut items = [0, 1, 2, 3, 4, 5];
        let mut iter = ShuffledMut::new(&mut items, rng);

        assert_eq!(iter.next(), Some(&mut 5));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 4));
        assert_eq!(iter.next(), Some(&mut 0));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}

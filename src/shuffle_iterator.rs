// TODO this is not exactly the same implementation as the one in Tortoise but
// it should work for proof of concept

use std::mem;

use crate::rng::NextInt;

pub struct ShuffleIterator<'a, T, R> {
    remaining: &'a mut [T],
    rng: R,
}

impl<'a, T, R> ShuffleIterator<'a, T, R> {
    pub fn new(slice: &'a mut [T], rng: R) -> Self {
        ShuffleIterator {
            remaining: slice,
            rng,
        }
    }
}

impl<'a, T, R> Iterator for ShuffleIterator<'a, T, R>
where
    R: NextInt,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let remaining = mem::take(&mut self.remaining);
        if remaining.is_empty() {
            return None;
        }

        let next = self.rng.next_int(remaining.len() as i32) as usize;
        remaining.swap(0, next);

        let (head, tail) = remaining.split_first_mut().expect("should not be empty");

        self.remaining = tail;
        Some(head)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[test]
    fn test_shuffle_iterator() {
        #[derive(Debug)]
        struct R {
            remaining: VecDeque<i32>,
        }
        impl NextInt for R {
            fn next_int(&mut self, max: i32) -> i32 {
                let next = self.remaining.pop_front().unwrap();
                assert!(0 <= next && next < max);
                next
            }
        }
        let rng = R {
            remaining: vec![5, 2, 2, 2, 0, 0].into(),
        };
        let mut items = [0, 1, 2, 3, 4, 5];
        let mut iter = ShuffleIterator::new(&mut items, rng);

        assert_eq!(iter.next(), Some(&mut 5));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 4));
        assert_eq!(iter.next(), Some(&mut 0));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}

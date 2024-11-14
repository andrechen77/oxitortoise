// TODO this is not exactly the same implementation as the one in Tortoise but
// it should work for proof of concept

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
        if self.remaining.is_empty() {
            return None;
        }

        let next = self.rng.next_int(self.remaining.len() as i32) as usize;
        self.remaining.swap(0, next);

        let (head, tail) = self
            .remaining
            .split_first_mut()
            .expect("should not be empty");

        // Safety: We know that both `head` and `tail` are valid mutable
        // reference that are valid for 'a, because `Self` lives for 'a, meaning
        // that it was initialized with a slice that lived for 'a, which means
        // every item in the slice also lives for &'a. For a split second, both
        // `self.remaining` and the temporary returned by `from_raw_parts` point
        // to overlapping memory, but that should be okay (verified by miri)
        // since they both originate from the same borrow. `item` is created
        // after `self.remaining` is updated, so it always has exclusive access
        // to that item.
        self.remaining = unsafe { std::slice::from_raw_parts_mut(tail.as_mut_ptr(), tail.len()) };
        let item: &'a mut T = unsafe { &mut *(head as *mut T) };

        Some(item)
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

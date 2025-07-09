use std::{collections::VecDeque, marker::PhantomData};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenIndex {
    pub index: u32,
    pub r#gen: u16,
}

#[derive(Debug, Default)]
pub struct GenSlotTracker {
    generations: Vec<u16>,
    free_list: VecDeque<u32>,
}

impl GenSlotTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate(&mut self) -> GenIndex {
        if let Some(index) = self.free_list.pop_front() {
            // use a slot from the free list

            // mark as occupied by setting lsb to 1
            let new_gen = self.generations[index as usize] | 1;
            self.generations[index as usize] = new_gen;
            GenIndex {
                index,
                r#gen: new_gen,
            }
        } else {
            // add a new slot
            let index = self.generations.len() as u32;
            self.generations.push(1); // start with generation 1 (odd, occupied)
            GenIndex { index, r#gen: 1 }
        }
    }

    /// Deallocates a slot, freeing it up for future reuse. Returns None if the
    /// slot is not occupied. Otherwise, returns whether freeing up this slot
    /// caused the generation number to wrap around, as a warning that keys
    /// may begin to collide.
    pub fn deallocate(&mut self, gen_index: GenIndex) -> Result<bool, ()> {
        if !self.has_key(gen_index) {
            return Err(());
        }

        let current_gen = self.generations[gen_index.index as usize];
        // increment generation to mark as unoccupied (even)
        let (new_gen, overflow) = current_gen.overflowing_add(1);
        self.generations[gen_index.index as usize] = new_gen;

        // add to free list
        self.free_list.push_back(gen_index.index);
        Ok(overflow)
    }

    pub fn has_key(&self, gen_index: GenIndex) -> bool {
        if gen_index.index as usize >= self.generations.len() {
            return false;
        }

        let stored_gen = self.generations[gen_index.index as usize];
        stored_gen == gen_index.r#gen
    }

    pub fn len(&self) -> usize {
        self.generations.len() - self.free_list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.generations.len()
    }

    /// Frees all slots without resetting generation numbers.
    pub fn clear(&mut self) {
        // go through each slot and deallocate if occupied (odd generation)
        for i in 0..self.generations.len() {
            let generation = self.generations[i];
            if generation & 1 == 1 {
                // slot is occupied, so deallocate it
                let _ = self.deallocate(GenIndex {
                    index: i as u32,
                    r#gen: generation,
                });
            }
        }
    }

    /// Iterates over all occupied slots.
    pub fn iter(&self) -> impl Iterator<Item = GenIndex> + '_ {
        self.generations
            .iter()
            .enumerate()
            .filter_map(|(i, &r#gen)| {
                if r#gen & 1 == 1 {
                    Some(GenIndex {
                        index: i as u32,
                        r#gen: r#gen,
                    })
                } else {
                    None
                }
            })
    }
}

#[derive(Debug, Clone)]
struct Entry<T> {
    generation: u16,
    value: Option<T>,
}

impl<T> Entry<T> {
    fn new(generation: u16, value: T) -> Self {
        Self {
            generation,
            value: Some(value),
        }
    }

    fn empty(generation: u16) -> Self {
        Self {
            generation,
            value: None,
        }
    }
}

#[derive(Debug)]
pub struct GenSlotMap<K, V> {
    entries: Vec<Entry<V>>,
    _phantom: PhantomData<fn(K)>,
}

impl<K, V> GenSlotMap<K, V>
where
    K: Into<GenIndex>,
    GenIndex: Into<K>,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a reference to the value at the given key.
    /// Returns None if the generation doesn't match or the slot is empty.
    pub fn get(&self, key: K) -> Option<&V> {
        let key = key.into();

        if key.index as usize >= self.entries.len() {
            return None;
        }

        let entry = &self.entries[key.index as usize];
        if entry.generation != key.r#gen {
            return None;
        }

        entry.value.as_ref()
    }

    /// Gets a mutable reference to the value at the given key.
    /// Returns None if the generation doesn't match or the slot is empty.
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let key = key.into();

        if key.index as usize >= self.entries.len() {
            return None;
        }

        let entry = &mut self.entries[key.index as usize];
        if entry.generation != key.r#gen {
            return None;
        }

        entry.value.as_mut()
    }

    /// Mutates the value at the given key if it exists, otherwise inserts a new
    /// value. If there is already a value at the same index with a different
    /// generation, it is removed and returned.
    #[must_use]
    pub fn mutate_or_insert(
        &mut self,
        key: K,
        mutate: impl FnOnce(&mut V),
        insert: impl FnOnce() -> V,
    ) -> Option<(K, V)> {
        let key = key.into();

        // ensure the entries vector is large enough
        while self.entries.len() <= key.index as usize {
            self.entries.push(Entry::empty(0));
        }

        let entry = &mut self.entries[key.index as usize];
        if entry.generation != key.r#gen {
            let new_value = insert();
            let old_key = GenIndex {
                index: key.index,
                r#gen: entry.generation,
            };
            let old_value = entry.value.replace(new_value);
            entry.generation = key.r#gen;
            old_value.map(|v| (old_key.into(), v))
        } else {
            let current_value = entry
                .value
                .as_mut()
                .expect("since the generation matches, the value must exist");
            mutate(current_value);
            None
        }
    }

    /// Inserts a value at the given key, replacing any existing value.
    /// Returns the old value as Option<(K, V)> if there was one.
    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)> {
        let key = key.into();
        // ensure the entries vector is large enough
        while self.entries.len() <= key.index as usize {
            self.entries.push(Entry::empty(0));
        }

        let entry = &mut self.entries[key.index as usize];
        let old_value = entry.value.take();
        let old_generation = entry.generation;
        entry.generation = key.r#gen;
        entry.value = Some(value);

        old_value.map(|v| {
            (
                GenIndex {
                    index: key.index,
                    r#gen: old_generation,
                }
                .into(),
                v,
            )
        })
    }

    /// Removes the value at the given key.
    /// Returns the removed value if it existed.
    pub fn remove(&mut self, key: K) -> Option<V> {
        let key = key.into();
        if key.index as usize >= self.entries.len() {
            return None;
        }

        let entry = &mut self.entries[key.index as usize];
        if entry.generation != key.r#gen {
            return None;
        }

        entry.value.take()
    }

    pub fn len(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.value.is_some())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.entries.len()
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn drain(&mut self) -> impl Iterator<Item = (K, V)> + '_ {
        self.entries.drain(..).enumerate().filter_map(|(i, entry)| {
            entry.value.map(|v| {
                (
                    GenIndex {
                        index: i as u32,
                        r#gen: entry.generation,
                    }
                    .into(),
                    v,
                )
            })
        })
    }
}

impl<K, V> Default for GenSlotMap<K, V>
where
    K: Into<GenIndex>,
    GenIndex: Into<K>,
{
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate() {
        let mut tracker = GenSlotTracker::new();

        let gen1 = tracker.allocate();
        assert_eq!(gen1.index, 0);
        assert_eq!(gen1.r#gen, 1);

        let gen2 = tracker.allocate();
        assert_eq!(gen2.index, 1);
        assert_eq!(gen2.r#gen, 1);
    }

    #[test]
    fn test_deallocate() {
        let mut tracker = GenSlotTracker::new();

        let gen1 = tracker.allocate();
        assert!(tracker.has_key(gen1));

        assert_eq!(tracker.deallocate(gen1), Ok(false));
        assert!(!tracker.has_key(gen1));

        // should be able to reallocate the same slot
        let gen2 = tracker.allocate();
        assert_eq!(gen2.index, 0);
        assert_eq!(gen2.r#gen, 3);
    }

    #[test]
    fn test_has_key() {
        let mut tracker = GenSlotTracker::new();

        let gen1 = tracker.allocate();
        assert!(tracker.has_key(gen1));

        // wrong generation
        let wrong_gen = GenIndex {
            index: gen1.index,
            r#gen: gen1.r#gen + 1,
        };
        assert!(!tracker.has_key(wrong_gen));

        // wrong index
        let wrong_index = GenIndex {
            index: gen1.index + 1,
            r#gen: gen1.r#gen,
        };
        assert!(!tracker.has_key(wrong_index));
    }

    #[test]
    fn test_deallocate_invalid() {
        let mut tracker = GenSlotTracker::new();

        let gen1 = tracker.allocate();
        let _ = tracker.deallocate(gen1);

        // try to deallocate again
        assert_eq!(tracker.deallocate(gen1), Err(()));
    }

    #[test]
    fn test_reuse_slots() {
        let mut tracker = GenSlotTracker::new();

        // allocate and deallocate multiple times
        for _ in 0..5 {
            let gen_index = tracker.allocate();
            assert_eq!(tracker.deallocate(gen_index), Ok(false));
        }

        // should reuse slots from free list
        let gen_index = tracker.allocate();
        assert_eq!(gen_index.index, 0); // should reuse the first slot
    }

    #[test]
    fn test_generation_overflow() {
        let mut tracker = GenSlotTracker::new();

        // deallocate and reallocate the same slot 2^15 - 1 times
        for i in 0..(1 << 15) - 1 {
            let key = tracker.allocate();
            assert_eq!(key.index, 0);
            assert_eq!(key.r#gen, (i << 1) | 1);

            let result = tracker.deallocate(key);
            assert_eq!(result, Ok(false));
        }

        // the next allocation-deallocation will cause a generation overflow
        let about_to_overflow = tracker.allocate();
        assert_eq!(about_to_overflow.index, 0);
        assert_eq!(about_to_overflow.r#gen, u16::MAX);

        let result = tracker.deallocate(about_to_overflow);
        assert_eq!(result, Ok(true));

        // allocating again will use the same slot with generation 1
        let overflowed = tracker.allocate();
        assert_eq!(overflowed.index, 0);
        assert_eq!(overflowed.r#gen, 1);
    }

    #[test]
    fn test_clear() {
        let mut tracker = GenSlotTracker::new();

        let key1 = tracker.allocate();
        let key2 = tracker.allocate();
        let key3 = tracker.allocate();

        assert!(tracker.has_key(key1));
        assert!(tracker.has_key(key2));
        assert!(tracker.has_key(key3));
        assert_eq!(tracker.len(), 3);

        tracker.clear();

        assert!(!tracker.has_key(key1));
        assert!(!tracker.has_key(key2));
        assert!(!tracker.has_key(key3));
        assert_eq!(tracker.len(), 0);

        let new_key = tracker.allocate();
        assert!(tracker.has_key(new_key));
        assert_eq!(new_key.index, 0);
        assert_eq!(new_key.r#gen, 3);
    }

    // gen slot map tests

    // empty key type for testing purposes only
    #[derive(Clone, Copy)]
    struct NothingKey;
    impl From<GenIndex> for NothingKey {
        fn from(_: GenIndex) -> Self {
            NothingKey
        }
    }
    impl From<NothingKey> for GenIndex {
        fn from(_: NothingKey) -> Self {
            GenIndex { index: 0, r#gen: 1 }
        }
    }

    #[test]
    fn test_gen_slot_map_new() {
        let map: GenSlotMap<NothingKey, i32> = GenSlotMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 0);
    }

    #[test]
    fn test_gen_slot_map_insert_new() {
        let mut map = GenSlotMap::new();

        let gen_index = GenIndex { index: 0, r#gen: 1 };
        let result = map.insert(gen_index, 42);
        assert_eq!(result, None); // no previous value
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());

        assert_eq!(map.get(gen_index), Some(&42));
    }

    #[test]
    fn test_gen_slot_map_get() {
        let mut map = GenSlotMap::new();

        let gen_index = GenIndex { index: 0, r#gen: 1 };
        let _ = map.insert(gen_index, "hello");
        assert_eq!(map.get(gen_index), Some(&"hello"));

        // test with wrong generation
        let wrong_gen = GenIndex {
            index: gen_index.index,
            r#gen: gen_index.r#gen + 2,
        };
        assert_eq!(map.get(wrong_gen), None);

        // test with wrong index
        let wrong_index = GenIndex {
            index: gen_index.index + 1,
            r#gen: gen_index.r#gen,
        };
        assert_eq!(map.get(wrong_index), None);
    }

    #[test]
    fn test_gen_slot_map_get_mut() {
        let mut map = GenSlotMap::new();

        let gen_index = GenIndex { index: 0, r#gen: 1 };
        let _ = map.insert(gen_index, String::from("hello"));

        // test mutable access
        if let Some(value) = map.get_mut(gen_index) {
            value.push_str(" world");
        }

        assert_eq!(map.get(gen_index), Some(&String::from("hello world")));

        // test with wrong generation
        let wrong_gen = GenIndex {
            index: gen_index.index,
            r#gen: gen_index.r#gen + 2,
        };
        assert_eq!(map.get_mut(wrong_gen), None);
    }

    #[test]
    fn test_gen_slot_map_insert() {
        let mut map = GenSlotMap::new();

        let gen_index = GenIndex { index: 0, r#gen: 1 };
        let _ = map.insert(gen_index, 10);

        // insert a new value, should return the old one with old generation
        let old_value = map.insert(gen_index, 20);
        assert_eq!(old_value, Some((GenIndex { index: 0, r#gen: 1 }, 10)));
        assert_eq!(map.get(gen_index), Some(&20));

        // insert again, should return the current value with current generation
        let old_value = map.insert(gen_index, 30);
        assert_eq!(old_value, Some((GenIndex { index: 0, r#gen: 1 }, 20)));
        assert_eq!(map.get(gen_index), Some(&30));
    }

    #[test]
    fn test_gen_slot_map_insert_invalid() {
        let mut map = GenSlotMap::new();

        // try to insert with an index that doesn't exist yet
        let invalid_index = GenIndex { index: 5, r#gen: 1 };
        let result = map.insert(invalid_index, 42);
        assert_eq!(result, None);
        assert_eq!(map.get(invalid_index), Some(&42)); // should work now since insert creates the slot
    }

    #[test]
    fn test_gen_slot_map_remove() {
        let mut map = GenSlotMap::new();

        let gen_index = GenIndex { index: 0, r#gen: 1 };
        let _ = map.insert(gen_index, 42);
        assert_eq!(map.len(), 1);

        // remove the value
        let removed = map.remove(gen_index);
        assert_eq!(removed, Some(42));
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        // try to get the removed value
        assert_eq!(map.get(gen_index), None);
    }

    #[test]
    fn test_gen_slot_map_remove_invalid() {
        let mut map: GenSlotMap<NothingKey, i32> = GenSlotMap::new();

        // try to remove with an index that doesn't exist
        let nonexistent_key = GenIndex { index: 5, r#gen: 1 }.into();
        let result = map.remove(nonexistent_key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_gen_slot_map_reuse_slots() {
        let mut map = GenSlotMap::new();

        // insert and remove multiple times
        for i in 0..5 {
            let gen_index = GenIndex {
                index: i as u32,
                r#gen: 1,
            };
            let _ = map.insert(gen_index, i);
            assert_eq!(map.get(gen_index), Some(&i));

            let removed = map.remove(gen_index);
            assert_eq!(removed, Some(i));
        }

        // should be able to insert at any index
        let new_gen_index = GenIndex {
            index: 10,
            r#gen: 1,
        };
        let _ = map.insert(new_gen_index, 100);
        assert_eq!(map.get(new_gen_index), Some(&100));
    }

    #[test]
    fn test_gen_slot_map_clear() {
        let mut map = GenSlotMap::new();

        let gen1 = GenIndex { index: 0, r#gen: 1 };
        let gen2 = GenIndex { index: 1, r#gen: 1 };
        let gen3 = GenIndex { index: 2, r#gen: 1 };

        let _ = map.insert(gen1, 1);
        let _ = map.insert(gen2, 2);
        let _ = map.insert(gen3, 3);

        assert_eq!(map.len(), 3);
        assert!(map.get(gen1).is_some());
        assert!(map.get(gen2).is_some());
        assert!(map.get(gen3).is_some());

        map.clear();

        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert!(map.get(gen1).is_none());
        assert!(map.get(gen2).is_none());
        assert!(map.get(gen3).is_none());

        // should be able to insert new values after clear
        let new_gen = GenIndex { index: 0, r#gen: 1 };
        let _ = map.insert(new_gen, 42);
        assert_eq!(map.get(new_gen), Some(&42));
    }

    #[test]
    fn test_gen_slot_map_multiple_types() {
        // test with different types
        let mut string_map = GenSlotMap::new();
        let string_index = GenIndex { index: 0, r#gen: 1 };
        let _ = string_map.insert(string_index, String::from("test"));
        assert_eq!(string_map.get(string_index), Some(&String::from("test")));

        let mut vec_map: GenSlotMap<NothingKey, Vec<i32>> = GenSlotMap::new();
        let vec_index = GenIndex { index: 0, r#gen: 1 }.into();
        let _ = vec_map.insert(vec_index, vec![1, 2, 3]);
        assert_eq!(vec_map.get(vec_index), Some(&vec![1, 2, 3]));

        let mut option_map: GenSlotMap<NothingKey, Option<i32>> = GenSlotMap::new();
        let option_index = GenIndex { index: 0, r#gen: 1 }.into();
        let _ = option_map.insert(option_index, Some(42));
        assert_eq!(option_map.get(option_index), Some(&Some(42)));
    }

    #[test]
    fn test_gen_slot_map_generation_mismatch() {
        let mut map = GenSlotMap::new();

        let gen_index = GenIndex { index: 0, r#gen: 1 };
        let _ = map.insert(gen_index, 42);

        // create a gen index with the same index but different generation
        let wrong_gen = GenIndex {
            index: gen_index.index,
            r#gen: gen_index.r#gen + 2, // even number (unoccupied)
        };

        // should not be able to access with wrong generation
        assert_eq!(map.get(wrong_gen), None);
        assert_eq!(map.get_mut(wrong_gen), None);

        // inserting with wrong generation replaces the value
        assert_eq!(
            map.insert(wrong_gen, 100),
            Some((GenIndex { index: 0, r#gen: 1 }, 42))
        );
        assert_eq!(map.remove(wrong_gen), Some(100));
    }

    #[test]
    fn test_gen_slot_map_generation_replacement() {
        let mut map = GenSlotMap::new();

        // insert at index 0 with generation 1
        let gen1 = GenIndex { index: 0, r#gen: 1 };
        let result = map.insert(gen1, 42);
        assert_eq!(result, None); // no previous value
        assert_eq!(map.get(gen1), Some(&42));

        // insert at the same index with generation 3
        let gen3 = GenIndex { index: 0, r#gen: 3 };
        let result = map.insert(gen3, 100);
        assert_eq!(result, Some((GenIndex { index: 0, r#gen: 1 }, 42))); // returns old gen and value
        assert_eq!(map.get(gen3), Some(&100));

        // the old generation should no longer be accessible
        assert_eq!(map.get(gen1), None);
        assert_eq!(map.get_mut(gen1), None);
        assert_eq!(map.remove(gen1), None);

        // but the new generation should work
        assert_eq!(map.get(gen3), Some(&100));
        assert_eq!(map.get_mut(gen3), Some(&mut 100));
    }

    #[test]
    fn test_gen_slot_map_mutate_or_insert() {
        let mut map = GenSlotMap::new();
        let gen_index = GenIndex { index: 0, r#gen: 1 };

        // Insert a new value using mutate_or_insert (should call insert closure)
        let result =
            map.mutate_or_insert(gen_index, |_v| panic!("should not mutate on insert"), || 10);
        assert_eq!(result, None);
        assert_eq!(map.get(gen_index), Some(&10));

        // Mutate the existing value (should call mutate closure)
        let result = map.mutate_or_insert(
            gen_index,
            |v| *v += 5,
            || panic!("should not insert on mutate"),
        );
        assert_eq!(result, None);
        assert_eq!(map.get(gen_index), Some(&15));

        // Insert at the same index but with a different generation (should replace and return old value)
        let new_gen_index = GenIndex { index: 0, r#gen: 3 };
        let result = map.mutate_or_insert(
            new_gen_index,
            |_v| panic!("should not mutate on insert"),
            || 100,
        );
        assert_eq!(result, Some((gen_index, 15)));
        assert_eq!(map.get(new_gen_index), Some(&100));
        assert_eq!(map.get(gen_index), None);
    }
}

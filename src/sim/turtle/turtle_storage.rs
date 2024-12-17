use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    ptr::NonNull,
};

use slotmap::SlotMap;

use crate::sim::turtle::Turtle;

use super::{TurtleData, TurtleId, TurtleWho};

#[derive(Debug, Default)]
pub(super) struct TurtleStorage {
    inner: RefCell<Inner>,
}

#[derive(Debug, Default)]
struct Inner {
    /// The who number to be given to the next turtle.
    next_who: TurtleWho,
    /// Maps turtle who numbers into `TurtleId`s. Removed turtles do not show
    /// up here, even if their memory still exists and is pointed to by
    /// owning_ptrs.
    who_map: HashMap<TurtleWho, TurtleId>,
    /// Stores pointers to the turtle data. References may exist to the turtle
    /// data regardless of whether this struct is borrowed, so care must be
    /// taken not to invalidate these references, such as by dropping the data.
    /// Therefore, removed turtles will still show up here until they are
    /// deallocated manually.
    owning_ptrs: SlotMap<TurtleId, NonNull<Turtle>>,
    /// A queue of turtle ids for turtles that have been removed but not yet
    /// deallocated.
    removed_queue: VecDeque<TurtleId>,
}

impl TurtleStorage {
    pub(super) fn translate_who(&self, who: TurtleWho) -> Option<TurtleId> {
        let turtle_storage = self.inner.borrow();
        turtle_storage.who_map.get(&who).copied()
    }

    pub(super) fn get_turtle(&self, turtle_id: TurtleId) -> Option<&Turtle> {
        let turtle_storage = self.inner.borrow();
        let &turtle_ptr = turtle_storage.owning_ptrs.get(turtle_id)?;
        // SAFETY: the turtle pointer is guaranteed to be valid for as long as
        // the refcell containing this struct is share-borrowed, because no
        // operation on a &self invalidates it
        let turtle = unsafe { turtle_ptr.as_ref() };
        Some(turtle)
    }

    // if lazy iteration is necessary, will have to create a custom
    // iterator type to hold on to the inner borrow. however, needing to do this
    // in the first place might be a red flag something bad, since it probably
    // means doing something else with the world in between iterations, which
    // might want to mutably borrow the inner data.
    pub(super) fn turtle_ids(&self) -> Vec<TurtleId> {
        let t = self.inner.borrow();
        t.owning_ptrs.keys().collect()
    }

    pub(super) fn add_turtle(&self, data: TurtleData) -> TurtleId {
        let mut turtle_storage = self.inner.borrow_mut();

        let who = turtle_storage.take_next_who();
        let turtle = Box::new(Turtle {
            who,
            data: RefCell::new(data),
        });
        let turtle = NonNull::new(Box::into_raw(turtle)).expect("should not be null");
        let turtle_id = turtle_storage.owning_ptrs.insert(turtle);
        turtle_storage.who_map.insert(who, turtle_id);
        turtle_id
    }

    /// Removes a turtle from the storage, also removing its who number.
    /// However, since this is done through a shared reference, the turtle data
    /// cannot actually be invalidated, since other references to it might
    /// exist. To actually deallocate the data, [`Self::deallocate_removed`]
    /// must be called.
    ///
    /// # Panics
    ///
    /// Panics if the given `TurtleId` does not refer to a turtle in this
    /// struct.
    pub(super) fn remove_turtle(&self, turtle_id: TurtleId) {
        let mut t = self.inner.borrow_mut();

        t.removed_queue.push_back(turtle_id);

        let &owning_ptr = t
            .owning_ptrs
            .get(turtle_id)
            .expect("by precondition turtle should exist");
        // SAFETY: the turtle pointer is guaranteed to be valid for as long as
        // the refcell containing this struct is share-borrowed, because no
        // operation on a &self invalidates it
        let turtle = unsafe { owning_ptr.as_ref() };
        t.who_map.remove(&turtle.who());
    }

    /// Deallocates the turtle data of all turtles that have been removed.
    pub(super) fn deallocate_removed(&mut self) {
        let mut t = self.inner.borrow_mut();
        let t = &mut *t; // convert to &mut because we need to access different parts concurrently
        for turtle_id in t.removed_queue.drain(0..) {
            let Some(owning_ptr) = t.owning_ptrs.remove(turtle_id) else {
                continue;
            };

            // SAFETY: since `Self` is exclusively borrowed, it is statically
            // guaranteed that there are no references to this turtle data;
            // `Self::get_turtle` only returns a reference that lives as long
            // as the shared borrow.
            drop(unsafe { Box::from_raw(owning_ptr.as_ptr()) });
        }
    }

    pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Turtle> {
        let turtle_storage = self.inner.get_mut();
        turtle_storage.owning_ptrs.values_mut().map(|ptr| {
            // SAFETY: since `Self` is exclusively borrowed, it is statically
            // guaranteed that there are no references to this turtle data;
            // `Self::get_turtle` only returns a reference that lives as long
            // as the shared borrow.
            unsafe { &mut *ptr.as_ptr() }
        })
    }

    /// Removes all turtles and resets the who numbering.
    pub(super) fn clear(&self) {
        let mut t = self.inner.borrow_mut();
        let t = &mut *t; // convert to &mut because we need to access different parts concurrently

        for turtle_id in t.owning_ptrs.keys() {
            t.removed_queue.push_back(turtle_id);
        }

        t.who_map.clear();
    }
}

impl Inner {
    /// Returns the next who number to be given to a turtle, and increments it
    /// again.
    fn take_next_who(&mut self) -> TurtleWho {
        let who = self.next_who;
        self.next_who.0 += 1;
        who
    }
}

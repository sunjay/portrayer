use std::marker::PhantomData;

use derivative::Derivative;

/// A handle to an item in the arena.
///
/// This handle can only be used with an arena that stores the same type. That means that
/// if multiple arenas store the same type you may lose the guarantee that this handle is
/// valid for that arena.
#[derive(Derivative)]
#[derivative(Debug(bound=""), Clone(bound=""), Copy(bound=""), PartialEq(bound=""), Eq(bound=""), Hash(bound=""))]
pub struct Handle<T> {
    index: usize,
    _d: PhantomData<T>,
    // NOTE: If in the future you want to allow multiple arenas that store the same type,
    // you can add an "arena_id" field here that allows you to disambiguate (at runtime)
    // which arena a given handle came from. The arena_id can be set by a global AtomicUsize.
}

// Need to implement these traits manually because Rust

/// A simple insert-only arena that provides typed handles to its contents
///
/// This is basically the same as keeping a vector of items and storing
/// the indexes instead of the items themselves. You get added type safety
/// and a guarantee that handles are valid as long as the arena is valid.
#[derive(Debug)]
pub struct Arena<T> {
    data: Vec<T>,
}

// Need this impl because derive(Default) requires T: Default which is not necessary here.
impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

impl<T> Arena<T> {
    pub fn insert(&mut self, item: T) -> Handle<T> {
        self.data.push(item);
        Handle {
            index: self.data.len() - 1,
            _d: PhantomData::default(),
        }
    }

    pub fn get(&self, handle: Handle<T>) -> &T {
        &self.data[handle.index]
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> &mut T {
        &mut self.data[handle.index]
    }
}

use std::any::{Any, TypeId};
use std::collections::BTreeMap;

#[derive(Default)]
/// A map of Type => Instance.
/// We use a BTreeMap here to allow for determinism. The number of insertion/removal isn't expected
/// to be high.
pub struct TypeMap(BTreeMap<TypeId, Box<dyn Any>>);

impl TypeMap {
    /// Create an empty `TypeMap`.
    #[inline]
    pub fn new() -> TypeMap {
        TypeMap(BTreeMap::default())
    }

    /// Insert a type into this `TypeMap`.
    ///
    /// If a extension of this type already existed, it will
    /// be returned.
    pub fn insert<T: 'static>(&mut self, val: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(val));
    }

    /// Check if container contains entry
    pub fn contains<T: 'static>(&self) -> bool {
        self.0.get(&TypeId::of::<T>()).is_some()
    }

    /// Get a reference to a type previously inserted on this `TypeMap`.
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
    }

    /// Get a mutable reference to a type previously inserted on this `TypeMap`.
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.0
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| (&mut **boxed as &mut (dyn Any + 'static)).downcast_mut())
    }

    /// Remove a type from this `TypeMap`.
    ///
    /// If a extension of this type existed, it will be returned.
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.0.remove(&TypeId::of::<T>()).and_then(|boxed| {
            (boxed as Box<dyn Any + 'static>)
                .downcast()
                .ok()
                .map(|boxed| *boxed)
        })
    }

    /// Clear the `TypeMap` of all inserted extensions.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{OnceLock, PoisonError, RwLock};

/// A trait for internable values.
pub trait Internable: Hash + Eq {
    /// Creates a static reference to `self`, possibly leaking memory.
    fn leak(&self) -> &'static Self;

    /// Returns `true` if the two references point to the same value.
    fn ref_eq(&self, other: &Self) -> bool;

    /// Feeds the reference to the hasher.
    fn ref_hash<H: Hasher>(&self, state: &mut H);
}

impl Internable for str {
    fn leak(&self) -> &'static Self {
        let str = self.to_owned().into_boxed_str();
        Box::leak(str)
    }

    fn ref_eq(&self, other: &Self) -> bool {
        self.as_ptr() == other.as_ptr() && self.len() == other.len()
    }

    fn ref_hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        self.as_ptr().hash(state);
    }
}

/// An interned value. Will stay valid until the end of the program and will not drop.
pub struct Interned<T: ?Sized + 'static>(pub &'static T);

/// A thread-safe interner which can be used to create [`Interned<T>`] from a `&T`.
pub struct Interner<T: ?Sized + 'static>(OnceLock<RwLock<HashSet<&'static T>>>);

impl<T: ?Sized> Default for Interner<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Interner<T> {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }
}

impl<T: Internable + ?Sized> Interner<T> {
    pub fn intern(&self, value: &T) -> Interned<T> {
        let lock = self.0.get_or_init(Default::default);
        {
            let set = lock.read().unwrap_or_else(PoisonError::into_inner);
            if let Some(value) = set.get(value) {
                return Interned(*value);
            }
        }
        {
            let mut set = lock.write().unwrap_or_else(PoisonError::into_inner);
            if let Some(value) = set.get(value) {
                Interned(*value)
            } else {
                let leaked = value.leak();
                set.insert(leaked);
                Interned(leaked)
            }
        }
    }
}

impl<T: ?Sized> Deref for Interned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T: ?Sized> AsRef<T> for Interned<T> {
    fn as_ref(&self) -> &T {
        self.0
    }
}

impl<T: ?Sized> Borrow<T> for Interned<T> {
    fn borrow(&self) -> &T {
        self.0
    }
}

impl<T: ?Sized> Clone for Interned<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Interned<T> {}

impl<T: ?Sized + Internable> PartialEq for Interned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.ref_eq(other.0)
    }
}

impl<T: ?Sized + Internable> Eq for Interned<T> {}

impl<T: ?Sized + Internable> Hash for Interned<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.ref_hash(state);
    }
}

impl<T: ?Sized + Debug> Debug for Interned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<&Interned<T>> for Interned<T> {
    fn from(value: &Interned<T>) -> Self {
        *value
    }
}

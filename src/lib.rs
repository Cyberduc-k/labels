pub mod intern;

use std::any::Any;
use std::hash::{Hash, Hasher};

#[doc(hidden)]
pub use paste as __paste;

/// An object safe version of [`Eq`].
pub trait DynEq: Any {
    /// Cast the type to `dyn Any`.
    fn as_any(&self) -> &dyn Any;

    /// This method tests for `self` and `other` values to be equal.
    fn dyn_eq(&self, other: &dyn DynEq) -> bool;
}

/// An object safe version of [`Hash`].
pub trait DynHash: DynEq {
    /// Cast the type to `dyn DynEq`.
    fn as_dyn_eq(&self) -> &dyn DynEq;

    /// Feeds this value into the given [`Hasher`].
    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<T: Any + Eq> DynEq for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn DynEq) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            return self == other;
        }
        false
    }
}

impl<T: DynEq + Hash> DynHash for T {
    fn as_dyn_eq(&self) -> &dyn DynEq {
        self
    }

    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        self.type_id().hash(&mut state);
        T::hash(self, &mut state);
    }
}

#[macro_export]
macro_rules! define_label {
    ($(#[$attr:meta])* $vis:vis $label_name:ident $(;)? $(
        extra_methods:{ $($(#[$method_attr:meta])* fn $method:ident(&$self:ident) -> $ret:ty $($body:block)?)* }
        extra_methods_impl:{ $(fn $impl_method:ident(&$impl_self:ident) -> $impl_ret:ty $impl_body:block)* })?
    ) => {
        $(#[$attr])*
        $vis trait $label_name: ::std::fmt::Debug + Send + Sync + 'static {
            $($(
                $(#[$method_attr])*
                fn $method(&$self) -> $ret $($body)?
            )*)?

            /// Clones this `
            #[doc = stringify!($label_name)]
            /// `.
            fn dyn_clone(&self) -> ::std::boxed::Box<dyn $label_name>;

            /// Cast the type to `dyn DynEq`.
            fn as_dyn_eq(&self) -> &dyn $crate::DynEq;

            /// Cast the type to `dyn DynHash`.
            fn as_dyn_hash(&self) -> &dyn $crate::DynHash;

            /// Returns an [`Interned`](labels::intern::Interned) value corresponding to `self`.
            fn intern(&self) -> $crate::intern::Interned<dyn $label_name>
            where
                Self: Sized,
            {
                $crate::__paste::paste! {
                    [<$label_name:upper _INTERNER>].intern(self)
                }
            }
        }

        impl $label_name for $crate::intern::Interned<dyn $label_name> {
            $($(
                fn $impl_method(&$impl_self) -> $impl_ret $impl_body
            )*)?

            fn dyn_clone(&self) -> ::std::boxed::Box<dyn $label_name> {
                (**self).dyn_clone()
            }

            fn as_dyn_eq(&self) -> &dyn $crate::DynEq {
                (**self).as_dyn_eq()
            }

            fn as_dyn_hash(&self) -> &dyn $crate::DynHash {
                (**self).as_dyn_hash()
            }

            fn intern(&self) -> Self {
                *self
            }
        }

        impl PartialEq for dyn $label_name {
            fn eq(&self, other: &Self) -> bool {
                self.as_dyn_eq().dyn_eq(other.as_dyn_eq())
            }
        }

        impl Eq for dyn $label_name {}

        impl ::std::hash::Hash for dyn $label_name {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                self.as_dyn_hash().dyn_hash(state);
            }
        }

        impl $crate::intern::Internable for dyn $label_name {
            fn leak(&self) -> &'static Self {
                Box::leak(self.dyn_clone())
            }

            fn ref_eq(&self, other: &Self) -> bool {
                if self.as_dyn_eq().type_id() != other.as_dyn_eq().type_id() {
                    return false;
                }
                (self as *const Self as *const ()) == (other as *const Self as *const ())
            }

            fn ref_hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                use ::std::hash::Hash;
                self.as_dyn_eq().type_id().hash(state);
                (self as *const Self as *const ()).hash(state);
            }
        }

        $crate::__paste::paste! {
            static [<$label_name:upper _INTERNER>]: $crate::intern::Interner<dyn $label_name> =
                $crate::intern::Interner::new();
        }
    };
}

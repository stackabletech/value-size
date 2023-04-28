#![doc = include_str!("../../README.md")]

use std::collections::BTreeMap;

pub use value_size_derive::Size;

/// Trait for objects that can analyze their own size.
///
/// Intended for memory profiling. Usually implemented using the [`derive@Size`] derive macro.
pub trait Size: Sized {
    /// Get the full size occupied by an object.
    fn full_size(&self) -> usize {
        std::mem::size_of::<Self>() + self.indirect_size()
    }

    /// Get the size of all values that are owned by this object but contained outside of it (such as in a Vec or Box's allocation).
    ///
    /// This should be 0 for simple scalars (such as [`u8`]).
    fn indirect_size(&self) -> usize;
}

impl<T: Size> Size for Option<T> {
    fn indirect_size(&self) -> usize {
        self.as_ref().map_or(0, Size::indirect_size)
    }
}
impl<T: Size> Size for Box<T> {
    fn indirect_size(&self) -> usize {
        T::full_size(self)
    }
}
impl<T: Size> Size for Vec<T> {
    fn indirect_size(&self) -> usize {
        self.capacity() * std::mem::size_of::<T>()
            + self.iter().map(Size::indirect_size).sum::<usize>()
    }
}
impl<K: Size, V: Size> Size for BTreeMap<K, V> {
    fn indirect_size(&self) -> usize {
        // optimistic but close enough
        self.iter()
            .map(|(k, v)| k.full_size() + v.full_size())
            .sum::<usize>()
    }
}
impl Size for serde_json::Value {
    fn indirect_size(&self) -> usize {
        match self {
            serde_json::Value::Null => 0,
            serde_json::Value::Bool(b) => b.indirect_size(),
            serde_json::Value::Number(n) => n.indirect_size(),
            serde_json::Value::String(s) => s.indirect_size(),
            serde_json::Value::Array(a) => a.indirect_size(),
            serde_json::Value::Object(o) => o.indirect_size(),
        }
    }
}
impl Size for serde_json::Map<String, serde_json::Value> {
    fn indirect_size(&self) -> usize {
        // optimistic but close enough
        self.iter()
            .map(|(k, v)| k.full_size() + v.full_size())
            .sum::<usize>()
    }
}
impl Size for String {
    fn indirect_size(&self) -> usize {
        self.capacity()
    }
}

macro_rules! impl_scalar {
    ($ty:ty) => {
        impl Size for $ty {
            fn indirect_size(&self) -> usize {
                0
            }
        }
    };
}
impl_scalar!(u8);
impl_scalar!(i32);
impl_scalar!(i64);
impl_scalar!(f64);
impl_scalar!(bool);
impl_scalar!(chrono::DateTime<chrono::Utc>);
impl_scalar!(serde_json::Number);

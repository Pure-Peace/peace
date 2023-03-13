pub use arc_swap::*;
pub use atomic_float::{AtomicF32, AtomicF64};

use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    sync::{atomic::*, Arc},
};

pub trait AtomicValue: Sized {
    type Value;
    fn val(&self) -> Self::Value;
    fn set(&self, val: Self::Value);
}

pub type Atomic<T> = AtomicAny<Arc<T>>;
pub type AtomicOption<T> = AtomicAny<Option<Arc<T>>>;

impl<T> Atomic<T> {
    pub fn new(val: T) -> Self {
        Self(Arc::new(val).into())
    }
}

impl<T> AtomicOption<T> {
    pub fn new(val: T) -> Self {
        Self(Some(Arc::new(val)).into())
    }

    pub fn from_option(option: Option<T>) -> Self {
        Self(option.map(|inner| inner.into()).into())
    }
}

impl<T> Serialize for Atomic<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.load_full().serialize(serializer)
    }
}

impl<T> Serialize for AtomicOption<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.load_full().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Atomic<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).and_then(|t| Ok(Atomic::new(t)))
    }
}

impl<'de, T> Deserialize<'de> for AtomicOption<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).and_then(|t| Ok(AtomicOption::new(t)))
    }
}

impl Into<Atomic<String>> for String {
    fn into(self) -> Atomic<String> {
        Atomic::new(self)
    }
}

impl Into<AtomicOption<String>> for Option<String> {
    fn into(self) -> AtomicOption<String> {
        AtomicOption::from_option(self)
    }
}

#[derive(Debug, Default)]
pub struct AtomicAny<T: RefCnt>(ArcSwapAny<T>);

impl<T> AtomicValue for AtomicAny<T>
where
    T: RefCnt,
{
    type Value = T;

    fn val(&self) -> Self::Value {
        self.0.load_full()
    }

    fn set(&self, val: Self::Value) {
        self.0.store(val)
    }
}

impl<T> Deref for AtomicAny<T>
where
    T: RefCnt,
{
    type Target = ArcSwapAny<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! implAtomicValue {
    ($($ty: ty$(,)*)*) => {
        paste::paste! {
            $(
                #[derive(Debug, Default)]
                pub struct [<$ty:camel>]([<Atomic $ty:camel>]);

                impl [<$ty:camel>] {
                    pub fn new(val: [<$ty:snake>]) -> Self {
                        Self([<Atomic $ty:camel>]::new(val))
                    }
                }

                impl AtomicValue for [<$ty:camel>] {
                    type Value = [<$ty:snake>];

                    fn val(&self) -> Self::Value {
                        self.0.load(Ordering::SeqCst)
                    }

                    fn set(&self, val: Self::Value) {
                        self.0.store(val, Ordering::SeqCst)
                    }
                }

                impl Deref for [<$ty:camel>] {
                    type Target = [<Atomic $ty:camel>];

                    fn deref(&self) -> &Self::Target {
                        &self.0
                    }
                }

                impl Into<[<Atomic $ty:camel>]> for [<$ty:camel>] {
                    fn into(self) -> [<Atomic $ty:camel>] {
                        self.0
                    }
                }

                impl Into<[<$ty:camel>]> for [<$ty:snake>] {
                    fn into(self) -> [<$ty:camel>] {
                        [<$ty:camel>]::new(self)
                    }
                }
            )*
        }
    };
}

implAtomicValue!(
    bool, i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64
);

macro_rules! implAtomicValueSerde {
    ($($ty: ty$(,)*)*) => {
        paste::paste! {
            $(
                impl Serialize for [<$ty:camel>] {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: serde::Serializer,
                    {
                        serializer.[<serialize_$ty:snake>](self.val())
                    }
                }
            )*
        }
    };
}

implAtomicValueSerde!(bool, i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);

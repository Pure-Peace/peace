use bitmask_enum::bitmask;
use enum_primitive_derive::Primitive;
use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    Primitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum ChannelType {
    #[default]
    Private       = 0,
    Public        = 1,
    Group         = 2,
    Multiplayer   = 3,
    Spectaor      = 4,
}

#[rustfmt::skip]
#[derive(Default)]
#[bitmask(i32)]
pub enum Platform {
    #[default]
    None    = 0,
    Bancho  = 1,
    Lazer   = 2,
    Web     = 3,
}

impl serde::Serialize for Platform {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(self.bits())
    }
}

impl<'de> serde::Deserialize<'de> for Platform {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        i32::deserialize(deserializer).map(Self::from)
    }
}

impl Platform {
    #[inline]
    pub const fn all_platforms() -> [Self; 3] {
        [Self::Bancho, Self::Lazer, Self::Web]
    }

    #[inline]
    pub fn platforms_array(&self) -> [Option<Self>; 3] {
        [
            self.contains(Self::Bancho).then_some(Self::Bancho),
            self.contains(Self::Lazer).then_some(Self::Lazer),
            self.contains(Self::Web).then_some(Self::Web),
        ]
    }

    #[inline]
    pub fn add(&mut self, platforms: &Platform) {
        self.bits |= platforms.bits()
    }

    #[inline]
    pub fn remove(&mut self, platforms: &Platform) {
        self.bits &= !platforms.bits()
    }
}

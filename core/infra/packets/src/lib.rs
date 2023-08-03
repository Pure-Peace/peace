use derive_deref::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, sync::Arc, vec::IntoIter};

pub mod queue;
pub use queue::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct PacketData(pub Vec<u8>);

impl PacketData {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_iter(&self) -> std::vec::IntoIter<u8> {
        self.to_vec().into_iter()
    }
}

impl From<&[u8]> for PacketData {
    fn from(value: &[u8]) -> Self {
        Self(Vec::from(value))
    }
}

impl From<Vec<u8>> for PacketData {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<Arc<Vec<u8>>> for PacketData {
    fn from(value: Arc<Vec<u8>>) -> Self {
        Self(Arc::try_unwrap(value).unwrap_or_else(|ptr| (*ptr).clone()))
    }
}

impl From<PacketDataPtr> for PacketData {
    fn from(value: PacketDataPtr) -> Self {
        value.0.into()
    }
}

impl FromIterator<u8> for PacketData {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl IntoIterator for PacketData {
    type Item = u8;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct PacketDataPtr(pub Arc<Vec<u8>>);

impl PacketDataPtr {
    pub fn into_inner(self) -> Arc<Vec<u8>> {
        self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_iter(&self) -> std::vec::IntoIter<u8> {
        self.to_vec().into_iter()
    }
}

impl From<&[u8]> for PacketDataPtr {
    fn from(value: &[u8]) -> Self {
        Self(Vec::from(value).into())
    }
}

impl From<Vec<u8>> for PacketDataPtr {
    fn from(value: Vec<u8>) -> Self {
        Self(value.into())
    }
}

impl From<Arc<Vec<u8>>> for PacketDataPtr {
    fn from(value: Arc<Vec<u8>>) -> Self {
        Self(value)
    }
}

impl From<PacketData> for PacketDataPtr {
    fn from(value: PacketData) -> Self {
        Self(value.0.into())
    }
}

impl FromIterator<u8> for PacketDataPtr {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self(Vec::from_iter(iter).into())
    }
}

impl IntoIterator for PacketDataPtr {
    type Item = u8;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        Arc::try_unwrap(self.0).unwrap_or_else(|ptr| (*ptr).clone()).into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    Data(PacketData),
    Ptr(PacketDataPtr),
}

impl AsRef<[u8]> for Packet {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}

impl std::ops::Deref for Packet {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Packet::Data(data) => data.0.deref(),
            Packet::Ptr(ptr) => ptr.0.deref(),
        }
    }
}

impl Default for Packet {
    fn default() -> Self {
        Self::Data(PacketData::default())
    }
}

impl IntoIterator for Packet {
    type Item = u8;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Packet::Data(data) => data.into_iter(),
            Packet::Ptr(ptr) => ptr.into_iter(),
        }
    }
}

impl From<Vec<u8>> for Packet {
    fn from(data: Vec<u8>) -> Self {
        Self::Data(data.into())
    }
}

impl From<Arc<Vec<u8>>> for Packet {
    fn from(ptr: Arc<Vec<u8>>) -> Self {
        Self::Ptr(ptr.into())
    }
}

impl From<PacketData> for Packet {
    fn from(data: PacketData) -> Self {
        Self::Data(data)
    }
}

impl From<PacketDataPtr> for Packet {
    fn from(ptr: PacketDataPtr) -> Self {
        Self::Ptr(ptr)
    }
}

impl Packet {
    pub fn new(data: Vec<u8>) -> Self {
        Self::Data(PacketData(data))
    }

    pub fn new_ptr(data: Vec<u8>) -> Self {
        Self::Ptr(data.into())
    }

    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Packet::Data(data) => data.to_vec(),
            Packet::Ptr(ptr) => ptr.to_vec(),
        }
    }

    pub fn to_iter(&self) -> std::vec::IntoIter<u8> {
        match self {
            Packet::Data(data) => data.to_iter(),
            Packet::Ptr(ptr) => ptr.to_iter(),
        }
    }
}

//! Deserialize postgres rows into a Rust data structure.
use crate::serde_postgres::raw::Raw;
use serde::de::{self, value::SeqDeserializer, Deserialize, IntoDeserializer, Visitor};
use std::iter::FromIterator;
use tokio_postgres::Row;

mod error;

pub use self::error::*;

/// A structure that deserialize Postgres rows into Rust values.
pub struct Deserializer<'a> {
    input: &'a Row,
    index: usize,
}

impl<'a> Deserializer<'a> {
    /// Create a `Row` deserializer from a `Row`.
    pub fn from_row(input: &'a Row) -> Self {
        Self { index: 0, input }
    }
}

/// Attempt to deserialize from a single `Row`.
pub fn from_row<'a, T: Deserialize<'a>>(input: &'a Row) -> DeResult<T> {
    let mut deserializer = Deserializer::from_row(input);
    Ok(T::deserialize(&mut deserializer)?)
}

/// Attempt to deserialize multiple `Rows`.
pub fn from_rows<'a, T: Deserialize<'a>, I: FromIterator<T>>(
    input: impl IntoIterator<Item = &'a Row>,
) -> DeResult<I> {
    input
        .into_iter()
        .map(|row| {
            let mut deserializer = Deserializer::from_row(row);
            T::deserialize(&mut deserializer)
        })
        .collect()
}

macro_rules! unsupported_type {
    ($($fn_name:ident),*,) => {
        $(
            fn $fn_name<V: Visitor<'de>>(self, _: V) -> DeResult<V::Value> {
                Err(DeError::UnsupportedType)
            }
        )*
    }
}

macro_rules! get_value {
    ($this:ident, $ty:ty) => {{
        $this
            .input
            .try_get::<_, $ty>($this.index)
            .map_err(|e| DeError::InvalidType(format!("{:?}", e)))?
    }};
}

macro_rules! visit_value {
    ($this:ident, $ty:ty, $v:ident . $vfn:ident) => {{
        $v.$vfn(get_value!($this, $ty))
    }};

    ($this:ident, $v:ident . $vfn:ident) => {
        visit_value!($this, _, $v.$vfn)
    };
}

impl<'de, 'a, 'b> de::Deserializer<'de> for &'b mut Deserializer<'a> {
    type Error = DeError;

    unsupported_type! {
        deserialize_any, // TODO
        deserialize_u8,
        deserialize_u16,
        deserialize_u64,
        deserialize_char,
        deserialize_unit,
        deserialize_identifier,
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_bool)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_i8)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_i16)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_i32)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_i64)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_u32)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_f64)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_str)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_string)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        // TODO: use visit_borrowed_bytes
        visit_value!(self, visitor.visit_bytes)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visit_value!(self, visitor.visit_byte_buf)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        match get_value!(self, Option<Raw>) {
            Some(_) => visitor.visit_some(self),
            None => visitor.visit_none(),
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        let raw = get_value!(self, Raw);
        visitor.visit_seq(SeqDeserializer::new(raw.iter().copied()))
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _: &str,
        _: &[&str],
        _visitor: V,
    ) -> DeResult<V::Value> {
        Err(DeError::UnsupportedType)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _: &str, _: V) -> DeResult<V::Value> {
        Err(DeError::UnsupportedType)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _: &str, _: V) -> DeResult<V::Value> {
        Err(DeError::UnsupportedType)
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _: usize, _: V) -> DeResult<V::Value> {
        Err(DeError::UnsupportedType)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _: &str,
        _: usize,
        _: V,
    ) -> DeResult<V::Value> {
        Err(DeError::UnsupportedType)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> DeResult<V::Value> {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        v: V,
    ) -> DeResult<V::Value> {
        self.deserialize_map(v)
    }
}

impl<'de, 'a> de::MapAccess<'de> for Deserializer<'a> {
    type Error = DeError;

    fn next_key_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> DeResult<Option<T::Value>> {
        if self.index >= self.input.columns().len() {
            return Ok(None);
        }

        self.input
            .columns()
            .get(self.index)
            .ok_or(DeError::UnknownField)
            .map(|c| c.name().to_owned().into_deserializer())
            .and_then(|n| seed.deserialize(n).map(Some))
    }

    fn next_value_seed<T: de::DeserializeSeed<'de>>(&mut self, seed: T) -> DeResult<T::Value> {
        let result = seed.deserialize(&mut *self);
        self.index += 1;
        if let Err(DeError::InvalidType(err)) = result {
            let name = self.input.columns().get(self.index - 1).unwrap().name();
            Err(DeError::InvalidType(format!("{} {}", name, err)))
        } else {
            result
        }
    }
}

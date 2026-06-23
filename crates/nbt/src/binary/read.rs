use crate::binary::{TypeTag, UnknownTagError};
use crate::{BedrockHeader, Compound, List, ListVariant, NamedTag, Variant};
use std::collections::BTreeMap;
use std::io::{self, Read};
use std::string::FromUtf8Error;

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("Unknown tag")]
    UnknownTag(#[from] UnknownTagError),
    #[error("Invalid UTF-8 in string")]
    InvalidUtf8(#[from] FromUtf8Error),
    #[error("Unexpected end tag")]
    UnexpectedEndTag,
}

pub trait Readable: Sized {
    fn read_le(reader: &mut impl Read) -> Result<Self, ReadError>;
}

impl Readable for BedrockHeader {
    fn read_le(reader: &mut impl Read) -> Result<Self, ReadError> {
        let version = u32::read_le(&mut *reader)?;
        let size = u32::read_le(&mut *reader)?;
        Ok(BedrockHeader { version, size })
    }
}

impl Readable for NamedTag {
    fn read_le(reader: &mut impl Read) -> Result<Self, ReadError> {
        let tag = u8::read_le(&mut *reader)?.try_into()?;
        let name = String::read_le(&mut *reader)?;
        let value = read_tag(&mut *reader, tag)?;

        Ok(NamedTag(name, value))
    }
}

impl Readable for Option<NamedTag> {
    fn read_le(reader: &mut impl Read) -> Result<Self, ReadError> {
        let tag = u8::read_le(&mut *reader)?;
        if tag == 0 {
            return Ok(None);
        }

        let tag = tag.try_into()?;
        let name = String::read_le(&mut *reader)?;
        let value = read_tag(&mut *reader, tag)?;

        Ok(Some(NamedTag(name, value)))
    }
}

fn read_tag(reader: &mut impl Read, tag: TypeTag) -> Result<Variant, ReadError> {
    let variant = match tag {
        TypeTag::Int8 => u8::read_le(reader).map(Variant::Int8)?,
        TypeTag::Int16 => u16::read_le(reader).map(Variant::Int16)?,
        TypeTag::Int32 => u32::read_le(reader).map(Variant::Int32)?,
        TypeTag::Int64 => u64::read_le(reader).map(Variant::Int64)?,
        TypeTag::Float32 => f32::read_le(reader).map(Variant::Float32)?,
        TypeTag::Float64 => f64::read_le(reader).map(Variant::Float64)?,
        TypeTag::String => String::read_le(reader).map(Variant::String)?,
        TypeTag::Int8List => ListVariant::read_le(reader).map(Variant::Int8List)?,
        TypeTag::List => List::read_le(reader).map(Variant::List)?,
        TypeTag::Compound => Compound::read_le(reader).map(Variant::Compound)?,
        TypeTag::Int32List => ListVariant::read_le(reader).map(Variant::Int32List)?,
        TypeTag::Int64List => ListVariant::read_le(reader).map(Variant::Int64List)?,
        TypeTag::EndCompound => return Err(ReadError::UnexpectedEndTag),
    };

    Ok(variant)
}

impl Readable for Compound {
    fn read_le(reader: &mut impl Read) -> Result<Self, ReadError> {
        let mut map = BTreeMap::new();
        while let Some(NamedTag(key, value)) = Option::<NamedTag>::read_le(&mut *reader)? {
            map.insert(key, value);
        }

        Ok(Compound(map))
    }
}

impl Readable for String {
    fn read_le(reader: &mut impl Read) -> Result<Self, ReadError> {
        let length = u16::read_le(&mut *reader)?;
        let mut buf = vec![0u8; length.into()];
        reader.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
}

impl Readable for List {
    fn read_le(reader: &mut impl Read) -> Result<Self, ReadError> {
        let tag = u8::read_le(&mut *reader)?;
        let tag = tag.try_into()?;
        let list = match tag {
            TypeTag::Int8 => ListVariant::read_le(reader).map(List::Int8)?,
            TypeTag::Int16 => ListVariant::read_le(reader).map(List::Int16)?,
            TypeTag::Int32 => ListVariant::read_le(reader).map(List::Int32)?,
            TypeTag::Int64 => ListVariant::read_le(reader).map(List::Int64)?,
            TypeTag::Float32 => ListVariant::read_le(reader).map(List::Float32)?,
            TypeTag::Float64 => ListVariant::read_le(reader).map(List::Float64)?,
            TypeTag::String => ListVariant::read_le(reader).map(List::String)?,
            TypeTag::Int8List => ListVariant::read_le(reader).map(List::Int8List)?,
            TypeTag::List => ListVariant::read_le(reader).map(List::List)?,
            TypeTag::Compound => ListVariant::read_le(reader).map(List::Compound)?,
            TypeTag::Int32List => ListVariant::read_le(reader).map(List::Int32List)?,
            TypeTag::Int64List => ListVariant::read_le(reader).map(List::Int64List)?,
            TypeTag::EndCompound => {
                let _length = u32::read_le(&mut *reader)?;
                List::Empty
            }
        };

        Ok(list)
    }
}

impl<T: Readable> Readable for ListVariant<T> {
    fn read_le(reader: &mut impl Read) -> Result<Self, ReadError> {
        let length = u32::read_le(&mut *reader)?;
        let mut items = Vec::new();

        for _ in 0..length {
            items.push(T::read_le(&mut *reader)?);
        }

        Ok(ListVariant(items))
    }
}

impl<T: ReadablePrimitive> Readable for T {
    fn read_le(reader: &mut impl Read) -> Result<Self, ReadError> {
        let mut buf = Self::EMPTY_BYTE_ARRAY;
        reader.read_exact(buf.as_mut())?;
        Ok(Self::from_le(buf))
    }
}

trait ReadablePrimitive {
    type ByteArray: AsMut<[u8]>;

    const EMPTY_BYTE_ARRAY: Self::ByteArray;

    fn from_le(byte_array: Self::ByteArray) -> Self;
}

macro_rules! impl_readable_primitive {
    ($T:ty, $size:expr) => {
        impl ReadablePrimitive for $T {
            type ByteArray = [u8; $size];

            const EMPTY_BYTE_ARRAY: Self::ByteArray = [0; $size];

            fn from_le(byte_array: Self::ByteArray) -> Self {
                Self::from_le_bytes(byte_array)
            }
        }
    };
}

impl_readable_primitive!(u8, 1);
impl_readable_primitive!(i8, 1);
impl_readable_primitive!(u16, 2);
impl_readable_primitive!(i16, 2);
impl_readable_primitive!(u32, 4);
impl_readable_primitive!(i32, 4);
impl_readable_primitive!(u64, 8);
impl_readable_primitive!(i64, 8);

impl_readable_primitive!(f32, 4);
impl_readable_primitive!(f64, 8);

#[test]
fn read_structure() {
    use bytes::Buf;

    let bytes = include_bytes!("../../tests/crossbow_piglin.mcstructure");
    let mut reader = bytes.reader();
    NamedTag::read_le(&mut reader).unwrap();
}

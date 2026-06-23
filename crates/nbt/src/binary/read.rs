use crate::binary::{Endianness, NbtPrimitive, TypeTag, UnknownTagError};
use crate::{BedrockHeader, Compound, List, ListVariant, NamedTag, Variant};
use std::collections::BTreeMap;
use std::io::{self, Read};
use std::string::FromUtf8Error;

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("{0}")]
    UnknownTag(#[from] UnknownTagError),
    #[error("invalid utf-8 in string")]
    InvalidUtf8(#[from] FromUtf8Error),
    #[error("unexpected end tag")]
    UnexpectedEndTag,
}

pub trait ReadNbt: Sized {
    fn read_nbt(reader: &mut impl Read, endianness: Endianness) -> Result<Self, ReadError>;
}

impl ReadNbt for BedrockHeader {
    fn read_nbt(reader: &mut impl Read, endianness: Endianness) -> Result<Self, ReadError> {
        let version = u32::read_nbt(&mut *reader, endianness)?;
        let size = u32::read_nbt(&mut *reader, endianness)?;
        Ok(BedrockHeader { version, size })
    }
}

impl ReadNbt for NamedTag {
    fn read_nbt(reader: &mut impl Read, endianness: Endianness) -> Result<Self, ReadError> {
        let tag = u8::read_nbt(&mut *reader, endianness)?.try_into()?;
        let name = String::read_nbt(&mut *reader, endianness)?;
        let value = read_tag(&mut *reader, endianness, tag)?;

        Ok(NamedTag(name, value))
    }
}

impl ReadNbt for Option<NamedTag> {
    fn read_nbt(reader: &mut impl Read, endianness: Endianness) -> Result<Self, ReadError> {
        let tag = u8::read_nbt(&mut *reader, endianness)?;
        if tag == 0 {
            return Ok(None);
        }

        let tag = tag.try_into()?;
        let name = String::read_nbt(&mut *reader, endianness)?;
        let value = read_tag(&mut *reader, endianness, tag)?;

        Ok(Some(NamedTag(name, value)))
    }
}

fn read_tag(
    reader: &mut impl Read,
    endianness: Endianness,
    tag: TypeTag,
) -> Result<Variant, ReadError> {
    let variant = match tag {
        TypeTag::Int8 => u8::read_nbt(reader, endianness).map(Variant::Int8)?,
        TypeTag::Int16 => u16::read_nbt(reader, endianness).map(Variant::Int16)?,
        TypeTag::Int32 => u32::read_nbt(reader, endianness).map(Variant::Int32)?,
        TypeTag::Int64 => u64::read_nbt(reader, endianness).map(Variant::Int64)?,
        TypeTag::Float32 => f32::read_nbt(reader, endianness).map(Variant::Float32)?,
        TypeTag::Float64 => f64::read_nbt(reader, endianness).map(Variant::Float64)?,
        TypeTag::String => String::read_nbt(reader, endianness).map(Variant::String)?,
        TypeTag::Int8List => ListVariant::read_nbt(reader, endianness).map(Variant::Int8List)?,
        TypeTag::List => List::read_nbt(reader, endianness).map(Variant::List)?,
        TypeTag::Compound => Compound::read_nbt(reader, endianness).map(Variant::Compound)?,
        TypeTag::Int32List => ListVariant::read_nbt(reader, endianness).map(Variant::Int32List)?,
        TypeTag::Int64List => ListVariant::read_nbt(reader, endianness).map(Variant::Int64List)?,
        TypeTag::EndCompound => return Err(ReadError::UnexpectedEndTag),
    };

    Ok(variant)
}

impl ReadNbt for Compound {
    fn read_nbt(reader: &mut impl Read, endianness: Endianness) -> Result<Self, ReadError> {
        let mut map = BTreeMap::new();
        while let Some(NamedTag(key, value)) =
            Option::<NamedTag>::read_nbt(&mut *reader, endianness)?
        {
            map.insert(key, value);
        }

        Ok(Compound(map))
    }
}

impl ReadNbt for String {
    fn read_nbt(reader: &mut impl Read, endianness: Endianness) -> Result<Self, ReadError> {
        let length = u16::read_nbt(&mut *reader, endianness)?;
        let mut buf = vec![0u8; length.into()];
        reader.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
}

impl ReadNbt for List {
    fn read_nbt(reader: &mut impl Read, endianness: Endianness) -> Result<Self, ReadError> {
        let tag = u8::read_nbt(&mut *reader, endianness)?;
        let tag = tag.try_into()?;
        let list = match tag {
            TypeTag::Int8 => ListVariant::read_nbt(reader, endianness).map(List::Int8)?,
            TypeTag::Int16 => ListVariant::read_nbt(reader, endianness).map(List::Int16)?,
            TypeTag::Int32 => ListVariant::read_nbt(reader, endianness).map(List::Int32)?,
            TypeTag::Int64 => ListVariant::read_nbt(reader, endianness).map(List::Int64)?,
            TypeTag::Float32 => ListVariant::read_nbt(reader, endianness).map(List::Float32)?,
            TypeTag::Float64 => ListVariant::read_nbt(reader, endianness).map(List::Float64)?,
            TypeTag::String => ListVariant::read_nbt(reader, endianness).map(List::String)?,
            TypeTag::Int8List => ListVariant::read_nbt(reader, endianness).map(List::Int8List)?,
            TypeTag::List => ListVariant::read_nbt(reader, endianness).map(List::List)?,
            TypeTag::Compound => ListVariant::read_nbt(reader, endianness).map(List::Compound)?,
            TypeTag::Int32List => ListVariant::read_nbt(reader, endianness).map(List::Int32List)?,
            TypeTag::Int64List => ListVariant::read_nbt(reader, endianness).map(List::Int64List)?,
            TypeTag::EndCompound => {
                let _length = u32::read_nbt(&mut *reader, endianness)?;
                List::Empty
            }
        };

        Ok(list)
    }
}

impl<T: ReadNbt> ReadNbt for ListVariant<T> {
    fn read_nbt(reader: &mut impl Read, endianness: Endianness) -> Result<Self, ReadError> {
        let length = u32::read_nbt(&mut *reader, endianness)?;
        let mut items = Vec::new();

        for _ in 0..length {
            items.push(T::read_nbt(&mut *reader, endianness)?);
        }

        Ok(ListVariant(items))
    }
}

impl<T: NbtPrimitive> ReadNbt for T {
    fn read_nbt(reader: &mut impl Read, endianness: Endianness) -> Result<Self, ReadError> {
        let mut buf = Self::EMPTY_BYTE_ARRAY;
        reader.read_exact(buf.as_mut())?;
        Ok(Self::from_bytes(buf, endianness))
    }
}

#[test]
fn read_structure() {
    use bytes::Buf;

    let bytes = include_bytes!("../../tests/crossbow_piglin.mcstructure");
    let mut reader = bytes.reader();
    NamedTag::read_nbt(&mut reader, Endianness::Little).unwrap();
}

use crate::binary::{Endianness, NbtPrimitive, TypeTag};
use crate::{BedrockHeader, Compound, List, ListVariant, NamedTag, Variant};
use std::io::{self, Write};

pub trait WriteNbt {
    fn write_nbt(&self, writer: &mut impl Write, endianness: Endianness) -> io::Result<()>;
}

impl WriteNbt for BedrockHeader {
    fn write_nbt(&self, writer: &mut impl Write, endianness: Endianness) -> io::Result<()> {
        self.version.write_nbt(&mut *writer, endianness)?;
        self.size.write_nbt(&mut *writer, endianness)?;
        Ok(())
    }
}

impl WriteNbt for NamedTag {
    fn write_nbt(&self, writer: &mut impl Write, endianness: Endianness) -> io::Result<()> {
        self.1.type_tag().write_nbt(&mut *writer, endianness)?;
        self.0.write_nbt(&mut *writer, endianness)?;
        self.1.write_nbt(&mut *writer, endianness)
    }
}

impl WriteNbt for String {
    fn write_nbt(&self, writer: &mut impl Write, endianness: Endianness) -> io::Result<()> {
        u16::try_from(self.len())
            .map_err(io::Error::other)?
            .write_nbt(&mut *writer, endianness)?;

        writer.write_all(self.as_bytes())
    }
}

impl WriteNbt for TypeTag {
    fn write_nbt(&self, writer: &mut impl Write, _: Endianness) -> io::Result<()> {
        let tag = (*self) as u8;
        writer.write_all(&[tag])
    }
}

impl WriteNbt for Variant {
    fn write_nbt(&self, writer: &mut impl Write, endianness: Endianness) -> io::Result<()> {
        match self {
            Self::Int8(value) => value.write_nbt(&mut *writer, endianness),
            Self::Int16(value) => value.write_nbt(&mut *writer, endianness),
            Self::Int32(value) => value.write_nbt(&mut *writer, endianness),
            Self::Int64(value) => value.write_nbt(&mut *writer, endianness),
            Self::Float32(value) => value.write_nbt(&mut *writer, endianness),
            Self::Float64(value) => value.write_nbt(&mut *writer, endianness),
            Self::String(value) => value.write_nbt(&mut *writer, endianness),
            Self::List(list) => list.write_nbt(&mut *writer, endianness),
            Self::Compound(compound) => compound.write_nbt(&mut *writer, endianness),
            Self::Int8List(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            Self::Int32List(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            Self::Int64List(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
        }
    }
}

impl Variant {
    fn type_tag(&self) -> TypeTag {
        match self {
            Self::Int8(_) => TypeTag::Int8,
            Self::Int16(_) => TypeTag::Int16,
            Self::Int32(_) => TypeTag::Int32,
            Self::Int64(_) => TypeTag::Int64,
            Self::Float32(_) => TypeTag::Float32,
            Self::Float64(_) => TypeTag::Float64,
            Self::String(_) => TypeTag::String,
            Self::List(_) => TypeTag::List,
            Self::Compound(_) => TypeTag::Compound,
            Self::Int8List(_) => TypeTag::Int8List,
            Self::Int32List(_) => TypeTag::Int32List,
            Self::Int64List(_) => TypeTag::Int64List,
        }
    }
}

impl WriteNbt for Compound {
    fn write_nbt(&self, writer: &mut impl Write, endianness: Endianness) -> io::Result<()> {
        for (key, value) in &self.0 {
            value.type_tag().write_nbt(&mut *writer, endianness)?;
            key.write_nbt(&mut *writer, endianness)?;
            value.write_nbt(&mut *writer, endianness)?;
        }

        TypeTag::EndCompound.write_nbt(&mut *writer, endianness)?;

        Ok(())
    }
}

impl WriteNbt for List {
    fn write_nbt(&self, writer: &mut impl Write, endianness: Endianness) -> io::Result<()> {
        self.type_tag().write_nbt(&mut *writer, endianness)?;

        match self {
            List::Int8(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::Int16(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::Int32(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::Int64(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::Float32(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::Float64(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::String(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::List(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::Compound(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::Int8List(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::Int32List(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::Int64List(list_variant) => list_variant.write_nbt(&mut *writer, endianness),
            List::Empty => {
                u32::write_nbt(&0, &mut *writer, endianness)?;
                Ok(())
            }
        }
    }
}

impl List {
    fn type_tag(&self) -> TypeTag {
        match self {
            Self::Int8(_) => TypeTag::Int8,
            Self::Int16(_) => TypeTag::Int16,
            Self::Int32(_) => TypeTag::Int32,
            Self::Int64(_) => TypeTag::Int64,
            Self::Float32(_) => TypeTag::Float32,
            Self::Float64(_) => TypeTag::Float64,
            Self::String(_) => TypeTag::String,
            Self::List(_) => TypeTag::List,
            Self::Compound(_) => TypeTag::Compound,
            Self::Int8List(_) => TypeTag::Int8List,
            Self::Int32List(_) => TypeTag::Int32List,
            Self::Int64List(_) => TypeTag::Int64List,
            Self::Empty => TypeTag::EndCompound,
        }
    }
}

impl<T: WriteNbt> WriteNbt for ListVariant<T> {
    fn write_nbt(&self, writer: &mut impl Write, endianness: Endianness) -> io::Result<()> {
        u32::try_from(self.0.len())
            .map_err(io::Error::other)?
            .write_nbt(&mut *writer, endianness)?;

        for item in &self.0 {
            item.write_nbt(&mut *writer, endianness)?;
        }

        Ok(())
    }
}

impl<T: NbtPrimitive> WriteNbt for T {
    fn write_nbt(&self, writer: &mut impl Write, endianness: Endianness) -> io::Result<()> {
        writer.write_all(self.to_bytes(endianness).as_ref())
    }
}

use crate::binary::TypeTag;
use crate::{BedrockHeader, BedrockNbtFile, Compound, List, ListVariant, NamedTag, Variant};
use std::io::{self, Write};

pub trait Writeable {
    fn write_le(&self, writer: &mut impl Write) -> io::Result<()>;
}

impl Writeable for BedrockNbtFile {
    fn write_le(&self, writer: &mut impl Write) -> io::Result<()> {
        if let Some(header) = &self.header {
            header.write_le(&mut *writer)?;
        }

        self.tag.write_le(&mut *writer)?;

        Ok(())
    }
}

impl Writeable for BedrockHeader {
    fn write_le(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&self.size.to_le_bytes())?;
        Ok(())
    }
}

impl Writeable for NamedTag {
    fn write_le(&self, writer: &mut impl Write) -> io::Result<()> {
        self.1.type_tag().write_le(&mut *writer)?;
        self.0.write_le(&mut *writer)?;
        self.1.write_value_le(&mut *writer)
    }
}

impl Writeable for String {
    fn write_le(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(
            &u16::try_from(self.len())
                .map_err(io::Error::other)?
                .to_le_bytes(),
        )?;
        writer.write_all(self.as_bytes())
    }
}

impl Writeable for TypeTag {
    fn write_le(&self, writer: &mut impl Write) -> io::Result<()> {
        let tag = (*self) as u8;
        writer.write_all(&[tag])
    }
}

impl Variant {
    fn write_value_le(&self, writer: &mut impl Write) -> io::Result<()> {
        match self {
            Self::Int8(value) => value.write_le(&mut *writer),
            Self::Int16(value) => value.write_le(&mut *writer),
            Self::Int32(value) => value.write_le(&mut *writer),
            Self::Int64(value) => value.write_le(&mut *writer),
            Self::Float32(value) => value.write_le(&mut *writer),
            Self::Float64(value) => value.write_le(&mut *writer),
            Self::String(value) => value.write_le(&mut *writer),
            Self::List(list) => list.write_le(&mut *writer),
            Self::Compound(compound) => compound.write_le(&mut *writer),
            Self::Int8List(list_variant) => list_variant.write_le(&mut *writer),
            Self::Int32List(list_variant) => list_variant.write_le(&mut *writer),
            Self::Int64List(list_variant) => list_variant.write_le(&mut *writer),
        }
    }

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

impl Writeable for Compound {
    fn write_le(&self, writer: &mut impl Write) -> io::Result<()> {
        for (key, value) in &self.0 {
            value.type_tag().write_le(&mut *writer)?;
            key.write_le(&mut *writer)?;
            value.write_value_le(&mut *writer)?;
        }

        TypeTag::EndCompound.write_le(&mut *writer)?;

        Ok(())
    }
}

impl Writeable for List {
    fn write_le(&self, writer: &mut impl Write) -> io::Result<()> {
        self.type_tag().write_le(&mut *writer)?;

        match self {
            List::Int8(list_variant) => list_variant.write_le(&mut *writer),
            List::Int16(list_variant) => list_variant.write_le(&mut *writer),
            List::Int32(list_variant) => list_variant.write_le(&mut *writer),
            List::Int64(list_variant) => list_variant.write_le(&mut *writer),
            List::Float32(list_variant) => list_variant.write_le(&mut *writer),
            List::Float64(list_variant) => list_variant.write_le(&mut *writer),
            List::String(list_variant) => list_variant.write_le(&mut *writer),
            List::List(list_variant) => list_variant.write_le(&mut *writer),
            List::Compound(list_variant) => list_variant.write_le(&mut *writer),
            List::Int8List(list_variant) => list_variant.write_le(&mut *writer),
            List::Int32List(list_variant) => list_variant.write_le(&mut *writer),
            List::Int64List(list_variant) => list_variant.write_le(&mut *writer),
            List::Empty => writer.write_all(&[0]),
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

impl<T: Writeable> Writeable for ListVariant<T> {
    fn write_le(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(
            &u32::try_from(self.0.len())
                .map_err(io::Error::other)?
                .to_le_bytes(),
        )?;

        for item in &self.0 {
            item.write_le(&mut *writer)?;
        }

        Ok(())
    }
}

impl<T: WriteablePrimitive> Writeable for T {
    fn write_le(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(self.to_le().as_ref())
    }
}

trait WriteablePrimitive {
    type ByteArray: AsRef<[u8]>;

    fn to_le(&self) -> Self::ByteArray;
}

macro_rules! impl_readable_primitive {
    ($T:ty, $size:expr) => {
        impl WriteablePrimitive for $T {
            type ByteArray = [u8; $size];

            fn to_le(&self) -> Self::ByteArray {
                self.to_le_bytes()
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

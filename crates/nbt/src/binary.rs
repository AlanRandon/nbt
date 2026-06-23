pub mod read;
pub mod write;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Endianness {
    Little,
    Big,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeTag {
    EndCompound = 0,
    Int8 = 1,
    Int16 = 2,
    Int32 = 3,
    Int64 = 4,
    Float32 = 5,
    Float64 = 6,
    String = 8,
    Int8List = 7,
    List = 9,
    Compound = 10,
    Int32List = 11,
    Int64List = 12,
}

#[derive(Debug, thiserror::Error)]
#[error("The tag '{0}' was unrecognised")]
pub struct UnknownTagError(u8);

impl TryFrom<u8> for TypeTag {
    type Error = UnknownTagError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::EndCompound),
            1 => Ok(Self::Int8),
            2 => Ok(Self::Int16),
            3 => Ok(Self::Int32),
            4 => Ok(Self::Int64),
            5 => Ok(Self::Float32),
            6 => Ok(Self::Float64),
            8 => Ok(Self::String),
            7 => Ok(Self::Int8List),
            9 => Ok(Self::List),
            10 => Ok(Self::Compound),
            11 => Ok(Self::Int32List),
            12 => Ok(Self::Int64List),
            _ => Err(UnknownTagError(value)),
        }
    }
}

trait NbtPrimitive {
    type ByteArray: AsMut<[u8]> + AsRef<[u8]>;

    const EMPTY_BYTE_ARRAY: Self::ByteArray;

    fn from_bytes(byte_array: Self::ByteArray, endianness: Endianness) -> Self;
    fn into_bytes(&self, endianness: Endianness) -> Self::ByteArray;
}

macro_rules! impl_primitive {
    ($T:ty, $size:expr) => {
        impl NbtPrimitive for $T {
            type ByteArray = [u8; $size];

            const EMPTY_BYTE_ARRAY: Self::ByteArray = [0; $size];

            fn from_bytes(byte_array: Self::ByteArray, endianness: Endianness) -> Self {
                match endianness {
                    Endianness::Big => Self::from_be_bytes(byte_array),
                    Endianness::Little => Self::from_le_bytes(byte_array),
                }
            }

            fn into_bytes(&self, endianness: Endianness) -> Self::ByteArray {
                match endianness {
                    Endianness::Big => self.to_be_bytes(),
                    Endianness::Little => self.to_le_bytes(),
                }
            }
        }
    };
}

impl_primitive!(u8, 1);
impl_primitive!(i8, 1);
impl_primitive!(u16, 2);
impl_primitive!(i16, 2);
impl_primitive!(u32, 4);
impl_primitive!(i32, 4);
impl_primitive!(u64, 8);
impl_primitive!(i64, 8);

impl_primitive!(f32, 4);
impl_primitive!(f64, 8);

#[test]
fn roundtrip_encoding_structure() {
    use crate::{List, NamedTag, Variant};
    use bytes::{Buf, BufMut};
    use read::ReadNbt;
    use write::WriteNbt;

    for endianness in [Endianness::Little, Endianness::Big] {
        for structure in [
            NamedTag(String::new(), Variant::List(List::Empty)),
            {
                let parser =
                    crate::snbt::read::parse::Parser::new(include_str!("../tests/structure.snbt"));
                let variant = parser.parse_variant_and_finish().unwrap();
                let variant = variant.try_into().unwrap();
                NamedTag(String::new(), variant)
            },
            {
                let bytes = include_bytes!("../tests/crossbow_piglin.mcstructure");
                let mut reader = bytes.reader();
                NamedTag::read_nbt(&mut reader, Endianness::Little).unwrap()
            },
            {
                let bytes = include_bytes!("../tests/big_endian_mansion.nbt");
                let mut reader = bytes.reader();
                NamedTag::read_nbt(&mut reader, Endianness::Big).unwrap()
            },
        ] {
            let mut writer = Vec::new().writer();
            structure.write_nbt(&mut writer, endianness).unwrap();
            let bytes = writer.into_inner();

            let mut reader = bytes.reader();
            let roundtrip_encoded_structure = NamedTag::read_nbt(&mut reader, endianness).unwrap();

            assert_eq!(structure, roundtrip_encoded_structure);
        }
    }
}

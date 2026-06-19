pub mod read;
pub mod write;

#[derive(Debug, Clone, Copy)]
enum TypeTag {
    EndCompound = 0,
    Byte = 1,
    Int16 = 2,
    Int32 = 3,
    Int64 = 4,
    Float32 = 5,
    Float64 = 6,
    String = 8,
    ByteList = 7,
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
            1 => Ok(Self::Byte),
            2 => Ok(Self::Int16),
            3 => Ok(Self::Int32),
            4 => Ok(Self::Int64),
            5 => Ok(Self::Float32),
            6 => Ok(Self::Float64),
            8 => Ok(Self::String),
            7 => Ok(Self::ByteList),
            9 => Ok(Self::List),
            10 => Ok(Self::Compound),
            11 => Ok(Self::Int32List),
            12 => Ok(Self::Int64List),
            _ => Err(UnknownTagError(value)),
        }
    }
}

#[test]
fn roundtrip_encoding_structure() {
    use crate::BedrockNbtFile;
    use bytes::{Buf, BufMut};
    use write::Writeable;

    let bytes = include_bytes!("../tests/crossbow_piglin.mcstructure");
    let mut reader = bytes.reader();
    let structure = BedrockNbtFile::read_le_without_header(&mut reader).unwrap();

    let mut writer = Vec::new().writer();
    structure.write_le(&mut writer).unwrap();
    let bytes = writer.into_inner();

    let mut reader = bytes.reader();
    let roundtrip_encoded_structure = BedrockNbtFile::read_le_without_header(&mut reader).unwrap();

    assert_eq!(structure, roundtrip_encoded_structure);
}

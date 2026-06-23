use std::collections::BTreeMap;

pub mod binary;
pub mod snbt;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Variant {
    Int8(u8),
    Int16(u16),
    Int32(u32),
    Int64(u64),
    Float32(f32),
    Float64(f64),
    String(String),
    List(List),
    Compound(Compound),
    Int8List(ListVariant<u8>),
    Int32List(ListVariant<u32>),
    Int64List(ListVariant<u64>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum List {
    Int8(ListVariant<u8>),
    Int16(ListVariant<u16>),
    Int32(ListVariant<u32>),
    Int64(ListVariant<u64>),
    Float32(ListVariant<f32>),
    Float64(ListVariant<f64>),
    String(ListVariant<String>),
    List(ListVariant<List>),
    Compound(ListVariant<Compound>),
    Int8List(ListVariant<ListVariant<u8>>),
    Int32List(ListVariant<ListVariant<u32>>),
    Int64List(ListVariant<ListVariant<u64>>),
    Empty,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ListVariant<T>(pub Vec<T>);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Compound(pub BTreeMap<String, Variant>);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct NamedTag(pub String, pub Variant);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BedrockHeader {
    pub version: u32,
    pub size: u32,
}

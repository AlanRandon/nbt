use crate::binary::TypeTag;
use crate::snbt::read::convert::Error;
use crate::snbt::read::parse;
use crate::snbt::read::parse::{ListHeaderKind, SpannedInt};
use crate::{Compound, List, ListVariant, Variant};
use std::collections::BTreeMap;

impl<'src> TryFrom<parse::List<'src>> for List {
    type Error = Error<'src>;

    fn try_from(list: parse::List<'src>) -> Result<Self, Self::Error> {
        let type_tag = list.type_tag();
        match type_tag {
            TypeTag::EndCompound => {
                assert!(list.list.is_empty());
                Ok(List::Empty)
            }
            TypeTag::Compound => Ok(List::Compound(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        if let parse::Variant::Compound(compound) = item {
                            compound.try_into()
                        } else {
                            let mut map = BTreeMap::new();
                            map.insert("".to_string(), item.try_into()?);
                            Ok(Compound(map))
                        }
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::Int8 => Ok(List::Int8(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        let parse::Variant::Int(integer) = item else {
                            unreachable!("list of int8s must contain only integers");
                        };

                        integer.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::Int16 => Ok(List::Int16(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        let parse::Variant::Int(integer) = item else {
                            unreachable!("list of int16s must contain only integers");
                        };

                        integer.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::Int32 => Ok(List::Int32(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        let parse::Variant::Int(integer) = item else {
                            unreachable!("list of int32s must contain only integers");
                        };

                        integer.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::Int64 => Ok(List::Int64(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        let parse::Variant::Int(integer) = item else {
                            unreachable!("list of int64s must contain only integers");
                        };

                        integer.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::Float32 => Ok(List::Float32(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        let parse::Variant::Float(float) = item else {
                            unreachable!("list of float32s must contain only floats");
                        };

                        float.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::Float64 => Ok(List::Float64(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        let parse::Variant::Float(float) = item else {
                            unreachable!("list of float64s must contain only floats");
                        };

                        float.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::String => Ok(List::String(ListVariant(
                list.list
                    .into_iter()
                    .map(|string| {
                        let parse::Variant::String(string) = string else {
                            unreachable!("list of strings must contain only strings")
                        };

                        string.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::Int8List => Ok(List::Int8List(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        let parse::Variant::IntList(list) = item else {
                            unreachable!("list of int8 arrays must contain only int arrays")
                        };

                        list.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::Int32List => Ok(List::Int32List(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        let parse::Variant::IntList(list) = item else {
                            unreachable!("list of int32 arrays must contain only int arrays")
                        };

                        list.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::Int64List => Ok(List::Int64List(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        let parse::Variant::IntList(list) = item else {
                            unreachable!("list of int64 arrays must contain only int arrays")
                        };

                        list.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
            TypeTag::List => Ok(List::List(ListVariant(
                list.list
                    .into_iter()
                    .map(|item| {
                        let parse::Variant::List(list) = item else {
                            unreachable!("list of lists must contain only lists");
                        };

                        list.try_into()
                    })
                    .collect::<Result<_, _>>()?,
            ))),
        }
    }
}

impl<'src> TryFrom<parse::IntList<'src>> for Variant {
    type Error = Error<'src>;

    fn try_from(list: parse::IntList<'src>) -> Result<Self, Self::Error> {
        match list.header.kind {
            ListHeaderKind::Int8 => Ok(Variant::Int8List(list.try_into()?)),
            ListHeaderKind::Int32 => Ok(Variant::Int32List(list.try_into()?)),
            ListHeaderKind::Int64 => Ok(Variant::Int64List(list.try_into()?)),
        }
    }
}

impl<'src, T> TryFrom<parse::IntList<'src>> for ListVariant<T>
where
    T: TryFrom<SpannedInt<'src>, Error = Error<'src>>,
{
    type Error = Error<'src>;

    fn try_from(list: parse::IntList<'src>) -> Result<Self, Self::Error> {
        Ok(ListVariant(
            list.list
                .into_iter()
                .map(T::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

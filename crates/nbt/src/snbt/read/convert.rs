use itertools::Itertools;

use crate::binary::TypeTag;
use crate::snbt::read::parse::{ListHeaderKind, OperationKind};
use crate::snbt::read::tokenize::StringContentToken;
use crate::snbt::read::tokenize::number::{FloatType, IntType};
use crate::snbt::read::{Span, parse};
use crate::{Compound, ListVariant, Variant};
use std::collections::{BTreeMap, HashSet};

mod list;
mod number;

#[derive(Debug, thiserror::Error)]
pub enum Error<'src> {
    #[error("compound cannot have duplicate fields")]
    DuplicateField {
        key: String,
        values: Vec<parse::Variant<'src>>,
        compound_span: Span,
    },
    #[error("integers may not be downcast for inclusion in list")]
    DowncastInvalid {
        integer: parse::SpannedInt<'src>,
        result_type: IntType,
    },
    #[error("integer must fit within specified type")]
    IntegerTooLarge {
        integer: parse::SpannedInt<'src>,
        result_type: IntType,
    },
    #[error("integer cannot be negative and unsigned")]
    NegativeUnsignedInteger {
        integer: parse::SpannedInt<'src>,
        result_type: IntType,
    },
    #[error("float must fit within specified type")]
    FloatTooLarge {
        float: parse::SpannedFloat<'src>,
        result_type: FloatType,
    },
    #[error("error parsing arguments for operation")]
    ArgumentError {
        operation_span: Span,
        operation_kind_span: Span,
        error: Box<Error<'src>>,
    },
    #[error("arity error")]
    OperationArityError {
        operation_span: Span,
        operation_kind: OperationKind,
        operation_kind_span: Span,
        expected_arity: usize,
        found_arity: usize,
    },
    #[error("type error")]
    OperationTypeError {
        operation_span: Span,
        operation_kind: OperationKind,
        operation_kind_span: Span,
        arguments: Vec<Variant>,
    },
    #[error("invalid uuid")]
    ParseUuidError {
        operation_span: Span,
        operation_kind_span: Span,
        error: uuid::Error,
    },
}

impl<'src> TryFrom<parse::Variant<'src>> for Variant {
    type Error = Error<'src>;

    fn try_from(variant: parse::Variant<'src>) -> Result<Self, Self::Error> {
        match variant {
            parse::Variant::Int(spanned_int) => spanned_int.try_into(),
            parse::Variant::Float(spanned_float) => spanned_float.try_into(),
            parse::Variant::String(spanned_string) => spanned_string.try_into(),
            parse::Variant::Compound(compound) => compound.try_into().map(Variant::Compound),
            parse::Variant::List(list) => list.try_into().map(Variant::List),
            parse::Variant::IntList(int_list) => int_list.try_into(),
            parse::Variant::Operation(operation) => (*operation).try_into(),
            parse::Variant::Bool(_, value) => Ok(Self::Int8(value.into())),
        }
    }
}

impl<'src> TryFrom<parse::SpannedString<'src>> for String {
    type Error = Error<'src>;

    fn try_from(string: parse::SpannedString<'src>) -> Result<Self, Self::Error> {
        let mut result = String::new();
        for token in string.content {
            match token {
                StringContentToken::Literal(content) => result.push_str(content),
                StringContentToken::Escaped(ch) => result.push(ch),
                StringContentToken::Named(_) => todo!("handle named escapes"),
            }
        }
        Ok(result)
    }
}

impl<'src> TryFrom<parse::SpannedString<'src>> for Variant {
    type Error = Error<'src>;

    fn try_from(string: parse::SpannedString<'src>) -> Result<Self, Self::Error> {
        string.try_into().map(Variant::String)
    }
}

impl<'src> TryFrom<parse::Compound<'src>> for Compound {
    type Error = Error<'src>;

    fn try_from(compound: parse::Compound<'src>) -> Result<Self, Self::Error> {
        let mut parse_fields = BTreeMap::<String, Vec<parse::Variant>>::new();
        for parse::Field { key, value } in compound.fields {
            let key = key.try_into()?;
            parse_fields.entry(key).or_default().push(value);
        }

        let mut fields = BTreeMap::<String, Variant>::new();
        for (key, mut values) in parse_fields {
            if values.len() > 1 {
                return Err(Error::DuplicateField {
                    key,
                    values,
                    compound_span: compound.span,
                });
            }

            fields.insert(key, values.pop().unwrap().try_into()?);
        }

        Ok(Compound(fields))
    }
}

impl<'src> TryFrom<parse::Operation<'src>> for Variant {
    type Error = Error<'src>;

    fn try_from(operation: parse::Operation<'src>) -> Result<Self, Self::Error> {
        let arguments = operation
            .arguments
            .into_iter()
            .map(Variant::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| Error::ArgumentError {
                operation_span: operation.span,
                operation_kind_span: operation.kind_span,
                error: Box::new(error),
            })?;

        match (operation.kind, arguments.get(..)) {
            (OperationKind::Bool | OperationKind::Uuid, Some([argument])) => {
                match (operation.kind, argument) {
                    (OperationKind::Bool, Variant::Int8(integer)) => {
                        Ok(Variant::Int8(if *integer == 0 { 0 } else { 1 }))
                    }
                    (OperationKind::Bool, Variant::Int16(integer)) => {
                        Ok(Variant::Int8(if *integer == 0 { 0 } else { 1 }))
                    }
                    (OperationKind::Bool, Variant::Int32(integer)) => {
                        Ok(Variant::Int8(if *integer == 0 { 0 } else { 1 }))
                    }
                    (OperationKind::Bool, Variant::Int64(integer)) => {
                        Ok(Variant::Int8(if *integer == 0 { 0 } else { 1 }))
                    }
                    (OperationKind::Uuid, Variant::String(string)) => {
                        let uuid = uuid::Uuid::try_parse(string).map_err(|error| {
                            Error::ParseUuidError {
                                operation_span: operation.span,
                                operation_kind_span: operation.kind_span,
                                error,
                            }
                        })?;

                        let integers = uuid
                            .into_bytes()
                            .iter()
                            .chunks(4)
                            .into_iter()
                            .map(|mut chunk| {
                                let array = chunk.next_array().unwrap();
                                assert_eq!(chunk.next(), None);
                                let [a, b, c, d] = array.map(|byte| u32::from(*byte));

                                a * 0x100_0000 + b * 0x1_0000 + c * 0x100 + d
                            })
                            .next_array::<4>()
                            .unwrap();

                        Ok(Variant::Int32List(ListVariant(integers.to_vec())))
                    }
                    _ => Err(Error::OperationTypeError {
                        operation_span: operation.span,
                        operation_kind: operation.kind,
                        operation_kind_span: operation.kind_span,
                        arguments,
                    }),
                }
            }
            _ => Err(Error::OperationArityError {
                operation_span: operation.span,
                operation_kind: operation.kind,
                operation_kind_span: operation.kind_span,
                expected_arity: 1,
                found_arity: arguments.len(),
            }),
        }
    }
}

impl<'src> parse::Variant<'src> {
    fn type_tag(&self) -> TypeTag {
        match self {
            parse::Variant::Int(integer) => match integer.integer.r#type {
                IntType::Int8 => TypeTag::Int8,
                IntType::Int16 => TypeTag::Int16,
                IntType::Int32 => TypeTag::Int32,
                IntType::Int64 => TypeTag::Int64,
            },
            parse::Variant::Float(float) => match float.float.r#type {
                FloatType::Float32 => TypeTag::Float32,
                FloatType::Float64 => TypeTag::Float64,
            },
            parse::Variant::String(_) => TypeTag::String,
            parse::Variant::Compound(_) => TypeTag::Compound,
            parse::Variant::IntList(int_list) => match int_list.header.kind {
                ListHeaderKind::Int8 => TypeTag::Int8List,
                ListHeaderKind::Int32 => TypeTag::Int32List,
                ListHeaderKind::Int64 => TypeTag::Int64List,
            },
            parse::Variant::Operation(operation) => match operation.kind {
                OperationKind::Uuid => TypeTag::Int32List,
                OperationKind::Bool => TypeTag::Int8,
            },
            parse::Variant::Bool(_, _) => TypeTag::Int8,
            parse::Variant::List(_) => TypeTag::List,
        }
    }
}

impl<'src> parse::List<'src> {
    fn type_tag(&self) -> TypeTag {
        let mut type_set = HashSet::new();
        for item in &self.list {
            type_set.insert(item.type_tag());
            if type_set.len() >= 2 {
                return TypeTag::Compound;
            }
        }

        if type_set.is_empty() {
            TypeTag::EndCompound
        } else {
            type_set.iter().next().unwrap().clone()
        }
    }
}

#[test]
fn convert_from_syntax_tree() {
    use crate::snbt::read::SourcePosition;
    use crate::snbt::read::parse::{IntList, ListHeader, ListHeaderKind};
    use crate::snbt::read::tokenize::number::{Int, IntBytes, IntType, Sign, Signedness};
    use crate::{List, ListVariant};

    let span = Span(std::range::Range {
        start: SourcePosition(0),
        end: SourcePosition(0),
    });

    let variant = parse::Variant::Compound(parse::Compound {
        span,
        fields: vec![
            parse::Field {
                key: parse::SpannedString {
                    span,
                    content: vec![StringContentToken::Literal("true")],
                },
                value: parse::Variant::Bool(span, true),
            },
            parse::Field {
                key: parse::SpannedString {
                    span,
                    content: vec![StringContentToken::Literal("false")],
                },
                value: parse::Variant::Bool(span, false),
            },
            parse::Field {
                key: parse::SpannedString {
                    span,
                    content: vec![StringContentToken::Literal("empty_list")],
                },
                value: parse::Variant::List(parse::List { span, list: vec![] }),
            },
            parse::Field {
                key: parse::SpannedString {
                    span,
                    content: vec![StringContentToken::Literal("list")],
                },
                value: parse::Variant::List(parse::List {
                    span,
                    list: vec![
                        parse::Variant::List(parse::List { span, list: vec![] }),
                        parse::Variant::List(parse::List {
                            span,
                            list: vec![parse::Variant::Compound(parse::Compound {
                                span,
                                fields: vec![],
                            })],
                        }),
                    ],
                }),
            },
            parse::Field {
                key: parse::SpannedString {
                    span,
                    content: vec![StringContentToken::Literal("int8_list")],
                },
                value: parse::Variant::List(parse::List {
                    span,
                    list: vec![parse::Variant::Int(parse::SpannedInt {
                        span,
                        integer: Int {
                            sign: Sign::Positive,
                            signedness: Signedness::Signed,
                            r#type: IntType::Int8,
                            digits: IntBytes::Hex(b"67"),
                        },
                    })],
                }),
            },
            parse::Field {
                key: parse::SpannedString {
                    span,
                    content: vec![StringContentToken::Literal("int64_list")],
                },
                value: parse::Variant::List(parse::List {
                    span,
                    list: vec![parse::Variant::Int(parse::SpannedInt {
                        span,
                        integer: Int {
                            sign: Sign::Negative,
                            signedness: Signedness::Signed,
                            r#type: IntType::Int64,
                            digits: IntBytes::Denary(b"420"),
                        },
                    })],
                }),
            },
            parse::Field {
                key: parse::SpannedString {
                    span,
                    content: vec![StringContentToken::Literal("int_array_list")],
                },
                value: parse::Variant::List(parse::List {
                    span,
                    list: vec![parse::Variant::IntList(IntList {
                        span,
                        header: ListHeader {
                            span,
                            kind: ListHeaderKind::Int32,
                        },
                        list: vec![
                            parse::SpannedInt {
                                span,
                                integer: Int {
                                    sign: Sign::Negative,
                                    signedness: Signedness::Signed,
                                    r#type: IntType::Int32,
                                    digits: IntBytes::Binary(b"010"),
                                },
                            },
                            parse::SpannedInt {
                                span,
                                integer: Int {
                                    sign: Sign::Positive,
                                    signedness: Signedness::Unsigned,
                                    r#type: IntType::Int8,
                                    digits: IntBytes::Binary(b"1010"),
                                },
                            },
                        ],
                    })],
                }),
            },
        ],
    });

    assert_eq!(
        Variant::try_from(variant).unwrap(),
        Variant::Compound(Compound(BTreeMap::from_iter(
            [
                ("true".to_string(), Variant::Int8(1)),
                ("false".to_string(), Variant::Int8(0)),
                ("empty_list".to_string(), Variant::List(List::Empty)),
                (
                    "list".to_string(),
                    Variant::List(List::List(ListVariant(vec![
                        List::Empty,
                        List::Compound(ListVariant(vec![Compound(BTreeMap::new())]))
                    ])))
                ),
                (
                    "int8_list".to_string(),
                    Variant::List(List::Int8(ListVariant(vec![0x67])))
                ),
                (
                    "int64_list".to_string(),
                    Variant::List(List::Int64(ListVariant(vec![(-420i64).cast_unsigned()])))
                ),
                (
                    "int_array_list".to_string(),
                    Variant::List(List::Int32List(ListVariant(vec![ListVariant(vec![
                        (-0b010i32).cast_unsigned(),
                        0b1010,
                    ])])))
                ),
            ]
            .into_iter()
        )))
    )
}

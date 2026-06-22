pub mod convert;
pub mod parse;
pub mod tokenize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span(std::range::Range<SourcePosition>);

#[test]
fn parse_and_convert_variant() {
    use crate::{ListVariant, Variant};

    for (source, expected_variant) in [
        ("0xDEADBEEFL", Variant::Int64(0xDEADBEEF)),
        ("0f", Variant::Float32(0.)),
        ("-42e10f", Variant::Float32(-42e10)),
        (
            "\"Hello\\tworld!\"",
            Variant::String("Hello\tworld!".to_string()),
        ),
        ("bool(1L)", Variant::Int8(1)),
        (
            "uuid(\"f81d4fae-7dec-11d0-a765-00a0c91e6bf6\")",
            Variant::Int32List(ListVariant(vec![
                (-132296786i32).cast_unsigned(),
                2112623056,
                (-1486552928i32).cast_unsigned(),
                (-920753162i32).cast_unsigned(),
            ])),
        ),
    ] {
        let parser = parse::Parser::new(source);
        let parser_variant = parser.parse_variant_and_finish().unwrap();
        let variant = Variant::try_from(parser_variant).unwrap();
        assert_eq!(variant, expected_variant)
    }
}

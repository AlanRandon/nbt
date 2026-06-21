pub mod convert;
pub mod parse;
pub mod tokenize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span(std::range::Range<SourcePosition>);

#[test]
fn parse_and_convert_variant() {
    use crate::Variant;

    for (source, expected_variant) in [
        ("0xDEADBEEFL", Variant::Int64(0xDEADBEEF)),
        ("-42e10f", Variant::Float32(-42e10)),
        (
            "\"Hello\\tworld!\"",
            Variant::String("Hello\tworld!".to_string()),
        ),
        ("bool(1L)", Variant::Int8(1)),
    ] {
        let parser = parse::Parser::new(source);
        let parser_variant = parser.parse_variant_and_finish().unwrap();
        let variant = Variant::try_from(parser_variant).unwrap();
        assert_eq!(variant, expected_variant)
    }
}

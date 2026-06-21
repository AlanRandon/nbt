use crate::snbt::read::SourcePosition;
use std::assert_matches;

impl<'src> super::Tokenizer<'src> {
    pub fn take_number_token(&mut self) -> Result<super::Token<'src>, super::Error> {
        let source = self.source.as_bytes().get(self.position..).unwrap();

        let mut number_parser = NumberParser::new(source);
        let number = number_parser
            .take_number()
            .map_err(|_| super::Error::InvalidNumber(SourcePosition(self.position)))?;
        self.position += number_parser.position;

        Ok(super::Token::Number(number))
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("number was invalid")]
pub struct Error;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Int<'src> {
    pub sign: Sign,
    pub signedness: Signedness,
    pub r#type: IntType,
    pub digits: IntBytes<'src>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum IntBytes<'src> {
    Denary(&'src [u8]),
    Binary(&'src [u8]),
    Hex(&'src [u8]),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Number<'src> {
    Int(Int<'src>),
    Float(Float<'src>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Float<'src> {
    pub sign: Sign,
    pub r#type: FloatType,
    pub integer_part: Option<&'src [u8]>,
    pub fractional_part: Option<&'src [u8]>,
    pub exponent_part: Option<(Sign, &'src [u8])>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Signedness {
    Unsigned,
    Signed,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum IntType {
    Int8,
    Int16,
    Int32,
    Int64,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum FloatType {
    Float32,
    Float64,
}

pub struct NumberParser<'src> {
    position: usize,
    source: &'src [u8],
}

impl<'src> NumberParser<'src> {
    pub fn new(source: &'src [u8]) -> Self {
        Self {
            position: 0,
            source,
        }
    }

    fn take_number(&mut self) -> Result<Number<'src>, Error> {
        let sign = self.take_sign();

        if matches!(
            self.source.get(self.position..=self.position + 2),
            Some([b'0', b'b', b'0' | b'1'])
        ) {
            let digits = self.take_binary_sequence();
            let (signedness, r#type) = self.take_int_type();
            return Ok(Number::Int(Int {
                sign,
                r#type,
                signedness,
                digits: IntBytes::Binary(digits),
            }));
        }

        if matches!(
            self.source.get(self.position..=self.position + 2),
            Some([b'0', b'x', b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'])
        ) {
            let digits = self.take_hex_sequence();
            let (signedness, r#type) = self.take_int_type();
            return Ok(Number::Int(Int {
                sign,
                r#type,
                signedness,
                digits: IntBytes::Hex(digits),
            }));
        }

        if matches!(self.source.get(self.position), Some(b'0'..=b'9')) {
            let mut is_float = false;

            let integer_part = self.take_denary_sequence();

            let fractional_part = if self.source.get(self.position) == Some(&b'.') {
                is_float = true;
                self.position += 1;
                matches!(self.source.get(self.position), Some(b'0'..=b'9'))
                    .then(|| self.take_denary_sequence())
            } else {
                None
            };

            let exponent_part = self.take_optional_exponent_part();
            is_float |= exponent_part.is_some();

            if is_float {
                let r#type = self.take_float_type();
                return Ok(Number::Float(Float {
                    sign,
                    r#type,
                    integer_part: Some(integer_part),
                    fractional_part,
                    exponent_part,
                }));
            } else {
                let (signedness, r#type) = self.take_int_type();
                return Ok(Number::Int(Int {
                    sign,
                    r#type,
                    signedness,
                    digits: IntBytes::Denary(integer_part),
                }));
            }
        }

        if matches!(
            self.source.get(self.position..=self.position + 1),
            Some([b'.', b'0'..=b'9'])
        ) {
            self.position += 1;

            let fractional_part = self.take_denary_sequence();
            let exponent_part = self.take_optional_exponent_part();
            let r#type = self.take_float_type();
            return Ok(Number::Float(Float {
                sign,
                r#type,
                integer_part: None,
                fractional_part: Some(fractional_part),
                exponent_part,
            }));
        }

        Err(Error)
    }

    fn take_sign(&mut self) -> Sign {
        if self.source.get(self.position) == Some(&b'-') {
            self.position += 1;
            Sign::Negative
        } else {
            Sign::Positive
        }
    }

    fn take_int_type(&mut self) -> (Signedness, IntType) {
        if let Some([signedness, ty]) = self.source.get(self.position..=self.position + 1) {
            let signedness = match signedness {
                b's' | b'S' => Some(Signedness::Signed),
                b'u' | b'U' => Some(Signedness::Unsigned),
                _ => None,
            };

            let ty = match ty {
                b'b' | b'B' => Some(IntType::Int8),
                b's' | b'S' => Some(IntType::Int16),
                b'i' | b'I' => Some(IntType::Int32),
                b'l' | b'L' => Some(IntType::Int64),
                _ => None,
            };

            if let (Some(signedness), Some(ty)) = (signedness, ty) {
                self.position += 2;
                return (signedness, ty);
            }
        }

        match self.source.get(self.position).and_then(|byte| match byte {
            b'b' | b'B' => Some(IntType::Int8),
            b's' | b'S' => Some(IntType::Int16),
            b'i' | b'I' => Some(IntType::Int32),
            b'l' | b'L' => Some(IntType::Int64),
            _ => None,
        }) {
            Some(ty) => {
                self.position += 1;
                (Signedness::Signed, ty)
            }
            None => (Signedness::Signed, IntType::Int32),
        }
    }

    fn take_float_type(&mut self) -> FloatType {
        match self.source.get(self.position).and_then(|byte| match byte {
            b'f' | b'F' => Some(FloatType::Float32),
            b'd' | b'D' => Some(FloatType::Float64),
            _ => None,
        }) {
            Some(ty) => {
                self.position += 1;
                ty
            }
            None => FloatType::Float64,
        }
    }

    fn take_optional_exponent_part(&mut self) -> Option<(Sign, &'src [u8])> {
        if matches!(
            self.source.get(self.position..=self.position + 1),
            Some([b'e' | b'E', b'0'..=b'9'])
        ) {
            self.position += 1;
            Some((Sign::Positive, self.take_denary_sequence()))
        } else if matches!(
            self.source.get(self.position..=self.position + 2),
            Some([b'e' | b'E', b'-', b'0'..=b'9'])
        ) {
            self.position += 2;
            Some((Sign::Negative, self.take_denary_sequence()))
        } else {
            None
        }
    }

    pub fn take_denary_sequence(&mut self) -> &'src [u8] {
        let source = self.source.get(self.position..).unwrap();
        assert_matches!(source.first(), Some(b'0'..=b'9'));

        let length = source
            .iter()
            .take_while(|byte| matches!(byte, b'_' | b'0'..=b'9'))
            .count();

        let seq = source.get(..length).unwrap();
        let seq = if seq.last() == Some(&b'_') {
            self.position += length - 1;
            seq.get(..length - 1).unwrap()
        } else {
            self.position += length;
            seq
        };

        seq
    }

    pub fn take_binary_sequence(&mut self) -> &'src [u8] {
        let source = self.source.get(self.position..).unwrap();
        assert_matches!(source.get(0..=2), Some([b'0', b'b', b'0' | b'1']));

        let length = source
            .iter()
            .skip(2)
            .take_while(|byte| matches!(byte, b'0' | b'1' | b'_'))
            .count();

        let seq = source.get(2..).unwrap().get(..length).unwrap();
        let seq = if seq.last() == Some(&b'_') {
            self.position += length + 1;
            seq.get(..length - 1).unwrap()
        } else {
            self.position += length + 2;
            seq
        };

        seq
    }

    pub fn take_hex_sequence(&mut self) -> &'src [u8] {
        let source = self.source.get(self.position..).unwrap();
        assert_matches!(
            source.get(0..=2),
            Some([b'0', b'x', b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'])
        );

        let length = source
            .iter()
            .skip(2)
            .take_while(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' | b'_'))
            .count();

        let seq = source.get(2..).unwrap().get(..length).unwrap();
        let seq = if seq.last() == Some(&b'_') {
            self.position += length + 1;
            seq.get(..length - 1).unwrap()
        } else {
            self.position += length + 2;
            seq
        };

        seq
    }
}

#[test]
fn take_hex_digits() {
    let mut parser = NumberParser::new(b"0xD_EAD_-abcd");
    assert_eq!(parser.take_hex_sequence(), b"D_EAD");
}

#[test]
fn take_binary_digits() {
    let mut parser = NumberParser::new(b"0b0_1_0_abc");
    assert_eq!(parser.take_binary_sequence(), b"0_1_0");
}

#[test]
fn take_denary_digits() {
    let mut parser = NumberParser::new(b"6_42_7_");
    assert_eq!(parser.take_denary_sequence(), b"6_42_7");
}

#[test]
fn take_number() {
    for (source, expected) in [
        (
            b".1" as &[u8],
            Number::Float(Float {
                sign: Sign::Positive,
                r#type: FloatType::Float64,
                integer_part: None,
                fractional_part: Some(b"1"),
                exponent_part: None,
            }),
        ),
        (
            b"1.",
            Number::Float(Float {
                sign: Sign::Positive,
                r#type: FloatType::Float64,
                integer_part: Some(b"1"),
                fractional_part: None,
                exponent_part: None,
            }),
        ),
        (
            b"1.2e3",
            Number::Float(Float {
                sign: Sign::Positive,
                r#type: FloatType::Float64,
                integer_part: Some(b"1"),
                fractional_part: Some(b"2"),
                exponent_part: Some((Sign::Positive, b"3")),
            }),
        ),
        (
            b"87E48",
            Number::Float(Float {
                sign: Sign::Positive,
                r#type: FloatType::Float64,
                integer_part: Some(b"87"),
                fractional_part: None,
                exponent_part: Some((Sign::Positive, b"48")),
            }),
        ),
        (
            b"0.1e-1",
            Number::Float(Float {
                sign: Sign::Positive,
                r#type: FloatType::Float64,
                integer_part: Some(b"0"),
                fractional_part: Some(b"1"),
                exponent_part: Some((Sign::Negative, b"1")),
            }),
        ),
        (
            b"0xbad",
            Number::Int(Int {
                sign: Sign::Positive,
                r#type: IntType::Int32,
                signedness: Signedness::Signed,
                digits: IntBytes::Hex(b"bad"),
            }),
        ),
        (
            b"0xCAFE",
            Number::Int(Int {
                sign: Sign::Positive,
                r#type: IntType::Int32,
                signedness: Signedness::Signed,
                digits: IntBytes::Hex(b"CAFE"),
            }),
        ),
        (
            b"0b101",
            Number::Int(Int {
                sign: Sign::Positive,
                r#type: IntType::Int32,
                signedness: Signedness::Signed,
                digits: IntBytes::Binary(b"101"),
            }),
        ),
        (
            b"0b10_01",
            Number::Int(Int {
                sign: Sign::Positive,
                r#type: IntType::Int32,
                signedness: Signedness::Signed,
                digits: IntBytes::Binary(b"10_01"),
            }),
        ),
        (
            b"0xAB_CD",
            Number::Int(Int {
                sign: Sign::Positive,
                r#type: IntType::Int32,
                signedness: Signedness::Signed,
                digits: IntBytes::Hex(b"AB_CD"),
            }),
        ),
        (
            b"1_2.3_4__5f",
            Number::Float(Float {
                sign: Sign::Positive,
                r#type: FloatType::Float32,
                integer_part: Some(b"1_2"),
                fractional_part: Some(b"3_4__5"),
                exponent_part: None,
            }),
        ),
        (
            b"1_2e3_4",
            Number::Float(Float {
                sign: Sign::Positive,
                r#type: FloatType::Float64,
                integer_part: Some(b"1_2"),
                fractional_part: None,
                exponent_part: Some((Sign::Positive, b"3_4")),
            }),
        ),
        (
            b"-16b",
            Number::Int(Int {
                sign: Sign::Negative,
                r#type: IntType::Int8,
                signedness: Signedness::Signed,
                digits: IntBytes::Denary(b"16"),
            }),
        ),
        (
            b"-16sb",
            Number::Int(Int {
                sign: Sign::Negative,
                r#type: IntType::Int8,
                signedness: Signedness::Signed,
                digits: IntBytes::Denary(b"16"),
            }),
        ),
        (
            b"240uB",
            Number::Int(Int {
                sign: Sign::Positive,
                r#type: IntType::Int8,
                signedness: Signedness::Unsigned,
                digits: IntBytes::Denary(b"240"),
            }),
        ),
        (
            b"15s",
            Number::Int(Int {
                sign: Sign::Positive,
                r#type: IntType::Int16,
                signedness: Signedness::Signed,
                digits: IntBytes::Denary(b"15"),
            }),
        ),
        (
            b"15sS",
            Number::Int(Int {
                sign: Sign::Positive,
                r#type: IntType::Int16,
                signedness: Signedness::Signed,
                digits: IntBytes::Denary(b"15"),
            }),
        ),
        (
            b"15Us",
            Number::Int(Int {
                sign: Sign::Positive,
                r#type: IntType::Int16,
                signedness: Signedness::Unsigned,
                digits: IntBytes::Denary(b"15"),
            }),
        ),
    ] {
        let mut parser = NumberParser::new(source);
        assert_eq!(parser.take_number().unwrap(), expected);
    }
}

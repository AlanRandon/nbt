use crate::Variant;
use crate::snbt::read::convert::Error;
use crate::snbt::read::parse;
use crate::snbt::read::tokenize::number::{FloatType, IntBytes, IntType, Sign, Signedness};

impl<'src> TryFrom<parse::SpannedInt<'src>> for Variant {
    type Error = Error<'src>;

    fn try_from(integer: parse::SpannedInt<'src>) -> Result<Self, Self::Error> {
        match integer.integer.r#type {
            IntType::Int8 => Ok(Variant::Int8(integer.try_into()?)),
            IntType::Int16 => Ok(Variant::Int16(integer.try_into()?)),
            IntType::Int32 => Ok(Variant::Int32(integer.try_into()?)),
            IntType::Int64 => Ok(Variant::Int64(integer.try_into()?)),
        }
    }
}

impl<'src> TryFrom<parse::SpannedInt<'src>> for u8 {
    type Error = Error<'src>;

    fn try_from(integer: parse::SpannedInt<'src>) -> Result<Self, Self::Error> {
        match integer.integer.r#type {
            IntType::Int8 => {}
            IntType::Int16 | IntType::Int32 | IntType::Int64 => {
                return Err(Error::DowncastInvalid {
                    integer,
                    result_type: IntType::Int8,
                });
            }
        }

        let value = integer
            .integer
            .digits
            .to_i128()
            .ok()
            .and_then(|value: i128| Self::try_from(value).ok())
            .and_then(
                |value| match (integer.integer.signedness, integer.integer.sign) {
                    (Signedness::Signed, Sign::Positive) => {
                        Some(Some(i8::try_from(value).ok()?.cast_unsigned()))
                    }
                    (Signedness::Signed, Sign::Negative) => Some(Some(
                        i8::try_from(value).ok()?.checked_neg()?.cast_unsigned(),
                    )),
                    (Signedness::Unsigned, Sign::Positive) => Some(Some(value)),
                    (Signedness::Unsigned, Sign::Negative) => Some(None),
                },
            );

        match value {
            Some(Some(value)) => Ok(value),
            None => Err(Error::IntegerTooLarge {
                integer,
                result_type: IntType::Int8,
            }),
            Some(None) => Err(Error::NegativeUnsignedInteger {
                integer,
                result_type: IntType::Int8,
            }),
        }
    }
}

impl<'src> TryFrom<parse::SpannedInt<'src>> for u16 {
    type Error = Error<'src>;

    fn try_from(integer: parse::SpannedInt<'src>) -> Result<Self, Self::Error> {
        match integer.integer.r#type {
            IntType::Int8 | IntType::Int16 => {}
            IntType::Int32 | IntType::Int64 => {
                return Err(Error::DowncastInvalid {
                    integer,
                    result_type: IntType::Int16,
                });
            }
        }

        let value = integer
            .integer
            .digits
            .to_i128()
            .ok()
            .and_then(|value: i128| Self::try_from(value).ok())
            .and_then(
                |value| match (integer.integer.signedness, integer.integer.sign) {
                    (Signedness::Signed, Sign::Positive) => {
                        Some(Some(i16::try_from(value).ok()?.cast_unsigned()))
                    }
                    (Signedness::Signed, Sign::Negative) => Some(Some(
                        i16::try_from(value).ok()?.checked_neg()?.cast_unsigned(),
                    )),
                    (Signedness::Unsigned, Sign::Positive) => Some(Some(value)),
                    (Signedness::Unsigned, Sign::Negative) => Some(None),
                },
            );

        match value {
            Some(Some(value)) => Ok(value),
            None => Err(Error::IntegerTooLarge {
                integer,
                result_type: IntType::Int16,
            }),
            Some(None) => Err(Error::NegativeUnsignedInteger {
                integer,
                result_type: IntType::Int16,
            }),
        }
    }
}

impl<'src> TryFrom<parse::SpannedInt<'src>> for u32 {
    type Error = Error<'src>;

    fn try_from(integer: parse::SpannedInt<'src>) -> Result<Self, Self::Error> {
        match integer.integer.r#type {
            IntType::Int8 | IntType::Int16 | IntType::Int32 => {}
            IntType::Int64 => {
                return Err(Error::DowncastInvalid {
                    integer,
                    result_type: IntType::Int32,
                });
            }
        }

        let value = integer
            .integer
            .digits
            .to_i128()
            .ok()
            .and_then(|value: i128| Self::try_from(value).ok())
            .and_then(
                |value| match (integer.integer.signedness, integer.integer.sign) {
                    (Signedness::Signed, Sign::Positive) => {
                        Some(Some(i32::try_from(value).ok()?.cast_unsigned()))
                    }
                    (Signedness::Signed, Sign::Negative) => Some(Some(
                        i32::try_from(value).ok()?.checked_neg()?.cast_unsigned(),
                    )),
                    (Signedness::Unsigned, Sign::Positive) => Some(Some(value)),
                    (Signedness::Unsigned, Sign::Negative) => Some(None),
                },
            );

        match value {
            Some(Some(value)) => Ok(value),
            None => Err(Error::IntegerTooLarge {
                integer,
                result_type: IntType::Int32,
            }),
            Some(None) => Err(Error::NegativeUnsignedInteger {
                integer,
                result_type: IntType::Int32,
            }),
        }
    }
}

impl<'src> TryFrom<parse::SpannedInt<'src>> for u64 {
    type Error = Error<'src>;

    fn try_from(integer: parse::SpannedInt<'src>) -> Result<Self, Self::Error> {
        let value = integer
            .integer
            .digits
            .to_i128()
            .ok()
            .and_then(|value: i128| Self::try_from(value).ok())
            .and_then(
                |value| match (integer.integer.signedness, integer.integer.sign) {
                    (Signedness::Signed, Sign::Positive) => {
                        Some(Some(i64::try_from(value).ok()?.cast_unsigned()))
                    }
                    (Signedness::Signed, Sign::Negative) => Some(Some(
                        i64::try_from(value).ok()?.checked_neg()?.cast_unsigned(),
                    )),
                    (Signedness::Unsigned, Sign::Positive) => Some(Some(value)),
                    (Signedness::Unsigned, Sign::Negative) => Some(None),
                },
            );

        match value {
            Some(Some(value)) => Ok(value),
            None => Err(Error::IntegerTooLarge {
                integer,
                result_type: IntType::Int64,
            }),
            Some(None) => Err(Error::NegativeUnsignedInteger {
                integer,
                result_type: IntType::Int64,
            }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("the number type was too large")]
pub struct TooLargeError;

impl<'src> IntBytes<'src> {
    fn to_i128(&self) -> Result<i128, TooLargeError> {
        match self {
            IntBytes::Denary(bytes) => bytes
                .iter()
                .rev()
                .filter(|byte| **byte != b'_')
                .map(|byte| byte - b'0')
                .enumerate()
                .fold(Some(0i128), |result, (place_value, digit)| {
                    let result = result?;
                    let digit = i128::from(digit);
                    let place_value = u32::try_from(place_value).ok()?;
                    let multiplier = 10i128.checked_pow(place_value)?;
                    let value = digit.checked_mul(multiplier)?;
                    result.checked_add(value)
                }),
            IntBytes::Binary(bytes) => bytes
                .iter()
                .rev()
                .filter(|byte| **byte != b'_')
                .map(|byte| byte - b'0')
                .enumerate()
                .fold(Some(0i128), |result, (place_value, digit)| {
                    let result = result?;
                    let digit = i128::from(digit);
                    let place_value = u32::try_from(place_value).ok()?;
                    let value = digit.checked_shl(place_value)?;
                    Some(result | value)
                }),
            IntBytes::Hex(bytes) => bytes
                .iter()
                .rev()
                .filter(|byte| **byte != b'_')
                .map(|byte| match byte {
                    b'0'..=b'9' => byte - b'0',
                    b'a'..=b'f' => byte - b'a' + 0xa,
                    b'A'..=b'F' => byte - b'A' + 0xa,
                    _ => unreachable!("hex bytes must not contain non-hex digit"),
                })
                .enumerate()
                .fold(Some(0i128), |result, (place_value, digit)| {
                    let result = result?;
                    let digit = i128::from(digit);
                    let place_value = u32::try_from(place_value).ok()?;
                    let value = digit.checked_shl(place_value.checked_mul(4)?)?;
                    Some(result | value)
                }),
        }
        .ok_or_else(|| TooLargeError)
    }
}

impl<'src> TryFrom<parse::SpannedFloat<'src>> for Variant {
    type Error = Error<'src>;

    fn try_from(float: parse::SpannedFloat<'src>) -> Result<Self, Self::Error> {
        match float.float.r#type {
            FloatType::Float32 => float.try_into().map(Variant::Float32),
            FloatType::Float64 => float.try_into().map(Variant::Float64),
        }
    }
}
impl<'src> TryFrom<parse::SpannedFloat<'src>> for f32 {
    type Error = Error<'src>;

    fn try_from(float: parse::SpannedFloat<'src>) -> Result<Self, Self::Error> {
        let integer_part = if let Some(integer_part) = float.float.integer_part {
            match integer_part
                .iter()
                .rev()
                .filter(|byte| **byte != b'_')
                .map(|byte| byte - b'0')
                .enumerate()
                .fold(Some(0f32), |result, (place_value, digit)| {
                    let result = result?;
                    let digit = f32::from(digit);
                    let place_value = i32::try_from(place_value).ok()?;
                    let value = digit * 10f32.powi(place_value);
                    Some(result + value)
                }) {
                Some(integer_part) => integer_part,
                None => {
                    return Err(Error::FloatTooLarge {
                        float,
                        result_type: FloatType::Float32,
                    });
                }
            }
        } else {
            0.
        };

        let fractional_part = if let Some(fractional_part) = float.float.fractional_part {
            match fractional_part
                .iter()
                .filter(|byte| **byte != b'_')
                .map(|byte| byte - b'0')
                .enumerate()
                .fold(Some(0f32), |result, (place_value, digit)| {
                    let result = result?;
                    let digit = f32::from(digit);
                    let place_value = i32::try_from(place_value)
                        .ok()?
                        .checked_add(1)?
                        .checked_neg()?;
                    let value = digit * 10f32.powi(place_value);
                    Some(result + value)
                }) {
                Some(fractional_part) => fractional_part,
                None => {
                    return Err(Error::FloatTooLarge {
                        float,
                        result_type: FloatType::Float32,
                    });
                }
            }
        } else {
            0.
        };

        let exponent_part = if let Some((exponent_sign, exponent_part)) = float.float.exponent_part
        {
            match exponent_part
                .iter()
                .rev()
                .filter(|byte| **byte != b'_')
                .map(|byte| byte - b'0')
                .enumerate()
                .fold(Some(0f32), |result, (place_value, digit)| {
                    let result = result?;
                    let digit = f32::from(digit);
                    let place_value = i32::try_from(place_value).ok()?;
                    let value = digit * 10f32.powi(place_value);
                    Some(result + value)
                }) {
                Some(exponent_part) => match exponent_sign {
                    Sign::Positive => exponent_part,
                    Sign::Negative => -exponent_part,
                },
                None => {
                    return Err(Error::FloatTooLarge {
                        float,
                        result_type: FloatType::Float32,
                    });
                }
            }
        } else {
            0.
        };

        let value = (integer_part + fractional_part) * 10f32.powf(exponent_part);
        let value = match float.float.sign {
            Sign::Positive => value,
            Sign::Negative => -value,
        };

        Ok(value)
    }
}

impl<'src> TryFrom<parse::SpannedFloat<'src>> for f64 {
    type Error = Error<'src>;

    fn try_from(float: parse::SpannedFloat<'src>) -> Result<Self, Self::Error> {
        let integer_part = if let Some(integer_part) = float.float.integer_part {
            match integer_part
                .iter()
                .rev()
                .filter(|byte| **byte != b'_')
                .map(|byte| byte - b'0')
                .enumerate()
                .fold(Some(0f64), |result, (place_value, digit)| {
                    let result = result?;
                    let digit = f64::from(digit);
                    let place_value = i32::try_from(place_value).ok()?;
                    let value = digit * 10f64.powi(place_value);
                    Some(result + value)
                }) {
                Some(integer_part) => integer_part,
                None => {
                    return Err(Error::FloatTooLarge {
                        float,
                        result_type: FloatType::Float64,
                    });
                }
            }
        } else {
            0.
        };

        let fractional_part = if let Some(fractional_part) = float.float.fractional_part {
            match fractional_part
                .iter()
                .filter(|byte| **byte != b'_')
                .map(|byte| byte - b'0')
                .enumerate()
                .fold(Some(0f64), |result, (place_value, digit)| {
                    let result = result?;
                    let digit = f64::from(digit);
                    let place_value = i32::try_from(place_value)
                        .ok()?
                        .checked_add(1)?
                        .checked_neg()?;
                    let value = digit * 10f64.powi(place_value);
                    Some(result + value)
                }) {
                Some(fractional_part) => fractional_part,
                None => {
                    return Err(Error::FloatTooLarge {
                        float,
                        result_type: FloatType::Float64,
                    });
                }
            }
        } else {
            0.
        };

        let exponent_part = if let Some((exponent_sign, exponent_part)) = float.float.exponent_part
        {
            match exponent_part
                .iter()
                .rev()
                .filter(|byte| **byte != b'_')
                .map(|byte| byte - b'0')
                .enumerate()
                .fold(Some(0f64), |result, (place_value, digit)| {
                    let result = result?;
                    let digit = f64::from(digit);
                    let place_value = i32::try_from(place_value).ok()?;
                    let value = digit * 10f64.powi(place_value);
                    Some(result + value)
                }) {
                Some(exponent_part) => match exponent_sign {
                    Sign::Positive => exponent_part,
                    Sign::Negative => -exponent_part,
                },
                None => {
                    return Err(Error::FloatTooLarge {
                        float,
                        result_type: FloatType::Float64,
                    });
                }
            }
        } else {
            0.
        };

        let value = (integer_part + fractional_part) * 10f64.powf(exponent_part);
        let value = match float.float.sign {
            Sign::Positive => value,
            Sign::Negative => -value,
        };

        Ok(value)
    }
}

#[test]
fn convert_integer() {
    use crate::snbt::read::tokenize::number::Int;
    use crate::snbt::read::{SourcePosition, Span};

    let span = Span(std::range::Range {
        start: SourcePosition(0),
        end: SourcePosition(0),
    });

    assert_eq!(
        u32::try_from(parse::SpannedInt {
            span,
            integer: Int {
                sign: Sign::Positive,
                signedness: Signedness::Unsigned,
                r#type: IntType::Int32,
                digits: IntBytes::Binary(b"1010")
            }
        })
        .unwrap(),
        0b1010
    );
}

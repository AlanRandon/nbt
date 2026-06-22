use crate::snbt::read::{SourcePosition, Span};
use std::char::CharTryFromError;
use std::str::Utf8Error;

pub mod number;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token<'src> {
    /// End of File
    Eof,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `B;`
    Int8ArrayHeader,
    /// `I;`
    Int32ArrayHeader,
    /// `L;`
    Int64ArrayHeader,
    /// `true`
    True,
    /// `false`
    False,
    /// `bool`
    Bool,
    /// `uuid`
    Uuid,
    /// `"<string-content>"`
    String(Vec<StringContentToken<'src>>),
    /// `<number>`
    Number(number::Number<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken<'src> {
    pub token: Token<'src>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum StringContentToken<'src> {
    Literal(&'src str),
    Escaped(char),
    Named(&'src str),
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'src> {
    position: usize,
    source: &'src str,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid token at {0:?}")]
    InvalidToken(SourcePosition),
    #[error("unclosed string escape at {0:?}")]
    UnclosedEscape(SourcePosition),
    #[error("unknown string escape at {0:?}")]
    UnknownEscape(SourcePosition),
    #[error("escape contains non-digits at {0:?}")]
    EscapeNonDigits(SourcePosition),
    #[error("escape contains invalid character at {0:?}")]
    InvalidEscapeChar(SourcePosition, CharTryFromError),
    #[error("invalid number at {0:?}")]
    InvalidNumber(SourcePosition),
    #[error("invalid utf-8 in string at {0:?}")]
    InvalidUtf8(SourcePosition, Utf8Error),
}

impl<'src> Tokenizer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            position: 0,
            source,
        }
    }

    pub fn take_spanned_token(&mut self) -> Result<SpannedToken<'src>, Error> {
        let Some(byte) = self.source.as_bytes().get(self.position) else {
            return Ok(SpannedToken {
                token: Token::Eof,
                span: Span(std::range::Range {
                    start: SourcePosition(self.position),
                    end: SourcePosition(self.position),
                }),
            });
        };

        if matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
            self.position += 1;
            return self.take_spanned_token();
        }

        let start = SourcePosition(self.position);
        let token = self.take_token()?;
        let end = SourcePosition(self.position);

        Ok(SpannedToken {
            token,
            span: Span(std::range::Range { start, end }),
        })
    }

    pub fn take_token(&mut self) -> Result<Token<'src>, Error> {
        let Some(byte) = self.source.as_bytes().get(self.position) else {
            return Ok(Token::Eof);
        };

        let token = match byte {
            b' ' | b'\t' | b'\n' | b'\r' => {
                self.position += 1;
                return self.take_token();
            }
            b'{' => Token::OpenBrace,
            b'}' => Token::CloseBrace,
            b'[' => Token::OpenBracket,
            b']' => Token::CloseBracket,
            b'(' => Token::OpenParen,
            b')' => Token::CloseParen,
            b':' => Token::Colon,
            b',' => Token::Comma,
            b'B' => return self.take_unit_token(b"B;", Token::Int8ArrayHeader),
            b'I' => return self.take_unit_token(b"I;", Token::Int32ArrayHeader),
            b'L' => return self.take_unit_token(b"L;", Token::Int64ArrayHeader),
            b't' => return self.take_unit_token(b"true", Token::True),
            b'f' => return self.take_unit_token(b"false", Token::False),
            b'b' => return self.take_unit_token(b"bool", Token::Bool),
            b'u' => return self.take_unit_token(b"uuid", Token::Uuid),
            b'.' | b'-' | b'0'..=b'9' => return self.take_number_token(),
            b'"' => return self.take_string_token(b'"'),
            b'\'' => return self.take_string_token(b'\''),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let length = self
                    .source
                    .as_bytes()
                    .get(self.position..)
                    .unwrap()
                    .iter()
                    .take_while(|byte| matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' | b'-' | b'.' | b'+'))
                    .count();

                let content = self
                    .source
                    .as_bytes()
                    .get(self.position..self.position + length)
                    .expect("at least length bytes to remain");

                let content = str::from_utf8(content).expect("found string is valid utf-8");

                self.position += length;

                return Ok(Token::String(vec![StringContentToken::Literal(content)]));
            }
            _ => return Err(Error::InvalidToken(SourcePosition(self.position))),
        };

        self.position += 1;
        Ok(token)
    }

    fn take_unit_token(
        &mut self,
        expected_bytes: &[u8],
        token: Token<'src>,
    ) -> Result<Token<'src>, Error> {
        let Some(bytes) = self
            .source
            .as_bytes()
            .get(self.position..self.position + expected_bytes.len())
        else {
            return Err(Error::InvalidToken(SourcePosition(self.position)));
        };

        if bytes == expected_bytes {
            self.position += expected_bytes.len();
            Ok(token)
        } else {
            Err(Error::InvalidToken(SourcePosition(self.position)))
        }
    }

    fn take_string_token(&mut self, quote: u8) -> Result<Token<'src>, Error> {
        let source = self.source.as_bytes();
        assert_eq!(source[self.position], quote);
        self.position += 1;

        let mut tokens = Vec::new();
        loop {
            let rest = source
                .get(self.position..)
                .expect("remainder of source in bounds of source");

            let Some(end_pos) = rest.iter().position(|byte| *byte == quote) else {
                return Err(Error::InvalidToken(SourcePosition(self.position)));
            };

            let rest = rest
                .get(0..end_pos)
                .expect("quote found in remaining source");

            if let Some(end_pos) = rest.iter().position(|byte| *byte == b'\\') {
                let rest = rest
                    .get(0..end_pos)
                    .expect("backslash found in remaining source");

                tokens.push(StringContentToken::Literal(str::from_utf8(rest).map_err(
                    |err| Error::InvalidUtf8(SourcePosition(self.position), err),
                )?));

                self.position += rest.len() + 1;

                tokens.push(self.take_string_escape()?);
            } else {
                tokens.push(StringContentToken::Literal(str::from_utf8(rest).map_err(
                    |err| Error::InvalidUtf8(SourcePosition(self.position), err),
                )?));

                self.position += rest.len() + 1;

                break;
            }
        }

        Ok(Token::String(tokens))
    }

    fn take_string_escape(&mut self) -> Result<StringContentToken<'src>, Error> {
        let Some(byte) = self.source.as_bytes().get(self.position) else {
            return Err(Error::UnclosedEscape(SourcePosition(self.position)));
        };

        let token = match byte {
            b'b' => StringContentToken::Escaped(0x08.into()),
            b'f' => StringContentToken::Escaped(0x0c.into()),
            b'n' => StringContentToken::Escaped('\n'),
            b'r' => StringContentToken::Escaped('\r'),
            b's' => StringContentToken::Escaped(' '),
            b't' => StringContentToken::Escaped('\t'),
            b'\\' => StringContentToken::Escaped('\\'),
            b'\'' => StringContentToken::Escaped('\''),
            b'"' => StringContentToken::Escaped('"'),
            b'x' => match self.source.as_bytes().get(self.position + 1..) {
                Some([a @ b'0'..=b'9', b @ b'0'..=b'9', ..]) => {
                    self.position += 3;
                    return Ok(StringContentToken::Escaped(char::from(
                        (a - b'0') * 0x10 + (b - b'0'),
                    )));
                }
                Some([_, _]) => return Err(Error::EscapeNonDigits(SourcePosition(self.position))),
                _ => return Err(Error::UnclosedEscape(SourcePosition(self.position))),
            },
            b'u' => match self.source.as_bytes().get(self.position + 1..) {
                Some(
                    [
                        a @ b'0'..=b'9',
                        b @ b'0'..=b'9',
                        c @ b'0'..=b'9',
                        d @ b'0'..=b'9',
                        ..,
                    ],
                ) => {
                    let ch = char::try_from(
                        u32::from(a - b'0') * 0x1000
                            + u32::from(b - b'0') * 0x100
                            + u32::from(c - b'0') * 0x10
                            + u32::from(d - b'0'),
                    )
                    .map_err(|err| Error::InvalidEscapeChar(SourcePosition(self.position), err))?;

                    self.position += 5;
                    return Ok(StringContentToken::Escaped(ch));
                }
                Some([_, _, _, _]) => {
                    return Err(Error::EscapeNonDigits(SourcePosition(self.position)));
                }
                _ => return Err(Error::UnclosedEscape(SourcePosition(self.position))),
            },
            b'U' => match self.source.as_bytes().get(self.position + 1..) {
                Some(
                    [
                        a @ b'0'..=b'9',
                        b @ b'0'..=b'9',
                        c @ b'0'..=b'9',
                        d @ b'0'..=b'9',
                        e @ b'0'..=b'9',
                        f @ b'0'..=b'9',
                        g @ b'0'..=b'9',
                        h @ b'0'..=b'9',
                        ..,
                    ],
                ) => {
                    let ch = char::try_from(
                        u32::from(a - b'0') * 0x1000_0000
                            + u32::from(b - b'0') * 0x100_0000
                            + u32::from(c - b'0') * 0x10_0000
                            + u32::from(d - b'0') * 0x1_0000
                            + u32::from(e - b'0') * 0x1000
                            + u32::from(f - b'0') * 0x100
                            + u32::from(g - b'0') * 0x10
                            + u32::from(h - b'0'),
                    )
                    .map_err(|err| Error::InvalidEscapeChar(SourcePosition(self.position), err))?;

                    self.position += 5;
                    return Ok(StringContentToken::Escaped(ch));
                }
                Some([_, _, _, _]) => {
                    return Err(Error::EscapeNonDigits(SourcePosition(self.position)));
                }
                _ => return Err(Error::UnclosedEscape(SourcePosition(self.position))),
            },
            b'N' => match self.source.as_bytes().get(self.position + 1..) {
                Some([b'{', rest @ ..]) => {
                    if let Some(close_pos) = rest.iter().position(|byte| *byte == b'}') {
                        let name = rest.get(..close_pos).expect("string to exist up to close");
                        let name = str::from_utf8(name).map_err(|err| {
                            Error::InvalidUtf8(SourcePosition(self.position), err)
                        })?;
                        self.position += 2 + close_pos;
                        return Ok(StringContentToken::Named(name));
                    } else {
                        return Err(Error::UnclosedEscape(SourcePosition(self.position)));
                    }
                }
                _ => return Err(Error::UnclosedEscape(SourcePosition(self.position))),
            },
            _ => return Err(Error::UnknownEscape(SourcePosition(self.position))),
        };

        self.position += 1;

        Ok(token)
    }
}

#[test]
fn tokenize_snbt() {
    let mut tokenizer = Tokenizer::new("B; true , \n :, \t 0b01010uB \"abc\"\"def\\nghi\": a+b [");
    assert_eq!(tokenizer.take_token().unwrap(), Token::Int8ArrayHeader);
    assert_eq!(tokenizer.take_token().unwrap(), Token::True);
    assert_eq!(tokenizer.take_token().unwrap(), Token::Comma);
    assert_eq!(tokenizer.take_token().unwrap(), Token::Colon);
    assert_eq!(tokenizer.take_token().unwrap(), Token::Comma);
    assert_eq!(
        tokenizer.take_token().unwrap(),
        Token::Number(number::Number::Int(number::Int {
            sign: number::Sign::Positive,
            signedness: number::Signedness::Unsigned,
            r#type: number::IntType::Int8,
            digits: number::IntBytes::Binary(b"01010"),
        }))
    );
    assert_eq!(
        tokenizer.take_token().unwrap(),
        Token::String(vec![StringContentToken::Literal("abc")])
    );
    assert_eq!(
        tokenizer.take_token().unwrap(),
        Token::String(vec![
            StringContentToken::Literal("def"),
            StringContentToken::Escaped('\n'),
            StringContentToken::Literal("ghi")
        ])
    );
    assert_eq!(tokenizer.take_token().unwrap(), Token::Colon);
    assert_eq!(
        tokenizer.take_token().unwrap(),
        Token::String(vec![StringContentToken::Literal("a+b")])
    );
    assert_eq!(tokenizer.take_token().unwrap(), Token::OpenBracket);
    assert_eq!(tokenizer.take_token().unwrap(), Token::Eof);
}

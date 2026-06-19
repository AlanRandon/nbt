use crate::snbt::read::SourcePosition;

mod number;

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
    ByteArrayHeader,
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
    /// `<byte>`
    Byte(u8),
    /// `<int16>`
    Int16(u32),
    /// `<int32>`
    Int32(u32),
    /// `<int64>`
    Int64(u32),
    /// `<float>`
    Float(f32),
    /// `<double>`
    Double(f64),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum StringContentToken<'src> {
    Literal(&'src str),
    Escaped(char),
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
}

impl<'src> Tokenizer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            position: 0,
            source,
        }
    }

    pub fn take_token(&mut self) -> Result<Token<'src>, Error> {
        let Some(byte) = self.source.as_bytes().get(self.position) else {
            return Ok(Token::Eof);
        };

        let token = match byte {
            b'{' => Token::OpenBrace,
            b'}' => Token::CloseBrace,
            b'[' => Token::OpenBracket,
            b']' => Token::CloseBracket,
            b'(' => Token::OpenParen,
            b')' => Token::CloseParen,
            b':' => Token::Colon,
            b',' => Token::Comma,
            b'B' => return self.take_unit_token(b"B;", Token::ByteArrayHeader),
            b'I' => return self.take_unit_token(b"I;", Token::Int32ArrayHeader),
            b'L' => return self.take_unit_token(b"L;", Token::Int64ArrayHeader),
            b't' => return self.take_unit_token(b"true", Token::True),
            b'f' => return self.take_unit_token(b"false", Token::False),
            b'b' => return self.take_unit_token(b"bool", Token::Bool),
            b'u' => return self.take_unit_token(b"uuid", Token::Uuid),
            b' ' | b'\t' | b'\n' | b'\r' => {
                self.position += 1;
                return self.take_token();
            }
            b'\"' => return self.take_string_token(),
            b'.' | b'-' | b'0'..=b'9' => return self.take_number_token(),
            _ => return Err(Error::InvalidToken(SourcePosition(self.position))),
        };

        self.position += 1;
        Ok(token)
    }

    pub fn take_unit_token(
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

    pub fn take_number_token(&mut self) -> Result<Token<'src>, Error> {
        let source = self.source.as_bytes().get(self.position..).unwrap();

        todo!();
    }

    pub fn take_string_token(&mut self) -> Result<Token<'src>, Error> {
        let source = self.source.as_bytes();
        assert_eq!(source[self.position], b'\"');
        self.position += 1;

        let mut tokens = Vec::new();
        loop {
            let rest = source
                .get(self.position..)
                .expect("remainder of source in bounds of source");

            let Some(end_pos) = rest.iter().position(|byte| *byte == b'\"') else {
                return Err(Error::InvalidToken(SourcePosition(self.position)));
            };

            let rest = rest
                .get(0..end_pos)
                .expect("quote found in remaining source");

            if let Some(end_pos) = rest.iter().position(|byte| *byte == b'\\') {
                let rest = rest
                    .get(0..end_pos)
                    .expect("backslash found in remaining source");

                tokens.push(StringContentToken::Literal(
                    str::from_utf8(rest).expect("string before backslash is valid utf-8"),
                ));

                self.position += rest.len() + 1;

                tokens.push(self.take_string_escape()?);
            } else {
                tokens.push(StringContentToken::Literal(
                    str::from_utf8(rest).expect("string before backslash is valid utf-8"),
                ));

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
            b'x' => todo!(),
            b'u' => todo!(),
            b'U' => todo!(),
            b'N' => todo!(),
            _ => return Err(Error::UnknownEscape(SourcePosition(self.position))),
        };

        self.position += 1;

        Ok(token)
    }
}

#[test]
fn tokenize_snbt() {
    let mut tokenizer = Tokenizer::new("B; true , \n :, \t \"abc\"\"def\\nghi\":");
    assert_eq!(tokenizer.take_token().unwrap(), Token::ByteArrayHeader);
    assert_eq!(tokenizer.take_token().unwrap(), Token::True);
    assert_eq!(tokenizer.take_token().unwrap(), Token::Comma);
    assert_eq!(tokenizer.take_token().unwrap(), Token::Colon);
    assert_eq!(tokenizer.take_token().unwrap(), Token::Comma);
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
    assert_eq!(tokenizer.take_token().unwrap(), Token::Eof);
}

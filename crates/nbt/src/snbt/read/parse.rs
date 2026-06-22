use crate::snbt::read::tokenize::{self, SpannedToken, Token, Tokenizer};
use crate::snbt::read::{SourcePosition, Span};

pub struct Parser<'src> {
    tokenizer: Tokenizer<'src>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error<'src> {
    #[error("failed to tokenize")]
    TokenizeError(tokenize::Error),
    #[error("unexpected token")]
    UnexpectedToken(SpannedToken<'src>),
    #[error("error parsing list")]
    List(SourcePosition, Box<Error<'src>>),
    #[error("list missing comma between elements")]
    MissingListComma(SourcePosition, SpannedToken<'src>),
    #[error("unexpected non-integer in integer list")]
    UnexpectedNonInteger(SourcePosition, Variant<'src>),
    #[error("error parsing compound")]
    Compound(SourcePosition, Box<Error<'src>>),
    #[error("compound missing comma between fields")]
    MissingCompoundComma(SourcePosition, SpannedToken<'src>),
    #[error("key in compound must be a string")]
    NonStringKey(SourcePosition, Variant<'src>),
    #[error("compound missing colon in field")]
    MissingColon(SourcePosition, SpannedToken<'src>),
    #[error("call missing open parenthesis")]
    MissingOpenParen(Span, SpannedToken<'src>),
    #[error("error parsing operation")]
    Operation(Span, Box<Error<'src>>),
    #[error("operation missing comma between arguments")]
    MissingOperationComma(Span, SpannedToken<'src>),
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            tokenizer: Tokenizer::new(source),
        }
    }

    pub fn parse_variant_and_finish(mut self) -> Result<Variant<'src>, Error<'src>> {
        let variant = self.parse_variant(None)?;
        self.expect_token(|token| token.token == Token::Eof)?;
        Ok(variant)
    }

    fn parse_variant(
        &mut self,
        first_token: Option<SpannedToken<'src>>,
    ) -> Result<Variant<'src>, Error<'src>> {
        let token = if let Some(token) = first_token {
            token
        } else {
            self.tokenizer
                .take_spanned_token()
                .map_err(|err| Error::TokenizeError(err))?
        };

        match token.token {
            Token::String(content) => Ok(Variant::String(SpannedString {
                span: token.span,
                content,
            })),
            Token::Number(number) => match number {
                tokenize::number::Number::Int(integer) => Ok(Variant::Int(SpannedInt {
                    span: token.span,
                    integer,
                })),
                tokenize::number::Number::Float(float) => Ok(Variant::Float(SpannedFloat {
                    span: token.span,
                    float,
                })),
            },
            Token::True => Ok(Variant::Bool(token.span, true)),
            Token::False => Ok(Variant::Bool(token.span, false)),
            Token::Bool => self
                .parse_operation(token.span, OperationKind::Bool)
                .map(Box::new)
                .map(Variant::Operation),
            Token::Uuid => self
                .parse_operation(token.span, OperationKind::Uuid)
                .map(Box::new)
                .map(Variant::Operation),
            Token::OpenBrace => self
                .parse_compound(token.span.0.start)
                .map(Variant::Compound),
            Token::OpenBracket => self.parse_list(token.span.0.start),
            _ => Err(Error::UnexpectedToken(token)),
        }
    }

    fn parse_list(&mut self, start: SourcePosition) -> Result<Variant<'src>, Error<'src>> {
        let token = self.take_list_token(start)?;

        match token.token {
            Token::Int8ArrayHeader => {
                return self
                    .parse_int_list(
                        start,
                        ListHeader {
                            kind: ListHeaderKind::Int8,
                            span: token.span,
                        },
                    )
                    .map(Variant::IntList);
            }
            Token::Int32ArrayHeader => {
                return self
                    .parse_int_list(
                        start,
                        ListHeader {
                            kind: ListHeaderKind::Int32,
                            span: token.span,
                        },
                    )
                    .map(Variant::IntList);
            }
            Token::Int64ArrayHeader => {
                return self
                    .parse_int_list(
                        start,
                        ListHeader {
                            kind: ListHeaderKind::Int64,
                            span: token.span,
                        },
                    )
                    .map(Variant::IntList);
            }
            _ => {}
        }

        let mut token = token;
        let mut list = Vec::new();
        let mut expecting_item = true;

        loop {
            match token.token {
                Token::CloseBracket => {
                    return Ok(Variant::List(List {
                        span: Span(std::range::Range {
                            start,
                            end: token.span.0.end,
                        }),
                        list,
                    }));
                }
                _ => {}
            }

            if !expecting_item {
                return Err(Error::MissingListComma(start, token));
            }

            list.push(
                self.parse_variant(Some(token))
                    .map_err(|err| Error::List(start, Box::new(err)))?,
            );

            token = self.take_list_token(start)?;

            if token.token == Token::Comma {
                expecting_item = true;
                token = self.take_list_token(start)?;
            } else {
                expecting_item = false;
            }
        }
    }

    fn parse_int_list(
        &mut self,
        start: SourcePosition,
        header: ListHeader,
    ) -> Result<IntList<'src>, Error<'src>> {
        let mut token = self.take_list_token(start)?;
        let mut list = Vec::new();
        let mut expecting_item = true;

        loop {
            match token.token {
                Token::CloseBracket => {
                    return Ok(IntList {
                        span: Span(std::range::Range {
                            start,
                            end: token.span.0.end,
                        }),
                        header,
                        list,
                    });
                }
                _ => {}
            }

            if !expecting_item {
                return Err(Error::MissingListComma(start, token));
            }

            let variant = self
                .parse_variant(Some(token))
                .map_err(|err| Error::List(start, Box::new(err)))?;

            match variant {
                Variant::Int(integer) => list.push(integer),
                variant => return Err(Error::UnexpectedNonInteger(start, variant)),
            }

            token = self.take_list_token(start)?;

            if token.token == Token::Comma {
                expecting_item = true;
                token = self.take_list_token(start)?;
            } else {
                expecting_item = false;
            }
        }
    }

    fn take_list_token(
        &mut self,
        start: SourcePosition,
    ) -> Result<SpannedToken<'src>, Error<'src>> {
        self.tokenizer
            .take_spanned_token()
            .map_err(|err| Error::List(start, Box::new(Error::TokenizeError(err))))
    }

    fn parse_compound(&mut self, start: SourcePosition) -> Result<Compound<'src>, Error<'src>> {
        let mut token = self.take_compound_token(start)?;
        let mut fields = Vec::new();
        let mut expecting_item = true;

        loop {
            match token.token {
                Token::CloseBrace => {
                    return Ok(Compound {
                        span: Span(std::range::Range {
                            start,
                            end: token.span.0.end,
                        }),
                        fields,
                    });
                }
                _ => {}
            }

            if !expecting_item {
                return Err(Error::MissingCompoundComma(start, token));
            }

            let key = match self.parse_variant(Some(token))? {
                Variant::String(string) => string,
                key => return Err(Error::NonStringKey(start, key)),
            };

            self.expect_token(|token| token.token == Token::Colon)?;

            let value = self
                .parse_variant(None)
                .map_err(|err| Error::Compound(start, Box::new(err)))?;

            fields.push(Field { key, value });

            token = self.take_compound_token(start)?;

            if token.token == Token::Comma {
                expecting_item = true;
                token = self.take_compound_token(start)?;
            } else {
                expecting_item = false;
            }
        }
    }

    fn take_compound_token(
        &mut self,
        start: SourcePosition,
    ) -> Result<SpannedToken<'src>, Error<'src>> {
        self.tokenizer
            .take_spanned_token()
            .map_err(|err| Error::Compound(start, Box::new(Error::TokenizeError(err))))
    }

    fn parse_operation(
        &mut self,
        kind_span: Span,
        kind: OperationKind,
    ) -> Result<Operation<'src>, Error<'src>> {
        let token = self.take_operation_token(kind_span)?;
        match token.token {
            Token::OpenParen => {}
            _ => return Err(Error::MissingOpenParen(kind_span, token)),
        }

        let mut token = self.take_operation_token(kind_span)?;
        let mut arguments = Vec::new();
        let mut expecting_item = true;

        loop {
            match token.token {
                Token::CloseParen => {
                    return Ok(Operation {
                        span: Span(std::range::Range {
                            start: kind_span.0.start,
                            end: token.span.0.end,
                        }),
                        kind_span,
                        kind,
                        arguments,
                    });
                }
                _ => {}
            }

            if !expecting_item {
                return Err(Error::MissingOperationComma(kind_span, token));
            }

            arguments.push(
                self.parse_variant(Some(token))
                    .map_err(|err| Error::Operation(kind_span, Box::new(err)))?,
            );

            token = self.take_operation_token(kind_span)?;

            if token.token == Token::Comma {
                expecting_item = true;
                token = self.take_operation_token(kind_span)?;
            } else {
                expecting_item = false;
            }
        }
    }

    fn take_operation_token(&mut self, kind_span: Span) -> Result<SpannedToken<'src>, Error<'src>> {
        self.tokenizer
            .take_spanned_token()
            .map_err(|err| Error::Operation(kind_span, Box::new(Error::TokenizeError(err))))
    }

    fn expect_token(
        &mut self,
        predicate: impl FnOnce(&SpannedToken<'src>) -> bool,
    ) -> Result<SpannedToken<'src>, Error<'src>> {
        let token = self
            .tokenizer
            .take_spanned_token()
            .map_err(|err| Error::TokenizeError(err))?;

        if predicate(&token) {
            Ok(token)
        } else {
            Err(Error::UnexpectedToken(token))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variant<'src> {
    Int(SpannedInt<'src>),
    Float(SpannedFloat<'src>),
    String(SpannedString<'src>),
    Compound(Compound<'src>),
    List(List<'src>),
    IntList(IntList<'src>),
    Operation(Box<Operation<'src>>),
    Bool(Span, bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Compound<'src> {
    pub span: Span,
    pub fields: Vec<Field<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct List<'src> {
    pub span: Span,
    pub list: Vec<Variant<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntList<'src> {
    pub span: Span,
    pub header: ListHeader,
    pub list: Vec<SpannedInt<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field<'src> {
    pub key: SpannedString<'src>,
    pub value: Variant<'src>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedInt<'src> {
    pub span: Span,
    pub integer: tokenize::number::Int<'src>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedFloat<'src> {
    pub span: Span,
    pub float: tokenize::number::Float<'src>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedString<'src> {
    pub span: Span,
    pub content: Vec<tokenize::StringContentToken<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Operation<'src> {
    pub span: Span,
    pub kind_span: Span,
    pub kind: OperationKind,
    pub arguments: Vec<Variant<'src>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationKind {
    Uuid,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListHeader {
    pub span: Span,
    pub kind: ListHeaderKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListHeaderKind {
    Int8,
    Int32,
    Int64,
}

#[test]
fn parse_variant() {
    use crate::snbt::read::tokenize::StringContentToken;
    use std::assert_matches;
    use tokenize::number::{Int, IntBytes, IntType, Sign, Signedness};

    let source = r#"
{
  key1: 123,
  'key2': 'somevalue1',
  "key3": {
    subkey1: 0x1C8,
    "subkey2": "somevalue2"
  }
}
"#;
    let mut parser = Parser::new(source);
    let variant = parser.parse_variant(None).unwrap();
    assert_matches!(
        variant,
        Variant::Compound(Compound { span: _, fields })
            if matches!(
                &fields as &[Field],
                [
                    Field {
                        key: SpannedString { span: _, content: key_1 },
                        value: Variant::Int(SpannedInt {
                            span: _,
                            integer: Int {
                                sign: Sign::Positive,
                                signedness: Signedness::Signed,
                                r#type: IntType::Int32,
                                digits: IntBytes::Denary(b"123"),
                            }
                        })
                    },
                    Field {
                        key: SpannedString { span: _, content: key_2 },
                        value: Variant::String(SpannedString { span: _, content: value_2 })
                    },
                    Field {
                        key: SpannedString { span: _, content: key_3 },
                        value: Variant::Compound(Compound {
                            span: _,
                            fields: value_3_fields
                        })
                    },
                ]
                if
                    *key_1 == vec![StringContentToken::Literal("key1")]
                    && *key_2 == vec![StringContentToken::Literal("key2")]
                    && *key_3 == vec![StringContentToken::Literal("key3")]
                    && *value_2 == vec![StringContentToken::Literal("somevalue1")]
                    && matches!(
                        value_3_fields as &[Field],
                        [
                            Field {
                                key: SpannedString { span: _, content: subkey_1 },
                                value: Variant::Int(SpannedInt {
                                    span: _,
                                    integer: Int {
                                        sign: Sign::Positive,
                                        signedness: Signedness::Signed,
                                        r#type: IntType::Int32,
                                        digits: IntBytes::Hex(b"1C8"),
                                    }
                                })
                            },
                            Field {
                                key: SpannedString { span: _, content: subkey_2 },
                                value: Variant::String(SpannedString { span: _, content: subvalue_2 })
                            },
                        ]
                        if
                            *subkey_1 == vec![StringContentToken::Literal("subkey1")]
                            && *subkey_2 == vec![StringContentToken::Literal("subkey2")]
                            && *subvalue_2 == vec![StringContentToken::Literal("somevalue2")]
                    )
            )
    );

    assert_eq!(
        parser
            .expect_token(|token| token.token == Token::Eof)
            .unwrap()
            .token,
        Token::Eof
    );
}

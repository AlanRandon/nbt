use clap::Parser as _;
use clap_file::{Input, Output};
use miette::{LabeledSpan, NamedSource, miette};
use nbt::binary::Endianness;
use nbt::binary::read::ReadNbt;
use nbt::binary::write::WriteNbt;
use nbt::snbt::read::{Span, convert, parse, tokenize};
use std::io::{Read, Write};

#[derive(clap::Parser)]
#[command(version, about)]
struct Arguments {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(Debug, Clone, Copy)]
struct ArgEndianness(Endianness);

impl clap::ValueEnum for ArgEndianness {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            ArgEndianness(Endianness::Little),
            ArgEndianness(Endianness::Big),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self(Endianness::Little) => Some(clap::builder::PossibleValue::new("little")),
            Self(Endianness::Big) => Some(clap::builder::PossibleValue::new("big")),
        }
    }
}

#[derive(clap::Subcommand)]
enum Subcommand {
    NbtToSnbt {
        #[arg(value_name = "NBT_FILE")]
        input: Input,
        #[arg(short, long, default_value = "-", value_name = "SNBT_FILE")]
        output: Output,
        #[arg(long)]
        bedrock_header: bool,
        #[arg(
            short,
            long,
            default_value = "little",
            conflicts_with = "bedrock_header"
        )]
        endianness: ArgEndianness,
    },
    SnbtToNbt {
        #[arg(value_name = "SNBT_FILE")]
        input: Input,
        #[arg(short, long, default_value = "-", value_name = "NBT_FILE")]
        output: Output,
        #[arg(long)]
        bedrock_header: bool,
        #[arg(long, default_value = "10")]
        bedrock_header_version: u32,
        #[arg(
            short,
            long,
            default_value = "little",
            conflicts_with = "bedrock_header"
        )]
        endianness: ArgEndianness,
    },
}

#[derive(Debug, Clone)]
struct MietteSpan(Span);

impl From<MietteSpan> for miette::SourceSpan {
    fn from(value: MietteSpan) -> Self {
        let length = value.0.0.end.0 - value.0.0.start.0;
        Self::new(value.0.0.start.0.into(), length)
    }
}

fn parse_error_into_labels<'src>(error: parse::Error<'src>) -> Vec<LabeledSpan> {
    match error {
        parse::Error::TokenizeError(error) => {
            let label = match error {
                tokenize::Error::InvalidToken(pos) => LabeledSpan::at(pos.0, "invalid token"),
                tokenize::Error::UnclosedEscape(pos) => {
                    LabeledSpan::at(pos.0, "unclosed string escape sequence")
                }
                tokenize::Error::UnknownEscape(pos) => {
                    LabeledSpan::at(pos.0, "unknown string escape sequence")
                }
                tokenize::Error::EscapeNonDigits(pos) => LabeledSpan::at(
                    pos.0,
                    "non-digits present in string escape sequence requiring only digits",
                ),
                tokenize::Error::InvalidEscapeChar(pos, _) => {
                    LabeledSpan::at(pos.0, "escaped character is not a valid character")
                }
                tokenize::Error::InvalidNumber(pos) => {
                    LabeledSpan::at(pos.0, "number token was malformed")
                }
                tokenize::Error::InvalidUtf8(pos, _) => {
                    LabeledSpan::at(pos.0, "string contains invalid utf-8")
                }
            };

            vec![label]
        }
        parse::Error::UnexpectedToken(token) => {
            vec![LabeledSpan::at(MietteSpan(token.span), "unexpected token")]
        }
        parse::Error::List(pos, error) => {
            let mut labels = vec![LabeledSpan::at(pos.0, "while parsing this list")];
            labels.extend(parse_error_into_labels(*error));
            labels
        }
        parse::Error::MissingListComma(pos, spanned_token) => vec![
            LabeledSpan::at(pos.0, "while parsing this list"),
            LabeledSpan::at(
                MietteSpan(spanned_token.span),
                "expected a comma, but found this",
            ),
        ],
        parse::Error::Compound(pos, error) => {
            let mut labels = vec![LabeledSpan::at(pos.0, "while parsing this compound")];
            labels.extend(parse_error_into_labels(*error));
            labels
        }
        parse::Error::MissingCompoundComma(pos, spanned_token) => {
            vec![
                LabeledSpan::at(pos.0, "while parsing this compound"),
                LabeledSpan::at(MietteSpan(spanned_token.span), "expected a comma"),
            ]
        }
        parse::Error::NonStringKey(pos, variant) => {
            vec![
                LabeledSpan::at(pos.0, "while parsing this compound"),
                LabeledSpan::at(MietteSpan(variant.span()), "only string keys are allowed"),
            ]
        }
        parse::Error::MissingColon(pos, spanned_token) => {
            vec![
                LabeledSpan::at(pos.0, "while parsing this compound"),
                LabeledSpan::at(
                    MietteSpan(spanned_token.span),
                    "expected a colon between key and value",
                ),
            ]
        }
        parse::Error::UnexpectedNonInteger(pos, variant) => vec![
            LabeledSpan::at(pos.0, "while parsing this integer array"),
            LabeledSpan::at(MietteSpan(variant.span()), "expected an integer"),
        ],
        parse::Error::MissingOpenParen(span, spanned_token) => vec![
            LabeledSpan::at(MietteSpan(span), "while parsing this operation"),
            LabeledSpan::at(
                MietteSpan(spanned_token.span),
                "expected an open parenthesis",
            ),
        ],
        parse::Error::Operation(span, error) => {
            let mut labels = vec![LabeledSpan::at(
                MietteSpan(span),
                "while parsing this operation",
            )];
            labels.extend(parse_error_into_labels(*error));
            labels
        }
        parse::Error::MissingOperationComma(span, spanned_token) => vec![
            LabeledSpan::at(MietteSpan(span), "while parsing this operation"),
            LabeledSpan::at(MietteSpan(spanned_token.span), "expected a comma"),
        ],
    }
}

fn convert_error_into_labels<'src>(error: convert::Error<'src>) -> Vec<LabeledSpan> {
    match error {
        convert::Error::DuplicateField {
            key,
            values,
            compound_span,
        } => {
            let mut labels = vec![LabeledSpan::at(
                MietteSpan(compound_span),
                format!("compound has duplicate field {key:?}"),
            )];
            labels.extend(values.into_iter().map(|value| {
                LabeledSpan::at(MietteSpan(value.span()), format!("instance of {key:?}"))
            }));
            labels
        }
        convert::Error::UpcastInvalid {
            integer,
            result_type,
        } => vec![LabeledSpan::at(
            MietteSpan(integer.span),
            format!("integer cannot be upcast into {result_type:?}"),
        )],
        convert::Error::IntegerTooLarge {
            integer,
            result_type,
        } => vec![LabeledSpan::at(
            MietteSpan(integer.span),
            format!("integer is too large to fit into {result_type:?}"),
        )],
        convert::Error::NegativeUnsignedInteger {
            integer,
            result_type: _,
        } => vec![LabeledSpan::at(
            MietteSpan(integer.span),
            "integers cannot be both unsigned and negative",
        )],
        convert::Error::FloatTooLarge { float, result_type } => vec![LabeledSpan::at(
            MietteSpan(float.span),
            format!("float is too large to fit into {result_type:?}"),
        )],
        convert::Error::ArgumentError {
            operation_span,
            operation_kind_span: _,
            error,
        } => {
            let mut labels = vec![LabeledSpan::at(
                MietteSpan(operation_span),
                "while converting this operation",
            )];
            labels.extend(convert_error_into_labels(*error));
            labels
        }
        convert::Error::OperationArityError {
            operation_span,
            operation_kind: _,
            operation_kind_span,
            expected_arity,
            found_arity,
        } => vec![
            LabeledSpan::at(
                MietteSpan(operation_span),
                format!("found {found_arity} arguments but expected {expected_arity}"),
            ),
            LabeledSpan::at(
                MietteSpan(operation_kind_span),
                format!("operation takes {expected_arity} arguments"),
            ),
        ],
        convert::Error::OperationTypeError {
            operation_span,
            operation_kind: _,
            operation_kind_span: _,
            arguments: _,
        } => vec![LabeledSpan::at(
            MietteSpan(operation_span),
            "found arguments of incorrect type",
        )],
        convert::Error::ParseUuidError {
            operation_span,
            operation_kind_span: _,
            error: _,
        } => vec![LabeledSpan::at(
            MietteSpan(operation_span),
            "failed to parse UUID",
        )],
    }
}

fn main() -> miette::Result<()> {
    let arguments = Arguments::parse();
    match arguments.command {
        Subcommand::NbtToSnbt {
            mut input,
            mut output,
            bedrock_header: header,
            endianness: ArgEndianness(endianness),
        } => {
            if header {
                let header = nbt::BedrockHeader::read_nbt(&mut input, endianness)
                    .map_err(miette::Report::msg)?;

                eprintln!("found header: version={}", header.version);
            }

            let nbt::NamedTag(key, value) =
                nbt::NamedTag::read_nbt(&mut input, endianness).map_err(miette::Report::msg)?;

            if !key.is_empty() {
                eprintln!("found non-empty root: {key:?}");
            }

            output
                .write_all(value.to_string().as_bytes())
                .map_err(miette::Report::msg)?;

            output.flush().map_err(miette::Report::msg)?;

            Ok(())
        }
        Subcommand::SnbtToNbt {
            mut input,
            mut output,
            bedrock_header: header,
            bedrock_header_version: header_version,
            endianness: ArgEndianness(endianness),
        } => {
            let mut source = String::new();
            input
                .read_to_string(&mut source)
                .map_err(miette::Report::msg)?;

            let parser = nbt::snbt::read::parse::Parser::new(&source);
            let nbt = match parser.parse_variant_and_finish() {
                Ok(nbt) => nbt,
                Err(error) => {
                    return Err(
                        miette!(labels = parse_error_into_labels(error), "parse error")
                            .with_source_code(NamedSource::new(
                                input
                                    .path()
                                    .and_then(std::path::Path::to_str)
                                    .unwrap_or("-"),
                                source,
                            )),
                    );
                }
            };

            let nbt = match nbt::Variant::try_from(nbt) {
                Ok(nbt) => nbt::NamedTag(String::new(), nbt),
                Err(error) => {
                    return Err(miette!(
                        labels = convert_error_into_labels(error),
                        "convert error"
                    )
                    .with_source_code(NamedSource::new(
                        input
                            .path()
                            .and_then(std::path::Path::to_str)
                            .unwrap_or("-"),
                        source,
                    )));
                }
            };

            if header {
                let mut payload = Vec::new();
                nbt.write_nbt(&mut payload, endianness)
                    .map_err(miette::Report::msg)?;
                let payload = payload;
                let header = nbt::BedrockHeader {
                    version: header_version,
                    size: payload.len().try_into().map_err(miette::Report::msg)?,
                };

                header
                    .write_nbt(&mut output, endianness)
                    .map_err(miette::Report::msg)?;

                output.write_all(&payload).map_err(miette::Report::msg)?;
                output.flush().map_err(miette::Report::msg)?;
            } else {
                nbt.write_nbt(&mut output, endianness)
                    .map_err(miette::Report::msg)?;

                output.flush().map_err(miette::Report::msg)?;
            }

            Ok(())
        }
    }
}

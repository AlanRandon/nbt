pub mod convert;
pub mod parse;
pub mod tokenize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span(std::range::Range<SourcePosition>);

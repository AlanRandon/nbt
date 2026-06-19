#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token {
    /// `-`
    Negative,
    /// `b'0'..=b'9'+`
    DigitSequence,
    /// `s` or `S`
    S,
}

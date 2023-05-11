use crate::lexer::LexerError;

pub(crate) struct ParseError {}

impl From<LexerError> for ParseError {
    fn from(_: LexerError) -> Self {
        Self {}
    }
}

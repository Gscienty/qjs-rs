use crate::{
    lexer::{Lexer, SourceReader, Token},
    vals::JSValue,
};

use super::parse_error;

pub(crate) struct Parser<'s> {
    pub(super) lexer: Lexer<'s>,
}

impl<'s> Parser<'s> {
    pub(crate) fn new(reader: &'s mut dyn SourceReader) -> Self {
        Parser {
            lexer: Lexer::new(reader),
        }
    }

    fn parse_value(&mut self) -> Result<JSValue, parse_error::ParseError> {
        match self.lexer.current() {
            Token::Str(val) => Ok(JSValue::Str(val.clone())),
            _ => Ok(JSValue::Null),
        }
    }
}

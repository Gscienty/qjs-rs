mod code_points;
mod lexer;
mod lexer_error;
mod reader;
mod token;

pub(crate) use lexer::Lexer;
pub(crate) use lexer_error::LexerError;
pub(crate) use reader::{InlineSourceReader, SourceReader};
pub(crate) use token::Token;

#[cfg(test)]
#[allow(non_snake_case)]
mod lexer_test;

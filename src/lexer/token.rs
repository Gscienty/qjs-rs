#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    EOF,

    Comment(String),
    HashbangComment(String),

    IdentifierName(String),
    PrivateIdentifier(String),
}

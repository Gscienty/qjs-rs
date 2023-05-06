mod inline;
mod reader;

pub(crate) use inline::InlineSourceReader;
pub(crate) use reader::SourceReader;

#[cfg(test)]
#[allow(non_snake_case)]
mod inline_test;

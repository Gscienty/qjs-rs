use crate::lexer::token::Token;

use super::{lexer::Lexer, reader};

#[test]
fn test_Lexer_parse_singleline_comment() {
    let mut src = reader::InlineSourceReader::new(
        r#"// hello world   
           //// foobar"#,
    );
    let mut lexer = Lexer::new(&mut src);

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::Comment(v) if v.eq(" hello world   ")))
    } else {
        panic!("next token failed")
    }

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::Comment(v) if v.eq("// foobar")))
    } else {
        panic!("next token failed")
    }
}

#[test]
fn test_Lexer_parse_multiline_comment() {
    let mut src = reader::InlineSourceReader::new(
        r#"/**
* hello
* world
*/"#,
    );
    let mut lexer = Lexer::new(&mut src);

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::Comment(v) if v.eq("*\n* hello\n* world\n")))
    } else {
        panic!("next token failed")
    }
}

#[test]
fn test_Lexer_parse_hashbang_comment() {
    let mut src = reader::InlineSourceReader::new(r#"#! hashbang"#);
    let mut lexer = Lexer::new(&mut src);

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::HashbangComment(v) if v.eq(" hashbang")))
    } else {
        panic!("next token failed")
    }
}

#[test]
fn test_Lexer_parse_identify_name() {
    let mut src = reader::InlineSourceReader::new(r#"$ _ h $hello _world foobar 张三"#);
    let mut lexer = Lexer::new(&mut src);

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::IdentifierName(v) if v.eq("$")))
    } else {
        panic!("next token failed")
    }

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::IdentifierName(v) if v.eq("_")))
    } else {
        panic!("next token failed")
    }

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::IdentifierName(v) if v.eq("h")))
    } else {
        panic!("next token failed")
    }

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::IdentifierName(v) if v.eq("$hello")))
    } else {
        panic!("next token failed")
    }

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::IdentifierName(v) if v.eq("_world")))
    } else {
        panic!("next token failed")
    }

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::IdentifierName(v) if v.eq("foobar")))
    } else {
        panic!("next token failed")
    }

    if let Ok(result) = lexer.next_token() {
        assert!(matches!(result, Token::IdentifierName(v) if v.eq("张三")))
    } else {
        panic!("next token failed")
    }
}

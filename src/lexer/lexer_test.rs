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

    let mut verify = |exp: &str| {
        if let Ok(result) = lexer.next_token() {
            println!("verify token: {:?} {}", result, exp);
            assert!(matches!(result, Token::IdentifierName(v) if v.eq(exp)));
            println!("verify token: {} success", exp);
        } else {
            println!("verify token: {} failed", exp);
            panic!("next token failed")
        }
    };

    verify("$");
    verify("_");
    verify("h");
    verify("$hello");
    verify("_world");
    verify("foobar");
    verify("张三");
}

#[test]
fn test_Lexer_parse_number() {
    let mut src = reader::InlineSourceReader::new(
        r#"123 1.23 .123 0x12a 0O123 0b10 0123 0129 1.e+5 .1e-5_6 1_2e3 0n 1n 123n"#,
    );
    let mut lexer = Lexer::new(&mut src);

    let mut verify = |exp: &str| {
        if let Ok(result) = lexer.next_token() {
            println!("verify token: {:?} {}", result, exp);
            assert!(matches!(result, Token::Number(v) if v.eq(exp)));
            println!("verify token: {} success", exp);
        } else {
            println!("verify token: {} failed", exp);
            panic!("next token failed")
        }
    };

    verify("123");
    verify("1.23");
    verify(".123");
    verify("0x12a");
    verify("0O123");
    verify("0b10");
    verify("0123");
    verify("0129");
    verify("1.e+5");
    verify(".1e-5_6");
    verify("1_2e3");
    verify("0n");
    verify("1n");
    verify("123n");
}

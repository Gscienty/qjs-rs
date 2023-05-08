use crate::lexer::token::Token;

use super::{lexer::Lexer, reader};

#[test]
fn test_Lexer_parse_singleline_comment() {
    let mut src = reader::InlineSourceReader::new(
        r#"// hello world   
           //// foobar"#,
    );
    let mut lexer = Lexer::new(&mut src);

    if lexer.next_token().is_ok() {
        assert!(matches!(lexer.current(), Token::Comment(v) if v.eq(" hello world   ")))
    } else {
        panic!("next token failed")
    }

    if lexer.next_token().is_ok() {
        assert!(matches!(lexer.current(), Token::Comment(v) if v.eq("// foobar")))
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

    if lexer.next_token().is_ok() {
        assert!(matches!(lexer.current(), Token::Comment(v) if v.eq("*\n* hello\n* world\n")))
    } else {
        panic!("next token failed")
    }
}

#[test]
fn test_Lexer_parse_hashbang_comment() {
    let mut src = reader::InlineSourceReader::new(r#"#! hashbang"#);
    let mut lexer = Lexer::new(&mut src);

    if lexer.next_token().is_ok() {
        assert!(matches!(lexer.current(), Token::HashbangComment(v) if v.eq(" hashbang")))
    } else {
        panic!("next token failed")
    }
}

#[test]
fn test_Lexer_parse_identify_name() {
    let mut src = reader::InlineSourceReader::new(r#"$ _ h $hello _world foobar 张三"#);
    let mut lexer = Lexer::new(&mut src);

    let mut verify = |exp: &str| {
        if lexer.next_token().is_ok() {
            println!("verify token: {:?} {}", lexer.current(), exp);
            assert!(matches!(lexer.current(), Token::IdentifierName(v) if v.eq(exp)));
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
        if lexer.next_token().is_ok() {
            println!("verify token: {:?} {}", lexer.current(), exp);
            assert!(matches!(lexer.current(), Token::Number(v) if v.eq(exp)));
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

#[test]
fn test_Lexer_parse_string() {
    let mut src = reader::InlineSourceReader::new(r#""hello" 'world' '"' "'" '\'' "\"" "#);
    let mut lexer = Lexer::new(&mut src);

    let mut verify = |exp: &str| {
        if lexer.next_token().is_ok() {
            println!("verify token: {:?} {}", lexer.current(), exp);
            assert!(matches!(lexer.current(), Token::Str(v) if v.eq(exp)));
            println!("verify token: {} success", exp);
        } else {
            println!("verify token: {} failed", exp);
            panic!("next token failed")
        }
    };

    verify("hello");
    verify("world");
    verify("\"");
    verify("'");
    verify("'");
    verify("\"");
}

#[test]
fn test_Lexer_parse_regular() {
    let mut src = reader::InlineSourceReader::new(r#"/.*?/ /^.*?\/$/ /[\]]/ "#);
    let mut lexer = Lexer::new(&mut src);

    let mut verify = |exp: &str| {
        if lexer.next_token().is_ok() {
            println!("verify token: {:?} {}", lexer.current(), exp);
            assert!(matches!(lexer.current(), Token::Regular(v) if v.eq(exp)));
            println!("verify token: {} success", exp);
        } else {
            println!("verify token: {} failed", exp);
            panic!("next token failed")
        }
    };

    verify(".*?");
    verify("^.*?\\/$");
    verify("[\\]]");
}

#[test]
fn test_Lexer_parse_template() {
    let mut src = reader::InlineSourceReader::new(r#"`hello ${world}${`你${好}`} foo ${bar}`"#);
    let mut lexer = Lexer::new(&mut src);

    let mut verify = |exp: Token| {
        if lexer.next_token().is_err() {
            println!("verify token: {:?} failed", exp);
            panic!("next token failed")
        }
        println!("verify token: {:?} {:?}", lexer.current(), exp);
        assert_eq!(lexer.current(), &exp);
        println!("verify token: {:?} success", exp);
    };

    verify(Token::TemplateHead("hello ".to_string()));
    verify(Token::IdentifierName("world".to_string()));
    verify(Token::TemplateMiddle("".to_string()));
    verify(Token::TemplateHead("你".to_string()));
    verify(Token::IdentifierName("好".to_string()));
    verify(Token::TemplateTail("".to_string()));
    verify(Token::TemplateMiddle(" foo ".to_string()));
    verify(Token::IdentifierName("bar".to_string()));
    verify(Token::TemplateTail("".to_string()));
}

use super::{code_points, lexer_error, reader, token::Token};

pub(crate) type LexerResult = Result<Token, lexer_error::LexerError>;

/// 词法分析器
///
/// 用于将 EMCAScript 源码拆解分析成为一组 Token
pub(crate) struct Lexer<'s> {
    reader: &'s mut dyn reader::SourceReader,

    line_number: usize,
    last_line: usize,

    tokenbuf: String,
    token: Token,
}

impl<'s> Lexer<'s> {
    /// 构建一个词法分析器
    ///
    /// # Arguments
    /// `reader` - EMCAScript 源码读取器
    /// # Returns
    /// 返回一个 EMCAScript 词法分析器
    pub(crate) fn new(reader: &'s mut dyn reader::SourceReader) -> Self {
        Self {
            reader,
            line_number: 1,
            last_line: 1,

            tokenbuf: String::new(),
            token: Token::EOF,
        }
    }

    /// 跳过 LineTerminator 或 LineTerminatorSequence，并将 line_number += 1
    ///
    /// LineTerminator ::
    ///     <LF>
    ///     <CR>
    ///     <LS>
    ///     <PS>
    ///
    /// LineTerminatorSequence ::
    ///     <LF>
    ///     <CR> [lookahead != <LF>]
    ///     <LS>
    ///     <PS>
    ///     <CR> <LF>
    fn newline(&mut self) {
        let chr = self.reader.read(0);

        if matches!(chr, None) {
            return;
        }
        if matches!(chr, Some(chr) if !code_points::is_line_terminator(chr)) {
            return;
        }

        self.reader.next();
        self.line_number += 1;

        // 如果是 <CR>, 则进一步判断下一个字符是否是 <LF>
        // 若凑成 <CR><LF>，即命中 LineTerminatorSequence 词法规则，则消费掉后续的 <LF>
        if !matches!(chr, Some(code_points::CR)) {
            return;
        }
        let chr = self.reader.read(0);
        if !matches!(chr, Some(code_points::LF)) {
            return;
        }

        self.reader.next();
    }

    /// 解析注释
    ///
    /// Comment ::
    ///     MultiLineComment
    ///     SingleLineComment
    ///
    /// # Returns
    /// 返回注释 Token （此处应直接摒弃）
    fn parse_comment(&mut self) -> LexerResult {
        panic!("not implemented")
    }

    /// 解析多行注释
    ///
    /// MultiLineComment ::
    ///     `/*` [<MultiLineCommentChars>] `*/`
    ///
    /// MultiLineCommentChars ::
    ///     MultiLineNotAsteriskChar [MultiLineCommentChars]
    ///     `*` [PostAsteriskCommentChars]
    ///
    /// MultiLineNotAsteriskChar ::
    ///     SourceCharacter (but not `*`)
    ///
    /// PostAsteriskCommentChars ::
    ///     MultiLineNotForwardSlashOrAsteriskChar [MultiLineCommentChars]
    ///     `*` [PostAsteriskCommentChars]
    ///
    /// MultiLineNotForwardSlashOrAsteriskChar ::
    ///     SourceCharacter (but not one of `/` or `*`)
    ///
    /// # Returns
    /// 返回注释 Token （此处应直接摒弃）
    fn parse_multiline_comment(&mut self) -> LexerResult {
        panic!("not implemented")
    }

    /// 解析单行注释
    ///
    /// SingleLineComment ::
    ///     `//` [SingleLineCommentChars]
    ///
    /// SingleLineCommentChars ::
    ///     SingleLineCommentChar [SingleLineCommentChars]
    ///
    /// SingleLineCommentChar ::
    ///     SourceCharacter (but not LineTerminator)
    ///
    /// # Returns
    /// 返回注释 Token （此处应直接摒弃）
    fn parse_singleline_comment(&mut self) -> LexerResult {
        panic!("not implemented")
    }

    /// 从 EMCAScript 源码的当前游标起进行扫描，获取下一个 Token
    ///
    /// # Returns
    /// 返回下一个 Token
    fn scan(&mut self) -> LexerResult {
        self.tokenbuf.clear();

        loop {
            let chr = self.reader.read(0);

            match chr {
                None => return Ok(Token::EOF),
                Some(chr) if code_points::is_line_terminator(chr) => {
                    self.newline();
                    continue;
                }
                Some(chr) if code_points::is_whitespace(chr) => {
                    self.reader.next();
                    continue;
                }

                Some('/') => {
                    let ahead_chr = self.reader.read(1);
                    if matches!(ahead_chr, Some('*' | '/')) {
                        return self.parse_comment();
                    }

                    panic!("not implemented")
                }
                Some(_) => {
                    panic!("not implemented")
                }
            }
        }
    }
}

use super::{code_points, lexer_error, reader, token::Token};

type LexerResult = Result<Token, lexer_error::LexerError>;
type LexerResultOnlyErr = Result<(), lexer_error::LexerError>;

/// 词法分析器
///
/// 用于将 EMCAScript 源码拆解分析成为一组 Token
pub(crate) struct Lexer<'s> {
    reader: &'s mut dyn reader::SourceReader,
    current_chr: Option<char>,
    lookahead_chr: Option<char>,

    line_number: usize,
    line_off: usize,

    tokenbuf: String,
}

impl<'s> Lexer<'s> {
    /// 构建一个词法分析器
    ///
    /// # Arguments
    /// `reader` - EMCAScript 源码读取器
    /// # Returns
    /// 返回一个 EMCAScript 词法分析器
    pub(crate) fn new(reader: &'s mut dyn reader::SourceReader) -> Self {
        let mut result = Self {
            reader,
            current_chr: None,
            lookahead_chr: None,

            line_number: 1,
            line_off: 1,

            tokenbuf: String::new(),
        };
        result.next(1);

        result
    }

    /// 将源码游标向下移动，并更新对应游标指向的字符
    fn next(&mut self, off: usize) {
        self.reader.next(off as isize);

        self.line_off += off;
    }

    /// 获取当前游标指向的字符
    #[inline(always)]
    fn current(&self) -> Option<char> {
        self.reader.current()
    }

    /// 获取游标指向的下一个字符
    #[inline(always)]
    fn lookahead(&self) -> Option<char> {
        self.reader.lookahead()
    }

    /// 保存字符到 token buffer
    #[inline(always)]
    fn save(&mut self, chr: char) {
        self.tokenbuf.push(chr);
    }

    /// 保存字符到 token buffer，并将游标向下移动
    #[inline(always)]
    fn savenext(&mut self, chr: char) {
        self.save(chr);
        self.next(1);
    }

    /// 清空 token buffer
    #[inline(always)]
    fn clear(&mut self) {
        self.tokenbuf.clear()
    }

    /// 将 token buffer 中的字符串获取出来
    #[inline(always)]
    fn get_tokenbuf(&self) -> String {
        self.tokenbuf.clone()
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
        if matches!(self.current(), None) {
            return;
        }
        if matches!(self.current(), Some(chr) if !code_points::is_line_terminator(chr)) {
            return;
        }

        self.line_number += 1;
        self.line_off = 1;

        // 如果是 <CR>, 则进一步判断下一个字符是否是 <LF>
        // 若凑成 <CR><LF>，即命中 LineTerminatorSequence 词法规则，则消费掉后续的 <LF>
        if !matches!(self.current(), Some(code_points::CR)) {
            self.next(1);
            return;
        }
        if !matches!(self.lookahead(), Some(code_points::LF)) {
            self.next(1);
            return;
        }
        self.next(2);
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
        match self.lookahead() {
            Some('/') => self.parse_singleline_comment()?,
            Some('*') => self.parse_multiline_comment()?,
            _ => {
                return Err(lexer_error::LexerError::new(
                    self.line_number,
                    self.line_off,
                ));
            }
        }

        Ok(Token::Comment(self.get_tokenbuf()))
    }

    /// 解析 Hashbang 注释
    ///
    /// HashbangComment ::
    ///     `#!` SingleLineCommentChars
    ///
    /// # Returns
    /// 返回 Hashbang 注释 Token
    fn parse_hashbang_comment(&mut self) -> LexerResult {
        self.parse_singleline_comment()?;
        Ok(Token::HashbangComment(self.get_tokenbuf()))
    }

    /// 解析多行注释
    ///
    /// MultiLineComment ::
    ///     `/*` [MultiLineCommentChars] `*/`
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
    fn parse_multiline_comment(&mut self) -> Result<(), lexer_error::LexerError> {
        self.next(2);

        loop {
            match self.current() {
                None => {
                    return Err(lexer_error::LexerError::new(
                        self.line_number,
                        self.line_off,
                    ))
                }
                Some('*') if matches!(self.lookahead(), Some('/')) => {
                    self.next(2);
                    break;
                }
                Some(chr) if code_points::is_line_terminator(chr) => {
                    self.save('\n');
                    self.newline();
                }
                Some(chr) => self.savenext(chr),
            }
        }

        Ok(())
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
    fn parse_singleline_comment(&mut self) -> LexerResultOnlyErr {
        self.next(2);

        loop {
            match self.current() {
                None => break,
                Some(chr) if code_points::is_line_terminator(chr) => {
                    self.newline();
                    break;
                }
                Some(chr) => self.savenext(chr),
            }
        }

        Ok(())
    }

    /// 解析 Unicode Escape Sequence
    ///
    /// UnicodeEscapeSequence ::
    ///     `u` Hex4Digits
    ///     `u{` CodePoint `}`
    ///
    /// Hex4Digits ::
    ///     HexDigit HexDigit HexDigit HexDigit
    ///
    /// CodePoint ::
    ///     HexDigits
    ///
    /// HexDigits ::
    ///     HexDigit
    ///     HexDigits HexDigit
    ///     HexDigits NumbericLiteralSeparator HexDigit
    ///
    /// NumbericLiteralSeparator ::
    ///     `_`
    fn parse_unicode_escape_sequence(&mut self) -> LexerResultOnlyErr {
        if !matches!(self.current(), Some('u')) {
            return Err(lexer_error::LexerError::new(
                self.line_number,
                self.line_off,
            ));
        }
        self.next(1);

        let mut val: u32 = 0;
        if matches!(self.current(), Some('{')) {
            self.next(1);

            let mut has_digit = false;
            let mut last_digit = false;
            loop {
                match self.current() {
                    Some('}') if has_digit => break,
                    Some(chr) if code_points::is_hex_digit(chr) => {
                        has_digit = true;
                        last_digit = true;

                        if let Some(digit) = chr.to_digit(16) {
                            val <<= 4;
                            val |= digit;
                        } else {
                            return Err(lexer_error::LexerError::new(
                                self.line_number,
                                self.line_off,
                            ));
                        }

                        if val > 0x10ffff {
                            return Err(lexer_error::LexerError::new(
                                self.line_number,
                                self.line_off,
                            ));
                        }

                        self.next(1);
                    }
                    Some('_') if last_digit => {
                        last_digit = false;

                        self.next(1);
                    }
                    _ => {
                        return Err(lexer_error::LexerError::new(
                            self.line_number,
                            self.line_off,
                        ))
                    }
                }
            }
        } else {
            for _ in 0..4 {
                match self.current() {
                    Some(chr) if code_points::is_hex_digit(chr) => {
                        if let Some(digit) = chr.to_digit(16) {
                            val <<= 4;
                            val |= digit;
                        } else {
                            return Err(lexer_error::LexerError::new(
                                self.line_number,
                                self.line_off,
                            ));
                        }

                        self.next(1);
                    }
                    _ => {
                        return Err(lexer_error::LexerError::new(
                            self.line_number,
                            self.line_off,
                        ))
                    }
                }
            }
        }

        if let Some(chr) = char::from_u32(val) {
            self.save(chr);

            Ok(())
        } else {
            Err(lexer_error::LexerError::new(
                self.line_number,
                self.line_off,
            ))
        }
    }

    /// 解析 IdentifierName
    ///
    /// IdentifierName ::
    ///     IdentifierStart
    ///     IdentifierName IdentifierPart
    ///
    /// IdentifierStart ::
    ///     IdentifierStartChar
    ///     `\` UnicodeEscapeSequence
    ///
    /// IdentifierPart ::
    ///     IdentifierPartChar
    ///     `\` UnicodeEscapeSequence
    ///
    /// IdentifierStartChar ::
    ///     UnicodeIDStart
    ///     `$`
    ///     `_`
    ///
    /// IdentifierPartChar ::
    ///     UnicodeIDContinue
    ///     `$`
    ///     <ZWNJ>
    ///     <ZWJ>
    ///
    /// # Returns
    /// 返回解析过程是否成功
    fn parse_identifier_name_part(&mut self) -> LexerResultOnlyErr {
        loop {
            match self.current() {
                Some(chr)
                    if code_points::is_id_start(chr)
                        || code_points::is_id_continue(chr)
                        || matches!(chr as u32, 0x200c | 0x200d)
                        || matches!(chr, '$' | '_') =>
                {
                    self.savenext(chr)
                }
                Some('\\') => {
                    self.next(1);

                    self.parse_unicode_escape_sequence()?;
                }
                _ => break,
            }
        }

        Ok(())
    }

    /// 解析 IdentifierName，若解析出的结果为关键字，则转换为对应的关键字 Token
    ///
    /// # Returns
    /// 返回 IdentifierName Token
    fn parse_identifier_name(&mut self) -> LexerResult {
        self.parse_identifier_name_part()?;

        let token = self.get_tokenbuf();

        Ok(match token.as_str() {
            "await" => Token::EOF,
            _ => Token::IdentifierName(token),
        })
    }

    /// 解析 PrivateIdentifier
    ///
    /// PrivateIdentifier ::
    ///     `#` IdentifierName
    ///
    /// # Returns
    /// 返回 PrivateIdentifier Token
    fn parse_private_identifier(&mut self) -> LexerResult {
        self.savenext('#');
        self.parse_identifier_name_part()?;

        Ok(Token::IdentifierName(self.get_tokenbuf()))
    }

    /// 从 EMCAScript 源码的当前游标起进行扫描，获取下一个 Token
    ///
    /// # Returns
    /// 返回下一个 Token
    pub(crate) fn next_token(&mut self) -> LexerResult {
        self.tokenbuf.clear();

        loop {
            match self.current() {
                // 结束
                None => return Ok(Token::EOF),

                // 换行
                Some(chr) if code_points::is_line_terminator(chr) => {
                    self.newline();
                    continue;
                }

                // White Space
                Some(chr) if code_points::is_whitespace(chr) => {
                    self.next(1);
                    continue;
                }

                // 注释 || 除法
                Some('/') => match self.lookahead() {
                    Some('*' | '/') => return self.parse_comment(),
                    Some('=') => {
                        self.next(2);
                        return Ok(Token::DivAssignOp);
                    }
                    _ => {
                        self.next(1);
                        return Ok(Token::DivOp);
                    }
                },

                Some('#') => match self.lookahead() {
                    // Hashbang Comment
                    Some('!') => {
                        return self.parse_hashbang_comment();
                    }

                    // PrivateIdentifier
                    Some(chr) if matches!(chr, '$' | '_') || code_points::is_id_start(chr) => {
                        return self.parse_private_identifier();
                    }

                    _ => {
                        return Err(lexer_error::LexerError::new(
                            self.line_number,
                            self.line_off,
                        ));
                    }
                },

                // IdentifierName || ReservedWord
                Some(chr) if matches!(chr, '$' | '_') || code_points::is_id_start(chr) => {
                    return self.parse_identifier_name();
                }

                _ => {
                    return Err(lexer_error::LexerError::new(
                        self.line_number,
                        self.line_off,
                    ));
                }
            }
        }
    }
}

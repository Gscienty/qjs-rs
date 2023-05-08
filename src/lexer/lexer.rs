use super::{code_points, lexer_error, reader, token::Token};

type LexerResult = Result<Token, lexer_error::LexerError>;
type LexerResultOnlyErr = Result<(), lexer_error::LexerError>;

/// 词法分析器
///
/// 用于将 EMCAScript 源码拆解分析成为一组 Token
pub(crate) struct Lexer<'s> {
    reader: &'s mut dyn reader::SourceReader,

    line_number: usize,
    line_off: usize,

    tokenbuf: String,

    tok: Token,

    template_expression: Vec<u8>,
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

            line_number: 1,
            line_off: 1,

            tokenbuf: String::new(),

            tok: Token::EOF,

            template_expression: Vec::new(),
        };
        result.next(1);

        result
    }

    /// 将源码游标向下移动，并更新对应游标指向的字符
    fn next(&mut self, off: usize) {
        self.reader.next(off as isize);

        self.line_off += off;
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

    /// 保存当前游标指向的字符到 token buffer，并移动游标到下一个字符
    ///
    /// # Arguments
    /// `n` - 保存几次
    fn savecurrent(&mut self, n: usize) {
        for _ in 0..n {
            if let Some(chr) = self.reader.current() {
                self.save(chr);
            }

            self.next(1);
        }
    }

    /// 构造一个单字符操作符 Token，并将游标向下移动
    #[inline(always)]
    fn operatornext(&mut self, chr: char) -> Token {
        self.next(1);
        Token::Operator(chr)
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

    /// 进入一个 template literal 的表达式部分
    #[inline(always)]
    fn template_enter_expression(&mut self) {
        self.template_expression.push(0);
    }

    /// 当前是否处于一个 template literal 的表达式部分中
    ///
    /// # Returns
    /// 返回当前是否处于 template literal 的表达式部分
    #[inline(always)]
    fn template_in_expression(&self) -> bool {
        !self.template_expression.is_empty()
    }

    /// 是否允许离开 template literal 的表达式部分
    ///
    /// # Returns
    /// 返回是否允许离开 template literal 的表达式部分
    #[inline(always)]
    fn template_could_leave_expression(&self) -> bool {
        if let Some(blocks) = self.template_expression.last() {
            *blocks == 0
        } else {
            false
        }
    }

    /// 离开 template literal 的表达式部分
    ///
    /// # Returns
    /// 若不允许离开，则返回报错
    #[inline(always)]
    fn template_leave_expression(&mut self) -> LexerResultOnlyErr {
        if let Some(blocks) = self.template_expression.pop() {
            if blocks != 0 {
                return Err(lexer_error::LexerError::new(
                    self.line_number,
                    self.line_off,
                ));
            }
            Ok(())
        } else {
            Err(lexer_error::LexerError::new(
                self.line_number,
                self.line_off,
            ))
        }
    }

    /// 在一个 template literal 的表达式中，进入一个 block
    ///
    /// # Returns
    /// 如果退出失败，
    #[inline(always)]
    fn template_expression_enter_block(&mut self) -> LexerResultOnlyErr {
        if let Some(blocks) = self.template_expression.last_mut() {
            *blocks += 1;

            Ok(())
        } else {
            Err(lexer_error::LexerError::new(
                self.line_number,
                self.line_off,
            ))
        }
    }

    /// 在一个 template literal 的表达式中，退出一个 block
    ///
    /// # Returns
    /// 如果退出失败，
    #[inline(always)]
    fn template_expression_leave_block(&mut self) -> LexerResultOnlyErr {
        if let Some(blocks) = self.template_expression.last_mut() {
            if *blocks == 0 {
                return Err(lexer_error::LexerError::new(
                    self.line_number,
                    self.line_off,
                ));
            }

            *blocks -= 1;

            Ok(())
        } else {
            Err(lexer_error::LexerError::new(
                self.line_number,
                self.line_off,
            ))
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
        if matches!(self.reader.current(), None) {
            return;
        }
        if matches!(self.reader.current(), Some(chr) if !code_points::is_line_terminator(chr)) {
            return;
        }

        self.line_number += 1;
        self.line_off = 1;

        // 如果是 <CR>, 则进一步判断下一个字符是否是 <LF>
        // 若凑成 <CR><LF>，即命中 LineTerminatorSequence 词法规则，则消费掉后续的 <LF>
        if !matches!(self.reader.current(), Some(code_points::CR)) {
            self.next(1);
            return;
        }
        if !matches!(self.reader.lookahead(), Some(code_points::LF)) {
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
        match self.reader.lookahead() {
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
    ///     `/*` MultiLineCommentChars? `*/`
    ///
    /// MultiLineCommentChars ::
    ///     MultiLineNotAsteriskChar MultiLineCommentChars?
    ///     `*` PostAsteriskCommentChars?
    ///
    /// MultiLineNotAsteriskChar ::
    ///     SourceCharacter (but not `*`)
    ///
    /// PostAsteriskCommentChars ::
    ///     MultiLineNotForwardSlashOrAsteriskChar MultiLineCommentChars?
    ///     `*` PostAsteriskCommentChars?
    ///
    /// MultiLineNotForwardSlashOrAsteriskChar ::
    ///     SourceCharacter (but not one of `/` or `*`)
    ///
    /// # Returns
    /// 返回注释 Token （此处应直接摒弃）
    fn parse_multiline_comment(&mut self) -> Result<(), lexer_error::LexerError> {
        self.next(2);

        loop {
            match self.reader.current() {
                None => {
                    return Err(lexer_error::LexerError::new(
                        self.line_number,
                        self.line_off,
                    ))
                }
                Some('*') if matches!(self.reader.lookahead(), Some('/')) => {
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
    ///     `//` SingleLineCommentChars?
    ///
    /// SingleLineCommentChars ::
    ///     SingleLineCommentChar SingleLineCommentChars?
    ///
    /// SingleLineCommentChar ::
    ///     SourceCharacter (but not LineTerminator)
    ///
    /// # Returns
    /// 返回注释 Token （此处应直接摒弃）
    fn parse_singleline_comment(&mut self) -> LexerResultOnlyErr {
        self.next(2);

        loop {
            match self.reader.current() {
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
    ///     HexDigits+
    ///
    /// HexDigits ::
    ///     HexDigit
    ///     HexDigits HexDigit
    ///     HexDigits NumbericLiteralSeparator HexDigit
    ///
    /// NumbericLiteralSeparator ::
    ///     `_`
    fn parse_unicode_escape_sequence(&mut self) -> LexerResultOnlyErr {
        if !matches!(self.reader.current(), Some('u')) {
            return Err(lexer_error::LexerError::new(
                self.line_number,
                self.line_off,
            ));
        }
        self.next(1);

        let mut val: u32 = 0;
        if matches!(self.reader.current(), Some('{')) {
            self.next(1);

            let mut has_digit = false;
            let mut last_digit = false;
            loop {
                match self.reader.current() {
                    Some('}') if has_digit => break,
                    Some(chr) if chr.is_digit(16) => {
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
                match self.reader.current() {
                    Some(chr) if chr.is_digit(16) => {
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
            match self.reader.current() {
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
            "await" => Token::Await,
            "break" => Token::Break,
            "case" => Token::Case,
            "catch" => Token::Catch,
            "class" => Token::Class,
            "const" => Token::Const,
            "continue" => Token::Continue,
            "debugger" => Token::Debugger,
            "default" => Token::Default,
            "delete" => Token::Delete,
            "do" => Token::Do,
            "else" => Token::Else,
            "enum" => Token::Enum,
            "export" => Token::Export,
            "extends" => Token::Extends,
            "false" => Token::False,
            "finally" => Token::Finally,
            "for" => Token::For,
            "function" => Token::Function,
            "if" => Token::If,
            "import" => Token::Import,
            "in" => Token::In,
            "instanceof" => Token::InstanceOf,
            "new" => Token::New,
            "null" => Token::Null,
            "return" => Token::Return,
            "super" => Token::Super,
            "switch" => Token::Switch,
            "this" => Token::This,
            "throw" => Token::Throw,
            "true" => Token::True,
            "try" => Token::Try,
            "typeof" => Token::TypeOf,
            "var" => Token::Var,
            "void" => Token::Void,
            "while" => Token::While,
            "with" => Token::With,
            "yield" => Token::Yield,
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

    /// 解析数字
    ///
    /// NumbericLiteral ::
    ///     DecimalLiteral
    ///     DecimalBigIntegerLiteral
    ///     NonDecimalIntegerLiteral+
    ///     NonDecimalIntegerLiteral+ BigIntLiteralSuffix
    ///     LegacyOctalIntegerLiteral
    ///
    /// DecimalLiteral ::
    ///     DecimalIntegerLiteral `.` [DecimalDigits] [ExponentPart]
    ///     `.` [DecimalDigits] [ExponentPart]
    ///     DecimalIntegerLiteral [ExponentPart]
    ///
    /// ExponentPart ::
    ///     ExponentIndicator SignedInteger
    ///
    /// ExponentIndicator ::
    ///     `e`
    ///     `E`
    ///
    /// SignedInteger ::
    ///     DecimalDigits
    ///     `+` DecimalDigits
    ///     `-` DecimalDigits
    ///
    /// DecimalBigIntegerLiteral ::
    ///     `0` BigIntLiteralSuffix
    ///     NonZeroDigit [DecimalDigits] BigIntLiteralSuffix
    ///     NonZeroDigit NumbericLiteralSeparator DecimalDigits BigIntLiteralSuffix
    ///
    /// NonDecimalIntegerLiteral ::
    ///     BinaryIntegerLiteral
    ///     OctalIntegerLiteral
    ///     HexIntegerLiteral
    ///
    /// BinaryIntegerLiteral ::
    ///     `0b` BinaryDigits
    ///     `0B` BinaryDigits
    ///
    /// OctalIntegerLiteral ::
    ///     `0o` OctalDigits
    ///     `0O` OctalDigits
    ///
    /// HexIntegerLiteral ::
    ///     `0x` HexDigits
    ///     `0X` HexDigits
    fn parse_number(&mut self) -> LexerResult {
        enum NumberType {
            MustDecimal,
            MustOctal,
            MustBinary,
            MustHex,
            MaybeOctal,
        }

        let mut has_digit = false;
        let mut only_dec = false;
        let mut may_allow_exp = false;
        let mut allow_exp = false;
        let mut allow_dot = false;
        let mut number_type = match self.reader.current() {
            Some('0') if matches!(self.reader.lookahead(), Some('b' | 'B')) => {
                self.savecurrent(2);
                NumberType::MustBinary
            }
            Some('0') if matches!(self.reader.lookahead(), Some('o' | 'O')) => {
                self.savecurrent(2);
                NumberType::MustOctal
            }
            Some('0') if matches!(self.reader.lookahead(), Some('x' | 'X')) => {
                self.savecurrent(2);
                NumberType::MustHex
            }
            Some('0') => {
                self.savecurrent(1);
                only_dec = true;
                allow_dot = true;
                allow_exp = true;
                NumberType::MaybeOctal
            }
            Some('.') => {
                self.savecurrent(1);
                may_allow_exp = true;
                NumberType::MustDecimal
            }
            _ => {
                self.savecurrent(1);
                has_digit = true;
                only_dec = true;
                allow_dot = true;
                allow_exp = true;
                NumberType::MustDecimal
            }
        };

        loop {
            match self.reader.current() {
                Some('n') if only_dec => {
                    self.savecurrent(1);
                    break;
                }
                Some('e' | 'E')
                    if allow_exp && matches!(self.reader.lookahead(), Some('+' | '-')) =>
                {
                    allow_exp = false;
                    only_dec = false;
                    has_digit = false;
                    allow_dot = false;
                    may_allow_exp = false;
                    number_type = NumberType::MustDecimal;
                    self.savecurrent(2);
                }
                Some('e' | 'E') if allow_exp => {
                    allow_exp = false;
                    only_dec = false;
                    has_digit = false;
                    allow_dot = false;
                    may_allow_exp = false;
                    number_type = NumberType::MustDecimal;
                    self.savecurrent(1);
                }
                Some('.') if allow_dot => {
                    allow_dot = false;
                    only_dec = false;
                    number_type = NumberType::MustDecimal;
                    self.savecurrent(1);
                }
                Some('_')
                    if has_digit
                        && matches!(number_type, NumberType::MustHex)
                        && matches!(self.reader.lookahead(), Some(chr) if chr.is_digit(16)) =>
                {
                    self.savecurrent(2);
                }
                Some('_')
                    if has_digit
                        && matches!(
                            number_type,
                            NumberType::MustDecimal | NumberType::MaybeOctal
                        )
                        && matches!(self.reader.lookahead(), Some(chr) if chr.is_digit(10)) =>
                {
                    if may_allow_exp {
                        may_allow_exp = false;
                        allow_exp = true;
                    }
                    self.savecurrent(2);
                }
                Some('_')
                    if has_digit
                        && matches!(number_type, NumberType::MustOctal)
                        && matches!(self.reader.lookahead(), Some(chr) if chr.is_digit(8)) =>
                {
                    self.savecurrent(2);
                }
                Some('_')
                    if has_digit
                        && matches!(number_type, NumberType::MustBinary)
                        && matches!(self.reader.lookahead(), Some(chr) if chr.is_digit(2)) =>
                {
                    self.savecurrent(2);
                }
                Some('0'..='7') if matches!(number_type, NumberType::MaybeOctal) => {
                    has_digit = true;
                    if may_allow_exp {
                        may_allow_exp = false;
                        allow_exp = true;
                    }
                    self.savecurrent(1);
                }
                Some('8'..='9') if matches!(number_type, NumberType::MaybeOctal) => {
                    number_type = NumberType::MustDecimal;
                    has_digit = true;
                    if may_allow_exp {
                        may_allow_exp = false;
                        allow_exp = true;
                    }
                    self.savecurrent(1);
                }
                Some(chr) if matches!(number_type, NumberType::MustHex) && chr.is_digit(16) => {
                    has_digit = true;
                    self.savenext(chr);
                }
                Some(chr) if matches!(number_type, NumberType::MustDecimal) && chr.is_digit(10) => {
                    has_digit = true;
                    if may_allow_exp {
                        may_allow_exp = false;
                        allow_exp = true;
                    }
                    self.savenext(chr);
                }
                Some(chr) if matches!(number_type, NumberType::MustOctal) && chr.is_digit(8) => {
                    has_digit = true;
                    self.savenext(chr);
                }
                Some(chr) if matches!(number_type, NumberType::MustBinary) && chr.is_digit(2) => {
                    has_digit = true;
                    self.savenext(chr);
                }
                _ => break,
            }
        }

        Ok(Token::Number(self.get_tokenbuf()))
    }

    /// 解析字符串内容字符
    ///
    /// # Returns
    /// 返回解析是否成功
    fn parse_string_content(&mut self) -> LexerResultOnlyErr {
        match self.reader.current() {
            Some('\u{2028}' | '\u{2029}') => self.savecurrent(1),
            Some('\\') if matches!(self.reader.lookahead(), Some(chr) if code_points::is_line_terminator(chr)) =>
            {
                self.save('\n');
                self.newline();
            }
            Some('\\') => {
                self.next(1);
                match self.reader.current() {
                    Some('\'') => self.savecurrent(1),
                    Some('\"') => self.savecurrent(1),
                    Some('\\') => self.savecurrent(1),
                    Some('b') => self.savenext('\x08'),
                    Some('f') => self.savenext('\x0c'),
                    Some('n') => self.savenext('\n'),
                    Some('r') => self.savenext('\r'),
                    Some('t') => self.savenext('\t'),
                    Some('v') => self.savenext('\x0b'),
                    Some('0') if !matches!(self.reader.lookahead(), Some(chr) if chr.is_digit(10)) =>
                    {
                        self.savenext('\0');
                    }
                    Some('0') if matches!(self.reader.lookahead(), Some('8' | '9')) => {
                        self.savenext('\0');
                    }
                    Some('0'..='3') if matches!(self.reader.lookahead(), Some(chr) if chr.is_digit(8)) =>
                    {
                        let mut val = 0u32;
                        if let Some(oct) = self.reader.current().and_then(|x| x.to_digit(8)) {
                            val |= oct;
                        }
                        self.next(1);

                        if let Some(oct) = self.reader.current().and_then(|x| x.to_digit(8)) {
                            val <<= 3;
                            val |= oct;
                        }
                        self.next(1);

                        if matches!(self.reader.current(), Some(chr) if chr.is_digit(8)) {
                            if let Some(oct) = self.reader.current().and_then(|x| x.to_digit(8)) {
                                val <<= 3;
                                val |= oct;
                            }
                            self.next(1);
                        }

                        if let Some(chr) = char::from_u32(val) {
                            self.save(chr);
                        } else {
                            return Err(lexer_error::LexerError::new(
                                self.line_number,
                                self.line_off,
                            ));
                        }
                    }
                    Some('4'..='7') if matches!(self.reader.lookahead(), Some(chr) if chr.is_digit(8)) =>
                    {
                        let mut val = 0u32;
                        for _ in 0..2 {
                            if let Some(digit) = self.reader.current().and_then(|x| x.to_digit(8)) {
                                val <<= 3;
                                val |= digit;
                            } else {
                                return Err(lexer_error::LexerError::new(
                                    self.line_number,
                                    self.line_off,
                                ));
                            }
                            self.next(1);
                        }
                        if let Some(chr) = char::from_u32(val) {
                            self.save(chr);
                        } else {
                            return Err(lexer_error::LexerError::new(
                                self.line_number,
                                self.line_off,
                            ));
                        }
                    }
                    Some('1'..='7') if !matches!(self.reader.lookahead(), Some(chr) if chr.is_digit(8)) => {
                        if let Some(chr) = self
                            .reader
                            .current()
                            .and_then(|x| x.to_digit(8))
                            .and_then(|x| char::from_u32(x))
                        {
                            self.savenext(chr);
                        } else {
                            return Err(lexer_error::LexerError::new(
                                self.line_number,
                                self.line_off,
                            ));
                        }
                    }
                    Some('x') => {
                        self.next(1);

                        let mut val = 0u32;
                        for _ in 0..2 {
                            if let Some(digit) = self.reader.current().and_then(|x| x.to_digit(16))
                            {
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
                        if let Some(chr) = char::from_u32(val) {
                            self.save(chr);
                        } else {
                            return Err(lexer_error::LexerError::new(
                                self.line_number,
                                self.line_off,
                            ));
                        }
                    }
                    Some('u') => self.parse_unicode_escape_sequence()?,
                    _ => {
                        return Err(lexer_error::LexerError::new(
                            self.line_number,
                            self.line_off,
                        ))
                    }
                }
            }
            Some(chr) => self.savenext(chr),
            _ => {
                return Err(lexer_error::LexerError::new(
                    self.line_number,
                    self.line_off,
                ))
            }
        }
        Ok(())
    }

    /// 解析字符串
    ///
    /// StringLiteral ::
    ///     `"` [DoubleStringCharacters] `"`
    ///     `'` [SingleStringCharacters] `'`
    ///
    /// DoubleStringCharacters ::
    ///     DoubleStringCharacter [DoubleStringCharacters]
    ///
    /// SingleStringCharacters ::
    ///     SingleStringCharacter [SingleStringCharacters]
    ///
    /// DoubleStringCharacter ::
    ///     SourceCharacter (but not one of `"` or `\` or LineTerminator)
    ///     <LS>
    ///     <PS>
    ///     \ EscapeSequence
    ///     LineContinuation
    ///
    /// SingleStringCharacter ::
    ///     SourceCharacter (but not one of `'` or `\` or LineTerminator)
    ///     <LS>
    ///     <PS>
    ///     \ EscapeSequence
    ///     LineContinuation
    fn parse_string(&mut self) -> LexerResult {
        let quota = self.reader.current();
        self.next(1);

        loop {
            if self.reader.current().eq(&quota) {
                self.next(1);
                break;
            }

            self.parse_string_content()?;
        }

        Ok(Token::Str(self.get_tokenbuf()))
    }

    /// 解析 template
    ///
    /// Template ::
    ///     NoSubstitutionTemplate
    ///     TemplateHead
    ///
    /// NoSubstitutionTemplate ::
    ///     ``` TemplateCharacters ```
    ///
    /// TemplateHead ::
    ///     ``` TemplateCharacters `${`
    ///
    /// TemplateSubstitutionTail ::
    ///     TemplateMiddle
    ///     TemplateTail
    ///
    /// TemplateMiddle ::
    ///     `}` TemplateCharacters `${`
    ///
    /// TemplateTail ::
    ///     `}` TemplateCharacters ```
    fn parse_template(&mut self) -> LexerResult {
        let is_head = matches!(self.reader.current(), Some('`'));
        self.next(1);

        loop {
            match self.reader.current() {
                Some('`') => {
                    self.next(1);
                    break Ok(if is_head {
                        Token::Str(self.get_tokenbuf())
                    } else {
                        Token::TemplateTail(self.get_tokenbuf())
                    });
                }
                Some('$') if matches!(self.reader.lookahead(), Some('{')) => {
                    self.next(2);
                    self.template_enter_expression();
                    break Ok(if is_head {
                        Token::TemplateHead(self.get_tokenbuf())
                    } else {
                        Token::TemplateMiddle(self.get_tokenbuf())
                    });
                }
                _ => self.parse_string_content()?,
            }
        }
    }

    /// 解析正则表达式
    ///
    /// RegularExpressionLiteral ::
    ///     `/` RegularExpressionBody `/` RegularExpressionFlags
    ///
    /// RegularExpressionBody ::
    ///     RegularExpressionFirstChar RegularExpressionChars
    ///
    /// RegularExpressionFirstChar ::
    ///     RegularExpressionNonTerminator (but not one of `*` or `\` or `/` or `[`)
    ///     RegularExpressionBackslashSequence
    ///     RegularExpressionClass
    ///
    /// RegularExpressionBackslashSequence ::
    ///     `\` RegularExpressionNonTerminator
    ///
    /// RegularExpressionClass ::
    ///     `[` RegularExpressionClassChars `]`
    ///
    /// RegularExpressionClassChars ::
    ///     [empty]
    ///     RegularExpressionClassChars RegularExpressionClassChar
    ///
    /// RegularExpressionClassChar ::
    ///     RegularExpressionNonTerminator (but not one of `]` or `\`)
    ///     RegularExpressionBackslashSequence
    ///
    /// RegularExpressionFlags ::
    ///     [empty]
    ///     RegularExpressionFlags IdentifierPartChar
    fn parse_regular(&mut self) -> LexerResult {
        self.next(1);

        let mut class_depth = 0;
        loop {
            if matches!(self.reader.current(), Some('/')) {
                self.next(1);
                break;
            }

            match self.reader.current() {
                Some('\\') if matches!(self.reader.lookahead(), Some(chr) if !code_points::is_line_terminator(chr)) => {
                    self.savecurrent(2)
                }
                Some('\\') if matches!(self.reader.lookahead(), Some(chr) if code_points::is_line_terminator(chr)) => {
                    return Err(lexer_error::LexerError::new(
                        self.line_number,
                        self.line_off,
                    ))
                }
                Some('\\') if matches!(self.reader.lookahead(), None) => {
                    return Err(lexer_error::LexerError::new(
                        self.line_number,
                        self.line_off,
                    ))
                }
                Some('[') => {
                    self.savenext('[');
                    class_depth += 1;
                }
                Some(']') if class_depth <= 0 => {
                    return Err(lexer_error::LexerError::new(
                        self.line_number,
                        self.line_off,
                    ))
                }
                Some(']') => {
                    self.savenext(']');
                    class_depth -= 1;
                }
                Some(chr) if code_points::is_line_terminator(chr) => {
                    return Err(lexer_error::LexerError::new(
                        self.line_number,
                        self.line_off,
                    ))
                }
                Some(chr) => self.savenext(chr),
                _ => {
                    return Err(lexer_error::LexerError::new(
                        self.line_number,
                        self.line_off,
                    ))
                }
            }
        }

        if class_depth != 0 {
            return Err(lexer_error::LexerError::new(
                self.line_number,
                self.line_off,
            ));
        }

        Ok(Token::Regular(self.get_tokenbuf()))
    }

    /// 获取下一个 Token
    ///
    /// # Returns
    /// 如果获取下一个 token 失败，则返回报错
    pub(crate) fn next_token(&mut self) -> LexerResultOnlyErr {
        self.tok = self.scan()?;

        Ok(())
    }

    /// 获取当前 Token
    #[inline(always)]
    pub(crate) const fn current(&self) -> &Token {
        &self.tok
    }

    /// 从 EMCAScript 源码的当前游标起进行扫描，获取下一个 Token
    ///
    /// # Returns
    /// 返回下一个 Token
    fn scan(&mut self) -> LexerResult {
        self.tokenbuf.clear();

        loop {
            match self.reader.current() {
                Some('#') if matches!(self.reader.lookahead(), Some('!')) => {
                    return self.parse_hashbang_comment(); // `#!`
                }
                Some('#')
                    if matches!(self.reader.lookahead(), Some('$' | '_'))
                        || matches!(self.reader.lookahead(), Some(chr) if code_points::is_id_start(chr)) =>
                {
                    return self.parse_private_identifier(); // PrivateIdentifier
                }

                // 注释
                Some('/') if matches!(self.reader.lookahead(), Some('*' | '/')) => {
                    return self.parse_comment()
                }
                // 正则表达式
                Some('/')
                    if !matches!(
                        self.current(),
                        Token::Number(..)
                            | Token::IdentifierName(..)
                            | Token::Str(..)
                            | Token::Operator(')' | ']')
                    ) =>
                {
                    return self.parse_regular();
                }
                // 除法运算符
                Some('/') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::DivAssign); // `/=`
                }

                Some('.') if matches!(self.reader.lookahead(), Some('0'..='9')) => {
                    return self.parse_number()
                }
                Some('.') => {
                    let op = self.operatornext('.');
                    if matches!(self.reader.current(), Some('.'))
                        && matches!(self.reader.lookahead(), Some('.'))
                    {
                        self.next(2);
                        return Ok(Token::Spread); // `...`
                    }
                    return Ok(op); // `.`
                }

                Some('<') if matches!(self.reader.lookahead(), Some('<')) => {
                    self.next(2);
                    match self.reader.current() {
                        Some('=') => {
                            self.next(1);
                            return Ok(Token::SHLAssign); // `<<=`
                        }
                        _ => return Ok(Token::SHL), // `<<`
                    }
                }
                Some('<') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::LE); // `<=`
                }

                Some('>') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::GE); // `>=`
                }
                Some('>') if matches!(self.reader.lookahead(), Some('>')) => {
                    self.next(2);
                    match self.reader.current() {
                        Some('>') if matches!(self.reader.lookahead(), Some('=')) => {
                            self.next(2);
                            return Ok(Token::USHRAssign); // `>>>=`
                        }
                        Some('>') => {
                            self.next(1);
                            return Ok(Token::USHR); // `>>>`
                        }
                        Some('=') => {
                            self.next(1);
                            return Ok(Token::SHRAssign); // `>>=`
                        }
                        _ => return Ok(Token::SHR), // `>>`
                    }
                }

                Some('=') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    if matches!(self.reader.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::StrictEqual); // `===`
                    }
                    return Ok(Token::Equal); // `==`
                }
                Some('=') if matches!(self.reader.lookahead(), Some('>')) => {
                    self.next(2);
                    return Ok(Token::ArrowFunction); // `=>`
                }

                Some('!') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    if matches!(self.reader.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::StrictNotEqual); // `!==`
                    }
                    return Ok(Token::NotEqual); // `!=`
                }

                Some('*') if matches!(self.reader.lookahead(), Some('*')) => {
                    self.next(2);
                    if matches!(self.reader.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::ExpAssign); // `**=`
                    }
                    return Ok(Token::Exp); // `**`
                }
                Some('*') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::MulAssign); // `*=`
                }

                Some('+') if matches!(self.reader.lookahead(), Some('+')) => {
                    self.next(2);
                    return Ok(Token::Incr); // `++`
                }
                Some('+') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::AddAssign); // `+=`
                }

                Some('-') if matches!(self.reader.lookahead(), Some('-')) => {
                    self.next(2);
                    return Ok(Token::Decr); // `--`
                }
                Some('-') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::SubAssign); // `-=`
                }

                Some('&') if matches!(self.reader.lookahead(), Some('&')) => {
                    self.next(2);
                    if matches!(self.reader.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::AndAssign); // `&&=`
                    }
                    return Ok(Token::And); // `&&`
                }
                Some('&') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::BitAndAssign); // `&=`
                }

                Some('|') if matches!(self.reader.lookahead(), Some('|')) => {
                    self.next(2);
                    if matches!(self.reader.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::OrAssign); // `||=`
                    }
                    return Ok(Token::Or); // `||`
                }
                Some('|') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::BitOrAssign); // `|=`
                }

                Some('^') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::XORAssign); // `^=`
                }

                Some('?') if matches!(self.reader.lookahead(), Some('?')) => {
                    self.next(2);
                    if matches!(self.reader.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::CoalNullAssign);
                    }
                    return Ok(Token::CoalNull);
                }
                Some('?') if matches!(self.reader.lookahead(), Some('.')) => {
                    self.next(2);
                    if matches!(self.reader.current(), Some(chr) if chr.is_digit(10)) {
                        return Err(lexer_error::LexerError::new(
                            self.line_number,
                            self.line_off,
                        ));
                    }
                    return Ok(Token::Chain);
                }

                Some('%') if matches!(self.reader.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::ModAssign);
                }

                // 字符串
                Some('"' | '\'') => return self.parse_string(),

                // template
                Some('`') => return self.parse_template(),

                // template literal 表达式内 `{`
                Some('{') if self.template_in_expression() => {
                    self.template_expression_enter_block()?;

                    return Ok(self.operatornext('{'));
                }
                // template literal 表达式内 `}`
                Some('}') if self.template_in_expression() => {
                    // 结束当前 template literal 表达式
                    if self.template_could_leave_expression() {
                        self.template_leave_expression()?;
                        return self.parse_template();
                    }
                    self.template_expression_leave_block()?;
                    return Ok(self.operatornext('}'));
                }

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

                // IdentifierName || ReservedWord
                Some(chr) if matches!(chr, '$' | '_') || code_points::is_id_start(chr) => {
                    return self.parse_identifier_name(); // IdentifierName
                }

                Some('0'..='9') => return self.parse_number(),

                // 单字符操作符
                Some(chr) => return Ok(self.operatornext(chr)),

                // 结束
                None => return Ok(Token::EOF),
            }
        }
    }
}

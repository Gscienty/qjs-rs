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

    /// 保存当前游标指向的字符到 token buffer，并移动游标到下一个字符
    ///
    /// # Arguments
    /// `n` - 保存几次
    fn savecurrent(&mut self, n: usize) {
        for _ in 0..n {
            if let Some(chr) = self.current() {
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
                match self.current() {
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
        let mut number_type = match self.current() {
            Some('0') if matches!(self.lookahead(), Some('b' | 'B')) => {
                self.savecurrent(2);
                NumberType::MustBinary
            }
            Some('0') if matches!(self.lookahead(), Some('o' | 'O')) => {
                self.savecurrent(2);
                NumberType::MustOctal
            }
            Some('0') if matches!(self.lookahead(), Some('x' | 'X')) => {
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
            match self.current() {
                Some('n') if only_dec => {
                    self.savecurrent(1);
                    break;
                }
                Some('e' | 'E') if allow_exp && matches!(self.lookahead(), Some('+' | '-')) => {
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
                        && matches!(self.lookahead(), Some(chr) if chr.is_digit(16)) =>
                {
                    self.savecurrent(2);
                }
                Some('_')
                    if has_digit
                        && matches!(
                            number_type,
                            NumberType::MustDecimal | NumberType::MaybeOctal
                        )
                        && matches!(self.lookahead(), Some(chr) if chr.is_digit(10)) =>
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
                        && matches!(self.lookahead(), Some(chr) if chr.is_digit(8)) =>
                {
                    self.savecurrent(2);
                }
                Some('_')
                    if has_digit
                        && matches!(number_type, NumberType::MustBinary)
                        && matches!(self.lookahead(), Some(chr) if chr.is_digit(2)) =>
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

    /// 从 EMCAScript 源码的当前游标起进行扫描，获取下一个 Token
    ///
    /// # Returns
    /// 返回下一个 Token
    pub(crate) fn next_token(&mut self) -> LexerResult {
        self.tokenbuf.clear();

        loop {
            match self.current() {
                Some('#') if matches!(self.lookahead(), Some('!')) => {
                    return self.parse_hashbang_comment(); // `#!`
                }
                Some('#')
                    if matches!(self.lookahead(), Some('$' | '_'))
                        || matches!(self.lookahead(), Some(chr) if code_points::is_id_start(chr)) =>
                {
                    return self.parse_private_identifier(); // PrivateIdentifier
                }

                Some('/') if matches!(self.lookahead(), Some('*' | '/')) => {
                    return self.parse_comment()
                }
                Some('/') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::DivAssign); // `/=`
                }

                Some('.') if matches!(self.lookahead(), Some('0'..='9')) => {
                    return self.parse_number()
                }
                Some('.') => {
                    let op = self.operatornext('.');
                    if matches!(self.current(), Some('.')) && matches!(self.lookahead(), Some('.'))
                    {
                        self.next(2);
                        return Ok(Token::Spread); // `...`
                    }
                    return Ok(op); // `.`
                }

                Some('<') if matches!(self.lookahead(), Some('<')) => {
                    self.next(2);
                    match self.current() {
                        Some('=') => {
                            self.next(1);
                            return Ok(Token::SHLAssign); // `<<=`
                        }
                        _ => return Ok(Token::SHL), // `<<`
                    }
                }
                Some('<') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::LE); // `<=`
                }

                Some('>') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::GE); // `>=`
                }
                Some('>') if matches!(self.lookahead(), Some('>')) => {
                    self.next(2);
                    match self.current() {
                        Some('>') if matches!(self.lookahead(), Some('=')) => {
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

                Some('=') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    if matches!(self.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::StrictEqual); // `===`
                    }
                    return Ok(Token::Equal); // `==`
                }
                Some('=') if matches!(self.lookahead(), Some('>')) => {
                    self.next(2);
                    return Ok(Token::ArrowFunction); // `=>`
                }

                Some('!') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    if matches!(self.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::StrictNotEqual); // `!==`
                    }
                    return Ok(Token::NotEqual); // `!=`
                }

                Some('*') if matches!(self.lookahead(), Some('*')) => {
                    self.next(2);
                    if matches!(self.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::ExpAssign); // `**=`
                    }
                    return Ok(Token::Exp); // `**`
                }
                Some('*') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::MulAssign); // `*=`
                }

                Some('+') if matches!(self.lookahead(), Some('+')) => {
                    self.next(2);
                    return Ok(Token::Incr); // `++`
                }
                Some('+') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::AddAssign); // `+=`
                }

                Some('-') if matches!(self.lookahead(), Some('-')) => {
                    self.next(2);
                    return Ok(Token::Decr); // `--`
                }
                Some('-') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::SubAssign); // `-=`
                }

                Some('&') if matches!(self.lookahead(), Some('&')) => {
                    self.next(2);
                    if matches!(self.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::AndAssign); // `&&=`
                    }
                    return Ok(Token::And); // `&&`
                }
                Some('&') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::BitAndAssign); // `&=`
                }

                Some('|') if matches!(self.lookahead(), Some('|')) => {
                    self.next(2);
                    if matches!(self.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::OrAssign); // `||=`
                    }
                    return Ok(Token::Or); // `||`
                }
                Some('|') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::BitOrAssign); // `|=`
                }

                Some('^') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::XORAssign); // `^=`
                }

                Some('?') if matches!(self.lookahead(), Some('?')) => {
                    self.next(2);
                    if matches!(self.current(), Some('=')) {
                        self.next(1);
                        return Ok(Token::CoalNullAssign);
                    }
                    return Ok(Token::CoalNull);
                }
                Some('?') if matches!(self.lookahead(), Some('.')) => {
                    self.next(2);
                    if matches!(self.current(), Some(chr) if chr.is_digit(10)) {
                        return Err(lexer_error::LexerError::new(
                            self.line_number,
                            self.line_off,
                        ));
                    }
                    return Ok(Token::Chain);
                }

                Some('%') if matches!(self.lookahead(), Some('=')) => {
                    self.next(2);
                    return Ok(Token::ModAssign);
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

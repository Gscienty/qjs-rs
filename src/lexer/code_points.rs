/// 判断当前字符是否为 White Space
///
/// WhiteSpace ::
///     <TAB>
///     <VT>
///     <FF>
///     <ZWNBSP>
///     <USP>
///
/// <TAB> == 0x0009
/// <VT> == 0x000b
/// <FF> == 0x000c
/// <ZWNBSP> == 0xfeff
/// <USP> == Unicode 空间中的 Space_Separator
///
/// # Arguments
/// `chr` - 字符
/// # Returns
/// 返回当前字段是否是 White Space
#[inline(always)]
pub(super) fn is_whitespace(chr: char) -> bool {
    match chr {
        '\u{0009}' | '\u{000b}' | '\u{000c}' | '\u{feff}' => true,
        _ => chr.is_whitespace(),
    }
}

pub(super) const LF: char = '\u{000a}';
pub(super) const CR: char = '\u{000d}';
pub(super) const LS: char = '\u{2028}';
pub(super) const PS: char = '\u{2029}';

/// 判断当前字符是否为行终止符 (Line Terminators)
///
/// # Arguments
/// `chr` - 字符
/// # Returns
/// 返回当前字段是否是 Line Terminators
#[inline(always)]
pub(super) const fn is_line_terminator(chr: char) -> bool {
    match chr {
        '\u{000a}' | '\u{000d}' | '\u{2028}' | '\u{2029}' => true,
        _ => false,
    }
}

/// 判断当前字符是否为 ID Start
///
/// # Arguments
/// `chr` - 字符
/// # Returns
/// 返回当前字段是否是 ID Start
#[inline(always)]
pub(super) const fn is_id_start(chr: char) -> bool {
    matches!(chr, 
        | '\u{0041}'..='\u{005a}'
        | '\u{0061}'..='\u{007a}'
        | '\u{00aa}'
        | '\u{00b5}'
        | '\u{00ba}'
        | '\u{00c0}'..='\u{00d6}'
        | '\u{00d8}'..='\u{00f6}'
        | '\u{00f8}'..='\u{02ff}'
        | '\u{0370}'..='\u{037d}'
        | '\u{037f}'..='\u{1fff}'
        | '\u{200c}'..='\u{200d}'
        | '\u{2070}'..='\u{218f}'
        | '\u{2c00}'..='\u{2fef}'
        | '\u{3001}'..='\u{d7ff}'
        | '\u{f900}'..='\u{fdff}'
        | '\u{fe70}'..='\u{fefe}'
        | '\u{ff10}'..='\u{ff19}'
        | '\u{ff21}'..='\u{ff3a}'
        | '\u{ff41}'..='\u{ff5a}'
        | '\u{ff65}'..='\u{ffdc}')
}

/// 判断当前字符是否为 ID Continue
///
/// # Arguments
/// `chr` - 字符
/// # Returns
/// 返回当前字段是否是 ID Continue
pub(super) const fn is_id_continue(chr: char) -> bool {
    matches!(chr,
        | '\u{0030}'..='\u{0039}'
        | '\u{0041}'..='\u{005a}'
        | '\u{005f}'
        | '\u{0061}'..='\u{007a}'
        | '\u{00aa}'
        | '\u{00b5}'
        | '\u{00ba}'
        | '\u{00c0}'..='\u{00d6}'
        | '\u{00d8}'..='\u{00f6}'
        | '\u{00f8}'..='\u{02ff}'
        | '\u{0300}'..='\u{036f}'
        | '\u{0370}'..='\u{037d}'
        | '\u{037f}'..='\u{1fff}'
        | '\u{200c}'..='\u{200d}'
        | '\u{203f}'..='\u{2040}'
        | '\u{2070}'..='\u{218f}'
        | '\u{2c00}'..='\u{2fef}'
        | '\u{3001}'..='\u{d7ff}'
        | '\u{f900}'..='\u{fdff}'
        | '\u{fe70}'..='\u{fefe}'
        | '\u{ff10}'..='\u{ff19}'
        | '\u{ff21}'..='\u{ff3a}'
        | '\u{ff41}'..='\u{ff5a}'
        | '\u{ff65}'..='\u{ffdc}')
}

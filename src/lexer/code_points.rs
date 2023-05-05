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
pub(super) fn is_whitespace(chr: char) -> bool {
    match chr as u32 {
        0x0009 | 0x000b | 0x000c | 0xfeff => true,
        _ => chr.is_whitespace(),
    }
}

pub(super) const CR: char = 0x000d as char;
pub(super) const LF: char = 0x000a as char;

/// 判断当前字符是否为行终止符 (Line Terminators)
///
/// # Arguments
/// `chr` - 字符
/// # Returns
/// 返回当前字段是否是 Line Terminators
pub(super) fn is_line_terminator(chr: char) -> bool {
    match chr as u32 {
        0x000a | 0x000d | 0x2028 | 0x2029 => true,
        _ => false,
    }
}

/// 判断当前字符是否为 Source Character
///
/// # Arguments
/// `chr` - 字符
/// # Returns
/// 返回当前字段是否是 Source Character
pub(super) fn is_source_character(chr: char) -> bool {
    match chr as u32 {
        0x0000..=0x10ffff => true,
        _ => false,
    }
}

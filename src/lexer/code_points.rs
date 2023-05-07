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
#[inline(always)]
pub(super) const fn is_line_terminator(chr: char) -> bool {
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
#[inline(always)]
pub(super) const fn is_source_character(chr: char) -> bool {
    match chr as u32 {
        0x0000..=0x10ffff => true,
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
    matches!(chr as u32, 
        | 0x0041..=0x005a
        | 0x0061..=0x007a
        | 0x00aa
        | 0x00b5
        | 0x00ba
        | 0x00c0..=0x00d6
        | 0x00d8..=0x00f6
        | 0x00f8..=0x02ff
        | 0x0370..=0x037d
        | 0x037f..=0x1fff
        | 0x200c..=0x200d
        | 0x2070..=0x218f
        | 0x2c00..=0x2fef
        | 0x3001..=0xd7ff
        | 0xf900..=0xfdff
        | 0xfe70..=0xfefe
        | 0xff10..=0xff19
        | 0xff21..=0xff3a
        | 0xff41..=0xff5a
        | 0xff65..=0xffdc)
}

/// 判断当前字符是否为 ID Continue
///
/// # Arguments
/// `chr` - 字符
/// # Returns
/// 返回当前字段是否是 ID Continue
pub(super) const fn is_id_continue(chr: char) -> bool {
    matches!(chr as u32,
        | 0x0030..=0x0039
        | 0x0041..=0x005a
        | 0x005f
        | 0x0061..=0x007a
        | 0x00aa
        | 0x00b5
        | 0x00ba
        | 0x00c0..=0x00d6
        | 0x00d8..=0x00f6
        | 0x00f8..=0x02ff
        | 0x0300..=0x036f
        | 0x0370..=0x037d
        | 0x037f..=0x1fff
        | 0x200c..=0x200d
        | 0x203f..=0x2040
        | 0x2070..=0x218f
        | 0x2c00..=0x2fef
        | 0x3001..=0xd7ff
        | 0xf900..=0xfdff
        | 0xfe70..=0xfefe
        | 0xff10..=0xff19
        | 0xff21..=0xff3a
        | 0xff41..=0xff5a
        | 0xff65..=0xffdc)
}

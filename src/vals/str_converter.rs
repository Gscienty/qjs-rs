pub(crate) mod strconv {
    use crate::vals::JSValue;

    enum ToNumberTarget {
        Binary,
        Oct,
        Decimal,
        Hex,
    }

    enum ToNumberDecimalState {
        IntPart,
        FracPart,
        ExpInitPart,
        ExpPart,
    }

    /// 根据明确的转化目标进行转化
    ///
    /// # Arguments
    /// `s` - 原始文本
    /// `target` - 文本的数字进制
    /// # Returns
    /// 返回文本转化成的数字，用 JSValue 表示
    fn to_number_with_target(s: &str, target: ToNumberTarget) -> JSValue {
        match target {
            ToNumberTarget::Binary => {
                let mut chars = s.chars().skip(2);
                let mut result = 0i64;
                while let Some(chr) = chars.next() {
                    result = match chr {
                        '0'..='1' => match chr.to_digit(2) {
                            Some(n) => (result << 1) | (n as i64),
                            _ => return JSValue::Float(f64::NAN),
                        },
                        _ => return JSValue::Float(f64::NAN),
                    }
                }
                JSValue::Int(result)
            }
            ToNumberTarget::Oct => {
                let mut chars = s.chars().skip(1).peekable();
                if matches!(chars.peek(), Some('o' | 'O')) {
                    chars.next();
                }

                let mut result = 0i64;
                while let Some(chr) = chars.next() {
                    result = match chr {
                        '0'..='7' => match chr.to_digit(8) {
                            Some(n) => (result << 3) | (n as i64),
                            _ => return JSValue::Float(f64::NAN),
                        },
                        _ => return JSValue::Float(f64::NAN),
                    };
                }
                JSValue::Int(result)
            }
            ToNumberTarget::Hex => {
                let mut chars = s.chars().skip(2);
                let mut result = 0i64;
                while let Some(chr) = chars.next() {
                    result = match chr {
                        '0'..='9' | 'a'..='f' | 'A'..='F' => match chr.to_digit(16) {
                            Some(n) => (result << 4) | (n as i64),
                            _ => return JSValue::Float(f64::NAN),
                        },
                        _ => return JSValue::Float(f64::NAN),
                    }
                }
                JSValue::Int(result)
            }
            ToNumberTarget::Decimal => {
                let mut chars = s.chars();
                let mut state = ToNumberDecimalState::IntPart;

                let mut intval = 0i64;
                let mut fracval = 0f64;
                let mut fracbase = 1f64;
                let mut expval = 0i64;

                let mut is_float = false;
                let mut has_exp = false;
                let mut negative_exp = false;

                while let Some(chr) = chars.next() {
                    match chr {
                        '0'..='9' if matches!(state, ToNumberDecimalState::IntPart) => {
                            match chr.to_digit(10) {
                                Some(n) => intval = intval * 10 + (n as i64),
                                _ => return JSValue::Float(f64::NAN),
                            }
                        }
                        '0'..='9' if matches!(state, ToNumberDecimalState::FracPart) => {
                            match chr.to_digit(10) {
                                Some(n) => {
                                    fracbase *= 0.1;
                                    fracval += fracbase * (n as f64);
                                }
                                _ => return JSValue::Float(f64::NAN),
                            }
                        }
                        '0'..='9'
                            if matches!(
                                state,
                                ToNumberDecimalState::ExpInitPart | ToNumberDecimalState::ExpPart
                            ) =>
                        {
                            state = ToNumberDecimalState::ExpPart;
                            match chr.to_digit(10) {
                                Some(n) => expval = expval * 10 + (n as i64),
                                _ => return JSValue::Float(f64::NAN),
                            }
                        }
                        '.' if matches!(state, ToNumberDecimalState::IntPart) => {
                            is_float = true;
                            fracval = intval as f64;
                            state = ToNumberDecimalState::FracPart;
                        }
                        'e' | 'E'
                            if matches!(
                                state,
                                ToNumberDecimalState::IntPart | ToNumberDecimalState::FracPart
                            ) =>
                        {
                            has_exp = true;
                            state = ToNumberDecimalState::ExpInitPart
                        }
                        '+' if matches!(state, ToNumberDecimalState::ExpInitPart) => {
                            state = ToNumberDecimalState::ExpPart;
                            negative_exp = false;
                        }
                        '-' if matches!(state, ToNumberDecimalState::ExpInitPart) => {
                            state = ToNumberDecimalState::ExpPart;
                            negative_exp = true;
                        }
                        _ => return JSValue::Float(f64::NAN),
                    }
                }

                if negative_exp {
                    expval = -expval;
                    if !is_float {
                        is_float = true;
                        fracval = intval as f64;
                    }
                }

                if is_float {
                    if has_exp {
                        JSValue::Float(fracval.powf(expval as f64))
                    } else {
                        JSValue::Float(fracval)
                    }
                } else {
                    if has_exp {
                        JSValue::Int(intval.pow(expval as u32))
                    } else {
                        JSValue::Int(intval)
                    }
                }
            }
        }
    }

    /// 将字符串转换为数字，用 JSValue 表示
    ///
    /// # Arguments
    /// `s` - 待转换为数字的字符串
    /// # Returns
    /// 返回 JSValue 表示的数字
    pub(crate) fn to_number(s: &str) -> JSValue {
        let mut chars = s.chars();

        match chars.next() {
            Some('0') => match chars.next() {
                Some('b' | 'B') => to_number_with_target(s, ToNumberTarget::Binary),
                Some('o' | 'O') => to_number_with_target(s, ToNumberTarget::Oct),
                Some('x' | 'X') => to_number_with_target(s, ToNumberTarget::Hex),
                _ => {
                    let mut is_oct = true;
                    while let Some(chr) = chars.next() {
                        if !matches!(chr, '0'..='7') {
                            is_oct = false;
                            break;
                        }
                    }

                    to_number_with_target(
                        s,
                        if is_oct {
                            ToNumberTarget::Oct
                        } else {
                            ToNumberTarget::Decimal
                        },
                    )
                }
            },
            _ => to_number_with_target(s, ToNumberTarget::Decimal),
        }
    }
}

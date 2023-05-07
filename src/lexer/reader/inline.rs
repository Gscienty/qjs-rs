use std::str::Chars;

use super::reader;

/// 读取在 Rust 代码内写 EMCAScript 源码
pub(crate) struct InlineSourceReader<'s> {
    source_chars: Chars<'s>,

    current_chr: Option<char>,
    lookahead_chr: Option<char>,
}

impl<'s> InlineSourceReader<'s> {
    /// 构造一个读取 Rust 代码内写 EMCAScript 源码的 SourceReader
    ///
    /// # Arguments
    /// `source` - JavaScript 源码
    /// # Returns
    /// SourceReader 的一个实现
    pub(crate) fn new(source: &'s str) -> Self {
        InlineSourceReader {
            source_chars: source.chars(),

            current_chr: None,
            lookahead_chr: None,
        }
    }
}

impl<'s> reader::SourceReader for InlineSourceReader<'s> {
    #[inline(always)]
    fn next(&mut self, off: isize) {
        for _ in 0..off {
            if self.lookahead_chr.is_some() {
                self.current_chr = self.lookahead_chr;
                self.lookahead_chr = None;
                continue;
            }
            self.current_chr = self.source_chars.next();
        }

        self.lookahead_chr = self.source_chars.next();
    }

    #[inline(always)]
    fn current(&self) -> Option<char> {
        self.current_chr
    }

    #[inline(always)]
    fn lookahead(&self) -> Option<char> {
        self.lookahead_chr
    }
}

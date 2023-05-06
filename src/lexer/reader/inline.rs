use std::{iter::Skip, str::Chars};

use super::reader;

/// 读取在 Rust 代码内写 EMCAScript 源码
pub(crate) struct InlineSourceReader<'s> {
    source: String,
    source_chars: Chars<'s>,

    current_chr: Option<char>,
    lookahead_chr: Option<char>,

    read_off: isize,
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
            source: String::from(source),
            source_chars: source.chars(),

            current_chr: None,
            lookahead_chr: None,

            read_off: -1,
        }
    }
}

impl<'s> reader::SourceReader for InlineSourceReader<'s> {
    #[inline(always)]
    fn next(&mut self, off: isize) {
        self.read_off += off;

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

    #[inline(always)]
    fn read(&self, reader_fn: &mut dyn FnMut(Skip<Chars>)) {
        if self.read_off < 0 {
            return;
        }
        reader_fn(self.source_chars.clone().skip(self.read_off as usize))
    }
}

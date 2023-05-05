use super::reader;

/// 读取在 Rust 代码内写 EMCAScript 源码
pub(crate) struct InlineSourceReader {
    source: String,

    read_off: isize,
    current: Option<char>,
    lookahead: Option<char>,
}

impl InlineSourceReader {
    /// 构造一个读取 Rust 代码内写 EMCAScript 源码的 SourceReader
    ///
    /// # Arguments
    /// `source` - JavaScript 源码
    /// # Returns
    /// SourceReader 的一个实现
    pub(super) fn new(source: &str) -> Self {
        InlineSourceReader {
            source: String::from(source),

            read_off: -1,
            current: None,
            lookahead: None,
        }
    }
}

impl reader::SourceReader for InlineSourceReader {
    #[inline(always)]
    fn next(&mut self) {
        self.read_off += 1;

        let off = self.read_off as usize;
        let mut chars = self.source[off..].chars();
        self.current = chars.next();
        self.lookahead = chars.next();
    }

    #[inline(always)]
    fn read(&self, off: isize) -> Option<char> {
        if off == 0 {
            return self.current;
        }
        if off == 1 && self.lookahead.is_some() {
            return self.lookahead;
        }

        let off = self.read_off + off;
        if off < 0 || self.source.len() <= off as usize {
            return None;
        }
        let off = off as usize;

        self.source[off..].chars().next()
    }
}

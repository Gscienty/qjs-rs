use std::isize;

/// 读取 EMCAScript 源码
///
/// 实现该 Trait 应维护一个读取源码的游标，从源码中读取游标指定的字符。
/// 由于 EMCAScript 的源码来源可能是多样的，可能是在 Rust 代码中直接写；
/// 可能是通过文件的方式进行读取。在做词法解析时应屏蔽这些细节，因此，
/// 词法分析器仅通过 SourceReader 读取对应的源码字符。
pub(crate) trait SourceReader {
    /// 将源码游标向下移动一个字符
    fn next(&mut self);

    /// 以当前游标基准，读取偏移量为 `off` 的字符
    ///
    /// # Arguments
    /// * `off` - 读取偏移量
    /// # Returns
    /// 返回以游标为基准，偏移量为 `off` 的字符
    fn read(&self, off: isize) -> Option<char>;
}

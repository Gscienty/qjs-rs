/// 读取 EMCAScript 源码
///
/// 实现该 Trait 应维护一个读取源码的游标，从源码中读取游标指定的字符。
/// 由于 EMCAScript 的源码来源可能是多样的，可能是在 Rust 代码中直接写；
/// 可能是通过文件的方式进行读取。在做词法解析时应屏蔽这些细节，因此，
/// 词法分析器仅通过 SourceReader 读取对应的源码字符。
pub(crate) trait SourceReader {
    /// 将源码游标向下移动一个字符
    ///
    /// # Arguments
    /// * `off` - 游标移动偏移量
    fn next(&mut self, off: isize);

    /// 获取当前游标指向的字符
    ///
    /// # Returns
    /// 返回当前游标指向的字符
    fn current(&self) -> Option<char>;

    /// 获取当前游标指向的下一个字符
    ///
    /// # Returns
    /// 返回当前游标指向的下一个字符
    fn lookahead(&self) -> Option<char>;
}

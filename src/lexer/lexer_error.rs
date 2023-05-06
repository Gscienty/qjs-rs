pub(crate) struct LexerError {
    line_number: usize,
    line_off: usize,
}

impl LexerError {
    pub(super) fn new(line_number: usize, line_off: usize) -> Self {
        LexerError {
            line_number,
            line_off,
        }
    }
}

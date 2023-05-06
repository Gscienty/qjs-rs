use super::{inline, reader::SourceReader};

#[test]
fn test_InlineSourceReader_next() {
    let mut reader = inline::InlineSourceReader::new(
        r#"function () {
            print("Hello World");
        }"#,
    );
    reader.next(1);
    assert_eq!(reader.current(), Some('f'));
    assert_eq!(reader.lookahead(), Some('u'));

    reader.next(2);
    assert_eq!(reader.current(), Some('n'));
    assert_eq!(reader.lookahead(), Some('c'));
}

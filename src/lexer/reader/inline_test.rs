use super::{inline, reader::SourceReader};

#[test]
fn test_InlineSourceReader_next() {
    let mut reader = inline::InlineSourceReader::new(
        r#"function () {
            print("Hello World");
        }"#,
    );

    reader.next();

    assert_eq!(reader.read(-1), None);
    assert_eq!(reader.read(0), Some('f'));
    assert_eq!(reader.read(1), Some('u'))
}

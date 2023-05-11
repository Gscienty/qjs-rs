#[derive(Debug)]
pub(crate) enum JSValue {
    Int(i64),
    Float(f64),
    Str(String),
    Null,
}

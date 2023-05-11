use crate::vals::strconv::to_number;

use super::JSValue;

#[test]
fn test_strconv_to_integer() {
    let verify = |s: &str, v: i64| {
        println!("verify: {:?} {:?}", to_number(s), v);
        assert!(matches!(to_number(s), JSValue::Int(a) if a == v));
    };

    verify("123", 123);
    verify("34e12", 34i64.pow(12));
    verify("34e+12", 34i64.pow(12));
    verify("0123", 0o123);
    verify("01238", 1238);
    verify("0b101", 0b101);
    verify("0o567", 0o567);
    verify("0x3abc", 0x3abc);
}

#[test]
fn test_strconv_to_float() {
    let verify = |s: &str, v: f64| {
        assert!(matches!(to_number(s), JSValue::Float(a) if a == v));
    };

    verify("123.", 123f64);
    verify("123.456", 123.456f64);
    verify(".456", 0.456f64);
    verify("123.456e2", 123.456f64.powf(2.0));
    verify(".456E-3", 0.456f64.powf(-3.0));
}

use std::fs;

use hessian_rs::{de::Deserializer, Value};
use hessian_rs::error::Error;


fn load_value_from_file(file_name: &str) -> Result<Value, Error>{
    let rdr = fs::read(file_name)?;
    let mut de = Deserializer::new(rdr);
    de.read_value()
}

#[test]
fn test_decode_long_binary() {
    let rdr = fs::read("tests/fixtures/bytes/65535.bin").unwrap();
    let mut de = Deserializer::new(rdr);
    let value = de.read_value().unwrap();
    match value {
        Value::Bytes(bytes) => assert_eq!(bytes, vec![0x41; 65535]),
        _ => panic!("expect bytes"),
    }
}

#[test]
fn test_decode_date() {
    assert_eq!(load_value_from_file("tests/fixtures/date/894621060000.bin").unwrap(), Value::Date(894621060000));
    assert_eq!(load_value_from_file("tests/fixtures/date/894621091000.bin").unwrap(), Value::Date(894621091000));
    assert_eq!(load_value_from_file("tests/fixtures/date/128849018880000.bin").unwrap(), Value::Date(128849018880000));
    assert_eq!(load_value_from_file("tests/fixtures/date/-128849018940000.bin").unwrap(), Value::Date(-128849018940000));
}
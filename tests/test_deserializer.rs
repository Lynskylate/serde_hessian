use std::fs;

use hessian_rs::{de::Deserializer, Value};

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

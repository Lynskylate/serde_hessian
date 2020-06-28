use std::fs;

use hessian_rs::Error;
use hessian_rs::{de::Deserializer, Value};

fn load_value_from_file(file_name: &str) -> Result<Value, Error> {
    let rdr = fs::read(file_name)?;
    let mut de = Deserializer::new(rdr);
    de.read_value()
}

#[test]
fn test_decode_long_binary() {
    let value = load_value_from_file("tests/fixtures/bytes/65535.bin").unwrap();
    match value {
        Value::Bytes(bytes) => assert_eq!(bytes, vec![0x41; 65535]),
        _ => panic!("expect bytes"),
    }
}

#[test]
fn test_decode_date() {
    assert_eq!(
        load_value_from_file("tests/fixtures/date/894621060000.bin").unwrap(),
        Value::Date(894621060000)
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/date/894621091000.bin").unwrap(),
        Value::Date(894621091000)
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/date/128849018880000.bin").unwrap(),
        Value::Date(128849018880000)
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/date/-128849018940000.bin").unwrap(),
        Value::Date(-128849018940000)
    );
}

#[test]
fn test_decode_string() {
    assert_eq!(
        load_value_from_file("tests/fixtures/string/empty.bin").unwrap(),
        Value::String("".to_string())
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/string/foo.bin").unwrap(),
        Value::String("foo".to_string())
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/string/chinese.bin").unwrap(),
        Value::String("中文 Chinese".to_string())
    );
}

#[test]
fn test_decode_list() {
    assert_eq!(
        load_value_from_file("tests/fixtures/list/untyped_list.bin").unwrap(),
        Value::List(vec![Value::Int(1), Value::Int(2), "foo".into()].into())
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/list/untyped_[].bin").unwrap(),
        Value::List(vec![].into())
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/list/untyped_list_8.bin").unwrap(),
        Value::List(
            vec!["1", "2", "3", "4", "5", "6", "7", "8"]
                .into_iter()
                .map(|x| x.into())
                .collect::<Vec<Value>>()
                .into()
        )
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/list/untyped_<String>[foo,bar].bin").unwrap(),
        Value::List(vec!["foo".into(), "bar".into()].into())
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/list/[int.bin").unwrap(),
        Value::List(("[int", vec![Value::Int(1), Value::Int(2), Value::Int(3)]).into())
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/list/[string.bin").unwrap(),
        Value::List(("[string", vec!["1".into(), "@".into(), "3".into()]).into())
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/list/typed_list.bin").unwrap(),
        Value::List(
            (
                "hessian.demo.SomeArrayList",
                vec!["ok".into(), "some list".into()]
            )
                .into()
        )
    );
    assert_eq!(
        load_value_from_file("tests/fixtures/list/typed_list_8.bin").unwrap(),
        Value::List(
            (
                "hessian.demo.SomeArrayList",
                vec!["1", "2", "3", "4", "5", "6", "7", "8"]
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<Value>>()
                    .into()
            )
                .into()
        )
    );
}

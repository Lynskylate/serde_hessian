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
        load_value_from_file("tests/fixtures/list/untyped_string_foo_bar.bin").unwrap(),
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
            )
                .into()
        )
    );
}

#[test]
fn test_decode_map() {
    use maplit::hashmap;

    assert_eq!(
        load_value_from_file("tests/fixtures/map/car.bin").unwrap(),
        Value::Map(
            (
                "hessian.demo.Car",
                hashmap! {
                    "a".into() => "a".into(),
                    "b".into() => "b".into(),
                    "c".into() => "c".into(),
                    "model".into() => "Beetle".into(),
                    "color".into() => "aquamarine".into(),
                    "mileage".into() => Value::Int(65536),
                }
            )
                .into()
        )
    );

    assert_eq!(
        load_value_from_file("tests/fixtures/map/car1.bin").unwrap(),
        Value::Map(
            (
                "hessian.demo.Car",
                hashmap! {
                    "prev".into() => Value::Null,
                    "self".into() => Value::Ref(0),
                    "model".into() => "Beetle".into(),
                    "color".into() => "aquamarine".into(),
                    "mileage".into() => Value::Int(65536),
                }
            )
                .into()
        )
    );

    assert_eq!(
        load_value_from_file("tests/fixtures/map/foo_empty.bin").unwrap(),
        Value::Map(
            hashmap! {
                "foo".into() => "".into(),
            }
            .into()
        )
    );

    assert_eq!(
        load_value_from_file("tests/fixtures/map/foo_bar.bin").unwrap(),
        Value::Map(
            hashmap! {
                "foo".into() => "bar".into(),
                "123".into() => Value::Int(456),
                "zero".into() => Value::Int(0),
                "中文key".into() => "中文哈哈value".into(),
            }
            .into()
        )
    );

    assert_eq!(
        load_value_from_file("tests/fixtures/map/hashtable.bin").unwrap(),
        Value::Map(
            (
                "java.util.Hashtable",
                hashmap! {
                    "foo".into() => "bar".into(),
                    "中文key".into() => "中文哈哈value".into(),
                }
            )
                .into()
        )
    );

    assert_eq!(
        load_value_from_file("tests/fixtures/map/generic.bin").unwrap(),
        Value::Map(
            hashmap! {
                Value::Long(123) => Value::Int(123456),
                Value::Long(123456) => Value::Int(123),
            }
            .into()
        )
    );

    let val = load_value_from_file("tests/fixtures/map/hashmap.bin").unwrap();
    let map = val.as_map().unwrap();
    let data = &map[&"data".into()];
    let data = data.as_map().unwrap();
    assert_eq!(data.len(), 2);

    let val = load_value_from_file("tests/fixtures/map/custom_map_type.bin").unwrap();
    let list = val.as_list().unwrap();
    assert_eq!(list.r#type().unwrap(), "com.alibaba.fastjson.JSONArray");
    let item0 = &list[0];
    let map = item0.as_map().unwrap();
    assert_eq!(map.r#type().unwrap(), "com.alibaba.fastjson.JSONObject");
    assert_eq!(map.len(), 3);
}

#[test]
fn test_decode_object() {
    let val = load_value_from_file("tests/fixtures/object/ConnectionRequest.bin").unwrap();
    let map = val.as_map().unwrap();
    assert_eq!(map.r#type().unwrap(), "hessian.ConnectionRequest");
    let ctx = &map[&"ctx".into()].as_map().unwrap();
    assert_eq!(
        ctx.r#type().unwrap(),
        "hessian.ConnectionRequest$RequestContext"
    );
    assert_eq!(ctx[&"id".into()], Value::Int(101));

    let val = load_value_from_file("tests/fixtures/object/AtomicLong0.bin").unwrap();
    let map = val.as_map().unwrap();
    assert_eq!(
        map.r#type().unwrap(),
        "java.util.concurrent.atomic.AtomicLong"
    );
    assert_eq!(map[&"value".into()], Value::Long(0));

    let val = load_value_from_file("tests/fixtures/object/AtomicLong1.bin").unwrap();
    let map = val.as_map().unwrap();
    assert_eq!(
        map.r#type().unwrap(),
        "java.util.concurrent.atomic.AtomicLong"
    );
    assert_eq!(map[&"value".into()], Value::Long(1));
}

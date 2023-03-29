use std::collections::HashMap;
use std::fmt::Debug;

use serde::de;
use serde::ser;
use serde::Deserialize;
use serde::Serialize;
use serde_hessian::{de::from_slice, ser::to_vec};

fn roundtrip_test<T: ser::Serialize + for<'a> de::Deserialize<'a> + PartialEq + Debug>(val: T) {
    let buf = to_vec(&val).unwrap();
    let decoded: T = from_slice(&buf).unwrap();
    assert_eq!(decoded, val);
}

#[test]
fn test_int_roundtrip() {
    roundtrip_test(0);
    roundtrip_test(-16);
    roundtrip_test(47);
    roundtrip_test(48);
    roundtrip_test(-2048);
    roundtrip_test(-256);
    roundtrip_test(2047);
    roundtrip_test(-262144);
    roundtrip_test(262143);
    roundtrip_test(262144);
}

#[test]
fn test_i64_roundtrip() {
    roundtrip_test(0);
    roundtrip_test(-16);
    roundtrip_test(47);
    roundtrip_test(48);
    roundtrip_test(-2048 as i64);
    roundtrip_test(-256 as i64);
    roundtrip_test(2047 as i64);
    roundtrip_test(-262144 as i64);
    roundtrip_test(262143 as i64);
    roundtrip_test(262144 as i64);
    roundtrip_test(i64::MAX);
    roundtrip_test(i64::MIN);
}

#[test]
fn test_double_roundtrip() {
    roundtrip_test(0.0);
    roundtrip_test(1.0);
    roundtrip_test(-128.0);
    roundtrip_test(10.0);
    roundtrip_test(127.0);
    roundtrip_test(-32768.0);
    roundtrip_test(32767.0);
    roundtrip_test(32766.0);
    roundtrip_test(12.25);
    roundtrip_test(32767.99999);
}

// #[test]
// fn test_date_roundtrip() {
//     roundtrip_test(Date(894621091000));
// }

#[test]
fn test_string_roundtrip() {
    roundtrip_test("".to_string());
    roundtrip_test("abc".to_string());
    roundtrip_test("中文 Chinese".to_string());
    roundtrip_test("abcdefghijklmnopqrstuvwxyz".to_string());
    roundtrip_test("abcdefghij".repeat(120));
    roundtrip_test("abcdefghij".repeat(1000));
}

#[test]
fn test_list_roundtrip() {
    roundtrip_test(vec![1, 2]);
    roundtrip_test(vec!["".to_string(), "abc".to_string(), "中文".to_string()]);
    roundtrip_test(vec![1; 13]);
    roundtrip_test(vec!["String".to_string(); 1024]);
}

#[test]
fn test_map_roundtrip() {
    let mut map = HashMap::new();
    map.insert(1, "fee".to_string());
    map.insert(1, "fie".to_string());
    map.insert(1, "foe".to_string());
    roundtrip_test(map);
    let mut car_map = HashMap::new();
    car_map.insert("color".to_string(), "aquamarine".to_string());
    car_map.insert("model".to_string(), "Beetle".to_string());
    roundtrip_test(car_map);
}

#[test]
fn test_basic_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct BasicStruct {
        a: i32,
        b: String,
    }
    roundtrip_test(BasicStruct {
        a: 1,
        b: "abc".to_string(),
    });
}

// todo: fix enum round trip
#[test]
fn test_enum() {
    #[derive(Deserialize, Serialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }
    roundtrip_test(E::Unit);
    // roundtrip_test(E::Newtype(1));
    // roundtrip_test(E::Tuple(1, 2));
}

use hessian_rs::{
    de::Deserializer,
    ser::Serializer,
    Value::{self, *},
};

fn roundtrip_test(val: Value) {
    let mut encoded = Vec::new();
    let mut ser = Serializer::new(&mut encoded);
    ser.serialize_value(&val)
        .expect(&format!("serialization failed for {:?}", val));
    let mut de = Deserializer::new(&encoded);
    let decoded = de
        .read_value()
        .expect(&format!("deserialization failed for {:?}", val));
    assert_eq!(decoded, val);
}

#[test]
fn test_int_roundtrip() {
    roundtrip_test(Int(0));
    roundtrip_test(Int(-16));
    roundtrip_test(Int(47));
    roundtrip_test(Int(48));
    roundtrip_test(Int(-2048));
    roundtrip_test(Int(-256));
    roundtrip_test(Int(2047));
    roundtrip_test(Int(-262144));
    roundtrip_test(Int(262143));
    roundtrip_test(Int(262144));
}

#[test]
fn test_long_roundtrip() {
    roundtrip_test(Long(0));
    roundtrip_test(Long(-16));
    roundtrip_test(Long(47));
    roundtrip_test(Long(48));
    roundtrip_test(Long(-2048));
    roundtrip_test(Long(-256));
    roundtrip_test(Long(2047));
    roundtrip_test(Long(-262144));
    roundtrip_test(Long(262143));
    roundtrip_test(Long(262144));
}

#[test]
fn test_string_roundtrip() {
    roundtrip_test(String("".to_string()));
    roundtrip_test(String("abc".to_string()));
    roundtrip_test(String("中文 Chinese".to_string()));
    roundtrip_test(String("abcdefghijklmnopqrstuvwxyz".to_string()));
    roundtrip_test(String("abcdefghij".repeat(120)));
    roundtrip_test(String("abcdefghij".repeat(1000)));
}

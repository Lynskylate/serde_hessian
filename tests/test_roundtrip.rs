use hessian_rs::{ser::Serializer, de::Deserializer, Value::{self, *}};

fn roundtrip_test(val: Value) {
    let mut encoded = Vec::new();
    let mut ser = Serializer::new(&mut encoded);
    ser.serialize_value(&val).unwrap();
    let mut de = Deserializer::new(&encoded);
    let decoded = de.read_value().unwrap();
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
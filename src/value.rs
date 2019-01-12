use std::cmp::Ordering;
// use std::collections::HashMap;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    // ClassDef,
    Bool(bool),
    Int(i32),
    Long(i64),
    Double(f64),
    Bytes(Vec<u8>),
    String(String),
    Ref(i32),
    List(Vec<Value>),
    Map(BTreeMap<Value, Value>),
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Value) -> Ordering {
        use self::Value::*;

        match *self {
            Null => match *other {
                Null => Ordering::Equal,
                _ => Ordering::Less,
            },
            Bool(b) => match *other {
                Null => Ordering::Greater,
                Int(i) => (b as i32).cmp(&i),
                Long(l) => (b as i64).cmp(&l),
                Double(d) => float_ord(b as i64 as f64, d),
                _ => Ordering::Less,
            },
            Int(i) => match *other {
                Null => Ordering::Greater,
                Bool(b) => i.cmp(&(b as i32)),
                Int(i2) => i.cmp(&i2),
                Long(l) => (i as i64).cmp(&l),
                Double(d) => float_ord(i as f64, d),
                _ => Ordering::Less,
            },
            Long(l) => match *other {
                Null => Ordering::Greater,
                Bool(b) => l.cmp(&(b as i64)),
                Int(i2) => l.cmp(&(i2 as i64)),
                Long(l2) => l.cmp(&l2),
                Double(d) => float_ord(l as f64, d),
                _ => Ordering::Less,
            },
            Double(d) => match *other {
                Null => Ordering::Greater,
                Bool(b) => float_ord(d, b as i64 as f64),
                Int(i) => float_ord(d, i as f64),
                Long(l) => float_ord(d, l as f64),
                Double(d2) => float_ord(d, d2),
                _ => Ordering::Less,
            },
            Bytes(ref bs) => match *other {
                String(_) | List(_) | Ref(_) | Map(_) => Ordering::Less,
                Bytes(ref bs2) => bs.cmp(bs2),
                _ => Ordering::Greater,
            },
            String(ref s) => match *other {
                Ref(_) | List(_) | Map(_) => Ordering::Less,
                String(ref s2) => s.cmp(s2),
                _ => Ordering::Greater,
            },
            Ref(i) => match *other {
                List(_) | Map(_) => Ordering::Less,
                Ref(i2) => i.cmp(&i2),
                _ => Ordering::Greater,
            },
            List(l) => match *other {
                Map(_) => Ordering::Less,
                List(l2) => l.len().cmp(&l2.len()),
                _ => Ordering::Greater,
            },
            Map(m) => match *other {
                Map(m2) => m.len().cmp(&m2.len()),
                _ => Ordering::Greater,
            },
        }
    }
}

fn float_ord(f: f64, g: f64) -> Ordering {
    match f.partial_cmp(&g) {
        Some(o) => o,
        None => Ordering::Less,
    }
}

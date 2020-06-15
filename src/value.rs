extern crate ordered_float;

use ordered_float::OrderedFloat;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
//use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    // ClassDef,
    Bool(bool),
    Int(i32),
    Long(i64),
    Double(f64),
    Date(i64),
    Bytes(Vec<u8>),
    String(String),
    Ref(u32),
    List(Vec<Value>),
    Map(HashMap<Value, Value>),
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Eq for Value {}

// Although we impl Hash for Map and List, we shouldn't use container type as key
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use self::Value::*;

        match *self {
            Null => ().hash(state),
            Bool(b) => b.hash(state),
            Int(i) => i.hash(state),
            Long(l) => l.hash(state),
            Double(d) => OrderedFloat(d).hash(state),
            Date(d) => d.hash(state),
            Bytes(ref bytes) => bytes.hash(state),
            String(ref s) => s.hash(state),
            Ref(i) => i.hash(state),
            List(ref l) => l.hash(state),
            // Hash each key-value is too expensive.
            Map(ref m) => std::ptr::hash(m, state),
        }
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
                Date(d) => (b as i64).cmp(&d),
                _ => Ordering::Less,
            },
            Int(i) => match *other {
                Null => Ordering::Greater,
                Bool(b) => i.cmp(&(b as i32)),
                Int(i2) => i.cmp(&i2),
                Long(l) => (i as i64).cmp(&l),
                Double(d) => float_ord(i as f64, d),
                Date(d) => (i as i64).cmp(&d),
                _ => Ordering::Less,
            },
            Long(l) => match *other {
                Null => Ordering::Greater,
                Bool(b) => l.cmp(&(b as i64)),
                Int(i2) => l.cmp(&(i2 as i64)),
                Long(l2) => l.cmp(&l2),
                Double(d) => float_ord(l as f64, d),
                Date(d) => l.cmp(&d),
                _ => Ordering::Less,
            },
            Double(d) => match *other {
                Null => Ordering::Greater,
                Bool(b) => float_ord(d, b as i64 as f64),
                Int(i) => float_ord(d, i as f64),
                Long(l) => float_ord(d, l as f64),
                Double(d2) => float_ord(d, d2),
                Date(d2) => float_ord(d, d2 as f64),
                _ => Ordering::Less,
            },
            Date(d) => match *other {
                Null => Ordering::Greater,
                Bool(b) => d.cmp(&(b as i64)),
                Int(i2) => d.cmp(&(i2 as i64)),
                Long(l2) => d.cmp(&l2),
                Double(d2) => float_ord(d as f64, d2),
                Date(d2) => d.cmp(&d2),
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
            List(ref l) => match other {
                Map(_) => Ordering::Less,
                List(l2) => l.cmp(l2),
                _ => Ordering::Greater,
            },
            Map(ref m) => match other {
                Map(m2) => {
                    let v1: Vec<_> = m.iter().collect();
                    let v2: Vec<_> = m2.iter().collect();
                    v1.cmp(&v2)
                }
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

pub trait ToHessian {
    fn to_hessian(self) -> Value;
}

macro_rules! to_hessian (
    ($t:ty, $v:expr) => (
        impl ToHessian for $t {
        fn to_hessian(self) -> Value {
            $v(self)
        }
    }
    );
);

to_hessian!(bool, Value::Bool);
to_hessian!(i32, Value::Int);
to_hessian!(i64, Value::Long);
to_hessian!(f64, Value::Double);
to_hessian!(String, Value::String);
to_hessian!(Vec<u8>, Value::Bytes);

impl ToHessian for () {
    fn to_hessian(self) -> Value {
        Value::Null
    }
}

impl<'a> ToHessian for &'a str {
    fn to_hessian(self) -> Value {
        Value::String(self.to_owned())
    }
}

impl<'a> ToHessian for &'a [u8] {
    fn to_hessian(self) -> Value {
        Value::Bytes(self.to_owned())
    }
}

impl<'a> ToHessian for &'a Vec<u8> {
    fn to_hessian(self) -> Value {
        Value::Bytes(self.to_owned())
    }
}

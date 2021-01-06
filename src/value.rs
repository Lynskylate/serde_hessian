extern crate ordered_float;

use ordered_float::OrderedFloat;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

/// class definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Definition {
    pub name: String,
    pub fields: Vec<String>,
}

/// hessian 2.0 list
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum List {
    Typed(String, Vec<Value>),
    Untyped(Vec<Value>),
}

impl List {
    pub fn r#type(&self) -> Option<&str> {
        match self {
            List::Typed(ref typ, _) => Some(typ),
            List::Untyped(_) => None,
        }
    }

    pub fn value(&self) -> &[Value] {
        match self {
            List::Typed(_, val) => val,
            List::Untyped(val) => val,
        }
    }

    pub fn value_mut(&mut self) -> &mut [Value] {
        match self {
            List::Typed(_, val) => val,
            List::Untyped(val) => val,
        }
    }
}

impl From<Vec<Value>> for List {
    fn from(val: Vec<Value>) -> Self {
        Self::Untyped(val)
    }
}

impl From<(String, Vec<Value>)> for List {
    fn from(val: (String, Vec<Value>)) -> Self {
        Self::Typed(val.0, val.1)
    }
}

impl From<(&str, Vec<Value>)> for List {
    fn from(val: (&str, Vec<Value>)) -> Self {
        Self::Typed(val.0.to_string(), val.1)
    }
}

impl Deref for List {
    type Target = [Value];

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value_mut()
    }
}

/// hessian 2.0 map
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Map {
    Typed(String, HashMap<Value, Value>),
    Untyped(HashMap<Value, Value>),
}

impl Map {
    pub fn r#type(&self) -> Option<&str> {
        match self {
            Map::Typed(ref typ, _) => Some(typ),
            Map::Untyped(_) => None,
        }
    }

    pub fn value(&self) -> &HashMap<Value, Value> {
        match self {
            Map::Typed(_, val) => val,
            Map::Untyped(val) => val,
        }
    }

    pub fn value_mut(&mut self) -> &mut HashMap<Value, Value> {
        match self {
            Map::Typed(_, val) => val,
            Map::Untyped(val) => val,
        }
    }
}

impl From<HashMap<Value, Value>> for Map {
    fn from(val: HashMap<Value, Value>) -> Self {
        Self::Untyped(val)
    }
}

impl From<(String, HashMap<Value, Value>)> for Map {
    fn from(val: (String, HashMap<Value, Value>)) -> Self {
        Self::Typed(val.0, val.1)
    }
}

impl From<(&str, HashMap<Value, Value>)> for Map {
    fn from(val: (&str, HashMap<Value, Value>)) -> Self {
        Self::Typed(val.0.to_string(), val.1)
    }
}

impl Deref for Map {
    type Target = HashMap<Value, Value>;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

impl DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value_mut()
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    /// null
    Null,
    // ClassDef,
    /// boolean
    Bool(bool),
    /// 32-bit int
    Int(i32),
    /// 64-bit int
    Long(i64),
    /// 64-bit double
    Double(f64),
    /// 64-bit millisecond date
    Date(i64),
    /// raw binary data
    Bytes(Vec<u8>),
    /// UTF8-encoded string
    String(String),
    /// shared and circular object references
    Ref(u32),
    // list for lists and arrays
    List(List),
    /// map for maps and dictionaries
    Map(Map),
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(lhs), Value::Bool(rhs)) => lhs == rhs,
            (Value::Int(lhs), Value::Int(rhs)) => lhs == rhs,
            (Value::Long(lhs), Value::Long(rhs)) => lhs == rhs,
            (Value::Double(lhs), Value::Double(rhs)) => lhs == rhs,
            (Value::Date(lhs), Value::Date(rhs)) => lhs == rhs,
            (Value::Bytes(lhs), Value::Bytes(rhs)) => lhs == rhs,
            (Value::String(lhs), Value::String(rhs)) => lhs == rhs,
            (Value::Ref(lhs), Value::Ref(rhs)) => lhs == rhs,
            (Value::List(lhs), Value::List(rhs)) => lhs == rhs,
            (Value::Map(lhs), Value::Map(rhs)) => {
                let mut left_v: Vec<_> = lhs.iter().collect();
                let mut right_v: Vec<_> = rhs.iter().collect();
                left_v.sort_by(|l_iter, r_iter| l_iter.0.cmp(r_iter.0));
                right_v.sort_by(|l_iter, r_iter| l_iter.0.cmp(r_iter.0));
                left_v == right_v
            }
            _ => false,
        }
    }
}

impl Value {
    /// Takes the value out of the `Value`, leaving a `Null` in its place.
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Value::Null)
    }

    /// If the `Value` is a `Null`, returns `()`. Returns `None` otherwise.
    pub fn as_null(&self) -> Option<()> {
        match self {
            Value::Null => Some(()),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    pub fn as_int(&self) -> Option<i32> {
        match *self {
            Value::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn is_int(&self) -> bool {
        self.as_int().is_some()
    }

    pub fn as_long(&self) -> Option<i64> {
        match *self {
            Value::Long(l) => Some(l),
            _ => None,
        }
    }

    pub fn is_long(&self) -> bool {
        self.as_long().is_some()
    }

    pub fn as_double(&self) -> Option<f64> {
        match *self {
            Value::Double(f) => Some(f),
            _ => None,
        }
    }

    pub fn is_double(&self) -> bool {
        self.as_double().is_some()
    }

    pub fn as_date(&self) -> Option<i64> {
        match *self {
            Value::Date(d) => Some(d),
            _ => None,
        }
    }

    pub fn is_date(&self) -> bool {
        self.as_date().is_some()
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(bs) => Some(bs),
            _ => None,
        }
    }

    pub fn is_bytes(&self) -> bool {
        self.as_bytes().is_some()
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    pub fn as_ref(&self) -> Option<u32> {
        match *self {
            Value::Ref(r) => Some(r),
            _ => None,
        }
    }

    pub fn is_ref(&self) -> bool {
        self.as_ref().is_some()
    }

    pub fn as_list(&self) -> Option<&List> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut List> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn is_list(&self) -> bool {
        self.as_int().is_some()
    }

    pub fn as_map(&self) -> Option<&Map> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut Map> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn is_map(&self) -> bool {
        self.as_map().is_some()
    }
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
                    let mut v1: Vec<_> = m.iter().collect();
                    let mut v2: Vec<_> = m2.iter().collect();
                    v1.sort_by(|l_iter, r_iter| l_iter.0.cmp(r_iter.0));
                    v2.sort_by(|l_iter, r_iter| l_iter.0.cmp(r_iter.0));
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

impl<'a> ToHessian for &'a String {
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

impl<K, V> ToHessian for HashMap<K, V>
where
    K: ToHessian,
    V: ToHessian,
{
    fn to_hessian(self) -> Value {
        let kv: HashMap<Value, Value> = self
            .into_iter()
            .map(|(k, v)| (k.to_hessian(), v.to_hessian()))
            .collect();
        Value::Map(kv.into())
    }
}

impl<K, V> ToHessian for (String, HashMap<K, V>)
where
    K: ToHessian,
    V: ToHessian,
{
    fn to_hessian(self) -> Value {
        let (typ, kv) = self;
        let kv: HashMap<Value, Value> = kv
            .into_iter()
            .map(|(k, v)| (k.to_hessian(), v.to_hessian()))
            .collect();
        Value::Map((typ, kv).into())
    }
}

impl<K, V> ToHessian for (&str, HashMap<K, V>)
where
    K: ToHessian,
    V: ToHessian,
{
    fn to_hessian(self) -> Value {
        let (typ, kv) = self;
        let kv: HashMap<Value, Value> = kv
            .into_iter()
            .map(|(k, v)| (k.to_hessian(), v.to_hessian()))
            .collect();
        Value::Map((typ, kv).into())
    }
}

impl<T: ToHessian> From<T> for Value {
    fn from(val: T) -> Self {
        val.to_hessian()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Null => write!(f, "None"),
            Value::Bool(b) => write!(f, "{}", if b { "True" } else { "False" }),
            Value::Int(ref i) => write!(f, "{}", i),
            Value::Long(ref i) => write!(f, "{}", i),
            Value::Double(ref v) => write!(f, "{}", v),
            Value::Date(v) => write!(f, "Date({})", v),
            Value::Bytes(ref b) => write!(f, "b{:?}", b), //
            Value::String(ref s) => write!(f, "{:?}", s),
            Value::List(ref l) => {
                write!(f, "[")?;
                for (inx, v) in l.iter().enumerate() {
                    if inx < l.len() - 1 {
                        write!(f, "{}, ", v)?;
                    } else {
                        write!(f, "{}", v)?;
                    }
                }
                write!(f, "]")
            }
            Value::Map(ref m) => {
                write!(f, "{{")?;
                for (key, value) in m.iter() {
                    write!(f, "{} : {},", key, value)?;
                }
                write!(f, "}}")
            }
            _ => write!(f, "<Unknown Type>"),
        }
    }
}

use hessian_rs::ser::Serializer;
use hessian_rs::value::Definition;
use pyo3::exceptions::PyTypeError;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::PyErr;
use std::io::Write;

use pyo3::types::timezone_utc;
use pyo3::types::PyBool;
use pyo3::types::PyBytes;
use pyo3::types::PyDateTime;
use pyo3::types::PyDict;
use pyo3::types::PyFloat;
use pyo3::types::PyInt;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::types::PyTuple;

#[pymodule]
fn hessian_codec(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    m.add_wrapped(wrap_pyfunction!(load))?;
    m.add_wrapped(wrap_pyfunction!(loads))?;

    m.add_wrapped(wrap_pyfunction!(dump))?;
    m.add_wrapped(wrap_pyfunction!(dumps))?;

    Ok(())
}

#[pyfunction]
pub fn load(py: Python, fp: PyObject, kwargs: Option<&PyDict>) -> PyResult<PyObject> {
    // Temporary workaround for
    // https://github.com/PyO3/pyo3/issues/145
    let io: &PyAny = fp.extract(py)?;

    // Reset file pointer to beginning See
    // https://github.com/PyO3/pyo3/issues/143 Note that we ignore the return
    // value, because `seek` does not strictly need to exist on the object
    let _success = io.call_method("seek", (0,), None);

    let s_obj = io.call_method0("read")?;
    loads(py, s_obj.to_object(py), None, None, None, kwargs)
}

#[pyfunction]
pub fn loads(
    py: Python,
    s: PyObject,
    encoding: Option<PyObject>,
    cls: Option<PyObject>,
    object_hook: Option<PyObject>,
    kwargs: Option<&PyDict>,
) -> PyResult<PyObject> {
    loads_impl(py, s, encoding, cls, object_hook, kwargs)
}

pub fn loads_impl(
    py: Python,
    s: PyObject,
    _encoding: Option<PyObject>,
    _cls: Option<PyObject>,
    _object_hook: Option<PyObject>,
    _kwargs: Option<&PyDict>,
) -> PyResult<PyObject> {
    let bytes: Vec<u8> = s.extract(py).map_err(|e| {
        PyTypeError::new_err(format!(
            "the hessian object must be bytes or bytearray, got: {:?}",
            e
        ))
    })?;

    let value = hessian_rs::from_slice(&bytes)
        .map_err(|e| PyTypeError::new_err(format!("Parse hessian error: {:?}", e)))?;

    Ok(HessianValueWrapper(value).to_object(py))
}

struct HessianValueWrapper(hessian_rs::Value);

impl ToPyObject for HessianValueWrapper {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match &self.0 {
            hessian_rs::Value::Null => py.None(),
            hessian_rs::Value::Bool(b) => b.to_object(py),
            hessian_rs::Value::Int(i) => i.to_object(py),
            hessian_rs::Value::Long(l) => l.to_object(py),
            hessian_rs::Value::Double(d) => d.to_object(py),
            hessian_rs::Value::Date(d) => {
                PyDateTime::from_timestamp(py, (*d as f64) / 1000.0, Some(timezone_utc(py)))
                    .unwrap()
                    .to_object(py)
            }
            hessian_rs::Value::String(s) => s.to_object(py),
            hessian_rs::Value::Bytes(b) => PyBytes::new(py, b).to_object(py),
            hessian_rs::Value::List(l) => l
                .value()
                .iter()
                .map(|v| HessianValueWrapper(v.clone()).to_object(py))
                .collect::<Vec<_>>()
                .to_object(py),
            hessian_rs::Value::Map(m) => {
                let dict = PyDict::new(py);
                for (k, v) in m.value().iter() {
                    dict.set_item(
                        HessianValueWrapper(k.clone()).to_object(py),
                        HessianValueWrapper(v.clone()).to_object(py),
                    )
                    .unwrap();
                }
                dict.to_object(py)
            }
            _ => py.None(),
        }
    }
}

#[pyfunction]
pub fn dump(
    py: Python,
    obj: PyObject,
    fp: PyObject,
    allow_nan: Option<PyObject>,
    cls: Option<PyObject>,
    default: Option<PyObject>,
    kwargs: Option<&PyDict>,
) -> PyResult<PyObject> {
    let s = dumps(py, obj, allow_nan, cls, default, kwargs)?;
    let fp_ref: &PyAny = fp.extract(py)?;
    fp_ref.call_method1("write", (s,))?;
    Ok(pyo3::Python::None(py))
}

#[pyfunction]
pub fn dumps(
    py: Python,
    obj: PyObject,
    _allow_nan: Option<PyObject>,
    _cls: Option<PyObject>,
    _default: Option<PyObject>,
    _kwargs: Option<&PyDict>,
) -> PyResult<PyObject> {
    let mut buf = Vec::new();
    let mut ser = hessian_rs::ser::Serializer::new(&mut buf);
    dump_value(obj.extract(py)?, &mut ser)?;
    Ok(PyBytes::new(py, &buf).into())
}

fn convert_err(e: hessian_rs::Error) -> PyErr {
    PyErr::new::<PyValueError, _>(format!("Cannot serialize value: {:?}", e))
}

fn dump_value<'a, W>(obj: &'a PyAny, ser: &'a mut Serializer<W>) -> PyResult<()>
where
    W: Write,
{
    if let Ok(val) = obj.extract::<PySerializeObject>() {
        let def = Definition {
            name: val.class_name,
            fields: val.fields,
        };
        ser.write_definition(&def).map_err(convert_err)?;
        ser.write_object_start(&def).map_err(convert_err)?;
        for v in val.values {
            dump_value(v, ser)?;
        }
        return Ok(());
    }

    if let Ok(val) = obj.extract::<&'a PyDict>() {
        ser.write_map_start(None).map_err(convert_err)?;
        for (k, v) in val.iter() {
            dump_value(k, ser)?;
            dump_value(v, ser)?;
        }
        ser.write_object_end().map_err(convert_err)?;
        return Ok(());
    }

    if let Ok(val) = obj.extract::<&'a PyList>() {
        ser.write_list_begin(val.len(), None).map_err(convert_err)?;
        for v in val.iter() {
            dump_value(v, ser)?;
        }
        ser.write_object_end().map_err(convert_err)?;
        return Ok(());
    }

    if let Ok(val) = obj.extract::<&'a PyTuple>() {
        ser.write_list_begin(val.len(), None).map_err(convert_err)?;
        for v in val.iter() {
            dump_value(v, ser)?;
        }
        ser.write_object_end().map_err(convert_err)?;
        return Ok(());
    }

    if let Ok(val) = obj.extract::<&'a PyDateTime>() {
        let timestamp = val.call_method0("timestamp")?.extract::<f64>()?;
        ser.serialize_date((timestamp * 1000.0) as i64)
            .map_err(convert_err)?;
        return Ok(());
    }

    if let Ok(val) = FromPyObject::extract(obj) {
        return ser.serialize_string(val).map_err(convert_err);
    }
    if let Ok(val) = FromPyObject::extract(obj) {
        return ser.serialize_binary(val).map_err(convert_err);
    }
    if let Ok(val) = FromPyObject::extract(obj) {
        return ser.serialize_bool(val).map_err(convert_err);
    }
    if let Ok(val) = FromPyObject::extract(obj) {
        return ser.serialize_int(val).map_err(convert_err);
    }
    if let Ok(val) = FromPyObject::extract(obj) {
        return ser.serialize_long(val).map_err(convert_err);
    }
    if let Ok(val) = FromPyObject::extract(obj) {
        return ser.serialize_double(val).map_err(convert_err);
    }
    if obj.is_none() {
        return ser.serialize_null().map_err(convert_err);
    }
    match obj.repr() {
        Ok(repr) => Err(PyErr::new::<PyValueError, _>(format!(
            "Value is not hessian serializable: {}",
            repr
        ))),
        Err(_) => Err(PyErr::new::<PyValueError, _>(format!(
            "Type is not JSON serializable: {}",
            obj.get_type().name()?
        ))),
    }
}

#[derive(FromPyObject)]
struct PySerializeObject<'a> {
    #[pyo3(attribute("hessian_class_name"))]
    class_name: String,
    #[pyo3(attribute("hessian_fields"))]
    fields: Vec<String>,
    #[pyo3(attribute("hessian_values"))]
    values: Vec<&'a PyAny>,
}

#[pyclass]
struct PyHessianSerializer {
    ser: hessian_rs::ser::Serializer<Vec<u8>>,
}

#[pymethods]
impl PyHessianSerializer {
    #[new]
    fn new() -> Self {
        Self {
            ser: hessian_rs::ser::Serializer::new(Vec::new()),
        }
    }

    fn serialize_bool(&mut self, b: &PyBool) -> PyResult<()> {
        self.ser.serialize_bool(b.is_true()).map_err(convert_err)?;
        Ok(())
    }

    fn serialize_bytes(&mut self, bytes: &PyBytes) -> PyResult<()> {
        self.ser
            .serialize_binary(bytes.as_bytes())
            .map_err(convert_err)?;
        Ok(())
    }

    fn serialize_string(&mut self, s: &PyString) -> PyResult<()> {
        self.ser
            .serialize_string(s.to_str().unwrap())
            .map_err(convert_err)?;
        Ok(())
    }

    fn serialize_long(&mut self, l: &PyInt) -> PyResult<()> {
        self.ser.serialize_long(l.extract()?).map_err(convert_err)?;
        Ok(())
    }

    fn serialize_double(&mut self, d: &PyFloat) -> PyResult<()> {
        self.ser
            .serialize_double(d.extract()?)
            .map_err(convert_err)?;
        Ok(())
    }

    fn serialize_date(&mut self, d: &PyDateTime) -> PyResult<()> {
        let timestamp = d.call_method0("timestamp")?.extract::<f64>()?;
        self.ser
            .serialize_date((timestamp * 1000.0) as i64)
            .map_err(convert_err)?;
        Ok(())
    }

    fn serialize_value(&mut self, d: &PyAny) -> PyResult<()> {
        dump_value(d, &mut self.ser)?;
        Ok(())
    }

    fn serialize_map(&mut self, d: &PyDict, typ: Option<&PyString>) -> PyResult<()> {
        self.ser
            .write_map_start(typ.map(|t| t.to_str().unwrap()))
            .map_err(convert_err)?;
        for (k, v) in d.iter() {
            self.serialize_string(k.extract()?)?;
            self.serialize_value(v)?;
        }
        self.ser.write_object_end().map_err(convert_err)?;
        Ok(())
    }
}

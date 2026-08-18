// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

//! Serialization and deserialization of pulse parameter values to/from JSON bytes.

use pyo3::intern;
use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyBytes;

/// Serialize a Python pulse parameter value to JSON bytes.
///
/// Converts the value with `_unstructure_pulse_parameter_value`, serializes to
/// JSON with `orjson.dumps`.
pub fn serialize_json(obj: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    let py = obj.py();
    let unstructure_fn =
        cached_model_fn(py, &UNSTRUCTURE_FN, "_unstructure_pulse_parameter_value")?;
    let orjson_dumps = cached_orjson_fn(py, &ORJSON_DUMPS, "dumps")?;

    let unstructured = unstructure_fn.call1((obj,))?;
    let json_bytes_obj = orjson_dumps.call1((unstructured,))?;
    let json_bytes: &[u8] = json_bytes_obj.cast::<PyBytes>()?.as_bytes();
    Ok(json_bytes.to_vec())
}

/// Deserialize JSON bytes back to a Python pulse parameter value.
///
/// Parses the JSON with `orjson.loads`, then reconstructs
/// the Python object with `_structure_pulse_parameter_value`.
pub fn deserialize_json<'py>(py: Python<'py>, bytes: &[u8]) -> PyResult<Bound<'py, PyAny>> {
    let structure_fn = cached_model_fn(py, &STRUCTURE_FN, "_structure_pulse_parameter_value")?;
    let orjson_loads = cached_orjson_fn(py, &ORJSON_LOADS, "loads")?;

    let parsed = orjson_loads.call1((bytes,))?;
    structure_fn.call1((parsed,))
}

static UNSTRUCTURE_FN: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
static STRUCTURE_FN: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
static ORJSON_DUMPS: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
static ORJSON_LOADS: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

/// Resolve `laboneq.serializers.implementations._models._experiment.<name>`, caching the
/// result so repeated calls skip the module import and attribute lookup.
fn cached_model_fn<'py>(
    py: Python<'py>,
    cell: &'static PyOnceLock<Py<PyAny>>,
    name: &str,
) -> PyResult<&'py Bound<'py, PyAny>> {
    cell.get_or_try_init(py, || {
        py.import(intern!(
            py,
            "laboneq.serializers.implementations._models._experiment"
        ))?
        .getattr(name)
        .map(Bound::unbind)
    })
    .map(|f| f.bind(py))
}

fn cached_orjson_fn<'py>(
    py: Python<'py>,
    cell: &'static PyOnceLock<Py<PyAny>>,
    name: &str,
) -> PyResult<&'py Bound<'py, PyAny>> {
    cell.get_or_try_init(py, || {
        py.import(intern!(py, "orjson"))?
            .getattr(name)
            .map(Bound::unbind)
    })
    .map(|f| f.bind(py))
}

#[cfg(test)]
mod tests {
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyList};

    use super::{deserialize_json, serialize_json};

    #[test]
    fn test_nested_list_and_dict_roundtrip() {
        Python::attach(|py| {
            // The values this module exists for: nested lists/dicts that aren't
            // handled by the native scalar/Parameter-ref fast path.
            let inner_list = PyList::new(py, [1i64, 2, 3]).unwrap();
            let dict = PyDict::new(py);
            dict.set_item("a", &inner_list).unwrap();
            dict.set_item("b", "some string").unwrap();
            dict.set_item("c", 1.5f64).unwrap();
            let obj = dict.into_any();

            let bytes = serialize_json(&obj).unwrap();

            let roundtripped = deserialize_json(py, &bytes).unwrap();
            assert!(obj.eq(&roundtripped).unwrap());
        });
    }

    #[test]
    fn test_deserialize_json_rejects_non_json_bytes() {
        Python::attach(|py| {
            assert!(deserialize_json(py, b"not json").is_err());
        });
    }
}

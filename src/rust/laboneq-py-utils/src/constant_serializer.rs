// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

//! Serialization and deserialization of pulse parameter values to/from JSON bytes.
//!
//! JSON bytes are stored in the capnp `rawBytesValue` field, distinguished from plain user
//! bytes by a fixed magic prefix (`JSON_PREFIX`). This avoids adding a new capnp union variant.

use pyo3::exceptions::PyValueError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyBytes;

/// Magic prefix prepended to all compiler-internal JSON bytes.
///
/// Valid JSON never starts with a null byte, so this prefix is unambiguous when
/// stored alongside user-provided raw bytes in the same capnp `rawBytesValue` field.
const JSON_PREFIX: &[u8] = b"\x00\x00JSON";

/// Serialize a Python pulse parameter value to prefixed JSON bytes.
///
/// Converts the value with `_unstructure_pulse_parameter_value`, serializes to
/// JSON with `orjson.dumps`, then prepends `JSON_PREFIX`.
pub fn serialize_json<'py>(py: Python<'py>, obj: &Bound<'py, PyAny>) -> PyResult<Vec<u8>> {
    let unstructure_fn =
        cached_model_fn(py, &UNSTRUCTURE_FN, "_unstructure_pulse_parameter_value")?;
    let orjson_dumps = cached_orjson_fn(py, &ORJSON_DUMPS, "dumps")?;

    let unstructured = unstructure_fn.call1((obj,))?;
    let json_bytes_obj = orjson_dumps.call1((unstructured,))?;
    let json_bytes: &[u8] = json_bytes_obj.cast::<PyBytes>()?.as_bytes();

    let mut prefixed = Vec::with_capacity(JSON_PREFIX.len() + json_bytes.len());
    prefixed.extend_from_slice(JSON_PREFIX);
    prefixed.extend_from_slice(json_bytes);
    Ok(prefixed)
}

/// Deserialize prefixed JSON bytes back to a Python pulse parameter value.
///
/// Strips `JSON_PREFIX`, parses the JSON with `orjson.loads`, then reconstructs
/// the Python object with `_structure_pulse_parameter_value`.
pub fn deserialize_json<'py>(py: Python<'py>, bytes: &[u8]) -> PyResult<Bound<'py, PyAny>> {
    let json_bytes = bytes
        .strip_prefix(JSON_PREFIX)
        .ok_or_else(|| PyValueError::new_err("bytes do not start with the expected JSON prefix"))?;

    let structure_fn = cached_model_fn(py, &STRUCTURE_FN, "_structure_pulse_parameter_value")?;
    let orjson_loads = cached_orjson_fn(py, &ORJSON_LOADS, "loads")?;

    let parsed = orjson_loads.call1((json_bytes,))?;
    structure_fn.call1((parsed,))
}

/// Returns `true` if `bytes` were produced by [`serialize_json`].
pub fn is_json_prefixed(bytes: &[u8]) -> bool {
    bytes.starts_with(JSON_PREFIX)
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

    use super::{deserialize_json, is_json_prefixed, serialize_json};

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

            let bytes = serialize_json(py, &obj).unwrap();
            assert!(is_json_prefixed(&bytes));

            let roundtripped = deserialize_json(py, &bytes).unwrap();
            assert!(obj.eq(&roundtripped).unwrap());
        });
    }

    #[test]
    fn test_is_json_prefixed_false_for_arbitrary_bytes() {
        assert!(!is_json_prefixed(b"not json"));
    }

    #[test]
    fn test_deserialize_json_rejects_unprefixed_bytes() {
        Python::attach(|py| {
            assert!(deserialize_json(py, b"not json").is_err());
        });
    }
}

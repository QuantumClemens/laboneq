// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;

use laboneq_ir::pulse_sheet_schedule::PulseSheetSchedule;

use crate::error::Error;
use crate::error::Result as CompilerResult;

/// Opaque carrier for a [`PulseSheetSchedule`] as it passes through the Python compilation steps.
#[pyclass]
pub(crate) struct PulseSheetSchedulePy {
    pub(crate) inner: PulseSheetSchedule,
}

impl PulseSheetSchedulePy {
    pub(crate) fn take_inner(&mut self) -> PulseSheetSchedule {
        std::mem::take(&mut self.inner)
    }
}

/// Convert a [`PulseSheetSchedule`] to a Python dict for consumption in Python.
///
/// # Returns
/// A Python dict containing:
/// - event_list: List of scheduler events
/// - event_list_truncated: Whether event generation hit the max_events limit
/// - section_info: Section metadata with preorder map
/// - section_signals_with_children: Signal hierarchy per section
/// - sampling_rates: Sampling rates per device type
pub(crate) fn schedule_to_py<'py>(
    py: Python<'py>,
    schedule: &PulseSheetSchedule,
) -> CompilerResult<Bound<'py, PyAny>> {
    // Convert to JSON then to Python dict
    let json_value = serde_json::to_value(schedule)
        .map_err(|e| Error::new(format!("Failed to serialize schedule: {}", e)))?;
    // Convert JSON to Python object
    json_to_py(py, &json_value)
}

/// Helper function to convert serde_json::Value to Python objects
fn json_to_py<'py>(
    py: Python<'py>,
    value: &serde_json::Value,
) -> CompilerResult<Bound<'py, PyAny>> {
    use pyo3::IntoPyObject;
    use pyo3::types::{PyDict, PyList};
    use serde_json::Value;

    match value {
        Value::Null => Ok(py.None().into_bound(py)),
        Value::Bool(b) => {
            let py_bool = b
                .into_pyobject(py)
                .map_err(|e| Error::new(format!("Failed to convert bool: {}", e)))?;
            Ok(py_bool.to_owned().into_any())
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                let py_int = i
                    .into_pyobject(py)
                    .map_err(|e| Error::new(format!("Failed to convert i64: {}", e)))?;
                Ok(py_int.to_owned().into_any())
            } else if let Some(u) = n.as_u64() {
                let py_int = u
                    .into_pyobject(py)
                    .map_err(|e| Error::new(format!("Failed to convert u64: {}", e)))?;
                Ok(py_int.to_owned().into_any())
            } else if let Some(f) = n.as_f64() {
                let py_float = f
                    .into_pyobject(py)
                    .map_err(|e| Error::new(format!("Failed to convert f64: {}", e)))?;
                Ok(py_float.to_owned().into_any())
            } else {
                Err(Error::new("Invalid JSON number"))
            }
        }
        Value::String(s) => {
            let py_str = s
                .into_pyobject(py)
                .map_err(|e| Error::new(format!("Failed to convert string: {}", e)))?;
            Ok(py_str.to_owned().into_any())
        }
        Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                py_list.append(json_to_py(py, item)?)?;
            }
            Ok(py_list.into_any())
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (key, val) in obj {
                py_dict.set_item(key, json_to_py(py, val)?)?;
            }
            Ok(py_dict.into_any())
        }
    }
}

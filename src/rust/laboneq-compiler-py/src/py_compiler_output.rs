// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::Arc;

use numpy::PyArray1;
use pyo3::IntoPyObjectExt;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use laboneq_dsl::types::ParameterUid;
use laboneq_dsl::types::SweepParameter;
use numeric_array::NumericArray;

use laboneq_common::named_id::NamedIdStore;
use laboneq_dsl::types::ExternalParameterUid;
use laboneq_py_utils::py_export::{acquisition_type_to_py, averaging_mode_to_py};
use laboneq_py_utils::py_object_store::PyObjectStore;

use crate::compiled_experiment::{CompiledExperiment, RealTimeProperties};
use crate::compiler_backend::CompilerArtifact;
use crate::py_execution::create_py_execution;
use crate::py_pulse_sheet_schedule::schedule_to_py;
use crate::result_shape::{AxisValues, HandleResultShape};

/// Builds the `ScheduledExperiment` Python object.
pub(crate) fn build_scheduled_experiment_py<'py>(
    py: Python<'py>,
    mut compiled_experiment: CompiledExperiment<impl CompilerArtifact>,
    id_store: &NamedIdStore,
    py_object_store: &PyObjectStore<ExternalParameterUid>,
) -> PyResult<Bound<'py, PyAny>> {
    let object_index = &mut PyObjectIndex::new(py);

    let scheduled_experiment_class = py
        .import(intern!(py, "laboneq.data.scheduled_experiment"))?
        .getattr(intern!(py, "ScheduledExperiment"))?;

    let kwargs = PyDict::new(py);
    kwargs.set_item(
        intern!(py, "device_setup_fingerprint"),
        compiled_experiment.device_setup_fingerprint,
    )?;
    let artifacts = compiled_experiment
        .artifacts
        .to_python(py, id_store, py_object_store)?;
    kwargs.set_item(intern!(py, "artifacts"), artifacts)?;
    let schedule_py = compiled_experiment
        .pulse_sheet_schedule
        .as_ref()
        .map(|s| schedule_to_py(py, s))
        .transpose()?;
    kwargs.set_item(intern!(py, "schedule"), schedule_py)?;

    let execution_py = create_py_execution(
        py,
        &compiled_experiment.execution,
        compiled_experiment.sweep_parameters.as_slice(),
        id_store,
        py_object_store,
    )?;
    kwargs.set_item(intern!(py, "execution"), execution_py)?;

    kwargs.set_item(
        intern!(py, "rt_loop_properties"),
        create_py_rt_loop_properties(py, &compiled_experiment.real_time_properties)?,
    )?;
    kwargs.set_item(
        intern!(py, "result_shape_info"),
        create_py_result_shape_info(
            py,
            compiled_experiment.result_shapes.handle_result_shapes,
            compiled_experiment.sweep_parameters.as_slice(),
            object_index,
            id_store,
        )?,
    )?;
    kwargs.set_item(
        intern!(py, "total_execution_time"),
        compiled_experiment.execution_timing.total_execution_time,
    )?;
    kwargs.set_item(
        intern!(py, "max_step_execution_time"),
        compiled_experiment.execution_timing.max_step_execution_time,
    )?;
    kwargs.set_item(
        intern!(py, "versions"),
        init_software_versions(py, &compiled_experiment.metadata.producer_version)?,
    )?;
    let scheduled_experiment = scheduled_experiment_class.call((), Some(&kwargs))?;
    Ok(scheduled_experiment)
}

/// A helper struct to keep track of already converted Python objects.
struct PyObjectIndex<'py> {
    py: Python<'py>,
    explicit_arrays: HashMap<*const NumericArray, Bound<'py, PyAny>>,
    sweep_parameters: HashMap<ParameterUid, Bound<'py, PyAny>>,
    shots_to_arr: HashMap<usize, Bound<'py, PyAny>>,
}

impl<'py> PyObjectIndex<'py> {
    fn new(py: Python<'py>) -> Self {
        Self {
            py,
            explicit_arrays: HashMap::new(),
            sweep_parameters: HashMap::new(),
            shots_to_arr: HashMap::new(),
        }
    }

    /// Get or insert a Python object for the given explicit array, caching it for future use.
    fn get_or_insert_explicit_array(
        &mut self,
        array: &Arc<NumericArray>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let ptr = Arc::as_ptr(array);
        if let Some(py_obj) = self.explicit_arrays.get(&ptr) {
            Ok(py_obj.clone())
        } else {
            let py_obj = array.to_py(self.py)?;
            self.explicit_arrays.insert(ptr, py_obj.clone());
            Ok(py_obj)
        }
    }

    /// Create a NumPy array with values from the sweep parameter and cache it for future use.
    fn get_or_insert_sweep_parameter_array(
        &mut self,
        param: &SweepParameter,
    ) -> PyResult<Bound<'py, PyAny>> {
        if let Some(py_obj) = self.sweep_parameters.get(&param.uid) {
            Ok(py_obj.clone())
        } else {
            let py_obj = param.values.to_py(self.py)?;
            self.sweep_parameters.insert(param.uid, py_obj.clone());
            Ok(py_obj)
        }
    }

    /// Create a NumPy array with values `0..shots`, as type float64.
    fn get_or_insert_shots_array(&mut self, shots: usize) -> PyResult<Bound<'py, PyAny>> {
        if let Some(py_obj) = self.shots_to_arr.get(&shots) {
            Ok(py_obj.clone())
        } else {
            let py_obj = PyArray1::from_iter(self.py, (0..shots).map(|v| v as f64))
                .into_bound_py_any(self.py)?;
            self.shots_to_arr.insert(shots, py_obj.clone());
            Ok(py_obj)
        }
    }
}

/// Build a `laboneq.data.scheduled_experiment.RtLoopProperties` Python object.
fn create_py_rt_loop_properties<'py>(
    py: Python<'py>,
    properties: &RealTimeProperties,
) -> PyResult<Bound<'py, PyAny>> {
    let acquisition_type = py
        .import(intern!(py, "laboneq.core.types.enums.acquisition_type"))?
        .getattr(intern!(py, "AcquisitionType"))?
        .getattr(acquisition_type_to_py(&properties.acquisition_type))?;
    let averaging_mode = py
        .import(intern!(py, "laboneq.core.types.enums.averaging_mode"))?
        .getattr(intern!(py, "AveragingMode"))?
        .getattr(averaging_mode_to_py(&properties.averaging_mode))?;

    let rt_loop_properties_class = py
        .import(intern!(py, "laboneq.data.scheduled_experiment"))?
        .getattr(intern!(py, "RtLoopProperties"))?;

    let kwargs = PyDict::new(py);
    kwargs.set_item(intern!(py, "acquisition_type"), acquisition_type)?;
    kwargs.set_item(intern!(py, "averaging_mode"), averaging_mode)?;
    kwargs.set_item(intern!(py, "shots"), properties.shots)?;
    kwargs.set_item(intern!(py, "chunk_count"), properties.chunk_count)?;
    rt_loop_properties_class.call((), Some(&kwargs))
}

/// Build a `laboneq.data.scheduled_experiment.ResultShapeInfo` Python object.
///
/// A shape's axis_names/axis_values carry one entry per loop axis; an axis with a
/// single sweep parameter is unwrapped to a bare value instead of a one-element list,
/// matching `laboneq.data.scheduled_experiment.HandleResultShape`'s expected shape.
fn create_py_result_shape_info<'py>(
    py: Python<'py>,
    handle_result_shapes: Vec<HandleResultShape>,
    sweep_parameters: &[SweepParameter],
    index: &mut PyObjectIndex<'py>,
    id_store: &NamedIdStore,
) -> PyResult<Bound<'py, PyAny>> {
    let handle_result_shape_class = py
        .import(intern!(py, "laboneq.data.scheduled_experiment"))?
        .getattr(intern!(py, "HandleResultShape"))?;

    let shapes = PyDict::new(py);
    for shape in handle_result_shapes {
        let resolved = create_result_shape_py(shape, sweep_parameters, index, id_store)?;

        let shape_tuple = PyTuple::new(py, resolved.shape)?;
        let axis_names = resolved
            .axis_names
            .into_iter()
            .map(|names| flatten_single_axis(py, names))
            .collect::<PyResult<Vec<_>>>()?;
        let axis_values = resolved
            .axis_values
            .into_iter()
            .map(|values| flatten_single_axis(py, values))
            .collect::<PyResult<Vec<_>>>()?;
        let match_case_mask = if resolved.match_case_mask.is_empty() {
            py.None()
        } else {
            resolved.match_case_mask.into_py_any(py)?
        };

        let kwargs = PyDict::new(py);
        kwargs.set_item(intern!(py, "shape"), shape_tuple)?;
        kwargs.set_item(intern!(py, "axis_names"), PyList::new(py, axis_names)?)?;
        kwargs.set_item(intern!(py, "axis_values"), PyList::new(py, axis_values)?)?;
        kwargs.set_item(
            intern!(py, "chunked_axis_index"),
            resolved.chunked_axis_index,
        )?;
        kwargs.set_item(intern!(py, "match_case_mask"), match_case_mask)?;
        let shape_py = handle_result_shape_class.call((), Some(&kwargs))?;

        shapes.set_item(resolved.handle, shape_py)?;
    }

    let result_shape_info_class = py
        .import(intern!(py, "laboneq.data.scheduled_experiment"))?
        .getattr(intern!(py, "ResultShapeInfo"))?;

    let kwargs = PyDict::new(py);
    kwargs.set_item(intern!(py, "shapes"), shapes)?;
    result_shape_info_class.call((), Some(&kwargs))
}

/// A [`HandleResultShape`] with its UIDs resolved to names and its axis values
/// converted to Python objects, ready to be assembled into the final
/// `laboneq.data.scheduled_experiment.HandleResultShape` dataclass.
struct ResolvedHandleResultShape<'py> {
    handle: String,
    shape: Vec<usize>,
    axis_names: Vec<Vec<String>>,
    axis_values: Vec<Vec<Bound<'py, PyAny>>>, // list[list[NumPyArray]]
    chunked_axis_index: Option<usize>,
    match_case_mask: HashMap<usize, Vec<usize>>, // dict[int, list[int]]
}

fn create_result_shape_py<'py>(
    result_shape: HandleResultShape,
    sweep_parameters: &[SweepParameter],
    index: &mut PyObjectIndex<'py>,
    id_store: &NamedIdStore,
) -> PyResult<ResolvedHandleResultShape<'py>> {
    let shape = result_shape.shape;
    let axis_names = result_shape
        .axis_names
        .into_iter()
        .map(|names| {
            names
                .into_iter()
                .map(|name| id_store.resolve(name).unwrap().to_string())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let axis_values = result_shape
        .axis_values
        .into_iter()
        .map(|values| {
            values
                .into_iter()
                .map(|value| match value {
                    AxisValues::Shots(shots) => index.get_or_insert_shots_array(shots),
                    AxisValues::Explicit(array) => index.get_or_insert_explicit_array(&array),
                    AxisValues::Parameter(param_uid) => {
                        let param = sweep_parameters
                            .iter()
                            .find(|p| p.uid == param_uid)
                            .expect("Sweep parameter not found");
                        index.get_or_insert_sweep_parameter_array(param)
                    }
                })
                .collect::<Result<Vec<Bound<'py, PyAny>>, PyErr>>()
        })
        .collect::<Result<Vec<Vec<_>>, PyErr>>()?;

    let chunked_axis_index = result_shape.chunked_axis_index;
    let match_case_mask = result_shape.match_case_mask;

    Ok(ResolvedHandleResultShape {
        handle: id_store.resolve(result_shape.handle).unwrap().to_string(),
        shape,
        axis_names,
        axis_values,
        chunked_axis_index,
        match_case_mask: match_case_mask.into_iter().collect(),
    })
}

/// Unwrap a single-element axis to its bare value, matching how the Python-side
/// dataclass represents an axis with just one name/value instead of a one-element list.
fn flatten_single_axis<'py, T>(py: Python<'py>, mut items: Vec<T>) -> PyResult<Bound<'py, PyAny>>
where
    T: IntoPyObject<'py>,
    Vec<T>: IntoPyObject<'py>,
{
    if items.len() == 1 {
        items.remove(0).into_bound_py_any(py)
    } else {
        items.into_bound_py_any(py)
    }
}

fn init_software_versions<'py>(py: Python<'py>, version: &str) -> PyResult<Bound<'py, PyAny>> {
    let versions_class = py
        .import(intern!(py, "laboneq.data.scheduled_experiment"))?
        .getattr(intern!(py, "SoftwareVersions"))?;

    let kwargs = PyDict::new(py);
    kwargs.set_item(intern!(py, "laboneq"), version)?;
    versions_class.call((), Some(&kwargs))
}

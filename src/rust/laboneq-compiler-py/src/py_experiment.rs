// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;
use std::sync::Arc;

use laboneq_common::compiler_settings::CompilerSettings;
use laboneq_ir::system::DeviceSetup;

use crate::compiler_backend::{DynCompilerBackend, PreprocessedBackendData};
use crate::experiment::Experiment;
use crate::py_helpers::precompensation_to_py;
use crate::setup_processor::DelayRegistry;

#[pyclass(name = "Experiment", frozen)]
pub struct ExperimentPy {
    pub(crate) inner: Experiment,
    // NOTE: The usage of Arc here is to allow sharing the id_store across Python bindings
    // Remove when Python bindings are no longer needed
    pub(crate) device_setup: Arc<DeviceSetup>,
    /// Delay compensation for signals on devices.
    pub(crate) delay_compensation: DelayRegistry,
    pub(crate) compiler_settings: CompilerSettings,
    pub(crate) backend: Arc<dyn DynCompilerBackend>,
    pub(crate) backend_data: Arc<dyn PreprocessedBackendData + Send + Sync>,
}

#[pymethods]
impl ExperimentPy {
    fn signal_delay_compensation(&self, signal_uid: &str) -> f64 {
        let uid = self.inner.id_store.get(signal_uid).unwrap().into();
        self.delay_compensation.signal_port_delay(uid).into()
    }

    fn device_lead_delay(&self, device_uid: &str) -> f64 {
        let uid = self.inner.id_store.get(device_uid).unwrap().into();
        self.delay_compensation.device_lead_delay(uid).into()
    }

    fn signal_precompensation<'py>(
        &self,
        py: Python<'py>,
        signal_uid: &str,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        let uid = self.inner.id_store.get(signal_uid).unwrap().into();
        if let Some(signal) = self.device_setup.signal_by_uid(&uid)
            && let Some(precomp) = &signal.precompensation
        {
            return precompensation_to_py(py, precomp).map(|d| Some(d.into_any()));
        }
        Ok(None)
    }
}

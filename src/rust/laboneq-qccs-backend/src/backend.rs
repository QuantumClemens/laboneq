// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

use pyo3::prelude::*;

use codegenerator_py::{HardwareSetup, SignalChannelProperties, artifacts_to_py, generate_code_py};
use laboneq_common::compiler_settings::CompilerSettings;
use laboneq_compiler_py::compiler_backend::{
    CodeGenArtifact, CompilerBackend, Error as CompilerError, ExperimentView, FeedbackCalculator,
    SignalView,
};
use laboneq_compiler_py::compiler_backend::{CompilerBackendResult, PreprocessOutput};
use laboneq_dsl::types::ExternalParameterUid;
use laboneq_error::LabOneQError;
use laboneq_ir::ExperimentIr;
use laboneq_py_utils::py_object_interner::PyObjectInterner;

use crate::preprocessor::QccsBackendPreprocessedData;
use crate::preprocessor::preprocess_experiment;
use crate::qccs_feedback_calculator::{FeedbackSignal, QccsFeedbackCalculator};

#[derive(Default)]
pub struct QccsBackend {}

impl CompilerBackend for QccsBackend {
    type Output = QccsBackendPreprocessedData;
    type CodeGenArtifact = CodeGenArtifactQccs;

    fn preprocess_experiment(
        &self,
        experiment: ExperimentView,
    ) -> CompilerBackendResult<PreprocessOutput<Self::Output>> {
        preprocess_experiment(experiment)
    }

    fn generate_code(
        &self,
        experiment: ExperimentIr,
        compiler_settings: &CompilerSettings,
        py_object_store: &PyObjectInterner<ExternalParameterUid>,
        backend_data: &Self::Output,
    ) -> CompilerBackendResult<Self::CodeGenArtifact> {
        let additional_signals = backend_data.signals().map(|s| SignalChannelProperties {
            signal_uid: s.uid,
            awg_key: s.awg_key,
            awg_index: s.awg_index,
            channels: s.channels.iter().map(|c| *c as u8).collect(),
            routed_output_channel_map: backend_data.routed_output_channel_map().clone(),
            ppc_channel: s.ppc_channel,
        });
        let setup_desc = HardwareSetup {
            signals: additional_signals.collect(),
            auxiliary_devices: backend_data.auxiliary_devices().to_vec(),
        };

        let id_store = &experiment.id_store;

        let out = Python::attach(|py| -> Result<Py<PyAny>, LabOneQError> {
            let artifacts = generate_code_py(
                py,
                experiment,
                &setup_desc,
                compiler_settings,
                py_object_store,
            )?;
            let artifacts_py = artifacts_to_py(py, artifacts, id_store, py_object_store)?;
            Ok(artifacts_py.into())
        })?;
        Ok(CodeGenArtifactQccs { inner: out })
    }

    fn device_class(&self) -> usize {
        0
    }

    fn feedback_calculator(
        &self,
        signals: &[SignalView],
        _compiler_settings: &CompilerSettings,
    ) -> Result<
        Option<Box<dyn FeedbackCalculator<Error = CompilerError> + Send + Sync + 'static>>,
        CompilerError,
    > {
        let feedback_signals = signals.iter().map(|s| FeedbackSignal {
            uid: s.uid(),
            awg_key: *s.awg_key(),
            signal_kind: s.signal_kind().clone(),
            device_kind: s.device_kind(),
            is_shfqc: s.device().is_shfqc(),
            sampling_rate: s.sampling_rate(),
            signal_delay: s.signal_delay(),
            port_delay: s.port_delay().copied(),
            start_delay: s.start_delay(),
        });
        let model = QccsFeedbackCalculator::new(feedback_signals)?;
        Ok(Some(Box::new(model)))
    }
}

pub struct CodeGenArtifactQccs {
    inner: Py<PyAny>,
}

impl CodeGenArtifact for CodeGenArtifactQccs {
    fn to_python<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        Ok(self.inner.bind(py).into())
    }
}

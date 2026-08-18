// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

use pyo3::{intern, prelude::*};

use codegenerator_py::{HardwareSetup, SignalChannelProperties, artifacts_to_py, generate_code_py};
use laboneq_common::compiler_settings::CompilerSettings;
use laboneq_common::named_id::NamedIdStore;
use laboneq_compiler_py::compiler_backend::{
    CodeGenArtifact, CombinedOutput, CompilerArtifact, CompilerBackend, Error as CompilerError,
    ExperimentView, FeedbackCalculator, SignalView,
};
use laboneq_compiler_py::compiler_backend::{CompilerBackendResult, PreprocessOutput};
use laboneq_dsl::types::{ExternalParameterUid, HandleUid, SignalUid};
use laboneq_error::LabOneQError;
use laboneq_ir::ExperimentIr;
use laboneq_py_utils::py_object_store::PyObjectStore;

use crate::preprocessor::QccsBackendPreprocessedData;
use crate::preprocessor::preprocess_experiment;
use crate::qccs_feedback_calculator::{FeedbackSignal, QccsFeedbackCalculator};

#[derive(Default, Clone)]
pub struct QccsBackend {}

impl CompilerBackend for QccsBackend {
    type Output = QccsBackendPreprocessedData;
    type CodeGenArtifact = CodeGenArtifactQccs;
    type CompilerArtifact = CompilationArtifactsQccs;
    type CombinedOutput = QccsCombinedOutput;

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
        py_object_store: &PyObjectStore<ExternalParameterUid>,
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

    fn combined_output_from_py(
        &self,
        obj: &Bound<'_, PyAny>,
    ) -> CompilerBackendResult<Self::CombinedOutput> {
        Ok(QccsCombinedOutput(obj.clone().unbind()))
    }

    fn finalize(
        &self,
        combined: Self::CombinedOutput,
    ) -> CompilerBackendResult<
        laboneq_compiler_py::compiler_backend::CompilationOutput<Self::CompilerArtifact>,
    > {
        Python::attach(|py| {
            let obj = combined.0.bind(py);
            let artifacts_py = obj.call_method0(intern!(py, "get_artifacts"))?;
            Ok(laboneq_compiler_py::compiler_backend::CompilationOutput {
                artifacts: CompilationArtifactsQccs {
                    artifacts: artifacts_py.into(),
                },
            })
        })
    }
}

/// QCCS's linked real-time compilation output. Still fully backed by the Python
/// `CombinedOutput` object, since QCCS's linker has not been ported to Rust.
pub struct QccsCombinedOutput(Py<PyAny>);

impl CombinedOutput for QccsCombinedOutput {
    fn raw_acquisition_lengths(
        &self,
        pairs: &[(SignalUid, HandleUid)],
        id_store: &NamedIdStore,
    ) -> CompilerBackendResult<Vec<(SignalUid, HandleUid, usize)>> {
        Python::attach(|py| {
            let obj = self.0.bind(py);
            pairs
                .iter()
                .map(|&(signal, handle)| -> CompilerBackendResult<_> {
                    let signal_str = id_store
                        .resolve(signal)
                        .expect("Internal error: signal not found in ID store");
                    let length: usize = obj
                        .call_method1(intern!(py, "get_raw_acquire_length"), (signal_str,))?
                        .extract()?;
                    Ok((signal, handle, length))
                })
                .collect()
        })
    }

    fn total_execution_time(&self) -> CompilerBackendResult<f64> {
        Python::attach(|py| {
            Ok(self
                .0
                .bind(py)
                .getattr(intern!(py, "total_execution_time"))?
                .extract()?)
        })
    }

    fn max_execution_time_per_step(&self) -> CompilerBackendResult<f64> {
        Python::attach(|py| {
            Ok(self
                .0
                .bind(py)
                .getattr(intern!(py, "max_execution_time_per_step"))?
                .extract()?)
        })
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

pub struct CompilationArtifactsQccs {
    artifacts: Py<PyAny>,
}

impl CompilerArtifact for CompilationArtifactsQccs {
    fn to_python<'py>(
        &mut self,
        py: Python<'py>,
        _id_store: &NamedIdStore,
        _py_object_store: &PyObjectStore<ExternalParameterUid>,
    ) -> PyResult<Bound<'py, PyAny>> {
        Ok(self.artifacts.bind(py).into())
    }
}

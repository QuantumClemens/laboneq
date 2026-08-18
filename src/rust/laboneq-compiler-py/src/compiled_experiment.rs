// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroU32;

use laboneq_dsl::types::{AcquisitionType, AveragingMode, SweepParameter};
use laboneq_ir::pulse_sheet_schedule::PulseSheetSchedule;

use crate::compiler_backend::CompilerArtifact;
use crate::execution::Execution;
use crate::result_shape::HandleResultShape;

pub(crate) struct CompiledExperiment<A: CompilerArtifact> {
    pub device_setup_fingerprint: String,
    pub artifacts: A,
    pub execution_timing: ExecutionTiming,
    pub metadata: Metadata,
    pub execution: Execution,
    pub real_time_properties: RealTimeProperties,
    pub result_shapes: ResultShapes,
    pub pulse_sheet_schedule: Option<PulseSheetSchedule>,

    pub sweep_parameters: Vec<SweepParameter>,
}

pub(crate) struct ExecutionTiming {
    /// Total execution time of the real-time steps in seconds.
    pub total_execution_time: f64,
    /// Maximum execution time of a single real-time step in seconds.
    pub max_step_execution_time: f64,
}

pub(crate) struct Metadata {
    /// Producer version. Follows Semantic Versioning (SemVer) format.
    pub producer_version: String,
}

pub(crate) struct RealTimeProperties {
    pub acquisition_type: AcquisitionType,
    pub averaging_mode: AveragingMode,
    pub shots: NonZeroU32,
    pub chunk_count: Option<u32>,
}

pub(crate) struct ResultShapes {
    pub handle_result_shapes: Vec<HandleResultShape>,
}

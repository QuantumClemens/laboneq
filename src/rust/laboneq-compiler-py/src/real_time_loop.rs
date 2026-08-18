// Copyright 2025 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroU32;

use laboneq_dsl::ExperimentNode;
use laboneq_dsl::operation::{AveragingLoop, Operation};
use laboneq_dsl::types::{AcquisitionType, AveragingMode};

use crate::error::{Error, Result};

/// Properties of the experiment's real-time averaging loop.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RealTimeLoopProperties {
    pub acquisition_type: AcquisitionType,
    pub averaging_mode: AveragingMode,
    pub shots: NonZeroU32,
}

impl From<&AveragingLoop> for RealTimeLoopProperties {
    fn from(loop_: &AveragingLoop) -> Self {
        RealTimeLoopProperties {
            acquisition_type: loop_.acquisition_type,
            averaging_mode: loop_.averaging_mode,
            shots: loop_.count,
        }
    }
}

/// Extract the properties of the experiment's real-time averaging loop.
///
/// The experiment is expected to contain exactly one averaging loop; the invariant is
/// established by the `resolve_timing_boundary` pass of the experiment processing.
pub(crate) fn real_time_loop_properties(root: &ExperimentNode) -> Result<RealTimeLoopProperties> {
    find_averaging_loop(root)
        .map(RealTimeLoopProperties::from)
        .ok_or_else(|| Error::new("Experiment must have exactly one real time acquisition loop."))
}

fn find_averaging_loop(node: &ExperimentNode) -> Option<&AveragingLoop> {
    if let Operation::AveragingLoop(loop_) = &node.kind {
        return Some(loop_);
    }
    node.children.iter().find_map(find_averaging_loop)
}

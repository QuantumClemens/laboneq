// Copyright 2025 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, sync::Arc};

use laboneq_common::types::SignalKind;
use laboneq_dsl::ExperimentNode;
use laboneq_dsl::hqcs::HqcsDeclarations;
use laboneq_dsl::signal_calibration::SignalCalibration;
use laboneq_dsl::types::{
    DeviceUid, ExternalParameterUid, PulseDef, PulseUid, SignalUid, SweepParameter,
};
use laboneq_py_utils::py_object_store::PyObjectStore;
use laboneq_units::duration::{Frequency, Hertz};

use crate::NamedIdStore;

pub(crate) struct Experiment {
    /// Root node of the experiment tree
    pub root: ExperimentNode,
    // NOTE: The usage of Arc here is to allow sharing the id_store across Python bindings
    // Remove when Python bindings are no longer needed
    pub id_store: Arc<NamedIdStore>,
    pub parameters: Vec<SweepParameter>,
    pub pulses: HashMap<PulseUid, PulseDef>,
    pub py_object_store: Arc<PyObjectStore<ExternalParameterUid>>,
    #[expect(dead_code)] // not yet consumed; wired up by the lowering pass later
    pub hqcs: HqcsDeclarations,
}

/// Device signal definition, representing a signal in the device setup.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceSignal {
    /// Identification parameters
    pub uid: SignalUid,
    pub device_uid: DeviceUid,

    /// Configuration parameters
    pub kind: SignalKind,
    pub calibration: SignalCalibration,
    /// Delay signal in samples
    pub delay_signal: i64,
    pub sampling_rate: Frequency<Hertz>,
}

// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

use laboneq_dsl::types::{DeviceUid, SignalUid};

use crate::signal::Signal;
// Re-export for convenience
pub use crate::device::AwgDevice;

/// Device and signal setup used in the experiment.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceSetup {
    signals: Vec<Signal>,
    awg_devices: Vec<AwgDevice>,
}

impl DeviceSetup {
    pub fn new(signals: Vec<Signal>, awg_devices: Vec<AwgDevice>) -> Result<Self, String> {
        // Validate all signals reference existing devices
        for signal in &signals {
            if !awg_devices.iter().any(|d| d.uid() == signal.device_uid) {
                return Err(format!(
                    "Signal '{}' references unknown device",
                    signal.uid.0
                ));
            }
        }

        Ok(Self {
            signals,
            awg_devices,
        })
    }

    pub fn signals(&self) -> impl Iterator<Item = &Signal> {
        self.signals.iter()
    }

    pub fn signal_by_uid(&self, uid: &SignalUid) -> Option<&Signal> {
        self.signals.iter().find(|signal| &signal.uid == uid)
    }

    pub fn device_by_uid(&self, uid: &DeviceUid) -> Option<&AwgDevice> {
        self.awg_devices.iter().find(|device| &device.uid() == uid)
    }

    pub fn awg_devices(&self) -> impl Iterator<Item = &AwgDevice> {
        self.awg_devices.iter()
    }
}

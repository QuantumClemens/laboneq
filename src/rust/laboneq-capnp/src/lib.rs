// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

//! Generated Cap'n Proto bindings for LabOne Q pulse-level schemas.

#![allow(unused_qualifications)]
#![allow(unreachable_pub)]

pub mod pulse {
    pub mod v1 {
        capnp::generated_code!(pub mod calibration_capnp, "pulse/v1/calibration_capnp.rs");
        capnp::generated_code!(pub mod common_capnp, "pulse/v1/common_capnp.rs");
        capnp::generated_code!(pub mod coprocessor_capnp, "pulse/v1/coprocessor_capnp.rs");
        capnp::generated_code!(pub mod device_setup_capnp, "pulse/v1/device_setup_capnp.rs");
        capnp::generated_code!(pub mod experiment_capnp, "pulse/v1/experiment_capnp.rs");
        capnp::generated_code!(pub mod operation_capnp, "pulse/v1/operation_capnp.rs");

        capnp::generated_code!(pub mod pulse_capnp, "pulse/v1/pulse_capnp.rs");
        capnp::generated_code!(pub mod section_capnp, "pulse/v1/section_capnp.rs");
        capnp::generated_code!(pub mod sweep_capnp, "pulse/v1/sweep_capnp.rs");
        capnp::generated_code!(
            pub mod setup_description_qccs_capnp,
            "pulse/v1/setup_description_qccs_capnp.rs"
        );
        capnp::generated_code!(
            pub mod setup_description_zqcs_capnp,
            "pulse/v1/setup_description_zqcs_capnp.rs"
        );
    }
}

pub mod compiled_experiment {
    pub mod v1 {
        capnp::generated_code!(pub mod common_capnp, "compiled_experiment/v1/common_capnp.rs");
        capnp::generated_code!(
            pub mod compiled_experiment_capnp,
            "compiled_experiment/v1/compiled_experiment_capnp.rs"
        );
        capnp::generated_code!(pub mod execution_capnp, "compiled_experiment/v1/execution_capnp.rs");
        capnp::generated_code!(pub mod result_shape_capnp, "compiled_experiment/v1/result_shape_capnp.rs");
        capnp::generated_code!(
            pub mod rt_loop_properties_capnp,
            "compiled_experiment/v1/rt_loop_properties_capnp.rs"
        );
        capnp::generated_code!(pub mod values_capnp, "compiled_experiment/v1/values_capnp.rs");
        capnp::generated_code!(pub mod pulse_sheet_capnp, "compiled_experiment/v1/pulse_sheet_capnp.rs");
    }
}

// Re-exported at the crate root so other crates that reference
// `compiled_experiment/v1/common.capnp` via `crate_provides("laboneq_capnp", ...)`
// (which assumes crate-root placement) resolve to this module.
pub use compiled_experiment::v1::common_capnp;

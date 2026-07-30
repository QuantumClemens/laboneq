# Copyright 2024 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

import laboneq.compiler.workflow.reporter  # noqa: F401

# reporter import is required to register the CompilationReportGenerator hook
from laboneq._version import get_version
from laboneq.compiler.workflow import compiler_hooks
from laboneq.compiler.workflow.compiler_hooks import (
    CompiledOutputParams,
    get_compiler_hooks,
)
from laboneq.compiler.workflow.neartime_execution import (
    NtCompilerExecutor,
)
from laboneq.core.types.enums.acquisition_type import AcquisitionType
from laboneq.core.types.enums.averaging_mode import AveragingMode
from laboneq.data.scheduled_experiment import (
    HandleResultShape,
    ResultShapeInfo,
    RtLoopProperties,
    ScheduledExperiment,
    SoftwareVersions,
)

from . import compat

if TYPE_CHECKING:
    from laboneq._rust import compiler as compiler_rs
    from laboneq.compiler.common import compiler_settings
    from laboneq.data.scheduled_experiment import (
        CompilerArtifact,
        ScheduledExperiment,
    )
    from laboneq.executor.executor import Statement

_logger = logging.getLogger(__name__)


def compile_capnp(
    capnp_bytes: bytes,
    device_class: int,
    compiler_settings: dict | None = None,
) -> ScheduledExperiment:
    """Compile the given capnp data which represents an experiment and device setup."""
    _logger.info("Starting LabOne Q Compiler run...")
    compiler_module = compiler_hooks.resolve_compiler_module(device_class)
    scheduled_experiment = compiler_module.compile_experiment(
        capnp_bytes,
        packed=compat.use_packed_capnp(),
        compiler_settings=compat.sanitize_compiler_settings(compiler_settings or {}),
    )
    _logger.info("Finished LabOne Q Compiler run.")
    return scheduled_experiment


@dataclass
class CompiledOutput:
    """Result shape that follows the field defined in `CompilationOutputPy` in Rust."""

    artifacts: CompilerArtifact
    schedule: dict[str, Any] | None
    execution: Statement
    rt_loop_properties: RtLoopProperties
    result_shape_info: ResultShapeInfo
    total_execution_time: float
    max_step_execution_time: float
    versions: SoftwareVersions


def compile_whole_or_with_chunks(
    experiment: compiler_rs.ProcessedExperiment,
    execution: Statement,
    chunk_count: int | None,
    device_class: int,
    compiler_settings: compiler_settings.CompilerSettings,
) -> CompiledOutput:
    """Compile the given experiment.

    This function is called from Rust Compiler in `compiler_module.compile_experiment()`
    """

    if chunk_count == 1:
        chunk_count = None
    executor = NtCompilerExecutor(
        experiment, compiler_settings, chunk_count, device_class
    )
    executor.run(execution)

    combined_compiler_output = executor.combined_compiler_output()
    assert combined_compiler_output is not None, (
        "Internal error: missing real-time compiler output"
    )

    executor.finalize()

    combined_output = combined_compiler_output.combined_output
    total_execution_time = combined_output._total_execution_time
    max_step_execution_time = combined_output._max_execution_time_per_step

    shapes = {
        shape.handle: HandleResultShape(
            shape=tuple(shape.shape),
            axis_names=[
                axis_names[0] if len(axis_names) == 1 else axis_names
                for axis_names in shape.axis_names
            ],
            axis_values=[
                axis_values[0] if len(axis_values) == 1 else axis_values
                for axis_values in shape.axis_values
            ],
            chunked_axis_index=shape.chunked_axis_index,
            match_case_mask=shape.match_case_mask or None,
        )
        for shape in experiment.get_result_shapes(combined_output)
    }

    artifacts = get_compiler_hooks(device_class).compiled_output(
        CompiledOutputParams(
            experiment_rs=experiment,
            combined_compiler_output=combined_output,
        )
    )
    result_shape_info = ResultShapeInfo(shapes=shapes)

    rt_loop_properties = experiment.rt_loop_properties()
    return CompiledOutput(
        artifacts=artifacts,
        schedule=combined_compiler_output.schedule,
        execution=execution,
        rt_loop_properties=RtLoopProperties(
            acquisition_type=AcquisitionType[rt_loop_properties.acquisition_type],
            averaging_mode=AveragingMode[rt_loop_properties.averaging_mode],
            shots=rt_loop_properties.count,
            chunk_count=chunk_count,
        ),
        result_shape_info=result_shape_info,
        total_execution_time=total_execution_time,
        max_step_execution_time=max_step_execution_time,
        versions=SoftwareVersions(laboneq=get_version()),
    )

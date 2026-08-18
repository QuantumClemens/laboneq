# Copyright 2022 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

from laboneq.core.validators import dicts_equal

if TYPE_CHECKING:
    from laboneq.data.scheduled_experiment import ScheduledExperiment
    from laboneq.dsl.device.device_setup import DeviceSetup
    from laboneq.dsl.experiment import Experiment


@dataclass(frozen=True)
class ResultProperties:
    """Data structure containing some information about measurement results.

    Attributes:
        shape: The shape of the result array.
        axis_names: This tuple has the same length as `shape`, and each entry describes what
                    the respective axis corresponds to. Usually they are ids of sweep parameters,
                    but can also contain other things - "samples" in case of raw acquisition, and the id of
                    the RT acquisition loop in case of single-shot readout. A dimension can also have more
                    than 1 associated names, if it corresponds to a parallel sweep over multiple parameters.
    """

    shape: tuple[int, ...]
    axis_names: tuple[str | tuple[str, ...], ...]


@dataclass(init=True, repr=True, order=True)
class CompiledExperiment:
    """Data structure to store the output of the compiler.

    Attributes:
        device_setup (DeviceSetup):
            The device setup the experiment was compiled for.
        experiment (Experiment):
            The (uncompiled) experiment.
        experiment_dict (deprecated):
            Deprecated. A representation of the source experiment, using
            primitive Python datatypes only (dicts, lists, etc).
            Use `.experiment` instead.
        scheduled_experiment (internal):
            Internal. The internal representation of the compiled
            experiment. Available for debugging but subject to
            change in any LabOne Q release.

    !!! version-changed "Changed in version 2.54.0"
        The following deprecated methods for saving and loading were removed:
        - `load`
        - `save`

        Use the `load` and `save` functions from the `laboneq.simple` module instead.

    !!! version-changed "Changed in version 2.56.0"
        The `.device_setup` and `.experiment` attributes were no longer deprecated.

    !!! version-changed "Deprecated in version 2.51.0"
        The `.device_setup` and `.experiment` attributes were
        deprecated in version 2.51.0 and will be removed in a
        future release. Manage and track the device setup and experiment
        separately if they are needed.

        Note that the `.device_setup` and `.experiment` were no longer
        deprecated in version 2.56.0.˝

    !!! version-changed "Deprecated in version 2.14.0"
        The `.experiment_dict` attribute was deprecated in
        version 2.14.0. Use `.experiment` instead.

    !!! version-changed "Changed in version 2.14.0"
        The `.scheduled_experiment` attribute was documented to
        be internal and subject to change.
    """

    scheduled_experiment: ScheduledExperiment

    # The source device setup.
    device_setup: DeviceSetup | None = None

    # The source experiment.
    experiment: Experiment | None = None

    # Settings passed to the compiler
    compiler_settings: dict[str, Any] | None = None

    # A representation of the source experiment, using primitive Python datatypes only
    # (dicts, lists, etc.)
    experiment_dict: dict[str, Any] | None = None

    # Post-compile coprocessor payload overrides; keyed by coprocessor label.
    payload_overrides: dict[str, bytes] = field(default_factory=dict)

    @property
    def estimated_runtime(self) -> float:
        """An estimation of the total runtime of the experiment in seconds.

        DISCLAIMER: This estimation does not include any overhead from network, IO,
        or python runtime.
        """
        return self.scheduled_experiment.total_execution_time

    @property
    def result_properties(self) -> dict[str, ResultProperties]:
        """Properties of results for each handle."""
        return {
            handle: ResultProperties(
                shape=properties.shape,
                axis_names=tuple(
                    x if isinstance(x, str) else tuple(x) for x in properties.axis_names
                ),
            )
            for handle, properties in self.scheduled_experiment.result_shape_info.shapes.items()
        }

    def set_coprocessor_payload(self, coprocessor_label: str, payload: bytes) -> None:
        """Override the payload for a coprocessor, post-compile.

        The experiment-body `Coprocessor.set_payload(...)` declares the
        initial payload; this method overrides it post-compilation.

        Args:
            coprocessor_label: The coprocessor label (as declared in the experiment body).
            payload: The new payload. Opaque to LabOne Q.
        """
        self.payload_overrides[coprocessor_label] = payload

    def __eq__(self, other):
        if other is self:
            return True
        if type(other) is not CompiledExperiment:
            return NotImplemented
        return (other.device_setup, other.experiment, other.scheduled_experiment) == (
            self.device_setup,
            self.experiment,
            self.scheduled_experiment,
        ) and dicts_equal(other.experiment_dict, self.experiment_dict)

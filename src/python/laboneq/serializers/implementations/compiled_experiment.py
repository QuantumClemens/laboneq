# Copyright 2022 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

import warnings
from typing import TYPE_CHECKING

import pybase64

from laboneq._version import get_version
from laboneq.core.types.compiled_experiment import CompiledExperiment
from laboneq.data.scheduled_experiment import ScheduledExperiment
from laboneq.serializers._legacy.classic import LabOneQClassicSerializer
from laboneq.serializers.base import VersionedClassSerializer
from laboneq.serializers.core import from_dict, to_dict
from laboneq.serializers.implementations._models._compiled_experiment import (
    ScheduledExperimentModel,
    make_converter,
)
from laboneq.serializers.serializer_registry import serializer

if TYPE_CHECKING:
    from laboneq.serializers.types import (
        DeserializationOptions,
        JsonSerializableType,
        SerializationOptions,
    )

_converter = make_converter()


def _reshape_v1_scheduled_experiment_dict(data: dict) -> dict:
    """Reshape a pre-refactor `ScheduledExperimentModel` dict.

    Before the Recipe/ArtifactsCodegen split, `total_execution_time`,
    `max_step_execution_time`, and `versions` lived nested under `recipe`, and the
    five QCCS-specific lists (`initializations`, `realtime_execution_init`,
    `oscillator_params`, `integrator_allocations`, `acquire_lengths`) lived there too
    instead of on `artifacts`. Lift/move them into the new shape in place.
    """
    data = dict(data)
    old_recipe = data.pop("recipe")
    data["total_execution_time"] = old_recipe["total_execution_time"]
    data["max_step_execution_time"] = old_recipe["max_step_execution_time"]
    data["versions"] = old_recipe["versions"]

    artifacts = dict(data["artifacts"])
    for key in (
        "initializations",
        "realtime_execution_init",
        "oscillator_params",
        "integrator_allocations",
        "acquire_lengths",
    ):
        artifacts[key] = old_recipe[key]
    data["artifacts"] = artifacts

    return data


def _unstructure_payload_overrides(overrides: dict[str, bytes]) -> dict[str, str]:
    """Base64-encode payload_overrides to a JSON-safe dict of ascii strings."""
    return {
        label: pybase64.b64encode(payload).decode("ascii")
        for label, payload in overrides.items()
    }


def _structure_payload_overrides(d: dict[str, str]) -> dict[str, bytes]:
    """Base64-decode payload_overrides from a JSON-safe dict."""
    return {
        label: pybase64.b64decode(value.encode("ascii")) for label, value in d.items()
    }


@serializer(types=CompiledExperiment, public=True)
class CompiledExperimentSerializer(VersionedClassSerializer[CompiledExperiment]):
    SERIALIZER_ID = "laboneq.serializers.implementations.CompiledExperimentSerializer"
    VERSION = 3

    @classmethod
    def to_dict(
        cls, obj: CompiledExperiment, options: SerializationOptions | None = None
    ) -> JsonSerializableType:
        device_setup = to_dict(obj.device_setup, options)
        experiment = to_dict(obj.experiment, options)
        experiment_dict = to_dict(obj.experiment_dict, options)
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", FutureWarning)
            scheduled_experiment = _converter.unstructure(
                obj.scheduled_experiment, ScheduledExperimentModel
            )
        return {
            "__serializer__": cls.serializer_id(),
            "__version__": cls.version(),
            "__laboneq_version__": get_version(),
            "__data__": {
                "device_setup": device_setup,
                "experiment": experiment,
                "experiment_dict": experiment_dict,
                "scheduled_experiment": scheduled_experiment,
                "payload_overrides": _unstructure_payload_overrides(
                    obj.payload_overrides
                ),
            },
        }

    @classmethod
    def _check_laboneq_version(
        cls,
        serialized_laboneq_version: str | None,
        options: DeserializationOptions | None = None,
    ) -> None:
        check_version = options is None or not options.force
        _not_found = "Could not find LabOne Q version in serialized data."
        _mismatch = (
            f"LabOne Q version mismatch. Check out the Labone Q with correct version "
            f"{serialized_laboneq_version} to load the serialized data. Otherwise, set "
            f"the `force` option to True to skip the version check."
        )
        if serialized_laboneq_version is None:
            if check_version:
                raise ValueError(_not_found)
            else:
                warnings.warn(_not_found, UserWarning, stacklevel=2)
        elif serialized_laboneq_version != get_version():
            if check_version:
                raise ValueError(_mismatch)
            else:
                warnings.warn(_mismatch, UserWarning, stacklevel=2)

    @classmethod
    def from_dict_v3(
        cls,
        serialized_data: JsonSerializableType,
        options: DeserializationOptions | None = None,
    ) -> CompiledExperiment:
        cls._check_laboneq_version(serialized_data.get("__laboneq_version__"), options)

        device_setup = from_dict(serialized_data["__data__"]["device_setup"], options)
        experiment = from_dict(serialized_data["__data__"]["experiment"], options)
        experiment_dict = from_dict(
            serialized_data["__data__"]["experiment_dict"], options
        )
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", FutureWarning)
            scheduled_experiment = _converter.structure(
                serialized_data["__data__"]["scheduled_experiment"],
                ScheduledExperimentModel,
            )

        compiled = CompiledExperiment(
            device_setup=device_setup,
            experiment=experiment,
            experiment_dict=experiment_dict,
            scheduled_experiment=scheduled_experiment,
        )
        raw_overrides = serialized_data["__data__"].get("payload_overrides") or {}
        compiled.payload_overrides = _structure_payload_overrides(raw_overrides)
        return compiled

    @classmethod
    def from_dict_v2(
        cls,
        serialized_data: JsonSerializableType,
        options: DeserializationOptions | None = None,
    ) -> CompiledExperiment:
        cls._check_laboneq_version(serialized_data.get("__laboneq_version__"), options)

        device_setup = from_dict(serialized_data["__data__"]["device_setup"], options)
        experiment = from_dict(serialized_data["__data__"]["experiment"], options)
        experiment_dict = from_dict(
            serialized_data["__data__"]["experiment_dict"], options
        )
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", FutureWarning)
            scheduled_experiment = _converter.structure(
                _reshape_v1_scheduled_experiment_dict(
                    serialized_data["__data__"]["scheduled_experiment"]
                ),
                ScheduledExperimentModel,
            )
        return CompiledExperiment(
            device_setup=device_setup,
            experiment=experiment,
            experiment_dict=experiment_dict,
            scheduled_experiment=scheduled_experiment,
        )

    @classmethod
    def from_dict_v1(
        cls, serialized_data, options: DeserializationOptions | None = None
    ) -> CompiledExperiment:
        cls._check_laboneq_version(serialized_data.get("__laboneq_version__"), options)
        return LabOneQClassicSerializer.from_dict_v1(serialized_data, options)


@serializer(types=ScheduledExperiment, public=True)
class ScheduledExperimentSerializer(VersionedClassSerializer[ScheduledExperiment]):
    SERIALIZER_ID = "laboneq.serializers.implementations.ScheduledExperimentSerializer"
    VERSION = 2

    @classmethod
    def to_dict(
        cls, obj: ScheduledExperiment, options: SerializationOptions | None = None
    ) -> JsonSerializableType:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", FutureWarning)
            scheduled_experiment = _converter.unstructure(obj, ScheduledExperimentModel)
        return {
            "__serializer__": cls.serializer_id(),
            "__version__": cls.version(),
            "__laboneq_version__": get_version(),
            "__data__": scheduled_experiment,
        }

    @classmethod
    def _check_laboneq_version(
        cls,
        serialized_laboneq_version: str | None,
        options: DeserializationOptions | None = None,
    ) -> None:
        check_version = options is None or not options.force
        _not_found = "Could not find LabOne Q version in serialized data."
        _mismatch = (
            f"LabOne Q version mismatch. Check out the Labone Q with correct version "
            f"{serialized_laboneq_version} to load the serialized data. Otherwise, set "
            f"the `force` option to True to skip the version check."
        )
        if serialized_laboneq_version is None:
            if check_version:
                raise ValueError(_not_found)
            else:
                warnings.warn(_not_found, UserWarning, stacklevel=2)
        elif serialized_laboneq_version != get_version():
            if check_version:
                raise ValueError(_mismatch)
            else:
                warnings.warn(_mismatch, UserWarning, stacklevel=2)

    @classmethod
    def from_dict_v2(
        cls,
        serialized_data: JsonSerializableType,
        options: DeserializationOptions | None = None,
    ) -> ScheduledExperiment:
        assert isinstance(serialized_data, dict)
        cls._check_laboneq_version(serialized_data.get("__laboneq_version__"), options)

        with warnings.catch_warnings():
            warnings.simplefilter("ignore", FutureWarning)
            scheduled_experiment = _converter.structure(
                serialized_data["__data__"],
                ScheduledExperimentModel,
            )
        return scheduled_experiment

    @classmethod
    def from_dict_v1(
        cls,
        serialized_data: JsonSerializableType,
        options: DeserializationOptions | None = None,
    ) -> ScheduledExperiment:
        assert isinstance(serialized_data, dict)
        cls._check_laboneq_version(serialized_data.get("__laboneq_version__"), options)

        with warnings.catch_warnings():
            warnings.simplefilter("ignore", FutureWarning)
            scheduled_experiment = _converter.structure(
                _reshape_v1_scheduled_experiment_dict(serialized_data["__data__"]),
                ScheduledExperimentModel,
            )
        return scheduled_experiment

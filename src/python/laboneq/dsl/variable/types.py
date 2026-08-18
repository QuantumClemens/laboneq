# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

"""The catalog is a closed set of class objects acting as type tags for the
fields of a Struct schema"""

from __future__ import annotations

from enum import Flag, auto
from typing import ClassVar

import attrs
import numpy as np


class Direction(Flag):
    """Whether a type may appear on an outbound or inbound stream (or both)."""

    OUT = auto()
    IN = auto()


class _VarType:
    """Base for the closed type catalog. Subclasses act as type tags."""

    direction: ClassVar[Direction]


class Int8(_VarType):
    direction = Direction.OUT | Direction.IN


class Int16(_VarType):
    direction = Direction.OUT | Direction.IN


class Int32(_VarType):
    direction = Direction.OUT | Direction.IN


class Int64(_VarType):
    direction = Direction.OUT | Direction.IN


class Uint8(_VarType):
    direction = Direction.OUT | Direction.IN


class Uint16(_VarType):
    direction = Direction.OUT | Direction.IN


class Uint32(_VarType):
    direction = Direction.OUT | Direction.IN


class Uint64(_VarType):
    direction = Direction.OUT | Direction.IN


class DiscriminationDataPacked(_VarType):
    direction = Direction.OUT


class IqDataPacked(_VarType):
    direction = Direction.OUT


class ScopeShot(_VarType):
    direction = Direction.OUT


class WaveformUpdate(_VarType):
    direction = Direction.IN


# --- Rich scalar types with value factories ---


@attrs.frozen
class PhaseValue:
    """A phase value in radians. Produced by Phase.from_radians()."""

    radians: float


class Phase(_VarType):
    direction = Direction.IN

    @staticmethod
    def from_radians(x: float) -> PhaseValue:
        return PhaseValue(radians=float(x))


@attrs.frozen
class FrequencyValue:
    """A frequency value in Hz. Produced by Frequency.from_hz()."""

    hz: float


class Frequency(_VarType):
    direction = Direction.IN

    @staticmethod
    def from_hz(x: float) -> FrequencyValue:
        return FrequencyValue(hz=float(x))


@attrs.frozen
class AmplitudeValue:
    """A dimensionless amplitude value. Produced by Amplitude.from_value()."""

    value: float


class Amplitude(_VarType):
    direction = Direction.IN

    @staticmethod
    def from_value(x: float) -> AmplitudeValue:
        return AmplitudeValue(value=float(x))


DTYPE_BY_TYPE: dict[type[_VarType], np.dtype] = {
    Int8: np.dtype("int8"),
    Int16: np.dtype("int16"),
    Int32: np.dtype("int32"),
    Int64: np.dtype("int64"),
    Uint8: np.dtype("uint8"),
    Uint16: np.dtype("uint16"),
    Uint32: np.dtype("uint32"),
    Uint64: np.dtype("uint64"),
    Phase: np.dtype("float64"),
    Frequency: np.dtype("float64"),
    Amplitude: np.dtype("float64"),
}

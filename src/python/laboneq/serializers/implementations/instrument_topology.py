# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

from typing import TYPE_CHECKING

from laboneq.data.instrument_topology import InstrumentTopology
from laboneq.serializers.base import VersionedClassSerializer
from laboneq.serializers.serializer_registry import serializer

if TYPE_CHECKING:
    from laboneq.serializers.types import (
        DeserializationOptions,
        JsonSerializableType,
        SerializationOptions,
    )


@serializer(types=InstrumentTopology, public=True)
class InstrumentTopologySerializer(VersionedClassSerializer[InstrumentTopology]):
    SERIALIZER_ID = "laboneq.serializers.implementations.InstrumentTopologySerializer"
    VERSION = 1

    @classmethod
    def to_dict(
        cls,
        obj: InstrumentTopology,
        options: SerializationOptions | None = None,
    ) -> JsonSerializableType:
        return {
            "__serializer__": cls.serializer_id(),
            "__version__": cls.version(),
            "__data__": obj.to_dict(),
        }

    @classmethod
    def from_dict_v1(
        cls,
        serialized_data: JsonSerializableType,
        options: DeserializationOptions | None = None,
    ) -> InstrumentTopology:
        return InstrumentTopology.from_dict(serialized_data["__data__"])

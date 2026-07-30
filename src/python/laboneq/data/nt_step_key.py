# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

from dataclasses import dataclass


@dataclass(frozen=True)
class NtStepKey:
    indices: tuple[int, ...]

    def __post_init__(self):
        # Required for JSON deserialization, as tuples are serialized as lists.
        if isinstance(self.indices, list):
            object.__setattr__(self, "indices", tuple(self.indices))

# Copyright 2023 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from laboneq.compiler.common.iface_compiler_output import (
        CombinedOutput,
    )


@dataclass
class CombinedRTCompilerOutputContainer:
    """Container for the compiler artifacts, after linking."""

    device_class: int
    combined_output: CombinedOutput
    schedule: dict[str, Any] | None = None

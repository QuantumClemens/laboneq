# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

"""Inventory entry for a coprocessor available on a DeviceSetup."""

from __future__ import annotations

import attrs


@attrs.frozen
class CoprocessorInventoryEntry:
    """A single coprocessor advertised by the system inventory."""

    key: str

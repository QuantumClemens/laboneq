# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0


from __future__ import annotations

from laboneq.core.exceptions import LabOneQException
from laboneq.dsl.experiment.experiment_context import current_experiment_context


class Coprocessor:
    """Coprocessor handle"""

    __slots__ = ("label", "_payload")

    def __init__(self, label: str) -> None:
        ctx = current_experiment_context()
        if ctx is None:
            raise LabOneQException(
                f"Coprocessor({label!r}) called outside an @experiment body."
            )
        self.label = label
        self._payload: bytes | None = None
        ctx.register_coprocessor(self)

    @property
    def payload(self) -> bytes | None:
        return self._payload

    def set_payload(self, payload: bytes) -> None:
        """Attach a payload to this coprocessor handle.

        The payload is opaque to LabOne Q. May be called more than once; the
        last call wins.
        """
        self._payload = payload

    def __eq__(self, other: object) -> bool:
        if self is other:
            return True
        if type(self) is not type(other):
            return NotImplemented
        return (self.label, self._payload) == (other.label, other._payload)

    def __repr__(self) -> str:
        return f"Coprocessor({self.label!r})"

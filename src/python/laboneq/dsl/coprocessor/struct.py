# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0


from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from laboneq.dsl.variable.types import _VarType


class Struct:
    """A flat, ordered record of named fields with closed-catalog types."""

    __slots__ = ("fields",)

    def __init__(self, **fields: type[_VarType]) -> None:
        self.fields: dict[str, type[_VarType]] = dict(fields)

    def __eq__(self, other: object) -> bool:
        if self is other:
            return True
        if type(self) is not type(other):
            return NotImplemented
        return self.fields == other.fields

    def __repr__(self) -> str:
        body = ", ".join(f"{k}={v.__name__}" for k, v in self.fields.items())
        return f"Struct({body})"

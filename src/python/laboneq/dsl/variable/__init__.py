# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

from typing import TYPE_CHECKING, Any

import attrs

if TYPE_CHECKING:
    from laboneq.dsl.variable.types import _VarType

from laboneq.dsl.coprocessor.predicate import _Predicate


@attrs.define(slots=False, eq=False)
class Variable:
    """A typed runtime-valued DSL value.

    The target of an inbound stream-field binding; usable as a play-parameter, a match
    discriminator, or a do_until condition term.
    """

    type: type[_VarType]
    name: str | None = None
    initial: Any | None = None
    log_handle: str | None = attrs.field(default=None, init=False)

    def __eq__(self, other: Any) -> _Predicate:  # type: ignore[override]
        """Build a symbolic expression.

        Comparisons against a Variable are deferred to compile time. The
        returned `_Predicate` is an inert placeholder the compiler
        currently rejects; it exists so that `do_until(condition=var == x)`
        has something to carry.
        """
        return _Predicate(self, "==", other)

    def __ne__(self, other: Any) -> _Predicate:  # type: ignore[override]
        return _Predicate(self, "!=", other)

    def __lt__(self, other: Any) -> _Predicate:
        return _Predicate(self, "<", other)

    def __le__(self, other: Any) -> _Predicate:
        return _Predicate(self, "<=", other)

    def __gt__(self, other: Any) -> _Predicate:
        return _Predicate(self, ">", other)

    def __ge__(self, other: Any) -> _Predicate:
        return _Predicate(self, ">=", other)

    __hash__ = object.__hash__

    def log(self, *, handle: str | None) -> None:
        """Opt this Variable into capture under the given Results handle.

        If the handle is omitted, the name of the variable (if available) is used as
        a fallback.
        """
        if handle is None:
            handle = self.name
        if handle is None:
            raise ValueError("Either handle or variable name must be provided")
        self.log_handle = handle

# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

"""HQCS predicate stand-in.

There is no real predicate machinery yet. Comparing a `Variable`
(e.g. `var != 0`) yields a `_Predicate`: an inert record of the
comparison. It carries no runtime semantics and the compiler will reject
any experiment that actually contains one until proper predicate support
has been implemented.
"""

from __future__ import annotations

from typing import Any

import attrs


@attrs.define(frozen=True, eq=False)
class _Predicate:
    """Opaque comparison record produced by Variable comparison operators.

    Mock placeholder: stores the operands and operator only. The compiler
    treats any surviving `_Predicate` as a hard error.
    """

    lhs: Any
    op: str
    rhs: Any

    def __bool__(self):
        raise NotImplementedError(
            "A predicate can be evaluated as a boolean only at runtime"
        )


@attrs.define(frozen=True, eq=False)
class _IsLive:
    """Arrival predicate produced by `is_live(x)`.

    `target` is a stream-bound Variable or Pulse. Unlike `_Predicate`,
    this is a supported condition: `do_until(is_live(x), ...)` exits the
    loop when an inbound arrival flips `target` from stale to live.
    """

    target: Any

# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

"""HQCS DSL builtins: register_stream, send, mark_stale, is_live, render_layout."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Union

from laboneq.core.exceptions import LabOneQException
from laboneq.dsl.coprocessor.operations import _MarkStale, _Send
from laboneq.dsl.coprocessor.predicate import _IsLive
from laboneq.dsl.coprocessor.stream import InboundStream, OutboundStream
from laboneq.dsl.experiment.experiment_context import current_experiment_context
from laboneq.dsl.experiment.section_context import current_section_context

if TYPE_CHECKING:
    from laboneq.dsl.coprocessor.coprocessor import Coprocessor
    from laboneq.dsl.coprocessor.stream import _Stream
    from laboneq.dsl.coprocessor.struct import Struct


def register_stream(
    *,
    src: Coprocessor | None = None,
    dst: Coprocessor | None = None,
    schema: Struct,
    link: str | None = None,
    uid: str | None = None,
) -> Union[OutboundStream, InboundStream]:
    """Declare a typed stream between two endpoints.

    Must be called inside an `@experiment` body. Dispatches on (src, dst):

    - `(None, Coprocessor)`         -> `OutboundStream`
    - `(Coprocessor, None)`         -> `InboundStream`
    - otherwise                     -> `TypeError` (no meaningful object)

    Args:
        src: The source coprocessor of this stream. Leave None when the source is
            the control system).
        dst: The destination coprocessor of this stream. Leave None when the
            destination is the control system).
        schema: The type definition of the messages handled by this stream.
        link: Optional pinning of the physical link.
        uid: Optional unique stream name. If provided, must not already be registered
              in this experiment. If None, auto-generated as `stream_N` (N chosen to avoid
              collisions with existing names).
    """
    ctx = current_experiment_context()
    if ctx is None:
        raise LabOneQException(
            "register_stream(...) called outside an @experiment body."
        )

    if src is None and dst is None:
        raise TypeError("register_stream(...): src and dst cannot both be None.")

    existing_uids = {s.uid for s in ctx.experiment.streams}
    if uid is not None:
        if uid in existing_uids:
            raise LabOneQException(
                f"register_stream(...): stream uid {uid!r} is already registered."
            )
    else:
        index = len(ctx.experiment.streams)
        uid = f"stream_{index}"
        while uid in existing_uids:
            index += 1
            uid = f"stream_{index}"

    stream: _Stream
    if src is None:
        stream = OutboundStream(schema=schema, dst=dst, link=link, uid=uid)  # type: ignore[arg-type]
    elif dst is None:
        stream = InboundStream(schema=schema, src=src, link=link, uid=uid)
    else:
        raise TypeError("Either src or dst must be specified")

    ctx.register_stream(stream)
    return stream  # type: ignore[return-value]


def _require_section(call_name: str):
    """Return the active section context, or raise."""
    ctx = current_section_context()
    if ctx is None:
        raise LabOneQException(f"{call_name}(...) called outside a section context.")
    return ctx


def send(stream, **kwargs) -> None:
    """Commit one logical packet on an outbound stream."""
    ctx = _require_section("send")
    ctx.section.add(_Send(stream=stream, literal_kwargs=dict(kwargs)))


def mark_stale(target) -> None:
    """Open an acceptance window on a stream-bound Variable or Pulse."""
    ctx = _require_section("mark_stale")
    ctx.section.add(_MarkStale(target=target))


def is_live(target) -> _IsLive:
    """Arrival predicate on a stream-bound Variable or Pulse.

    `do_until(is_live(x), max_count=...)` exits the loop when an inbound
    arrival flips `x` from stale to live during the loop body. Does not
    implicitly mark `x` stale; open the acceptance window explicitly with
    `mark_stale(x)`.
    """
    return _IsLive(target=target)


def render_layout(compiled: Any, *, target: str, coprocessor: str) -> str:
    """Render the kernel-side layout artifact for a coprocessor.

    Currently, returns a placeholder string. The compiler does not yet emit a
    layout artifact.

    NOTE: this function will most likely be relocated to another module.
    """
    return "the layout is not available yet"

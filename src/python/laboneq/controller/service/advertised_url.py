# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

"""Derivation of the base URL under which clients reach this service.

The address a client must dispatch experiments to is not necessarily the one
this process is bound to: behind a reverse or authenticating proxy the client
reaches the service under a different scheme, host, port and path prefix.
Only the proxy knows that outside address, and it communicates it through the
forwarded headers parsed here.  An operator can bypass the guesswork entirely
with the service's ``--public-url`` option.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from fastapi import Request

logger = logging.getLogger(__name__)

#: Ports that are implied by the scheme and therefore omitted from the URL.
_DEFAULT_PORTS = {"http": "80", "https": "443"}


def advertised_base_url(request: Request, configured: str | None = None) -> str:
    """Return the base URL under which the client reached this service.

    The result carries no trailing slash, so a path can be appended directly.

    Args:
        request: The request to derive the address from.
        configured: Explicitly configured base URL (the service's
            ``--public-url``).  Used verbatim when set, since the operator
            knows better than any header.

    Returns:
        Base URL, e.g. ``https://lab.example.com/laboneq``.
    """
    if configured:
        return configured.rstrip("/")

    forwarded = _forwarded_params(request)
    scheme = _scheme(request, forwarded)
    return f"{scheme}://{_authority(request, forwarded, scheme)}{_prefix(request)}"


def _first(request: Request, header: str) -> str | None:
    """Return the leftmost value of a possibly comma-separated *header*.

    Chained proxies append to these headers, so the leftmost entry is the one
    closest to the client, i.e. the outside address.  Starlette joins repeated
    headers with ``", "``, which this handles the same way.
    """
    raw = request.headers.get(header)
    if not raw:
        return None
    return raw.split(",")[0].strip() or None


def _forwarded_params(request: Request) -> dict[str, str]:
    """Parse the first element of the RFC 7239 ``Forwarded`` header.

    Returns an empty mapping if the header is absent or unparseable; a
    malformed header must degrade to the other sources rather than fail the
    request.
    """
    raw = request.headers.get("forwarded")
    if not raw:
        return {}
    params: dict[str, str] = {}
    for pair in raw.split(",")[0].split(";"):
        key, sep, value = pair.partition("=")
        if not sep:
            continue
        # Values may be quoted, e.g. host="lab.example.com:8443".
        params.setdefault(key.strip().lower(), value.strip().strip('"'))
    return params


def _scheme(request: Request, forwarded: dict[str, str]) -> str:
    candidate = forwarded.get("proto") or _first(request, "x-forwarded-proto")
    if candidate is None:
        return request.url.scheme
    scheme = candidate.lower()
    if scheme not in _DEFAULT_PORTS:
        logger.warning(
            "Ignoring unsupported forwarded scheme %r; using %r instead.",
            candidate,
            request.url.scheme,
        )
        return request.url.scheme
    return scheme


def _authority(request: Request, forwarded: dict[str, str], scheme: str) -> str:
    """Return the ``host[:port]`` part of the advertised URL."""
    authority = (
        forwarded.get("host")
        or _first(request, "x-forwarded-host")
        or request.headers.get("host")
        or request.url.netloc
    )
    host, port = _split_host_port(authority)
    if port is None:
        # A host without a port means the port is the one implied by the
        # scheme: clients spell out any other port in `Host`, and a proxy that
        # rewrites the host reports the outside port separately.  Falling back
        # to the bound port would be wrong, as it is not reachable from
        # outside.
        port = _first(request, "x-forwarded-port")
    if port is None or port == _DEFAULT_PORTS[scheme]:
        return host
    return f"{host}:{port}"


def _split_host_port(authority: str) -> tuple[str, str | None]:
    """Split ``host[:port]``, tolerating IPv6 literals.

    An unbracketed IPv6 literal is bracketed, since a URL requires it.
    """
    if authority.startswith("["):
        host, _, rest = authority.partition("]")
        port = rest.lstrip(":")
        return f"{host}]", port if port.isdigit() else None
    host, sep, port = authority.rpartition(":")
    if sep and port.isdigit() and ":" not in host:
        return host, port
    if authority.count(":") > 1:
        return f"[{authority}]", None
    return authority, None


def _prefix(request: Request) -> str:
    """Return the path prefix the service is mounted under, if any.

    RFC 7239 has no equivalent of ``X-Forwarded-Prefix``, so this is always
    taken from the ``X-`` header even when ``Forwarded`` is present.
    """
    prefix = (_first(request, "x-forwarded-prefix") or "").rstrip("/")
    if prefix and not prefix.startswith("/"):
        prefix = f"/{prefix}"
    return prefix

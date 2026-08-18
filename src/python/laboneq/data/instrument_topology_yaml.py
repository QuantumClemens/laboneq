# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

from typing import TYPE_CHECKING

import yaml

from laboneq.data.instrument_topology import (
    ConnectionEntry,
    InstrumentEntry,
    InstrumentTopology,
    ServerEntry,
)

if TYPE_CHECKING:
    from pathlib import Path

_SIGNAL_TYPE_KEYS = ("rf_signal", "iq_signal", "acquire_signal")


def _parse_servers(raw: list[dict]) -> list[ServerEntry]:
    return [
        ServerEntry(
            uid=s["uid"],
            host=s.get("host"),
            port=s.get("port"),
            url=s.get("url"),
        )
        for s in raw
    ]


def _parse_instruments(raw: dict[str, list[dict]]) -> list[InstrumentEntry]:
    entries = []
    for device_type, devices in raw.items():
        for d in devices:
            options_raw = d.get("options")
            options = options_raw.split("/") if isinstance(options_raw, str) else []
            entries.append(
                InstrumentEntry(
                    uid=d["uid"],
                    device_type=device_type,
                    address=d["address"],
                    options=options,
                    server_uid=d.get("server_uid"),
                    interface=d.get("interface"),
                    reference_clock_source=d.get("reference_clock_source"),
                )
            )
    return entries


def _parse_connections(raw: dict[str, list[dict]]) -> list[ConnectionEntry]:
    entries = []
    for instrument_uid, conns in raw.items():
        for c in conns:
            signal_type = next(
                (k for k in _SIGNAL_TYPE_KEYS if k in c),
                None,
            )
            if signal_type is None:
                raise ValueError(
                    f"Connection for instrument {instrument_uid!r} has none of "
                    f"{_SIGNAL_TYPE_KEYS}; got keys: {list(c.keys())}"
                )
            signal_name = c[signal_type]
            if "port" in c and "ports" in c:
                raise ValueError(
                    f"Connection for instrument {instrument_uid!r} signal {signal_name!r} "
                    f"has both 'port' and 'ports' defined; use only one."
                )
            if "port" in c:
                port = c["port"]
                if not isinstance(port, str):
                    raise ValueError(
                        f"Connection for instrument {instrument_uid!r} signal {signal_name!r} "
                        f"has a 'port' that is not a single string: {port!r}; use 'ports' "
                        f"for more than one port."
                    )
                ports = [port]
            else:
                ports = c.get("ports", [])
                if isinstance(ports, str):
                    ports = [ports]
            if not ports:
                raise ValueError(
                    f"Connection for instrument {instrument_uid!r} signal {signal_name!r} "
                    f"has no ports defined."
                )
            primary_port = ports[0]
            secondary_port = ports[1] if len(ports) > 1 else None
            entries.append(
                ConnectionEntry(
                    instrument_uid=instrument_uid,
                    signal_name=signal_name,
                    primary_port=primary_port,
                    secondary_port=secondary_port,
                    signal_type=signal_type,
                )
            )
    return entries


def instrument_topology_from_yaml_dict(raw: dict) -> InstrumentTopology:
    """Construct an InstrumentTopology from a dict in the YAML setup-file shape
    (as produced by e.g. ``yaml.safe_load`` on a topology file).

    The returned topology always has setup_description=None; set it after
    construction if a ZQCS description is available.

    Raises:
        ValueError: If the file sets ``controller_service_url``. That field
            records the address of the controller service that *served* a
            topology, so a file cannot supply it; a service pins the address it
            advertises with its ``--public-url`` option instead.
    """
    if "controller_service_url" in raw:
        raise ValueError(
            "'controller_service_url' cannot be set in a topology file: it records "
            "the address of the controller service that served the topology. Use the "
            "service's --public-url option to pin the address it advertises."
        )
    return InstrumentTopology(
        servers=_parse_servers(raw.get("servers") or []),
        instruments=_parse_instruments(raw.get("instruments") or {}),
        connections=_parse_connections(raw.get("connections") or {}),
    )


def instrument_topology_from_yaml_text(yaml_text: str) -> InstrumentTopology:
    """Parse a YAML setup-file string into an InstrumentTopology."""
    raw = yaml.safe_load(yaml_text) or {}
    return instrument_topology_from_yaml_dict(raw)


def load_instrument_topology(path: str | Path) -> InstrumentTopology:
    """Parse a YAML setup file into an InstrumentTopology.

    The returned topology always has setup_description=None; set it after
    construction if a ZQCS description is available.
    """
    with open(path) as f:
        return instrument_topology_from_yaml_text(f.read())

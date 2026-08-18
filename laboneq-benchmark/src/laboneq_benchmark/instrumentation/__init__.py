# Copyright 2025 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

"""Backwards compatibility instrumentors for LabOne Q."""

from packaging.version import Version


def _laboneq_version() -> str:
    """Get the LabOne Q version."""
    from laboneq import __version__ as laboneq_version

    return Version(laboneq_version)


# Backwards compatibility for old versions without scheduler/codegenerator instrumentation.
if _laboneq_version() <= Version("26.4.0"):
    from .instrumentor import LabOneQInstrumentor
else:
    from laboneq.instrumentation import LabOneQInstrumentor

__all__ = ["LabOneQInstrumentor"]

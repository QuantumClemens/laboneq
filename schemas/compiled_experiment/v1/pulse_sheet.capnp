# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

@0xba289912c6ed43bd;

struct PulseSheet {
  # Describes the pulse timing after scheduling.

  json @0 :Data;
  # Scheduled event timeline as UTF-8 JSON. For debugging and visualization only.
}

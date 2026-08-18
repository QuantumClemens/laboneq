# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

@0xfe7a6bb19ed59a15;

struct CodegenArtifactsQccs @0xdb7669badae93a7a {
    # Interim envelope for QCCS-backend codegen artifacts, pending a native capnp schema.

    json @0 :Data;
    # `ArtifactsCodegen` (laboneq.data.scheduled_experiment) serialized via
    # `laboneq.serializers.core.to_json()` (orjson, UTF-8). Opaque to this schema;
    # the reader must deserialize using the matching laboneq version.
}

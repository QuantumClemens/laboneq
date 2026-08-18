# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

@0xe97a4f365316b8d4;

struct RtLoopProperties {
  acquisitionType @0 :AcquisitionType;

  averagingMode @1 :AveragingMode;

  shots @2 :UInt32;

  chunkCount :union {
    none  @3 :Void;
    value @4 :UInt32;
  }
}

enum AcquisitionType {
  integration     @0;
  spectroscopyIq  @1;
  spectroscopyPsd @2;
  discrimination  @3;
  raw             @4;
}

enum AveragingMode {
  sequential  @0;
  cyclic      @1;
  singleShot  @2;
}

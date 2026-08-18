# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

@0xac5db2d4edba11e4;

struct NumericArray {
  # A typed 1D numeric array, stored as raw little-endian bytes.

  data @0 :Data;
  # Raw bytes, little-endian.

  length @1 :UInt64;
  # Number of elements. For complex data, this is the number of complex elements (not
  # the number of float values).

  dtype @2 :NumericDType;
}

enum NumericDType {
  # Numeric format of the array data.
  #
  # All formats use little-endian byte order. Complex formats interleave real and
  # imaginary components per element.

  unspecified @0;

  float64 @1;
  # 64-bit IEEE 754 floating point (real).

  complex128 @2;
  # Two 64-bit floats (real, imag) per element. 16 bytes per element.

  int64 @3;
  # 64-bit signed integer.
}

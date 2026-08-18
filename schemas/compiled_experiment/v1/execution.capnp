# Copyright 2026 Zurich Instruments AG
# SPDX-License-Identifier: Apache-2.0

@0x857d98856a3d0527;

using Common = import "common.capnp";

struct Execution {
  # Near-time execution program for the controller.
  #
  # A flat list of instructions to be executed. `execRealTime` statements mark the
  # boundaries where the real-time device program runs.

  instructions @0 :List(Instruction);
}

struct Instruction {
  union {
    setParameter @0 :SetParameter;
    # Assign the parameter's value for the current innermost loop index.

    loopStart @1 :LoopStart;
    # Push a loop-stack entry; contributes one axis to the near-time step.

    loopEnd @2 :Void;
    # Increment the innermost index; jump back to the matching loopStart
    # until its count is reached, then pop.

    execRealTime @3 :ExecRealTime;
    # Run the real-time device program for the current near-time step.

    execCallback @4 :ExecCallback;
    # Invoke a user-defined near-time callback.

    setNode @5 :SetNode;
    # Set a device node to a value at near-time.
  }
}

struct SetParameter {
  # Assigns parameter values for the current near-time step.

  parameterRef @0 :Common.Id;
  # Index into `CompiledExperiment.parameters`.
}

struct LoopStart {
  count @0 :UInt32;
  # Number of iterations. Must be >= 1.
}

struct ExecRealTime {}
# Deliberately an empty struct, not Void: reserves the slot for extension.

struct ExecCallback {
  # Invoke a near-time callback.

  callbackId @0 :Text;
  # Callback UID.

  args @1 :List(Common.ValueEntry);
  # Named arguments passed to the callback.
}

struct SetNode {
  # Set a device node to a literal or parameter value.

  path @0 :Text;
  # Device node path.

  value @1 :Common.Value;
}

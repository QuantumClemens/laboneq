// Copyright 2025 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

use laboneq_dsl::types::{
    HandleUid, ParameterUid, SectionTimingMode, SectionUid, SignalUid, SweepParameter,
};
use laboneq_units::tinysample::TinySamples;
use num_integer::lcm;
use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::parameter_resolver::ParameterResolver;
use crate::utils::{SignalGridInfo, compute_signal_grids};

pub(super) struct LocalContext<'a> {
    pub section_uid: Option<SectionUid>,
    resolver_stack: Vec<ParameterResolver<'a>>,
    pub system_grid: TinySamples,
    signal_grids: HashMap<SignalUid, (TinySamples, TinySamples)>,
    pub section_timing_mode: SectionTimingMode,
    /// Map from acquisition handle UIDs to signal UIDs, populated as
    /// `Acquire` operations are visited in program order.
    handle_to_signal: HashMap<HandleUid, SignalUid>,
}

impl<'a> LocalContext<'a> {
    pub(super) fn new<'b>(
        parameters: &'a HashMap<ParameterUid, SweepParameter>,
        nt_parameters: &'a crate::ParameterStore,
        system_grid: TinySamples,
        signals: impl Iterator<Item = &'b (impl SignalGridInfo + 'b)>,
    ) -> Result<Self> {
        let resolver: ParameterResolver = ParameterResolver::new(parameters, nt_parameters);
        let signal_grids = signals
            .map(|s| Ok((s.uid(), compute_signal_grids(s)?)))
            .collect::<Result<HashMap<_, _>>>()?;
        Ok(LocalContext {
            section_uid: None,
            section_timing_mode: SectionTimingMode::Relaxed,
            resolver_stack: vec![resolver],
            system_grid,
            signal_grids,
            handle_to_signal: HashMap::new(),
        })
    }

    /// Record that `handle` is associated with `signal`, as observed at an
    /// `Acquire` operation.
    pub(super) fn set_handle_signal(&mut self, handle: HandleUid, signal: SignalUid) -> Result<()> {
        if let Some(existing_signal) = self.handle_to_signal.get(&handle)
            && existing_signal != &signal
        {
            return Err(Error::new(format!(
                "Acquisition handle '{}' is associated with multiple signals, only one allowed.",
                handle.0
            )));
        }
        self.handle_to_signal.insert(handle, signal);
        Ok(())
    }

    /// Look up the signal most recently associated with `handle` by an
    /// `Acquire` operation earlier in program order.
    pub(super) fn handle_signal(&self, handle: &HandleUid) -> Result<&SignalUid> {
        self.handle_to_signal.get(handle).ok_or_else(|| {
            Error::new(format!(
                "Handle '{}' is used in a match before being acquired.",
                handle.0
            ))
        })
    }

    pub(super) fn signal_grids(&self, signal: &SignalUid) -> (TinySamples, TinySamples) {
        self.signal_grids[signal]
    }

    pub(crate) fn calculate_grids(
        &self,
        signals: impl Iterator<Item = SignalUid>,
        escalate_to_sequencer_grid: bool,
        on_system_grid: bool,
    ) -> (TinySamples, TinySamples) {
        let mut signals_grid = 1;
        let mut sequencer_grid = 1;
        let mut multiple_grids = false;

        for signal in signals {
            let (grid, sequencer) = self.signal_grids(&signal);
            if !multiple_grids && signals_grid != 1 && signals_grid != grid.value() {
                multiple_grids = true;
            }
            signals_grid = lcm(signals_grid, grid.value());
            sequencer_grid = lcm(sequencer_grid, sequencer.value());
        }
        let mut grid = 1;
        if on_system_grid {
            grid = lcm(grid, self.system_grid.value());
        }
        if multiple_grids || escalate_to_sequencer_grid {
            // two different sample rates -> escalate to sequencer grid
            grid = lcm(grid, sequencer_grid);
        } else {
            grid = lcm(grid, signals_grid);
        }
        (grid.into(), sequencer_grid.into())
    }

    pub(super) fn with_loop<R, T: FnMut(&mut Self) -> R>(
        &mut self,
        section_uid: SectionUid,
        parameters: &[ParameterUid],
        section_timing_mode: SectionTimingMode,
        mut f: T,
    ) -> Result<R> {
        let previous_section_uid = self.section_uid;
        let previous_section_timing_mode = self.section_timing_mode;
        self.section_uid = Some(section_uid);
        self.section_timing_mode = section_timing_mode;
        let resolver = self
            .resolver_stack
            .last()
            .unwrap()
            .child_scope(parameters)?;
        self.resolver_stack.push(resolver);
        let result = f(self);
        self.resolver_stack.pop();
        self.section_uid = previous_section_uid;
        self.section_timing_mode = previous_section_timing_mode;
        Ok(result)
    }

    pub(super) fn with_section<R, T: FnMut(&mut Self) -> R>(
        &mut self,
        section_uid: SectionUid,
        section_timing_mode: SectionTimingMode,
        mut f: T,
    ) -> R {
        let previous_section_uid = self.section_uid;
        let previous_section_timing_mode = self.section_timing_mode;
        self.section_uid = Some(section_uid);
        self.section_timing_mode = section_timing_mode;
        let result = f(self);
        self.section_uid = previous_section_uid;
        self.section_timing_mode = previous_section_timing_mode;
        result
    }

    pub(super) fn parameter_resolver(&self) -> &ParameterResolver<'a> {
        self.resolver_stack.last().unwrap()
    }

    pub(super) fn section_name(&self) -> String {
        self.section_uid
            .map_or_else(|| "unknown".to_string(), |s| s.0.to_string())
    }
}

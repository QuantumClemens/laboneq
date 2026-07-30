// Copyright 2026 Zurich Instruments AG
// SPDX-License-Identifier: Apache-2.0

//! Logging utilities for the Python bindings.
//!
//! This module provides a function to initialize logging that bridges between Python and Rust,
//! allowing Rust logs to be captured and displayed according to the log level and loggers set in Python.

use std::ffi::CString;
use std::fmt;
use std::sync::LazyLock;

use log::LevelFilter;

use pyo3::prelude::*;
use pyo3_log::Logger;
use pyo3_log::ResetHandle;

use laboneq_log::init_logging;

static PY_RESET_HANDLE: LazyLock<ResetHandle> = LazyLock::new(|| {
    let py_logger = Box::new(Logger::default());
    let handle = py_logger.reset_handle();
    log::set_boxed_logger(Box::new(py_logger)).unwrap();
    handle
});

/// A bridge between Python and Rust logging.
///
/// This function will setup logging that respects the log level and loggers set in Python.
///
/// Therefore this function should be called at the start of each compilation to ensure that Python log level
/// changes made between compilations are picked up.
///
/// Arguments:
/// - `log_level`: The Python log level as an integer. Follows the standard Python logging levels.
pub fn init_logging_py(log_level: i64) -> PyResult<()> {
    // Diagnostics is a custom `laboneq`-specific logging level between info and debug.
    // Therefore it needs to be handled separately here, and cannot be directly mapped to a Python log level.
    const DIAGNOSTICS_LEVEL: i64 = 15;
    let log_diagnostics = log_level <= DIAGNOSTICS_LEVEL;
    init_logging(log_diagnostics);
    PY_RESET_HANDLE.reset();

    let level_filter = match log_level {
        0 => LevelFilter::Warn,
        1..=10 => LevelFilter::Debug,
        11..=20 => LevelFilter::Info,
        21..=30 => LevelFilter::Warn,
        31..=40 => LevelFilter::Error,
        _ => LevelFilter::Error,
    };
    log::set_max_level(level_filter);
    Ok(())
}

/// Issues a Python-visible `FutureWarning` and mirrors it to the Rust log,
/// routed through `PyErr_WarnEx` so it respects the `warnings` module's
/// filters (`-W` flags, `simplefilter("error")`, pytest capture, ...).
///
/// `FutureWarning` is used rather than `DeprecationWarning` because it isn't
/// hidden by Python's default warning filters.
///
/// Takes [`fmt::Arguments`] (like `log::warn!`/`format!`) so callers can pass
/// either a plain static message or a format string with arguments; prefer
/// the [`deprecation_warning!`] macro over calling this directly.
pub fn deprecation_warning(py: Python<'_>, message: fmt::Arguments<'_>) -> PyResult<()> {
    log::warn!("Deprecation warning: {message}");
    let message = CString::new(message.to_string())
        .expect("deprecation warning message must not contain NUL bytes");
    PyErr::warn(
        py,
        py.get_type::<pyo3::exceptions::PyFutureWarning>().as_any(),
        &message,
        2,
    )
}

/// Emits a deprecation warning, formatting its arguments like `format!`.
///
/// ```ignore
/// deprecation_warning!(py, "`foo` is deprecated")?;
/// deprecation_warning!(py, "`{name}` is deprecated, use `{replacement}` instead")?;
/// ```
#[macro_export]
macro_rules! deprecation_warning {
    ($py:expr, $($arg:tt)*) => {
        $crate::logging::deprecation_warning($py, format_args!($($arg)*))
    };
}

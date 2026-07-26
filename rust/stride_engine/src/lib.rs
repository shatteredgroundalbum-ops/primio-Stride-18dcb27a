//! stride_engine
//!
//! Core walking/running workout engine for S.T.R.I.D.E.
//!
//! This crate is the single source of truth for all workout-session logic:
//! GPS validation, distance/speed/pace calculation, walk-vs-run
//! classification, auto-pause, calorie estimation, elevation, splits, goal
//! tracking, coaching events, personal records, and crash recovery.
//!
//! It is compiled as a `cdylib`/`staticlib` and consumed by the Flutter UI
//! through a C-ABI FFI boundary (see [`ffi`]) using `dart:ffi`. All inputs
//! and outputs across that boundary are JSON strings so the Dart side never
//! needs to know Rust struct layouts.
//!
//! Canonical units used everywhere internally: meters, meters/second,
//! meters (elevation), kilograms, epoch milliseconds. Conversion to
//! user-facing units (miles, feet, lb, mph) happens only in
//! [`engine::units`] at presentation time.

pub mod models;
pub mod engine;
pub mod ffi;

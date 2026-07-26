//! The workout engine: stateful subsystems that process the live stream of
//! GPS/sensor data into a complete workout record. Each submodule is
//! independently unit-tested and (mostly) side-effect free — the
//! [`controller`] module wires them together into the single
//! `WorkoutSessionController` the FFI boundary exposes to Dart.

pub mod achievements;
pub mod account;
pub mod autopause;
pub mod background;
pub mod battery;
pub mod calories;
pub mod classification;
pub mod coaching;
pub mod coaching_plan;
pub mod controller;
pub mod distance;
pub mod elevation;
pub mod gps_filter;
pub mod gps_quality;
pub mod maps;
pub mod music;
pub mod heart_rate;
pub mod movement;
pub mod pace;
pub mod permissions;
pub mod personal_records;
pub mod recovery;
pub mod route;
pub mod route_format;
pub mod security;
pub mod speed;
pub mod splits;
pub mod steps;
pub mod sync;
pub mod timekeeping;
pub mod units;
pub mod validation;
pub mod wearable;

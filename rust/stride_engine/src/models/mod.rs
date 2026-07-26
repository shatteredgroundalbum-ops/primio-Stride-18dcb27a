//! Core data models shared across the workout engine.
//!
//! Canonical units (per spec section 34 — Unit System):
//!   - distance   -> meters
//!   - speed      -> meters per second
//!   - elevation  -> meters
//!   - weight     -> kilograms
//!   - time       -> milliseconds since Unix epoch (UTC) for instants,
//!                   milliseconds (u64) for durations
//!
//! All conversions to miles/feet/mph/lb etc. happen only at the display
//! layer (Dart side or `units.rs` helpers), never internally.

pub mod point;
pub mod session;
pub mod split;
pub mod goal;
pub mod checkpoint;
pub mod sensor;
pub mod events;
pub mod summary;

pub use point::*;
pub use session::*;
pub use split::*;
pub use goal::*;
pub use checkpoint::*;
pub use sensor::*;
pub use events::*;
pub use summary::*;

/// Epoch milliseconds (UTC). Using an integer instead of `DateTime` at the
/// FFI boundary keeps serialization trivial and avoids timezone ambiguity.
pub type EpochMillis = i64;

/// Generates a new unique identifier as a string (UUID v4).
pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

//! Route file format & storage layout (spec section 4).
//!
//! The spec is explicit: **do not save thousands of raw GPS points inside a
//! single Firestore document.** Firestore documents are cheap to read but
//! expensive to write at scale, and a 10 km run can easily produce 3 000+
//! GPS fixes. Instead the architecture splits storage across four tiers:
//!
//!   1. **SQLite** — live recording buffer (the app writes every accepted
//!      point here in real time so a crash never loses the route).
//!   2. **Cloud Storage** — the *full-resolution* route is uploaded as a
//!      compressed binary blob or GPX file. This is the authoritative
//!      source of truth for the raw track.
//!   3. **Firestore** — a *summary* document per workout holds the
//!      aggregate metrics (distance, pace, calories, …) **plus a
//!      `route_file_path`** pointing at the Cloud Storage object. The
//!      summary is small (a few hundred bytes) and cheap to query/list.
//!   4. **Encoded polyline** — a Google-style polyline string is embedded
//!      in the Firestore summary for *fast map previews* without needing
//!      to download the full route file from Cloud Storage.
//!
//! This module provides the Rust-side decision logic and serialization for
//! that split:
//!
//!   - [`serialize_gpx`] turns a slice of [`WorkoutPoint`] into a valid
//!     GPX 1.1 XML document (the human-readable interchange format).
//!   - [`RouteFileFormat`] / [`decide_route_format`] choose between GPX,
//!     a compact binary format, and polyline-only, based on point count
//!     and whether the device is offline (GPX is larger but
//!     universally compatible; the binary format is ~40 % smaller).
//!   - [`generate_route_file_path`] builds the Cloud Storage object key
//!     (`routes/{user_id}/{workout_id}.{ext}`) so the summary document
//!     can reference it.
//!   - [`estimate_route_file_size`] predicts the byte size of a route
//!     file before it is serialized, so the sync layer can decide whether
//!     to upload over mobile data or wait for Wi-Fi.
//!   - [`RouteFileMetadata`] captures the per-route sync state (distinct
//!     from the per-*summary* sync state in `engine::sync`): a route
//!     file can be `LocalOnly`, `Uploading`, `Uploaded`, or `Failed`
//!     independently of whether the Firestore summary row is synced.
//!   - [`RouteSummary`] is the Rust model of the Firestore summary
//!     document — every field the spec lists, with the route data
//!     reduced to start/end coordinates + a polyline + a file path.

use serde::{Deserialize, Serialize};

use crate::models::{ActivityType, LocationSource, WorkoutPoint};
use crate::models::{EpochMillis, WorkoutSummary};

// ──────────────────────────────────────────────────────────────────────
// Route file format selection
// ──────────────────────────────────────────────────────────────────────

/// Which on-disk / Cloud-Storage format the full-resolution route uses.
///
/// The choice is a *decision*, not a configuration: the engine picks the
/// best format for the current situation (point count, offline status,
/// available storage) so the Dart layer never has to reason about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteFileFormat {
    /// GPX 1.1 XML — human-readable, universally compatible with
    /// Strava / Garmin / Google Earth import. Largest of the three
    /// (~180 bytes/point) but the safest interchange format.
    Gpx,
    /// App-specific compact binary — ~110 bytes/point. Smaller than
    /// GPX but requires the app to decode it; not portable to other
    /// platforms. Chosen when storage/mobile-data is constrained and
    /// the route is long.
    CompressedBinary,
    /// No separate route file at all — the route is stored *only* as
    /// an encoded polyline inside the Firestore summary document.
    /// Chosen for very short workouts (<50 points) where the polyline
    /// alone is small enough to live in Firestore without violating
    /// the "no raw point dumps" rule.
    PolylineOnly,
}

impl RouteFileFormat {
    /// File extension used in the Cloud Storage object key.
    pub fn extension(self) -> &'static str {
        match self {
            RouteFileFormat::Gpx => "gpx",
            RouteFileFormat::CompressedBinary => "croute",
            RouteFileFormat::PolylineOnly => "",
        }
    }

    /// Estimated bytes per point for this format (used by
    /// [`estimate_route_file_size`] before the route is actually
    /// serialized).
    pub fn bytes_per_point(self) -> usize {
        match self {
            RouteFileFormat::Gpx => 180,
            RouteFileFormat::CompressedBinary => 110,
            // Polyline-only has no separate file; the polyline lives in
            // the summary. Report 0 so size-based gating logic treats it
            // as "no upload needed."
            RouteFileFormat::PolylineOnly => 0,
        }
    }

    /// Whether this format produces a separate file that must be
    /// uploaded to Cloud Storage (as opposed to being embedded in the
    /// Firestore summary).
    pub fn requires_file_upload(self) -> bool {
        !matches!(self, RouteFileFormat::PolylineOnly)
    }
}

/// Inputs to [`decide_route_format`]. All fields are caller-supplied so
/// the decision logic is pure and unit-testable without any I/O.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteFormatInput {
    /// Number of accepted GPS points in the route.
    pub point_count: usize,
    /// Whether the device is currently offline (no network). When
    /// offline the engine prefers the smallest format to minimise the
    /// bytes queued for later upload.
    pub is_offline: bool,
    /// Whether the user has enabled "export to Strava/Garmin" for this
    /// workout. If so, GPX is forced so the file is directly importable
    /// by third-party platforms without a server-side conversion.
    pub wants_gpx_export: bool,
}

/// Threshold below which a polyline-only summary is sufficient (no
/// separate route file). 50 points × ~5 s/point ≈ a 4-minute walk —
/// small enough that the encoded polyline is well under Firestore's
/// 1 MiB document limit and a separate file is overkill.
const POLYLINE_ONLY_THRESHOLD: usize = 50;

/// Decides which route file format to use for a given workout.
///
/// Decision rules (in priority order):
///   1. If `wants_gpx_export` is true → **GPX** (the user needs a
///      portable file).
///   2. If `point_count` < `POLYLINE_ONLY_THRESHOLD` → **PolylineOnly**
///      (the route is short enough to live in the summary).
///   3. If `is_offline` → **CompressedBinary** (minimise queued bytes).
///   4. Otherwise → **GPX** (online + long route: the safer, more
///      compatible format, since storage cost is less of a concern when
///      uploading over Wi-Fi).
pub fn decide_route_format(input: &RouteFormatInput) -> RouteFileFormat {
    if input.wants_gpx_export {
        return RouteFileFormat::Gpx;
    }
    if input.point_count < POLYLINE_ONLY_THRESHOLD {
        return RouteFileFormat::PolylineOnly;
    }
    if input.is_offline {
        return RouteFileFormat::CompressedBinary;
    }
    RouteFileFormat::Gpx
}

// ──────────────────────────────────────────────────────────────────────
// Route file path generation
// ──────────────────────────────────────────────────────────────────────

/// Generates the Cloud Storage object key for a route file.
///
/// Format: `routes/{user_id}/{workout_id}.{ext}`
///
/// The `user_id` prefix means Firestore security rules can verify
/// ownership by checking that the path prefix matches the authenticated
/// user (see §7). The workout_id is already a unique opaque ID, so no
/// collision-avoidance suffix is needed.
///
/// Returns `None` for [`RouteFileFormat::PolylineOnly`] since no file
/// is uploaded in that case.
pub fn generate_route_file_path(
    user_id: &str,
    workout_id: &str,
    format: RouteFileFormat,
) -> Option<String> {
    if !format.requires_file_upload() {
        return None;
    }
    Some(format!(
        "routes/{user_id}/{workout_id}.{ext}",
        ext = format.extension()
    ))
}

// ──────────────────────────────────────────────────────────────────────
// Route file size estimation
// ──────────────────────────────────────────────────────────────────────

/// Fixed overhead bytes for the GPX XML envelope (the `<gpx>` root
/// element, metadata, and the single `<trk><trkseg>` wrapper).
const GPX_OVERHEAD_BYTES: usize = 512;

/// Estimated size, in bytes, of a route file before it is serialized.
///
/// This is a *prediction* (bytes-per-point × point_count + overhead),
/// not a measurement. The sync layer uses it to decide whether to
/// upload immediately over Wi-Fi or defer until the user is on a
/// cheaper network. The actual serialized size may differ slightly
/// (coordinate precision varies), but the estimate is conservative
/// enough for gating decisions.
///
/// Returns 0 for [`RouteFileFormat::PolylineOnly`] since no separate
/// file exists.
pub fn estimate_route_file_size(format: RouteFileFormat, point_count: usize) -> usize {
    if !format.requires_file_upload() {
        return 0;
    }
    format.bytes_per_point() * point_count + GPX_OVERHEAD_BYTES
}

/// Network-cost threshold (bytes) above which the sync layer should
/// prefer Wi-Fi over mobile data for the route file upload. 512 KiB
/// is small enough to upload in a few seconds over a decent mobile
/// connection but large enough that a 3 000-point GPX (~560 KiB)
/// triggers the Wi-Fi preference.
const WIFI_PREFERRED_THRESHOLD_BYTES: usize = 512 * 1024;

/// Decides whether a route file upload should wait for Wi-Fi rather
/// than proceeding over mobile data.
///
/// Returns `true` (prefer Wi-Fi) when the estimated file size exceeds
/// [`WIFI_PREFERRED_THRESHOLD_BYTES`] **and** the device is not
/// currently on Wi-Fi. Small files and Wi-Fi connections upload
/// immediately.
pub fn should_prefer_wifi_for_upload(
    format: RouteFileFormat,
    point_count: usize,
    is_on_wifi: bool,
) -> bool {
    if is_on_wifi {
        return false;
    }
    estimate_route_file_size(format, point_count) > WIFI_PREFERRED_THRESHOLD_BYTES
}

// ──────────────────────────────────────────────────────────────────────
// GPX 1.1 serialization
// ──────────────────────────────────────────────────────────────────────

/// GPX 1.1 XML namespace URI.
const GPX_NS: &str = "http://www.topografix.com/GPX/1/1";

/// Serializes a slice of accepted [`WorkoutPoint`]s into a GPX 1.1 XML
/// document.
///
/// The output is a complete, valid GPX file:
///
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <gpx version="1.1" creator="S.T.R.I.D.E." xmlns="http://www.topografix.com/GPX/1/1">
///   <metadata>
///     <name>{workout_id}</name>
///     <time>{started_at_iso8601}</time>
///   </metadata>
///   <trk>
///     <name>{workout_id}</name>
///     <trkseg>
///       <trkpt lat="40.0" lon="-73.0">
///         <ele>12.5</ele>
///         <time>2024-01-01T12:00:00Z</time>
///       </trkpt>
///       ...
///     </trkseg>
///   </trk>
/// </gpx>
/// ```
///
/// Only accepted points are emitted (the caller is expected to have
/// already filtered rejected points, but this function double-checks
/// the `accepted` flag and skips any point that was rejected — a
/// rejected point should never appear in the exported route).
///
/// Altitude (`<ele>`) is emitted only when present. Speed and accuracy
/// are not standard GPX fields and are omitted to keep the file
/// portable; they remain in the SQLite raw table and the compressed
/// binary format for the app's own use.
pub fn serialize_gpx(
    workout_id: &str,
    started_at_ms: EpochMillis,
    points: &[WorkoutPoint],
) -> String {
    let mut out = String::with_capacity(estimate_route_file_size(RouteFileFormat::Gpx, points.len()));

    // XML declaration + root element
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<gpx version=\"1.1\" creator=\"S.T.R.I.D.E.\" xmlns=\"{ns}\">\n",
        ns = GPX_NS
    ));

    // Metadata block
    out.push_str("  <metadata>\n");
    // XML-escape the workout ID in case it ever contains characters
    // that are illegal in XML (it shouldn't — it's a UUID — but
    // defensive escaping is cheap and prevents a malformed export).
    out.push_str(&format!(
        "    <name>{}</name>\n",
        xml_escape(workout_id)
    ));
    out.push_str(&format!(
        "    <time>{}</time>\n",
        epoch_to_iso8601(started_at_ms)
    ));
    out.push_str("  </metadata>\n");

    // Track
    out.push_str("  <trk>\n");
    out.push_str(&format!("    <name>{}</name>\n", xml_escape(workout_id)));
    out.push_str("    <trkseg>\n");

    for p in points.iter().filter(|p| p.accepted) {
        out.push_str(&format!(
            "      <trkpt lat=\"{lat}\" lon=\"{lon}\">\n",
            lat = format_coord(p.latitude),
            lon = format_coord(p.longitude)
        ));
        if let Some(alt) = p.altitude_meters {
            out.push_str(&format!("        <ele>{}</ele>\n", format_altitude(alt)));
        }
        out.push_str(&format!(
            "        <time>{}</time>\n",
            epoch_to_iso8601(p.recorded_at)
        ));
        out.push_str("      </trkpt>\n");
    }

    out.push_str("    </trkseg>\n");
    out.push_str("  </trk>\n");
    out.push_str("</gpx>\n");

    out
}

/// Escapes the five XML special characters in a text string.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Formats a latitude/longitude to 7 decimal places (GPX convention;
/// ~1 cm precision at the equator, more than enough for GPS).
fn format_coord(v: f64) -> String {
    format!("{:.7}", v)
}

/// Formats an altitude to 1 decimal place (GPS altitude is rarely
/// better than ~0.5 m accuracy, so more precision is misleading).
fn format_altitude(v: f64) -> String {
    format!("{:.1}", v)
}

/// Converts an epoch-millisecond timestamp to an ISO 8601 UTC string
/// (`YYYY-MM-DDTHH:MM:SSZ`), the format GPX `<time>` elements require.
///
/// This is a pure arithmetic conversion (no `chrono` dependency) so the
/// engine stays lightweight. It handles leap years correctly for all
/// dates after 1970-01-01.
fn epoch_to_iso8601(epoch_ms: EpochMillis) -> String {
    let total_seconds = (epoch_ms / 1000) as i64;
    let millis_part = epoch_ms % 1000;

    // Time-of-day
    let seconds_of_day = total_seconds.rem_euclid(86400);
    let hour = seconds_of_day / 3600;
    let minute = (seconds_of_day % 3600) / 60;
    let second = seconds_of_day % 60;

    // Date: day number since epoch → year/month/day. We count forward
    // from 1970-01-01, accounting for leap years.
    let day_number = total_seconds.div_euclid(86400);
    let (year, month, day) = day_number_to_date(day_number);

    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z",
        year = year,
        month = month,
        day = day,
        hour = hour,
        minute = minute,
        second = second,
        millis = millis_part
    )
}

/// Converts a day number (days since 1970-01-01) to a (year, month, day)
/// triple. Handles leap years.
fn day_number_to_date(days_since_epoch: i64) -> (i64, u32, u32) {
    // Start from 1970-01-01 and walk forward.
    let mut year = 1970i64;
    let mut remaining = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    // Now `remaining` is the day-of-year (0-indexed). Find the month.
    let leap = is_leap_year(year);
    let month_days: [u32; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    let mut day_remaining = remaining as u32;
    for &md in &month_days {
        if day_remaining < md {
            break;
        }
        day_remaining -= md;
        month += 1;
    }

    (year, month, day_remaining + 1)
}

/// Whether a given year is a Gregorian leap year.
fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// ──────────────────────────────────────────────────────────────────────
// Route file metadata & sync state
// ──────────────────────────────────────────────────────────────────────

/// Sync state of a route file, tracked **independently** from the
/// Firestore summary document's sync state. A route can be uploaded to
/// Cloud Storage while the summary row is still pending, or vice
/// versa — the two uploads are separate operations with separate retry
/// counts (see `engine::sync` for the summary-side state machine).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteFileSyncState {
    /// Route exists only in local SQLite; not yet uploaded.
    LocalOnly,
    /// Upload to Cloud Storage is in progress.
    Uploading,
    /// Route file is in Cloud Storage; the Firestore summary may or may
    /// not be synced yet (that's tracked separately).
    Uploaded,
    /// Upload failed after all retries; the route is stuck locally.
    /// The user can manually retry or the next app open may re-queue it.
    Failed,
}

/// Whether a route file sync state is terminal (no further automatic
/// action will be taken). `Uploaded` and `Failed` are terminal; the
/// sync loop stops retrying once it reaches either.
pub fn is_route_sync_terminal(state: RouteFileSyncState) -> bool {
    matches!(state, RouteFileSyncState::Uploaded | RouteFileSyncState::Failed)
}

/// Metadata about a route file — what the Dart layer needs to track the
/// file's location and sync status, and what gets stored in the SQLite
/// `route_files` table alongside the raw points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteFileMetadata {
    /// Cloud Storage object key (e.g.
    /// `routes/{user_id}/{workout_id}.gpx`). `None` for
    /// `PolylineOnly` routes (no file exists).
    pub file_path: Option<String>,
    /// Format of the file (determines how the Dart side decodes it).
    pub format: RouteFileFormat,
    /// Estimated or actual byte size of the serialized file. Used by
    /// the sync layer for network-cost gating.
    pub size_bytes: usize,
    /// Number of GPS points in the route (accepted points only).
    pub point_count: usize,
    /// Which device recorded the route (for multi-device sync; see
    /// `engine::sync::DeviceSyncState`). Stored so a download from
    /// Cloud Storage can show "recorded on Pixel 8" in the UI.
    pub device_source: String,
    /// Sync state of the route file specifically.
    pub sync_state: RouteFileSyncState,
    /// Epoch milliseconds when the route file was first created
    /// (i.e. when the workout finished and the file was serialized).
    pub created_at: EpochMillis,
    /// Epoch milliseconds when the route file metadata was last
    /// updated (e.g. sync state changed from Uploading → Uploaded).
    pub updated_at: EpochMillis,
}

/// Builds a [`RouteFileMetadata`] from the inputs the controller has
/// at workout-finish time. This is the pure decision logic; the actual
/// file serialization and Cloud Storage upload happen on the Dart
/// side (the engine never does I/O).
///
/// `device_source` is the identifier of the device that recorded the
/// workout (e.g. "pixel-8-pro" or a random device UUID). It's stored
/// on the route file so multi-device sync can attribute the route
/// correctly (see `engine::sync::decide_device_sync_action`).
pub fn build_route_file_metadata(
    user_id: &str,
    workout_id: &str,
    points: &[WorkoutPoint],
    device_source: &str,
    is_offline: bool,
    wants_gpx_export: bool,
    finished_at: EpochMillis,
) -> RouteFileMetadata {
    let accepted_count = points.iter().filter(|p| p.accepted).count();
    let format = decide_route_format(&RouteFormatInput {
        point_count: accepted_count,
        is_offline,
        wants_gpx_export,
    });
    let file_path = generate_route_file_path(user_id, workout_id, format);
    let size_bytes = estimate_route_file_size(format, accepted_count);

    RouteFileMetadata {
        file_path,
        format,
        size_bytes,
        point_count: accepted_count,
        device_source: device_source.to_string(),
        sync_state: RouteFileSyncState::LocalOnly,
        created_at: finished_at,
        updated_at: finished_at,
    }
}

// ──────────────────────────────────────────────────────────────────────
// Firestore summary document model
// ──────────────────────────────────────────────────────────────────────

/// Compact, sync-ready summary of a workout for Firestore.
///
/// This is the **Firestore document** shape — every field the spec
/// lists, with the full-resolution route reduced to:
///   - start/end coordinates (two pairs of floats),
///   - an encoded polyline (a few hundred bytes),
///   - a `route_file_path` pointing at the Cloud Storage object.
///
/// The raw GPS points are **never** stored in this document. They live
/// in Cloud Storage (via `route_file_path`) and in the local SQLite
/// raw table. This keeps every Firestore document under ~2 KiB so
/// listing the user's workout history is fast and cheap.
///
/// The Rust [`WorkoutSummary`] produced by the engine has more fields
/// (splits, goal results, validation warnings) that are useful
/// locally but too large for a Firestore summary row. This struct is
/// the *projection* of that summary into the Firestore-ready shape;
/// use [`build_route_summary`] to convert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSummary {
    /// Unique workout identifier (UUID).
    pub workout_id: String,
    /// Owning user's Firebase Auth UID.
    pub user_id: String,
    /// Activity type (walk / run / hike).
    pub activity_type: ActivityType,
    /// Epoch ms when the workout started.
    pub started_at: EpochMillis,
    /// Epoch ms when the workout ended.
    pub ended_at: EpochMillis,
    /// Total duration in milliseconds (including pauses).
    pub duration_ms: i64,
    /// Total distance in meters.
    pub distance_meters: f64,
    /// Average pace in seconds per kilometer.
    pub average_pace_sec_per_km: f64,
    /// Average speed in meters per second.
    pub average_speed_mps: f64,
    /// Total step count.
    pub steps: u32,
    /// Average heart rate in bpm, if any HR data was recorded.
    pub average_heart_rate_bpm: Option<f64>,
    /// Maximum heart rate in bpm, if any HR data was recorded.
    pub max_heart_rate_bpm: Option<u16>,
    /// Estimated calories burned.
    pub calories: f64,
    /// Method used for calorie estimation (e.g. "wearable", "hr",
    /// "met", "distance_weight").
    pub calorie_method: String,
    /// Start latitude.
    pub start_latitude: Option<f64>,
    /// Start longitude.
    pub start_longitude: Option<f64>,
    /// End latitude.
    pub end_latitude: Option<f64>,
    /// End longitude.
    pub end_longitude: Option<f64>,
    /// Google-style encoded polyline of the simplified route, for
    /// fast map previews without downloading the full route file.
    pub encoded_polyline: Option<String>,
    /// Cloud Storage object key for the full-resolution route file.
    /// `None` if the route was polyline-only (no separate file).
    pub route_file_path: Option<String>,
    /// Format of the route file (so the Dart side knows how to
    /// decode it on download).
    pub route_file_format: RouteFileFormat,
    /// Sync state of the Firestore summary document (uses the
    /// `engine::sync` SyncQueueState: pending / uploading / synced /
    /// failed).
    pub sync_state: crate::models::SyncQueueState,
    /// Which device recorded the workout (for multi-device sync).
    pub device_source: String,
    /// Epoch ms when the summary was first created (workout finish).
    pub created_at: EpochMillis,
    /// Epoch ms when the summary was last updated (e.g. re-synced,
    /// re-uploaded after an edit).
    pub updated_at: EpochMillis,
}

/// Builds a Firestore-ready [`RouteSummary`] from the engine's
/// [`WorkoutSummary`], the route file metadata, and the encoded
/// polyline.
///
/// This is the bridge between the engine's rich internal summary and
/// the compact Firestore document. The `WorkoutSummary` carries the
/// full computation results; this function projects the subset the
/// spec requires for the Firestore row and stamps it with the route
/// file path + sync state.
pub fn build_route_summary(
    summary: &WorkoutSummary,
    route_file: &RouteFileMetadata,
    device_source: &str,
    now_ms: EpochMillis,
) -> RouteSummary {
    RouteSummary {
        workout_id: summary.workout_id.clone(),
        user_id: summary.user_id.clone(),
        activity_type: summary.activity_type,
        started_at: summary.started_at,
        ended_at: summary.ended_at,
        duration_ms: summary.total_duration_ms,
        distance_meters: summary.distance_meters,
        average_pace_sec_per_km: summary.average_pace_sec_per_km,
        average_speed_mps: summary.average_speed_mps,
        steps: summary.step_count,
        average_heart_rate_bpm: summary.average_heart_rate_bpm,
        max_heart_rate_bpm: summary.max_heart_rate_bpm,
        calories: summary.calories_estimated,
        calorie_method: summary.calorie_estimate_method.clone(),
        start_latitude: summary.start_latitude,
        start_longitude: summary.start_longitude,
        end_latitude: summary.end_latitude,
        end_longitude: summary.end_longitude,
        encoded_polyline: summary.encoded_polyline.clone(),
        route_file_path: route_file.file_path.clone(),
        route_file_format: route_file.format,
        // Summary starts pending; the sync layer advances it.
        sync_state: crate::models::SyncQueueState::Pending,
        device_source: device_source.to_string(),
        created_at: route_file.created_at,
        updated_at: now_ms,
    }
}

/// Validates that a [`RouteSummary`] is safe to write to Firestore:
///   - `workout_id` and `user_id` must be non-empty.
///   - `route_file_path` (if present) must start with `routes/{user_id}/`
///     so Firestore security rules can verify ownership by path prefix
///     (see §7).
///   - `encoded_polyline` (if present) must be under 1 MiB so the
///     document stays within Firestore's 1 MiB document limit.
///
/// Returns `Ok(())` if valid, or a descriptive error string otherwise.
/// The Dart layer calls this *before* writing to Firestore so a bug
/// never produces an invalid or oversized document.
pub fn validate_route_summary(summary: &RouteSummary) -> Result<(), String> {
    if summary.workout_id.is_empty() {
        return Err("workout_id is empty".to_string());
    }
    if summary.user_id.is_empty() {
        return Err("user_id is empty".to_string());
    }
    if let Some(ref path) = summary.route_file_path {
        let expected_prefix = format!("routes/{}/", summary.user_id);
        if !path.starts_with(&expected_prefix) {
            return Err(format!(
                "route_file_path does not start with expected prefix '{}'",
                expected_prefix
            ));
        }
    }
    if let Some(ref polyline) = summary.encoded_polyline {
        // 1 MiB = 1 048 576 bytes. UTF-8 ASCII polyline chars are 1 byte
        // each, so byte length == char count.
        const FIRESTORE_MAX_DOC_BYTES: usize = 1_048_576;
        if polyline.len() > FIRESTORE_MAX_DOC_BYTES {
            return Err(format!(
                "encoded_polyline is {} bytes, exceeds Firestore 1 MiB document limit",
                polyline.len()
            ));
        }
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────
// GPS source dominance for route attribution
// ──────────────────────────────────────────────────────────────────────

/// Determines the dominant [`LocationSource`] of a route — the source
/// that contributed the most accepted points. Used to attribute the
/// route to "Phone GPS", "Wear OS", or "Health Connect" in the UI and
/// to pick the right sync priority (a Wear-OS-recorded route should
/// sync from the watch, not the phone, when both are available).
///
/// Returns `None` if the route has no accepted points.
pub fn dominant_location_source(points: &[WorkoutPoint]) -> Option<LocationSource> {
    let accepted: Vec<&WorkoutPoint> = points.iter().filter(|p| p.accepted).collect();
    if accepted.is_empty() {
        return None;
    }

    let mut counts: [(LocationSource, usize); 6] = [
        (LocationSource::PhoneGps, 0),
        (LocationSource::WearOs, 0),
        (LocationSource::HealthConnect, 0),
        (LocationSource::Manual, 0),
        (LocationSource::ServerCorrected, 0),
        (LocationSource::Estimated, 0),
    ];

    for p in &accepted {
        if let Some(slot) = counts.iter_mut().find(|(s, _)| *s == p.source) {
            slot.1 += 1;
        }
    }

    counts
        .iter()
        .max_by_key(|(_, c)| *c)
        .filter(|(_, c)| *c > 0)
        .map(|(s, _)| *s)
}

/// Human-readable label for a [`LocationSource`], for UI display.
pub fn location_source_label(source: LocationSource) -> &'static str {
    match source {
        LocationSource::PhoneGps => "Phone GPS",
        LocationSource::WearOs => "Wear OS",
        LocationSource::HealthConnect => "Health Connect",
        LocationSource::Manual => "Manual entry",
        LocationSource::ServerCorrected => "Server corrected",
        LocationSource::Estimated => "Estimated",
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn accepted_point(workout_id: &str, lat: f64, lon: f64, t: i64) -> WorkoutPoint {
        let mut p = WorkoutPoint::new(workout_id, lat, lon, t, LocationSource::PhoneGps);
        p.accepted = true;
        p
    }

    fn accepted_point_with_alt(
        workout_id: &str,
        lat: f64,
        lon: f64,
        alt: f64,
        t: i64,
    ) -> WorkoutPoint {
        let mut p = WorkoutPoint::new(workout_id, lat, lon, t, LocationSource::PhoneGps);
        p.altitude_meters = Some(alt);
        p.accepted = true;
        p
    }

    // ── Route file format selection ──────────────────────────────

    #[test]
    fn gpx_export_forced_when_requested() {
        let input = RouteFormatInput {
            point_count: 5000,
            is_offline: true,
            wants_gpx_export: true,
        };
        assert_eq!(decide_route_format(&input), RouteFileFormat::Gpx);
    }

    #[test]
    fn polyline_only_for_short_routes() {
        let input = RouteFormatInput {
            point_count: 49,
            is_offline: false,
            wants_gpx_export: false,
        };
        assert_eq!(decide_route_format(&input), RouteFileFormat::PolylineOnly);
    }

    #[test]
    fn compressed_binary_when_offline_and_long() {
        let input = RouteFormatInput {
            point_count: 1000,
            is_offline: true,
            wants_gpx_export: false,
        };
        assert_eq!(
            decide_route_format(&input),
            RouteFileFormat::CompressedBinary
        );
    }

    #[test]
    fn gpx_when_online_and_long() {
        let input = RouteFormatInput {
            point_count: 1000,
            is_offline: false,
            wants_gpx_export: false,
        };
        assert_eq!(decide_route_format(&input), RouteFileFormat::Gpx);
    }

    #[test]
    fn polyline_only_takes_priority_over_offline() {
        // Short route + offline → still polyline-only (no file to upload).
        let input = RouteFormatInput {
            point_count: 30,
            is_offline: true,
            wants_gpx_export: false,
        };
        assert_eq!(decide_route_format(&input), RouteFileFormat::PolylineOnly);
    }

    // ── Format metadata ───────────────────────────────────────────

    #[test]
    fn format_extensions() {
        assert_eq!(RouteFileFormat::Gpx.extension(), "gpx");
        assert_eq!(
            RouteFileFormat::CompressedBinary.extension(),
            "croute"
        );
        assert_eq!(RouteFileFormat::PolylineOnly.extension(), "");
    }

    #[test]
    fn only_gpx_and_compressed_require_upload() {
        assert!(RouteFileFormat::Gpx.requires_file_upload());
        assert!(RouteFileFormat::CompressedBinary.requires_file_upload());
        assert!(!RouteFileFormat::PolylineOnly.requires_file_upload());
    }

    #[test]
    fn polyline_only_has_zero_bytes_per_point() {
        assert_eq!(RouteFileFormat::PolylineOnly.bytes_per_point(), 0);
        assert!(RouteFileFormat::Gpx.bytes_per_point() > 0);
        assert!(RouteFileFormat::CompressedBinary.bytes_per_point() > 0);
    }

    // ── Route file path generation ───────────────────────────────

    #[test]
    fn gpx_path_format() {
        let path = generate_route_file_path("user123", "wk-abc", RouteFileFormat::Gpx);
        assert_eq!(path, Some("routes/user123/wk-abc.gpx".to_string()));
    }

    #[test]
    fn compressed_path_format() {
        let path =
            generate_route_file_path("user123", "wk-abc", RouteFileFormat::CompressedBinary);
        assert_eq!(path, Some("routes/user123/wk-abc.croute".to_string()));
    }

    #[test]
    fn polyline_only_has_no_file_path() {
        let path =
            generate_route_file_path("user123", "wk-abc", RouteFileFormat::PolylineOnly);
        assert_eq!(path, None);
    }

    // ── File size estimation ─────────────────────────────────────

    #[test]
    fn gpx_size_scales_with_points() {
        let small = estimate_route_file_size(RouteFileFormat::Gpx, 10);
        let large = estimate_route_file_size(RouteFileFormat::Gpx, 1000);
        assert!(large > small);
        // 1000 points × 180 bytes + 512 overhead = 180 512
        assert_eq!(large, 180_512);
    }

    #[test]
    fn compressed_is_smaller_than_gpx() {
        let gpx = estimate_route_file_size(RouteFileFormat::Gpx, 500);
        let compressed = estimate_route_file_size(RouteFileFormat::CompressedBinary, 500);
        assert!(compressed < gpx);
    }

    #[test]
    fn polyline_only_size_is_zero() {
        assert_eq!(
            estimate_route_file_size(RouteFileFormat::PolylineOnly, 500),
            0
        );
    }

    // ── Wi-Fi preference ──────────────────────────────────────────

    #[test]
    fn wifi_always_ok_when_on_wifi() {
        // Even a huge file uploads immediately if on Wi-Fi.
        assert!(!should_prefer_wifi_for_upload(
            RouteFileFormat::Gpx,
            10_000,
            true
        ));
    }

    #[test]
    fn small_file_ok_over_mobile() {
        assert!(!should_prefer_wifi_for_upload(
            RouteFileFormat::Gpx,
            10,
            false
        ));
    }

    #[test]
    fn large_file_prefers_wifi_off_wifi() {
        // 4000 points × 180 = 720 000 + 512 = 720 512 > 512 KiB
        assert!(should_prefer_wifi_for_upload(
            RouteFileFormat::Gpx,
            4000,
            false
        ));
    }

    #[test]
    fn polyline_only_never_needs_wifi() {
        assert!(!should_prefer_wifi_for_upload(
            RouteFileFormat::PolylineOnly,
            10_000,
            false
        ));
    }

    // ── GPX serialization ─────────────────────────────────────────

    #[test]
    fn gpx_has_valid_xml_envelope() {
        let points = vec![accepted_point("wk1", 40.0, -73.0, 1_000_000_000)];
        let gpx = serialize_gpx("wk1", 1_000_000_000, &points);
        assert!(gpx.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(gpx.contains("<gpx version=\"1.1\""));
        assert!(gpx.contains("xmlns=\"http://www.topografix.com/GPX/1/1\""));
        assert!(gpx.contains("</gpx>"));
    }

    #[test]
    fn gpx_emits_trkpt_with_lat_lon() {
        let points = vec![accepted_point("wk1", 40.0, -73.0, 1_000_000_000)];
        let gpx = serialize_gpx("wk1", 1_000_000_000, &points);
        assert!(gpx.contains("<trkpt lat=\"40.0000000\" lon=\"-73.0000000\">"));
    }

    #[test]
    fn gpx_includes_altitude_when_present() {
        let points =
            vec![accepted_point_with_alt("wk1", 40.0, -73.0, 12.5, 1_000_000_000)];
        let gpx = serialize_gpx("wk1", 1_000_000_000, &points);
        assert!(gpx.contains("<ele>12.5</ele>"));
    }

    #[test]
    fn gpx_omits_altitude_when_absent() {
        let points = vec![accepted_point("wk1", 40.0, -73.0, 1_000_000_000)];
        let gpx = serialize_gpx("wk1", 1_000_000_000, &points);
        assert!(!gpx.contains("<ele>"));
    }

    #[test]
    fn gpx_skips_rejected_points() {
        let mut p1 = accepted_point("wk1", 40.0, -73.0, 1_000_000_000);
        let mut p2 = WorkoutPoint::new("wk1", 40.001, -73.001, 1_001_000_000, LocationSource::PhoneGps);
        p2.accepted = false; // rejected
        let _ = &mut p1;
        let points = vec![p1, p2];
        let gpx = serialize_gpx("wk1", 1_000_000_000, &points);
        // Only one trkpt should appear
        assert_eq!(gpx.matches("<trkpt").count(), 1);
    }

    #[test]
    fn gpx_includes_metadata_with_name_and_time() {
        let points = vec![accepted_point("wk1", 40.0, -73.0, 1_000_000_000)];
        let gpx = serialize_gpx("wk1", 1_000_000_000, &points);
        assert!(gpx.contains("<metadata>"));
        assert!(gpx.contains("<name>wk1</name>"));
        assert!(gpx.contains("<time>"));
    }

    #[test]
    fn gpx_escapes_special_characters_in_workout_id() {
        let points = vec![accepted_point("wk<x>&\"1", 40.0, -73.0, 1_000_000_000)];
        let gpx = serialize_gpx("wk<x>&\"1", 1_000_000_000, &points);
        assert!(gpx.contains("&lt;"));
        assert!(gpx.contains("&gt;"));
        assert!(gpx.contains("&amp;"));
        assert!(gpx.contains("&quot;"));
        // Raw characters must not appear
        assert!(!gpx.contains("wk<x>&\"1</name>"));
    }

    // ── ISO 8601 conversion ───────────────────────────────────────

    #[test]
    fn epoch_zero_is_1970_01_01() {
        assert_eq!(epoch_to_iso8601(0), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn known_timestamp_conversion() {
        // 2024-01-01T00:00:00Z = 1704067200000 ms
        assert_eq!(
            epoch_to_iso8601(1_704_067_200_000),
            "2024-01-01T00:00:00.000Z"
        );
    }

    #[test]
    fn handles_leap_year_2024() {
        // 2024-02-29 is a valid leap day.
        // 2024-02-29T00:00:00Z = 1709164800000 ms
        assert_eq!(
            epoch_to_iso8601(1_709_164_800_000),
            "2024-02-29T00:00:00.000Z"
        );
    }

    #[test]
    fn handles_non_leap_year_2023() {
        // 2023-02-28 is the last day of Feb in a non-leap year.
        // 2023-02-28T00:00:00Z = 1677542400000 ms
        assert_eq!(
            epoch_to_iso8601(1_677_542_400_000),
            "2023-02-28T00:00:00.000Z"
        );
    }

    #[test]
    fn handles_time_of_day() {
        // 2024-01-01T13:45:30.123Z
        // base 1704067200 + 13*3600 + 45*60 + 30 = 1704067200 + 49530
        // = 1704116730 seconds → ×1000 + 123 ms
        let ts = 1_704_067_200_000 + 13 * 3600 * 1000 + 45 * 60 * 1000 + 30 * 1000 + 123;
        assert_eq!(epoch_to_iso8601(ts), "2024-01-01T13:45:30.123Z");
    }

    // ── Route file sync state ─────────────────────────────────────

    #[test]
    fn uploaded_and_failed_are_terminal() {
        assert!(is_route_sync_terminal(RouteFileSyncState::Uploaded));
        assert!(is_route_sync_terminal(RouteFileSyncState::Failed));
    }

    #[test]
    fn local_only_and_uploading_are_not_terminal() {
        assert!(!is_route_sync_terminal(RouteFileSyncState::LocalOnly));
        assert!(!is_route_sync_terminal(RouteFileSyncState::Uploading));
    }

    // ── build_route_file_metadata ─────────────────────────────────

    #[test]
    fn metadata_for_long_online_route() {
        let points: Vec<WorkoutPoint> = (0..200)
            .map(|i| accepted_point("wk1", 40.0 + i as f64 * 0.0001, -73.0, i * 1000))
            .collect();
        let meta = build_route_file_metadata(
            "user1",
            "wk1",
            &points,
            "pixel-8",
            false,
            false,
            1_000_000,
        );
        assert_eq!(meta.format, RouteFileFormat::Gpx);
        assert_eq!(meta.file_path, Some("routes/user1/wk1.gpx".to_string()));
        assert_eq!(meta.point_count, 200);
        assert_eq!(meta.device_source, "pixel-8");
        assert_eq!(meta.sync_state, RouteFileSyncState::LocalOnly);
        assert!(meta.size_bytes > 0);
    }

    #[test]
    fn metadata_for_short_route_is_polyline_only() {
        let points = vec![
            accepted_point("wk1", 40.0, -73.0, 1000),
            accepted_point("wk1", 40.001, -73.001, 2000),
        ];
        let meta = build_route_file_metadata(
            "user1",
            "wk1",
            &points,
            "pixel-8",
            false,
            false,
            1_000_000,
        );
        assert_eq!(meta.format, RouteFileFormat::PolylineOnly);
        assert_eq!(meta.file_path, None);
        assert_eq!(meta.size_bytes, 0);
    }

    #[test]
    fn metadata_counts_only_accepted_points() {
        let mut p1 = accepted_point("wk1", 40.0, -73.0, 1000);
        let _ = &mut p1;
        let mut p2 = WorkoutPoint::new("wk1", 40.001, -73.001, 2000, LocationSource::PhoneGps);
        p2.accepted = false;
        // 49 accepted + 1 rejected → 49 → polyline-only
        let mut points: Vec<WorkoutPoint> = (0..49)
            .map(|i| accepted_point("wk1", 40.0 + i as f64 * 0.0001, -73.0, i * 1000))
            .collect();
        points.push(p2);
        let meta = build_route_file_metadata(
            "user1",
            "wk1",
            &points,
            "pixel-8",
            false,
            false,
            1_000_000,
        );
        assert_eq!(meta.point_count, 49);
        assert_eq!(meta.format, RouteFileFormat::PolylineOnly);
    }

    #[test]
    fn metadata_for_gpx_export_request() {
        let points: Vec<WorkoutPoint> = (0..200)
            .map(|i| accepted_point("wk1", 40.0 + i as f64 * 0.0001, -73.0, i * 1000))
            .collect();
        let meta = build_route_file_metadata(
            "user1",
            "wk1",
            &points,
            "pixel-8",
            true,
            true, // wants_gpx_export
            1_000_000,
        );
        // GPX export forced even though offline
        assert_eq!(meta.format, RouteFileFormat::Gpx);
        assert_eq!(meta.file_path, Some("routes/user1/wk1.gpx".to_string()));
    }

    // ── build_route_summary ───────────────────────────────────────

    fn sample_summary() -> WorkoutSummary {
        WorkoutSummary {
            workout_id: "wk1".to_string(),
            user_id: "user1".to_string(),
            activity_type: ActivityType::Run,
            started_at: 1_000_000,
            ended_at: 1_900_000,
            total_duration_ms: 900_000,
            moving_ms: 800_000,
            paused_ms: 100_000,
            distance_meters: 5000.0,
            average_pace_sec_per_km: 300.0,
            average_speed_mps: 3.33,
            max_speed_mps: 5.0,
            calories_estimated: 350.0,
            calorie_estimate_method: "hr".to_string(),
            step_count: 6500,
            average_cadence_spm: 150.0,
            average_heart_rate_bpm: Some(140.0),
            max_heart_rate_bpm: Some(165),
            min_heart_rate_bpm: Some(110),
            elevation_gain_meters: 50.0,
            elevation_loss_meters: 48.0,
            splits: vec![],
            start_latitude: Some(40.0),
            start_longitude: Some(-73.0),
            end_latitude: Some(40.05),
            end_longitude: Some(-73.05),
            encoded_polyline: Some("_p~iF".to_string()),
            goal_result: None,
            has_flagged_segments: false,
            validation_warnings: vec![],
            is_blocked: false,
            block_reasons: vec![],
        }
    }

    #[test]
    fn route_summary_has_all_spec_fields() {
        let summary = sample_summary();
        let route_file = RouteFileMetadata {
            file_path: Some("routes/user1/wk1.gpx".to_string()),
            format: RouteFileFormat::Gpx,
            size_bytes: 180_512,
            point_count: 1000,
            device_source: "pixel-8".to_string(),
            sync_state: RouteFileSyncState::LocalOnly,
            created_at: 1_900_000,
            updated_at: 1_900_000,
        };
        let rs = build_route_summary(&summary, &route_file, "pixel-8", 1_900_000);

        assert_eq!(rs.workout_id, "wk1");
        assert_eq!(rs.user_id, "user1");
        assert_eq!(rs.activity_type, ActivityType::Run);
        assert_eq!(rs.started_at, 1_000_000);
        assert_eq!(rs.ended_at, 1_900_000);
        assert_eq!(rs.duration_ms, 900_000);
        assert_eq!(rs.distance_meters, 5000.0);
        assert_eq!(rs.average_pace_sec_per_km, 300.0);
        assert_eq!(rs.average_speed_mps, 3.33);
        assert_eq!(rs.steps, 6500);
        assert_eq!(rs.average_heart_rate_bpm, Some(140.0));
        assert_eq!(rs.max_heart_rate_bpm, Some(165));
        assert_eq!(rs.calories, 350.0);
        assert_eq!(rs.calorie_method, "hr");
        assert_eq!(rs.start_latitude, Some(40.0));
        assert_eq!(rs.start_longitude, Some(-73.0));
        assert_eq!(rs.end_latitude, Some(40.05));
        assert_eq!(rs.end_longitude, Some(-73.05));
        assert_eq!(rs.encoded_polyline, Some("_p~iF".to_string()));
        assert_eq!(rs.route_file_path, Some("routes/user1/wk1.gpx".to_string()));
        assert_eq!(rs.route_file_format, RouteFileFormat::Gpx);
        assert_eq!(rs.sync_state, crate::models::SyncQueueState::Pending);
        assert_eq!(rs.device_source, "pixel-8");
        assert_eq!(rs.created_at, 1_900_000);
        assert_eq!(rs.updated_at, 1_900_000);
    }

    #[test]
    fn route_summary_starts_pending() {
        let summary = sample_summary();
        let route_file = RouteFileMetadata {
            file_path: None,
            format: RouteFileFormat::PolylineOnly,
            size_bytes: 0,
            point_count: 10,
            device_source: "pixel-8".to_string(),
            sync_state: RouteFileSyncState::LocalOnly,
            created_at: 1_900_000,
            updated_at: 1_900_000,
        };
        let rs = build_route_summary(&summary, &route_file, "pixel-8", 1_900_000);
        assert_eq!(rs.sync_state, crate::models::SyncQueueState::Pending);
    }

    // ── validate_route_summary ────────────────────────────────────

    #[test]
    fn valid_summary_passes() {
        let summary = sample_summary();
        let route_file = RouteFileMetadata {
            file_path: Some("routes/user1/wk1.gpx".to_string()),
            format: RouteFileFormat::Gpx,
            size_bytes: 1000,
            point_count: 100,
            device_source: "pixel-8".to_string(),
            sync_state: RouteFileSyncState::LocalOnly,
            created_at: 1_900_000,
            updated_at: 1_900_000,
        };
        let rs = build_route_summary(&summary, &route_file, "pixel-8", 1_900_000);
        assert!(validate_route_summary(&rs).is_ok());
    }

    #[test]
    fn empty_workout_id_rejected() {
        let mut rs = build_route_summary(
            &sample_summary(),
            &RouteFileMetadata {
                file_path: None,
                format: RouteFileFormat::PolylineOnly,
                size_bytes: 0,
                point_count: 10,
                device_source: "pixel-8".to_string(),
                sync_state: RouteFileSyncState::LocalOnly,
                created_at: 1_900_000,
                updated_at: 1_900_000,
            },
            "pixel-8",
            1_900_000,
        );
        rs.workout_id = "".to_string();
        let result = validate_route_summary(&rs);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("workout_id"));
    }

    #[test]
    fn empty_user_id_rejected() {
        let mut rs = build_route_summary(
            &sample_summary(),
            &RouteFileMetadata {
                file_path: None,
                format: RouteFileFormat::PolylineOnly,
                size_bytes: 0,
                point_count: 10,
                device_source: "pixel-8".to_string(),
                sync_state: RouteFileSyncState::LocalOnly,
                created_at: 1_900_000,
                updated_at: 1_900_000,
            },
            "pixel-8",
            1_900_000,
        );
        rs.user_id = "".to_string();
        let result = validate_route_summary(&rs);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("user_id"));
    }

    #[test]
    fn route_path_must_match_user_prefix() {
        let summary = sample_summary();
        let mut rs = build_route_summary(
            &summary,
            &RouteFileMetadata {
                file_path: Some("routes/other_user/wk1.gpx".to_string()),
                format: RouteFileFormat::Gpx,
                size_bytes: 1000,
                point_count: 100,
                device_source: "pixel-8".to_string(),
                sync_state: RouteFileSyncState::LocalOnly,
                created_at: 1_900_000,
                updated_at: 1_900_000,
            },
            "pixel-8",
            1_900_000,
        );
        // user_id is "user1" but path says "other_user" → mismatch
        let result = validate_route_summary(&rs);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("prefix"));
        let _ = &mut rs; // silence unused mut
    }

    #[test]
    fn oversized_polyline_rejected() {
        let mut rs = build_route_summary(
            &sample_summary(),
            &RouteFileMetadata {
                file_path: None,
                format: RouteFileFormat::PolylineOnly,
                size_bytes: 0,
                point_count: 10,
                device_source: "pixel-8".to_string(),
                sync_state: RouteFileSyncState::LocalOnly,
                created_at: 1_900_000,
                updated_at: 1_900_000,
            },
            "pixel-8",
            1_900_000,
        );
        // 2 MiB polyline > 1 MiB Firestore limit
        rs.encoded_polyline = Some("A".repeat(2 * 1_048_576));
        let result = validate_route_summary(&rs);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1 MiB"));
    }

    // ── dominant_location_source ──────────────────────────────────

    #[test]
    fn dominant_source_phone_when_majority() {
        let points: Vec<WorkoutPoint> = (0..10)
            .map(|i| accepted_point("wk1", 40.0, -73.0, i * 1000))
            .collect();
        assert_eq!(
            dominant_location_source(&points),
            Some(LocationSource::PhoneGps)
        );
    }

    #[test]
    fn dominant_source_wear_os_when_majority() {
        let mut points: Vec<WorkoutPoint> = (0..3)
            .map(|i| accepted_point("wk1", 40.0, -73.0, i * 1000))
            .collect();
        let mut watch_pt =
            WorkoutPoint::new("wk1", 40.0, -73.0, 4000, LocationSource::WearOs);
        watch_pt.accepted = true;
        points.push(watch_pt);
        // 3 phone + 1 watch → phone still dominant
        assert_eq!(
            dominant_location_source(&points),
            Some(LocationSource::PhoneGps)
        );

        // Add 3 more watch points → watch dominant
        for i in 0..3 {
            let mut p =
                WorkoutPoint::new("wk1", 40.0, -73.0, 5000 + i * 1000, LocationSource::WearOs);
            p.accepted = true;
            points.push(p);
        }
        assert_eq!(
            dominant_location_source(&points),
            Some(LocationSource::WearOs)
        );
    }

    #[test]
    fn dominant_source_none_when_no_accepted_points() {
        let points = vec![];
        assert_eq!(dominant_location_source(&points), None);
    }

    #[test]
    fn dominant_source_none_when_all_rejected() {
        let mut p = WorkoutPoint::new("wk1", 40.0, -73.0, 1000, LocationSource::PhoneGps);
        p.accepted = false;
        let points = vec![p];
        assert_eq!(dominant_location_source(&points), None);
    }

    // ── location_source_label ─────────────────────────────────────

    #[test]
    fn labels_are_human_readable() {
        assert_eq!(location_source_label(LocationSource::PhoneGps), "Phone GPS");
        assert_eq!(location_source_label(LocationSource::WearOs), "Wear OS");
        assert_eq!(
            location_source_label(LocationSource::HealthConnect),
            "Health Connect"
        );
    }

    // ── GPX round-trip with multiple points ───────────────────────

    #[test]
    fn gpx_multiple_points_all_emitted() {
        let points: Vec<WorkoutPoint> = (0..5)
            .map(|i| accepted_point_with_alt("wk1", 40.0 + i as f64 * 0.001, -73.0, 10.0 + i as f64, i * 1000))
            .collect();
        let gpx = serialize_gpx("wk1", 0, &points);
        // 5 trkpt elements
        assert_eq!(gpx.matches("<trkpt").count(), 5);
        // Each has an ele
        assert_eq!(gpx.matches("<ele>").count(), 5);
        // Properly closed
        assert!(gpx.contains("</trkseg>"));
        assert!(gpx.contains("</trk>"));
        assert!(gpx.ends_with("</gpx>\n"));
    }
}

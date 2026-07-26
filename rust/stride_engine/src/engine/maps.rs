//! Maps and location services (spec section 5).
//!
//! The spec lists a large set of UI-level map features (markers, controls,
//! views, rotation, recenter). Most of those are handled by the Flutter/
//! Dart UI layer using `flutter_map` or `google_maps_flutter`. The Rust
//! engine's role is the **pure decision logic** behind the map subsystem:
//!
//!   - **Tile cache management** — how many tiles to cache, when to evict
//!     (LRU), the maximum cache size, and whether a tile cache entry is
//!     stale.
//!   - **Offline region management** — a region is a bounding box + zoom
//!     range. The engine computes the tile count for a region (so the
//!     download size can be estimated *before* the user taps "download"),
//!     builds a region manifest, and decides whether a downloaded region
//!     is still needed or can be deleted.
//!   - **Storage-availability checks** — before starting a region
//!     download, the engine checks whether the estimated tile cache size
//!     fits within the user's storage budget and the device's free space.
//!   - **Map attribution** — the legal text the map UI must display for
//!     each tile provider (OSM, satellite, etc.).
//!   - **GPS accuracy indicator** — converts a horizontal accuracy in
//!     meters to a 5-level quality bucket (excellent / good / fair / poor
//!     / unavailable) for the on-map accuracy dot colour.
//!   - **Route preview** — selects the right polyline simplification
//!     precision based on the current map zoom level (high zoom → full
//!     detail, low zoom → aggressive simplification).
//!   - **Map view type** — tracks which tile layer the user is viewing
//!     (standard / satellite / hybrid / terrain) so attribution and
//!     licensing can be enforced per-provider.

use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────────────
// Map view type & tile providers
// ──────────────────────────────────────────────────────────────────────

/// Which tile layer the user is currently viewing. Each type may come
/// from a different provider with different licensing and attribution
/// requirements (see [`attribution_for_provider`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapViewType {
    /// Standard street map (typically OpenStreetMap tiles).
    Standard,
    /// Satellite imagery (requires a licensed satellite-tile provider;
    /// OSM does not provide unrestricted production satellite tiles).
    Satellite,
    /// Satellite imagery with street labels overlaid.
    Hybrid,
    /// Topographic/terrain map with elevation contours.
    Terrain,
}

impl Default for MapViewType {
    fn default() -> Self {
        MapViewType::Standard
    }
}

/// Which tile provider supplies the tiles for a given map view. This
/// determines the attribution text and licensing constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TileProvider {
    /// OpenStreetMap standard tiles (open license, attribution required).
    OpenStreetMap,
    /// Licensed satellite-tile provider (requires API key + commercial
    /// license; the spec says the app "will need a provider with
    /// appropriate licensing and API terms for satellite tiles").
    SatelliteProvider,
    /// ESRI tiles (free tier available for some products).
    Esri,
    /// Custom/self-hosted tile server.
    Custom,
}

/// Returns the tile provider for a given map view type. The Dart layer
/// uses this to build the correct tile URL template and display the
/// correct attribution.
pub fn provider_for_view(view: MapViewType) -> TileProvider {
    match view {
        MapViewType::Standard => TileProvider::OpenStreetMap,
        MapViewType::Satellite => TileProvider::SatelliteProvider,
        MapViewType::Hybrid => TileProvider::SatelliteProvider,
        MapViewType::Terrain => TileProvider::Esri,
    }
}

// ──────────────────────────────────────────────────────────────────────
// Map attribution
// ──────────────────────────────────────────────────────────────────────

/// Returns the legal attribution text the map UI must display for a
/// given tile provider. Attribution is **required** by the license
/// terms of every tile provider; the app must show this text in the
/// map view (typically a small text overlay in the corner).
///
/// The text is plain (not HTML) so the Dart layer can render it in any
/// widget (Text, RichText, etc.) without needing an HTML parser.
pub fn attribution_for_provider(provider: TileProvider) -> &'static str {
    match provider {
        TileProvider::OpenStreetMap => {
            "© OpenStreetMap contributors"
        }
        TileProvider::SatelliteProvider => {
            "Satellite imagery © [Satellite Provider]. All rights reserved."
        }
        TileProvider::Esri => {
            "© Esri, HERE, Garmin, and the GIS user community"
        }
        TileProvider::Custom => {
            "Map tiles © [Custom Provider]"
        }
    }
}

/// Convenience: attribution for the provider associated with a given
/// map view type.
pub fn attribution_for_view(view: MapViewType) -> &'static str {
    attribution_for_provider(provider_for_view(view))
}

/// The URL template the Dart tile layer should use for a given
/// provider. The `{z}`, `{x}`, `{y}` placeholders are standard slippy-
/// map tile coordinates (zoom level, column, row).
///
/// For the satellite provider, the template includes a placeholder
/// for the API key (`{api_key}`) that the Dart layer must fill in
/// from the app's configuration. OSM and ESRI URLs are public.
pub fn tile_url_template(provider: TileProvider) -> &'static str {
    match provider {
        TileProvider::OpenStreetMap => {
            "https://tile.openstreetmap.org/{z}/{x}/{y}.png"
        }
        TileProvider::SatelliteProvider => {
            "https://tiles.satellite-provider.com/{z}/{x}/{y}.jpg?api_key={api_key}"
        }
        TileProvider::Esri => {
            "https://server.arcgisonline.com/ArcGIS/rest/services/World_Topo_Map/MapServer/tile/{z}/{y}/{x}"
        }
        TileProvider::Custom => {
            "https://tiles.example.com/{z}/{x}/{y}.png"
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// GPS accuracy indicator
// ──────────────────────────────────────────────────────────────────────

/// Display-quality bucket for the on-map GPS accuracy indicator dot.
/// The dot colour changes with accuracy so the user can tell at a
/// glance whether the route is being recorded accurately.
///
/// This is *separate* from the engine's internal [`GpsQualityState`]
/// (in `models::events`), which is used for coaching nudges. This
/// enum is purely for the map UI's accuracy dot colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpsAccuracyLevel {
    /// <5 m — excellent GPS fix, green dot.
    Excellent,
    /// 5–10 m — good fix, light green dot.
    Good,
    /// 10–20 m — fair fix, yellow dot.
    Fair,
    /// 20–50 m — poor fix, orange dot. Route may be imprecise.
    Poor,
    /// >50 m or no fix — red dot / "no GPS" indicator.
    Unavailable,
}

/// Classifies a horizontal accuracy (in meters) into a display
/// quality level for the on-map accuracy indicator. `None` (no fix)
/// maps to [`GpsAccuracyLevel::Unavailable`].
///
/// Thresholds are based on typical consumer GPS performance:
///   - Phone GPS in open sky: ~3–5 m
///   - Phone GPS in urban canyon: ~10–20 m
///   - Phone GPS indoors: >20 m or no fix
pub fn classify_gps_accuracy(accuracy_meters: Option<f64>) -> GpsAccuracyLevel {
    let acc = match accuracy_meters {
        Some(a) if a >= 0.0 => a,
        _ => return GpsAccuracyLevel::Unavailable,
    };
    if acc < 5.0 {
        GpsAccuracyLevel::Excellent
    } else if acc < 10.0 {
        GpsAccuracyLevel::Good
    } else if acc < 20.0 {
        GpsAccuracyLevel::Fair
    } else if acc < 50.0 {
        GpsAccuracyLevel::Poor
    } else {
        GpsAccuracyLevel::Unavailable
    }
}

/// Returns a human-readable description of a GPS accuracy level,
/// for the accuracy indicator tooltip or a settings screen.
pub fn gps_accuracy_description(level: GpsAccuracyLevel) -> &'static str {
    match level {
        GpsAccuracyLevel::Excellent => "Excellent GPS (< 5 m)",
        GpsAccuracyLevel::Good => "Good GPS (5–10 m)",
        GpsAccuracyLevel::Fair => "Fair GPS (10–20 m)",
        GpsAccuracyLevel::Poor => "Poor GPS (20–50 m)",
        GpsAccuracyLevel::Unavailable => "No GPS (> 50 m)",
    }
}

/// Returns the recommended colour (as a hex string) for the on-map
/// accuracy indicator dot. The Dart layer converts this to a `Color`.
pub fn gps_accuracy_color(level: GpsAccuracyLevel) -> &'static str {
    match level {
        GpsAccuracyLevel::Excellent => "#4CAF50", // green
        GpsAccuracyLevel::Good => "#8BC34A",      // light green
        GpsAccuracyLevel::Fair => "#FFEB3B",     // yellow
        GpsAccuracyLevel::Poor => "#FF9800",     // orange
        GpsAccuracyLevel::Unavailable => "#F44336", // red
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tile coordinates & tile count estimation
// ──────────────────────────────────────────────────────────────────────

/// A single slippy-map tile coordinate (zoom, column, row). This is the
/// standard XYZ tile addressing scheme used by OSM, Google Maps, and
/// most tile providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    pub z: u8,
    pub x: u32,
    pub y: u32,
}

/// Converts a (latitude, longitude) pair to a tile coordinate at a
/// given zoom level. Uses the standard Web Mercator projection.
///
/// Returns `None` if the latitude is outside the valid range
/// (\u00b1 85.05\u00b0, beyond which Web Mercator is undefined).
pub fn lat_lon_to_tile(lat: f64, lon: f64, zoom: u8) -> Option<TileCoord> {
    if !(-85.0511..=85.0511).contains(&lat) {
        return None;
    }
    if !(-180.0..=180.0).contains(&lon) {
        return None;
    }

    let n = 2f64.powi(zoom as i32);
    let max = n as u32 - 1;
    let x = ((lon + 180.0) / 360.0 * n).floor() as u32;
    let y = lat_to_tile_y(lat, zoom).min(max);

    Some(TileCoord {
        z: zoom,
        x: x.min(max),
        y,
    })
}

/// Counts the total number of tiles needed to cover a bounding box
/// (min lat, min lon, max lat, max lon) across a range of zoom levels
/// (min_zoom to max_zoom inclusive).
///
/// This is the key function for offline region downloads: the Dart
/// layer calls this *before* starting a download to estimate the total
/// tile count, which determines the download size and time.
pub fn count_tiles_in_region(
    min_lat: f64,
    min_lon: f64,
    max_lat: f64,
    max_lon: f64,
    min_zoom: u8,
    max_zoom: u8,
) -> u64 {
    if min_zoom > max_zoom {
        return 0;
    }

    let mut total: u64 = 0;
    for zoom in min_zoom..=max_zoom {
        let tiles_wide = tiles_across_range(min_lon, max_lon, zoom);
        let tiles_high = tiles_across_range_lat(min_lat, max_lat, zoom);
        total += tiles_wide as u64 * tiles_high as u64;
    }
    total
}

/// Number of tiles the longitude range [min_lon, max_lon] spans at a
/// given zoom level.
fn tiles_across_range(min_lon: f64, max_lon: f64, zoom: u8) -> u32 {
    let n = 2f64.powi(zoom as i32) as u32;
    let min_x = lon_to_tile_x(min_lon, zoom);
    let max_x = lon_to_tile_x(max_lon, zoom);
    // +1 because both endpoints are inclusive (the tile containing
    // max_lon must also be downloaded).
    (max_x - min_x + 1).min(n)
}

/// Number of tiles the latitude range [min_lat, max_lat] spans at a
/// given zoom level.
fn tiles_across_range_lat(min_lat: f64, max_lat: f64, zoom: u8) -> u32 {
    let n = 2f64.powi(zoom as i32) as u32;
    let min_y = lat_to_tile_y(max_lat, zoom); // higher lat → lower y
    let max_y = lat_to_tile_y(min_lat, zoom); // lower lat → higher y
    (max_y - min_y + 1).min(n)
}

/// Converts a longitude to a tile X column at a given zoom level.
fn lon_to_tile_x(lon: f64, zoom: u8) -> u32 {
    let n = 2f64.powi(zoom as i32);
    ((lon + 180.0) / 360.0 * n).floor() as u32
}

/// Converts a latitude to a tile Y row at a given zoom level.
fn lat_to_tile_y(lat: f64, zoom: u8) -> u32 {
    let n = 2f64.powi(zoom as i32);
    let lat_rad = lat.to_radians();
    // Standard Web Mercator projection:
    //   y = (1 - ln(tan(π/4 + lat/2)) / π) / 2 * n
    // Equivalently, asinh(tan(lat)) can be used, but the tan(π/4 + lat/2)
    // form is the most widely referenced standard.
    ((1.0 - (std::f64::consts::PI / 4.0 + lat_rad / 2.0).tan().ln() / std::f64::consts::PI)
        / 2.0
        * n)
        .floor() as u32
}

// ──────────────────────────────────────────────────────────────────────
// Offline region manifest
// ──────────────────────────────────────────────────────────────────────

/// A saved offline region the user has downloaded for use without
/// network connectivity. This is the manifest stored in the local
/// SQLite database; the actual tile files live in the on-device tile
/// cache directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineRegion {
    /// Unique ID for this region (UUID, generated at download time).
    pub region_id: String,
    /// Human-readable name the user gave the region (e.g. "Home
    /// neighborhood", "Sunday run route").
    pub name: String,
    /// Bounding box: minimum latitude.
    pub min_lat: f64,
    /// Bounding box: minimum longitude.
    pub min_lon: f64,
    /// Bounding box: maximum latitude.
    pub max_lat: f64,
    /// Bounding box: maximum longitude.
    pub max_lon: f64,
    /// Minimum zoom level included in the download (e.g. 10).
    pub min_zoom: u8,
    /// Maximum zoom level included in the download (e.g. 16).
    pub max_zoom: u8,
    /// Total number of tiles in this region (computed by
    /// [`count_tiles_in_region`] at download time).
    pub tile_count: u64,
    /// Estimated total size in bytes (tile_count × avg_tile_size).
    pub estimated_size_bytes: u64,
    /// Which tile provider the tiles came from (determines
    /// attribution for offline tiles too).
    pub provider: TileProvider,
    /// Epoch milliseconds when the region was downloaded.
    pub downloaded_at: i64,
    /// Epoch milliseconds when the region was last accessed (i.e.
    /// when the user last viewed a map in this region). Used by
    /// the LRU eviction policy.
    pub last_accessed_at: i64,
}

/// Average size (in bytes) of a single map tile. PNG tiles at zoom
/// 13–16 average ~10–15 KiB; we use 12 KiB as a conservative estimate.
const AVG_TILE_SIZE_BYTES: u64 = 12 * 1024;

/// Maximum total size of all offline regions combined. 500 MiB is a
/// reasonable budget for a fitness app's map cache — enough for
/// several city-scale regions at high zoom without overwhelming the
/// device's storage.
const MAX_TOTAL_OFFLINE_CACHE_BYTES: u64 = 500 * 1024 * 1024;

/// Maximum number of offline regions a user can save. Prevents the
/// region list from growing unbounded.
const MAX_OFFLINE_REGIONS: usize = 20;

/// Builds an [`OfflineRegion`] manifest from the user's download
/// request parameters. Computes the tile count and estimated size.
///
/// This is the pure logic called at the start of a region download;
/// the actual tile fetching happens on the Dart side (the engine
/// never does network I/O).
pub fn build_offline_region(
    region_id: &str,
    name: &str,
    min_lat: f64,
    min_lon: f64,
    max_lat: f64,
    max_lon: f64,
    min_zoom: u8,
    max_zoom: u8,
    provider: TileProvider,
    now_ms: i64,
) -> OfflineRegion {
    let tile_count = count_tiles_in_region(min_lat, min_lon, max_lat, max_lon, min_zoom, max_zoom);
    let estimated_size_bytes = tile_count * AVG_TILE_SIZE_BYTES;

    OfflineRegion {
        region_id: region_id.to_string(),
        name: name.to_string(),
        min_lat,
        min_lon,
        max_lat,
        max_lon,
        min_zoom,
        max_zoom,
        tile_count,
        estimated_size_bytes,
        provider,
        downloaded_at: now_ms,
        last_accessed_at: now_ms,
    }
}

// ──────────────────────────────────────────────────────────────────────
// Storage-availability checks
// ──────────────────────────────────────────────────────────────────────

/// Checks whether the device has enough free storage to download a
/// new offline region, given the current total cache size and the
/// device's available bytes.
///
/// Returns `Ok(())` if the download fits within both the device's free
/// space and the app's total cache budget. Returns `Err` with a
/// descriptive message otherwise.
///
/// The `current_total_cache_bytes` is the sum of all existing
/// offline regions' `estimated_size_bytes`; the engine tracks this
/// so it can enforce the global cache limit.
pub fn check_storage_availability(
    new_region_size_bytes: u64,
    current_total_cache_bytes: u64,
    device_free_bytes: u64,
) -> Result<(), String> {
    // Check device free space (with a 10% safety margin for non-tile
    // storage the app might need during the download).
    let needed_with_margin = new_region_size_bytes + new_region_size_bytes / 10;
    if needed_with_margin > device_free_bytes {
        return Err(format!(
            "Not enough free space: need ~{} bytes (with 10% margin), have {} bytes",
            needed_with_margin, device_free_bytes
        ));
    }

    // Check total cache budget
    let new_total = current_total_cache_bytes + new_region_size_bytes;
    if new_total > MAX_TOTAL_OFFLINE_CACHE_BYTES {
        let over = new_total - MAX_TOTAL_OFFLINE_CACHE_BYTES;
        return Err(format!(
            "Offline cache limit exceeded: would be {} bytes, limit is {} bytes (over by {} bytes). Delete some regions to free space.",
            new_total, MAX_TOTAL_OFFLINE_CACHE_BYTES, over
        ));
    }

    Ok(())
}

/// Checks whether the user can save a new offline region, given the
/// current number of saved regions. The limit prevents the region
/// list from growing unbounded.
pub fn can_add_region(current_region_count: usize) -> bool {
    current_region_count < MAX_OFFLINE_REGIONS
}

// ──────────────────────────────────────────────────────────────────────
// Tile cache eviction (LRU)
// ──────────────────────────────────────────────────────────────────────

/// Decides whether a region should be evicted to make space for a new
/// download. Uses a least-recently-used policy: the region with the
/// oldest `last_accessed_at` is the eviction candidate.
///
/// The Dart layer calls this when a new download would exceed the cache
/// budget ([`check_storage_availability`] fails). It passes the list
/// of existing regions; the engine picks the LRU candidate whose
/// deletion would free enough space for the new region.
///
/// Returns the index of the region to evict, or `None` if no single
/// region's deletion would free enough space (in which case the Dart
/// layer should offer the user a multi-select to delete several).
pub fn select_eviction_candidate(
    regions: &[OfflineRegion],
    bytes_needed: u64,
) -> Option<usize> {
    if regions.is_empty() {
        return None;
    }

    // Find the LRU region whose size >= bytes_needed.
    // If none is large enough, return the single largest LRU region
    // (it frees the most space, which is the best we can do with a
    // single eviction).
    let mut lru_index = 0;
    let mut lru_time = regions[0].last_accessed_at;
    let mut largest_index = 0;
    let mut largest_size = regions[0].estimated_size_bytes;

    for (i, r) in regions.iter().enumerate() {
        // Track the overall LRU
        if r.last_accessed_at < lru_time {
            lru_time = r.last_accessed_at;
            lru_index = i;
        }
        // Track the LRU that's large enough
        if r.estimated_size_bytes >= bytes_needed && r.last_accessed_at <= lru_time {
            lru_index = i;
            lru_time = r.last_accessed_at;
        }
        // Track the largest
        if r.estimated_size_bytes > largest_size {
            largest_size = r.estimated_size_bytes;
            largest_index = i;
        }
    }

    // If the LRU region is large enough, evict it
    if regions[lru_index].estimated_size_bytes >= bytes_needed {
        return Some(lru_index);
    }
    // Otherwise, return the largest (best single eviction)
    Some(largest_index)
}

/// Decides whether a downloaded region is stale enough to be
/// auto-deleted. A region is "stale" if it hasn't been accessed in
/// over 90 days (the user likely doesn't need it anymore).
const STALE_REGION_AGE_MS: i64 = 90 * 24 * 60 * 60 * 1000; // 90 days

pub fn is_region_stale(region: &OfflineRegion, now_ms: i64) -> bool {
    now_ms - region.last_accessed_at > STALE_REGION_AGE_MS
}

/// Updates the `last_accessed_at` timestamp on a region (called when
/// the user views a map in that region). Returns the updated region.
pub fn touch_region(mut region: OfflineRegion, now_ms: i64) -> OfflineRegion {
    region.last_accessed_at = now_ms;
    region
}

// ──────────────────────────────────────────────────────────────────────
// Route preview — zoom-based simplification
// ──────────────────────────────────────────────────────────────────────

/// Selects the Douglas-Peucker simplification epsilon (in degrees)
/// appropriate for the current map zoom level. Higher zoom (more
/// detail visible) uses a smaller epsilon (less simplification);
/// lower zoom (zoomed out) uses a larger epsilon (aggressive
/// simplification to avoid drawing thousands of micro-points).
///
/// The returned epsilon is in degrees of latitude/longitude, matching
/// the `simplify_epsilon_degrees` field in `RouteBuilderConfig`.
pub fn simplification_epsilon_for_zoom(zoom: u8) -> f64 {
    match zoom {
        0..=5 => 0.001,     // continent view: ~100m
        6..=9 => 0.0005,    // country/region: ~50m
        10..=13 => 0.0001,  // city: ~10m
        14..=17 => 0.00003, // street: ~3m (same as default)
        _ => 0.00001,       // building level: ~1m
    }
}

/// Whether the current zoom level is high enough that the user can
/// see individual GPS points on the route. At zoom >= 15, points are
/// typically 2–3 pixels apart and the full-resolution route should be
/// shown; at lower zoom, the simplified polyline is sufficient.
pub fn should_show_full_resolution_route(zoom: u8) -> bool {
    zoom >= 15
}

// ──────────────────────────────────────────────────────────────────────
// Map controls decision logic
// ──────────────────────────────────────────────────────────────────────

/// Whether the "recenter" button should be visible. The recenter
/// control re-centers the map on the user's current location. It
/// should only be visible when the map has been panned away from the
/// user's location (i.e., the user is not following their current
/// position).
///
/// `distance_meters` is the distance between the map's center and
/// the user's current location. If it's more than ~50 m, the recenter
/// button appears.
pub fn should_show_recenter_button(distance_from_center_meters: f64) -> bool {
    distance_from_center_meters > 50.0
}

/// Whether map rotation should follow the user's heading (compass
/// mode) or stay north-up. The engine recommends follow-heading mode
/// when the user is moving fast enough that heading is meaningful
/// (walking/running) and not when stationary.
///
/// `speed_mps` is the user's current speed in m/s. `is_heading_valid`
/// is whether the device has a reliable compass bearing.
pub fn should_rotate_with_heading(speed_mps: f64, is_heading_valid: bool) -> bool {
    // ~1.4 m/s ≈ 5 km/h, a slow walk — below this, heading is
    // too jittery to be useful for map rotation.
    is_heading_valid && speed_mps > 1.4
}

// ──────────────────────────────────────────────────────────────────────
// Saved routes
// ──────────────────────────────────────────────────────────────────────

/// A saved route the user can replay or use as a route plan. This is
/// distinct from a *workout* (a recorded activity) — a saved route is
/// a path the user has bookmarked for future reference, potentially
/// to follow as a planned run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedRoute {
    /// Unique ID for this saved route.
    pub route_id: String,
    /// User-assigned name (e.g. "Morning 5K loop").
    pub name: String,
    /// User ID who saved the route.
    pub user_id: String,
    /// Encoded polyline of the route (Google-style, for compact
    /// storage and fast map preview).
    pub encoded_polyline: String,
    /// Start latitude (for the route list's preview thumbnail).
    pub start_lat: f64,
    /// Start longitude.
    pub start_lon: f64,
    /// Total route distance in meters.
    pub distance_meters: f64,
    /// Epoch ms when the route was saved.
    pub saved_at: i64,
}

/// Validates that a saved route is safe to persist: name non-empty,
/// polyline non-empty, distance non-negative.
pub fn validate_saved_route(route: &SavedRoute) -> Result<(), String> {
    if route.name.is_empty() {
        return Err("route name is empty".to_string());
    }
    if route.encoded_polyline.is_empty() {
        return Err("encoded polyline is empty".to_string());
    }
    if route.distance_meters < 0.0 {
        return Err(format!("distance is negative: {}", route.distance_meters));
    }
    if route.user_id.is_empty() {
        return Err("user_id is empty".to_string());
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────
// Tile cache key
// ──────────────────────────────────────────────────────────────────────

/// Builds the cache key for a tile in the on-device tile cache.
///
/// Format: `tiles/{provider}/{z}/{x}/{y}` (no extension — the Dart
/// layer adds `.png` or `.jpg` based on the provider). This directory
/// structure lets the Dart layer check tile existence with a simple
/// file-exists call and enumerate all cached tiles for a provider.
pub fn tile_cache_key(provider: TileProvider, tile: TileCoord) -> String {
    format!(
        "tiles/{provider}/{z}/{x}/{y}",
        provider = tile_provider_slug(provider),
        z = tile.z,
        x = tile.x,
        y = tile.y
    )
}

/// Short slug for a tile provider, used in cache paths. Lowercase,
/// no spaces, suitable for filesystem directory names.
pub fn tile_provider_slug(provider: TileProvider) -> &'static str {
    match provider {
        TileProvider::OpenStreetMap => "osm",
        TileProvider::SatelliteProvider => "satellite",
        TileProvider::Esri => "esri",
        TileProvider::Custom => "custom",
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Map view type & providers ──────────────────────────────────

    #[test]
    fn standard_view_uses_osm() {
        assert_eq!(
            provider_for_view(MapViewType::Standard),
            TileProvider::OpenStreetMap
        );
    }

    #[test]
    fn satellite_view_uses_satellite_provider() {
        assert_eq!(
            provider_for_view(MapViewType::Satellite),
            TileProvider::SatelliteProvider
        );
    }

    #[test]
    fn hybrid_view_uses_satellite_provider() {
        assert_eq!(
            provider_for_view(MapViewType::Hybrid),
            TileProvider::SatelliteProvider
        );
    }

    #[test]
    fn terrain_view_uses_esri() {
        assert_eq!(
            provider_for_view(MapViewType::Terrain),
            TileProvider::Esri
        );
    }

    #[test]
    fn default_view_is_standard() {
        assert_eq!(MapViewType::default(), MapViewType::Standard);
    }

    // ── Attribution ────────────────────────────────────────────────

    #[test]
    fn osm_attribution_mentions_contributors() {
        let attr = attribution_for_provider(TileProvider::OpenStreetMap);
        assert!(attr.contains("OpenStreetMap"));
        assert!(attr.contains("contributors"));
    }

    #[test]
    fn satellite_attribution_mentions_rights() {
        let attr = attribution_for_provider(TileProvider::SatelliteProvider);
        assert!(attr.contains("Satellite"));
        assert!(attr.contains("rights reserved"));
    }

    #[test]
    fn esri_attribution_mentions_esri() {
        let attr = attribution_for_provider(TileProvider::Esri);
        assert!(attr.contains("Esri"));
    }

    #[test]
    fn attribution_for_view_matches_provider() {
        // Standard view → OSM attribution
        assert_eq!(
            attribution_for_view(MapViewType::Standard),
            attribution_for_provider(TileProvider::OpenStreetMap)
        );
        // Satellite view → satellite attribution
        assert_eq!(
            attribution_for_view(MapViewType::Satellite),
            attribution_for_provider(TileProvider::SatelliteProvider)
        );
    }

    // ── Tile URL templates ─────────────────────────────────────────

    #[test]
    fn osm_url_has_zxy_placeholders() {
        let url = tile_url_template(TileProvider::OpenStreetMap);
        assert!(url.contains("{z}"));
        assert!(url.contains("{x}"));
        assert!(url.contains("{y}"));
    }

    #[test]
    fn satellite_url_has_api_key_placeholder() {
        let url = tile_url_template(TileProvider::SatelliteProvider);
        assert!(url.contains("{api_key}"));
        assert!(url.contains("{z}"));
    }

    #[test]
    fn osm_url_is_https() {
        let url = tile_url_template(TileProvider::OpenStreetMap);
        assert!(url.starts_with("https://"));
    }

    // ── GPS accuracy classification ───────────────────────────────

    #[test]
    fn accuracy_none_is_unavailable() {
        assert_eq!(
            classify_gps_accuracy(None),
            GpsAccuracyLevel::Unavailable
        );
    }

    #[test]
    fn accuracy_under_5m_is_excellent() {
        assert_eq!(
            classify_gps_accuracy(Some(3.0)),
            GpsAccuracyLevel::Excellent
        );
    }

    #[test]
    fn accuracy_5m_is_good() {
        // 5.0 is the boundary; < 5 → excellent, 5 → good
        assert_eq!(classify_gps_accuracy(Some(5.0)), GpsAccuracyLevel::Good);
    }

    #[test]
    fn accuracy_10m_is_fair() {
        assert_eq!(classify_gps_accuracy(Some(10.0)), GpsAccuracyLevel::Fair);
    }

    #[test]
    fn accuracy_20m_is_poor() {
        assert_eq!(classify_gps_accuracy(Some(20.0)), GpsAccuracyLevel::Poor);
    }

    #[test]
    fn accuracy_50m_is_unavailable() {
        assert_eq!(
            classify_gps_accuracy(Some(50.0)),
            GpsAccuracyLevel::Unavailable
        );
    }

    #[test]
    fn accuracy_3m_is_excellent() {
        assert_eq!(
            classify_gps_accuracy(Some(3.0)),
            GpsAccuracyLevel::Excellent
        );
    }

    #[test]
    fn negative_accuracy_is_unavailable() {
        assert_eq!(
            classify_gps_accuracy(Some(-1.0)),
            GpsAccuracyLevel::Unavailable
        );
    }

    // ── GPS accuracy descriptions & colors ────────────────────────

    #[test]
    fn accuracy_descriptions_are_human_readable() {
        assert!(gps_accuracy_description(GpsAccuracyLevel::Excellent).contains("Excellent"));
        assert!(gps_accuracy_description(GpsAccuracyLevel::Good).contains("Good"));
        assert!(gps_accuracy_description(GpsAccuracyLevel::Fair).contains("Fair"));
        assert!(gps_accuracy_description(GpsAccuracyLevel::Poor).contains("Poor"));
        assert!(gps_accuracy_description(GpsAccuracyLevel::Unavailable).contains("No GPS"));
    }

    #[test]
    fn accuracy_colors_are_hex_strings() {
        for level in [
            GpsAccuracyLevel::Excellent,
            GpsAccuracyLevel::Good,
            GpsAccuracyLevel::Fair,
            GpsAccuracyLevel::Poor,
            GpsAccuracyLevel::Unavailable,
        ] {
            let color = gps_accuracy_color(level);
            assert!(color.starts_with('#'));
            assert_eq!(color.len(), 7);
        }
    }

    #[test]
    fn excellent_is_green() {
        assert_eq!(gps_accuracy_color(GpsAccuracyLevel::Excellent), "#4CAF50");
    }

    #[test]
    fn unavailable_is_red() {
        assert_eq!(gps_accuracy_color(GpsAccuracyLevel::Unavailable), "#F44336");
    }

    // ── Tile coordinate conversion ─────────────────────────────────

    #[test]
    fn lat_lon_to_tile_at_zoom_0_is_origin() {
        // At zoom 0, there's only 1 tile (0, 0) for any valid lat/lon.
        let tile = lat_lon_to_tile(0.0, 0.0, 0).unwrap();
        assert_eq!(tile, TileCoord { z: 0, x: 0, y: 0 });
    }

    #[test]
    fn lat_lon_to_tile_at_zoom_1_splits_hemispheres() {
        // At zoom 1, there are 2x2 tiles.
        // (0, 0) → tile (1, 1) — equator/prime meridian
        let tile = lat_lon_to_tile(0.0, 0.0, 1).unwrap();
        assert_eq!(tile.z, 1);
        assert_eq!(tile.x, 1);
        assert_eq!(tile.y, 1);
    }

    #[test]
    fn north_pole_area_has_low_y() {
        // High latitude → low y (near the top of the map)
        let tile_north = lat_lon_to_tile(80.0, 0.0, 10).unwrap();
        let tile_south = lat_lon_to_tile(-80.0, 0.0, 10).unwrap();
        assert!(tile_north.y < tile_south.y);
    }

    #[test]
    fn east_has_higher_x_than_west() {
        let tile_east = lat_lon_to_tile(0.0, 50.0, 10).unwrap();
        let tile_west = lat_lon_to_tile(0.0, -50.0, 10).unwrap();
        assert!(tile_east.x > tile_west.x);
    }

    #[test]
    fn lat_above_85_returns_none() {
        assert!(lat_lon_to_tile(86.0, 0.0, 10).is_none());
    }

    #[test]
    fn lat_below_neg_85_returns_none() {
        assert!(lat_lon_to_tile(-86.0, 0.0, 10).is_none());
    }

    #[test]
    fn lon_above_180_returns_none() {
        assert!(lat_lon_to_tile(0.0, 181.0, 10).is_none());
    }

    #[test]
    fn lon_below_neg_180_returns_none() {
        assert!(lat_lon_to_tile(0.0, -181.0, 10).is_none());
    }

    // ── Tile count estimation ──────────────────────────────────────

    #[test]
    fn tile_count_at_zoom_0_is_1() {
        // Any bounding box at zoom 0 → 1 tile (the whole world is 1 tile)
        let count = count_tiles_in_region(-80.0, -170.0, 80.0, 170.0, 0, 0);
        assert_eq!(count, 1);
    }

    #[test]
    fn tile_count_at_zoom_1_small_region_is_1_or_4() {
        // A small region at zoom 1 might span 1 or a few tiles
        let count = count_tiles_in_region(0.0, 0.0, 1.0, 1.0, 1, 1);
        assert!(count >= 1);
    }

    #[test]
    fn tile_count_increases_with_zoom_range() {
        let single_zoom = count_tiles_in_region(40.0, -74.0, 41.0, -73.0, 12, 12);
        let multi_zoom = count_tiles_in_region(40.0, -74.0, 41.0, -73.0, 12, 14);
        assert!(multi_zoom > single_zoom);
    }

    #[test]
    fn tile_count_larger_box_has_more_tiles() {
        let small = count_tiles_in_region(40.0, -74.0, 40.01, -73.99, 14, 14);
        let large = count_tiles_in_region(40.0, -74.0, 41.0, -73.0, 14, 14);
        assert!(large > small);
    }

    #[test]
    fn tile_count_zero_when_min_zoom_gt_max_zoom() {
        let count = count_tiles_in_region(40.0, -74.0, 41.0, -73.0, 15, 10);
        assert_eq!(count, 0);
    }

    #[test]
    fn tile_count_for_nyc_area_at_common_zooms() {
        // NYC area: ~1 degree lat × 1 degree lon, zoom 12-16
        let count = count_tiles_in_region(40.5, -74.5, 41.0, -73.5, 12, 16);
        // Should be thousands of tiles
        assert!(count > 1000);
        // But not absurdly large (zoom 16 at ~1 sq degree is ~6000 tiles)
        assert!(count < 100_000);
    }

    // ── Offline region manifest ────────────────────────────────────

    #[test]
    fn build_offline_region_computes_tile_count() {
        let region = build_offline_region(
            "r1",
            "Test",
            40.0,
            -74.0,
            41.0,
            -73.0,
            12,
            14,
            TileProvider::OpenStreetMap,
            1_000_000,
        );
        assert!(region.tile_count > 0);
        assert!(region.estimated_size_bytes > 0);
        assert_eq!(region.region_id, "r1");
        assert_eq!(region.name, "Test");
        assert_eq!(region.provider, TileProvider::OpenStreetMap);
        assert_eq!(region.downloaded_at, 1_000_000);
        assert_eq!(region.last_accessed_at, 1_000_000);
    }

    #[test]
    fn estimated_size_is_tile_count_times_avg() {
        let region = build_offline_region(
            "r1",
            "Test",
            40.0,
            -74.0,
            41.0,
            -73.0,
            12,
            14,
            TileProvider::OpenStreetMap,
            1_000_000,
        );
        assert_eq!(
            region.estimated_size_bytes,
            region.tile_count * AVG_TILE_SIZE_BYTES
        );
    }

    // ── Storage availability ──────────────────────────────────────

    #[test]
    fn storage_ok_when_enough_space() {
        let result = check_storage_availability(
            1_000_000,  // new region: 1 MB
            0,          // current cache: empty
            100_000_000, // 100 MB free
        );
        assert!(result.is_ok());
    }

    #[test]
    fn storage_fails_when_not_enough_device_space() {
        let result = check_storage_availability(
            100_000_000,  // 100 MB
            0,
            50_000_000,   // only 50 MB free
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("free space"));
    }

    #[test]
    fn storage_fails_when_cache_budget_exceeded() {
        let result = check_storage_availability(
            400 * 1024 * 1024,  // 400 MB new region
            200 * 1024 * 1024,  // 200 MB already cached
            10 * 1024 * 1024 * 1024, // 10 GB free (plenty of device space)
        );
        // 400 + 200 = 600 > 500 MB limit
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cache limit"));
    }

    #[test]
    fn storage_ok_at_exact_limit() {
        let result = check_storage_availability(
            100 * 1024 * 1024,   // 100 MB
            400 * 1024 * 1024,   // 400 MB cached → total 500 MB = exactly the limit
            10 * 1024 * 1024 * 1024,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn storage_margin_accounts_for_overhead() {
        // 10 MB region + 10% margin = 11 MB needed from device
        // If device has exactly 10 MB free, it should fail
        let result = check_storage_availability(
            10_000_000,
            0,
            10_000_000, // exactly equal, no margin room
        );
        assert!(result.is_err());
    }

    // ── Region count limits ────────────────────────────────────────

    #[test]
    fn can_add_region_under_limit() {
        assert!(can_add_region(0));
        assert!(can_add_region(10));
        assert!(can_add_region(MAX_OFFLINE_REGIONS - 1));
    }

    #[test]
    fn cannot_add_region_at_limit() {
        assert!(!can_add_region(MAX_OFFLINE_REGIONS));
        assert!(!can_add_region(MAX_OFFLINE_REGIONS + 1));
    }

    // ── Eviction candidate selection ───────────────────────────────

    fn make_region(id: &str, size: u64, accessed: i64) -> OfflineRegion {
        OfflineRegion {
            region_id: id.to_string(),
            name: id.to_string(),
            min_lat: 40.0,
            min_lon: -74.0,
            max_lat: 41.0,
            max_lon: -73.0,
            min_zoom: 12,
            max_zoom: 14,
            tile_count: 100,
            estimated_size_bytes: size,
            provider: TileProvider::OpenStreetMap,
            downloaded_at: 0,
            last_accessed_at: accessed,
        }
    }

    #[test]
    fn eviction_picks_lru_when_large_enough() {
        let regions = vec![
            make_region("r1", 100_000, 500),  // newer
            make_region("r2", 100_000, 100),  // older (LRU)
            make_region("r3", 100_000, 300),  // middle
        ];
        let candidate = select_eviction_candidate(&regions, 50_000);
        // r2 is the LRU and is large enough
        assert_eq!(candidate, Some(1));
    }

    #[test]
    fn eviction_picks_largest_when_none_large_enough() {
        let regions = vec![
            make_region("r1", 30_000, 100),  // LRU but small
            make_region("r2", 50_000, 200),  // larger, more recent
        ];
        let candidate = select_eviction_candidate(&regions, 100_000);
        // No single region is large enough; r2 is the largest
        assert_eq!(candidate, Some(1));
    }

    #[test]
    fn eviction_returns_none_for_empty_list() {
        let candidate = select_eviction_candidate(&[], 50_000);
        assert_eq!(candidate, None);
    }

    #[test]
    fn eviction_picks_lru_that_is_large_enough_over_largest() {
        // r1 is LRU and large enough → should be picked over the larger r2
        let regions = vec![
            make_region("r1", 60_000, 100),  // LRU, large enough
            make_region("r2", 100_000, 500),  // larger, more recent
        ];
        let candidate = select_eviction_candidate(&regions, 50_000);
        assert_eq!(candidate, Some(0));
    }

    // ── Stale region detection ─────────────────────────────────────

    #[test]
    fn fresh_region_is_not_stale() {
        let region = make_region("r1", 100_000, 1_000_000);
        assert!(!is_region_stale(&region, 1_000_000));
    }

    #[test]
    fn old_region_is_stale() {
        let region = make_region("r1", 100_000, 1_000_000);
        // 100 days later
        let now = 1_000_000 + 100 * 24 * 60 * 60 * 1000;
        assert!(is_region_stale(&region, now));
    }

    #[test]
    fn region_just_under_stale_threshold_is_not_stale() {
        let region = make_region("r1", 100_000, 1_000_000);
        // 89 days later (just under 90-day threshold)
        let now = 1_000_000 + 89 * 24 * 60 * 60 * 1000;
        assert!(!is_region_stale(&region, now));
    }

    // ── touch_region ────────────────────────────────────────────────

    #[test]
    fn touch_updates_last_accessed() {
        let region = make_region("r1", 100_000, 1_000_000);
        let touched = touch_region(region, 2_000_000);
        assert_eq!(touched.last_accessed_at, 2_000_000);
        // Other fields unchanged
        assert_eq!(touched.region_id, "r1");
        assert_eq!(touched.estimated_size_bytes, 100_000);
    }

    // ── Route preview simplification ───────────────────────────────

    #[test]
    fn simplification_epsilon_decreases_with_zoom() {
        let low_zoom = simplification_epsilon_for_zoom(5);
        let high_zoom = simplification_epsilon_for_zoom(16);
        assert!(high_zoom < low_zoom);
    }

    #[test]
    fn street_zoom_uses_default_epsilon() {
        // Zoom 14-17 uses 0.00003 (the same as RouteBuilderConfig default)
        let eps = simplification_epsilon_for_zoom(15);
        assert_eq!(eps, 0.00003);
    }

    #[test]
    fn building_zoom_uses_finest_epsilon() {
        let eps = simplification_epsilon_for_zoom(20);
        assert_eq!(eps, 0.00001);
    }

    #[test]
    fn continent_zoom_uses_coarsest_epsilon() {
        let eps = simplification_epsilon_for_zoom(3);
        assert_eq!(eps, 0.001);
    }

    // ── Full resolution route ──────────────────────────────────────

    #[test]
    fn full_resolution_at_high_zoom() {
        assert!(should_show_full_resolution_route(15));
        assert!(should_show_full_resolution_route(17));
    }

    #[test]
    fn simplified_at_low_zoom() {
        assert!(!should_show_full_resolution_route(10));
        assert!(!should_show_full_resolution_route(14));
    }

    // ── Recenter button ────────────────────────────────────────────

    #[test]
    fn recenter_hidden_when_centered() {
        assert!(!should_show_recenter_button(10.0));
        assert!(!should_show_recenter_button(50.0));
    }

    #[test]
    fn recenter_shown_when_off_center() {
        assert!(should_show_recenter_button(51.0));
        assert!(should_show_recenter_button(100.0));
    }

    // ── Heading rotation ──────────────────────────────────────────

    #[test]
    fn no_rotation_when_stationary() {
        assert!(!should_rotate_with_heading(0.0, true));
    }

    #[test]
    fn no_rotation_when_heading_invalid() {
        assert!(!should_rotate_with_heading(3.0, false));
    }

    #[test]
    fn rotation_when_moving_with_valid_heading() {
        assert!(should_rotate_with_heading(2.0, true));
        assert!(should_rotate_with_heading(5.0, true));
    }

    #[test]
    fn no_rotation_at_walk_threshold_boundary() {
        // 1.4 m/s is the threshold; at exactly 1.4, should not rotate
        // (the condition is > 1.4, not >=)
        assert!(!should_rotate_with_heading(1.4, true));
    }

    // ── Saved route validation ─────────────────────────────────────

    fn make_saved_route() -> SavedRoute {
        SavedRoute {
            route_id: "r1".to_string(),
            name: "Morning 5K".to_string(),
            user_id: "u1".to_string(),
            encoded_polyline: "_p~iF~ps|U".to_string(),
            start_lat: 40.0,
            start_lon: -73.0,
            distance_meters: 5000.0,
            saved_at: 1_000_000,
        }
    }

    #[test]
    fn valid_saved_route_passes() {
        assert!(validate_saved_route(&make_saved_route()).is_ok());
    }

    #[test]
    fn empty_name_rejected() {
        let mut r = make_saved_route();
        r.name = "".to_string();
        assert!(validate_saved_route(&r).is_err());
    }

    #[test]
    fn empty_polyline_rejected() {
        let mut r = make_saved_route();
        r.encoded_polyline = "".to_string();
        assert!(validate_saved_route(&r).is_err());
    }

    #[test]
    fn negative_distance_rejected() {
        let mut r = make_saved_route();
        r.distance_meters = -100.0;
        assert!(validate_saved_route(&r).is_err());
    }

    #[test]
    fn empty_user_id_rejected() {
        let mut r = make_saved_route();
        r.user_id = "".to_string();
        assert!(validate_saved_route(&r).is_err());
    }

    // ── Tile cache key ─────────────────────────────────────────────

    #[test]
    fn tile_cache_key_format() {
        let key = tile_cache_key(
            TileProvider::OpenStreetMap,
            TileCoord { z: 12, x: 1234, y: 5678 },
        );
        assert_eq!(key, "tiles/osm/12/1234/5678");
    }

    #[test]
    fn satellite_cache_key_uses_satellite_slug() {
        let key = tile_cache_key(
            TileProvider::SatelliteProvider,
            TileCoord { z: 10, x: 100, y: 200 },
        );
        assert_eq!(key, "tiles/satellite/10/100/200");
    }

    // ── Provider slugs ─────────────────────────────────────────────

    #[test]
    fn osm_slug() {
        assert_eq!(tile_provider_slug(TileProvider::OpenStreetMap), "osm");
    }

    #[test]
    fn satellite_slug() {
        assert_eq!(
            tile_provider_slug(TileProvider::SatelliteProvider),
            "satellite"
        );
    }

    #[test]
    fn esri_slug() {
        assert_eq!(tile_provider_slug(TileProvider::Esri), "esri");
    }

    #[test]
    fn custom_slug() {
        assert_eq!(tile_provider_slug(TileProvider::Custom), "custom");
    }
}

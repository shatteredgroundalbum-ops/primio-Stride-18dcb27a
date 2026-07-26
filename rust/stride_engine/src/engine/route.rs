//! Route builder (spec section 4).
//!
//! Converts the stream of *accepted* GPS points into:
//!   - a full-resolution point list retained for calculation,
//!   - a simplified point list for map display / preview,
//!   - a Google-style encoded polyline string for compact transport.
//!
//! Route segments are split after long pauses so the displayed path
//! doesn't draw a straight line across a rest stop gap.

use crate::models::WorkoutPoint;

#[derive(Debug, Clone, Copy)]
pub struct RouteBuilderConfig {
    /// A pause longer than this (ms) starts a new visual route segment.
    pub segment_break_after_pause_ms: i64,
    /// Douglas-Peucker epsilon (degrees) used to simplify the display
    /// route. Calculation always uses the full-resolution list regardless
    /// of this setting.
    pub simplify_epsilon_degrees: f64,
}

impl Default for RouteBuilderConfig {
    fn default() -> Self {
        Self {
            segment_break_after_pause_ms: 60_000,
            simplify_epsilon_degrees: 0.00003, // ~3m at mid-latitudes
        }
    }
}

pub struct RouteSegment {
    pub points: Vec<(f64, f64)>,
}

pub struct RouteBuilder {
    config: RouteBuilderConfig,
    /// Full-resolution accepted points, retained for calculation.
    full_resolution: Vec<WorkoutPoint>,
    segments: Vec<RouteSegment>,
    last_point_at: Option<i64>,
}

impl RouteBuilder {
    pub fn new(config: RouteBuilderConfig) -> Self {
        Self {
            config,
            full_resolution: Vec::new(),
            segments: vec![RouteSegment { points: Vec::new() }],
            last_point_at: None,
        }
    }

    pub fn default_config() -> Self {
        Self::new(RouteBuilderConfig::default())
    }

    /// Adds an accepted point (caller must have already passed it through
    /// `GpsFilter`). Starts a new segment if the gap since the previous
    /// point exceeds `segment_break_after_pause_ms`.
    pub fn add_accepted_point(&mut self, point: WorkoutPoint) {
        if let Some(last_at) = self.last_point_at {
            if point.recorded_at - last_at > self.config.segment_break_after_pause_ms {
                self.segments.push(RouteSegment { points: Vec::new() });
            }
        }
        self.last_point_at = Some(point.recorded_at);

        self.segments
            .last_mut()
            .unwrap()
            .points
            .push((point.latitude, point.longitude));
        self.full_resolution.push(point);
    }

    pub fn full_resolution_points(&self) -> &[WorkoutPoint] {
        &self.full_resolution
    }

    pub fn segments(&self) -> &[RouteSegment] {
        &self.segments
    }

    pub fn start_location(&self) -> Option<(f64, f64)> {
        self.full_resolution
            .first()
            .map(|p| (p.latitude, p.longitude))
    }

    pub fn end_location(&self) -> Option<(f64, f64)> {
        self.full_resolution
            .last()
            .map(|p| (p.latitude, p.longitude))
    }

    /// Produces a simplified point list (all segments concatenated) for
    /// fast map preview rendering, and its encoded polyline.
    pub fn simplified_polyline(&self) -> String {
        let all_points: Vec<(f64, f64)> = self
            .segments
            .iter()
            .flat_map(|s| s.points.iter().copied())
            .collect();
        let simplified = simplify_douglas_peucker(&all_points, self.config.simplify_epsilon_degrees);
        encode_polyline(&simplified)
    }
}

/// Douglas-Peucker polyline simplification. Reduces point count for map
/// display while preserving the overall route shape within `epsilon`.
pub fn simplify_douglas_peucker(points: &[(f64, f64)], epsilon: f64) -> Vec<(f64, f64)> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    simplify_range(points, 0, points.len() - 1, epsilon, &mut keep);

    points
        .iter()
        .zip(keep.iter())
        .filter_map(|(p, k)| if *k { Some(*p) } else { None })
        .collect()
}

fn simplify_range(points: &[(f64, f64)], start: usize, end: usize, epsilon: f64, keep: &mut [bool]) {
    if end <= start + 1 {
        return;
    }
    let (mut max_dist, mut max_index) = (0.0, start);
    for i in (start + 1)..end {
        let d = perpendicular_distance(points[i], points[start], points[end]);
        if d > max_dist {
            max_dist = d;
            max_index = i;
        }
    }
    if max_dist > epsilon {
        keep[max_index] = true;
        simplify_range(points, start, max_index, epsilon, keep);
        simplify_range(points, max_index, end, epsilon, keep);
    }
}

fn perpendicular_distance(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (x, y) = p;
    let (x1, y1) = a;
    let (x2, y2) = b;
    let dx = x2 - x1;
    let dy = y2 - y1;
    if dx == 0.0 && dy == 0.0 {
        return ((x - x1).powi(2) + (y - y1).powi(2)).sqrt();
    }
    let numerator = (dy * x - dx * y + x2 * y1 - y2 * x1).abs();
    let denominator = (dx * dx + dy * dy).sqrt();
    numerator / denominator
}

/// Encodes a sequence of (lat, lon) points using the standard Google
/// polyline algorithm (5 decimal-place precision).
pub fn encode_polyline(points: &[(f64, f64)]) -> String {
    let mut result = String::new();
    let mut prev_lat = 0i64;
    let mut prev_lon = 0i64;

    for &(lat, lon) in points {
        let lat_i = (lat * 1e5).round() as i64;
        let lon_i = (lon * 1e5).round() as i64;

        encode_value(lat_i - prev_lat, &mut result);
        encode_value(lon_i - prev_lon, &mut result);

        prev_lat = lat_i;
        prev_lon = lon_i;
    }

    result
}

fn encode_value(value: i64, out: &mut String) {
    let mut v = value << 1;
    if value < 0 {
        v = !v;
    }
    let mut v = v as u64;
    while v >= 0x20 {
        let chunk = ((v & 0x1f) as u8) | 0x20;
        out.push((chunk + 63) as char);
        v >>= 5;
    }
    out.push((v as u8 + 63) as char);
}

/// Decodes a Google-style polyline back into (lat, lon) points. Provided
/// for symmetry/testability; the Dart side may use its own map-library
/// decoder, but having this in Rust lets us unit test encode/decode
/// round-trips.
pub fn decode_polyline(encoded: &str) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let chars: Vec<u8> = encoded.bytes().collect();
    let mut index = 0;
    let mut lat = 0i64;
    let mut lon = 0i64;

    while index < chars.len() {
        let (dlat, new_index) = decode_value(&chars, index);
        index = new_index;
        lat += dlat;

        let (dlon, new_index2) = decode_value(&chars, index);
        index = new_index2;
        lon += dlon;

        points.push((lat as f64 / 1e5, lon as f64 / 1e5));
    }

    points
}

fn decode_value(chars: &[u8], mut index: usize) -> (i64, usize) {
    let mut result: i64 = 0;
    let mut shift = 0;
    loop {
        let b = chars[index] as i64 - 63;
        index += 1;
        result |= (b & 0x1f) << shift;
        shift += 5;
        if b < 0x20 {
            break;
        }
    }
    let value = if result & 1 != 0 { !(result >> 1) } else { result >> 1 };
    (value, index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LocationSource;

    fn accepted_point(workout_id: &str, lat: f64, lon: f64, t: i64) -> WorkoutPoint {
        let mut p = WorkoutPoint::new(workout_id, lat, lon, t, LocationSource::PhoneGps);
        p.accepted = true;
        p
    }

    #[test]
    fn tracks_start_and_end_location() {
        let mut b = RouteBuilder::default_config();
        b.add_accepted_point(accepted_point("w1", 40.0, -73.0, 1_000));
        b.add_accepted_point(accepted_point("w1", 40.001, -73.001, 2_000));
        assert_eq!(b.start_location(), Some((40.0, -73.0)));
        assert_eq!(b.end_location(), Some((40.001, -73.001)));
    }

    #[test]
    fn long_pause_starts_new_segment() {
        let mut b = RouteBuilder::default_config();
        b.add_accepted_point(accepted_point("w1", 40.0, -73.0, 0));
        b.add_accepted_point(accepted_point("w1", 40.001, -73.001, 5_000));
        // Gap > 60s -> new segment
        b.add_accepted_point(accepted_point("w1", 40.002, -73.002, 70_000));
        assert_eq!(b.segments().len(), 2);
    }

    #[test]
    fn polyline_encode_decode_roundtrip() {
        let points = vec![(38.5, -120.2), (40.7, -120.95), (43.252, -126.453)];
        let encoded = encode_polyline(&points);
        let decoded = decode_polyline(&encoded);
        assert_eq!(decoded.len(), points.len());
        for (a, b) in points.iter().zip(decoded.iter()) {
            assert!((a.0 - b.0).abs() < 1e-4);
            assert!((a.1 - b.1).abs() < 1e-4);
        }
    }

    #[test]
    fn known_polyline_encoding_matches_reference() {
        // Classic example from Google's polyline algorithm documentation.
        let points = vec![(38.5, -120.2), (40.7, -120.95), (43.252, -126.453)];
        let encoded = encode_polyline(&points);
        assert_eq!(encoded, "_p~iF~ps|U_ulLnnqC_mqNvxq`@");
    }

    #[test]
    fn simplify_reduces_point_count_on_straight_line_noise() {
        // A near-straight line with tiny jitter should simplify down a lot.
        let mut points = Vec::new();
        for i in 0..100 {
            let jitter = if i % 2 == 0 { 0.0000001 } else { -0.0000001 };
            points.push((40.0 + i as f64 * 0.0001, -73.0 + jitter));
        }
        let simplified = simplify_douglas_peucker(&points, 0.00005);
        assert!(simplified.len() < points.len());
        assert_eq!(simplified.first(), points.first());
        assert_eq!(simplified.last(), points.last());
    }

    #[test]
    fn simplify_keeps_short_lists_untouched() {
        let points = vec![(1.0, 1.0), (2.0, 2.0)];
        let simplified = simplify_douglas_peucker(&points, 0.001);
        assert_eq!(simplified, points);
    }
}

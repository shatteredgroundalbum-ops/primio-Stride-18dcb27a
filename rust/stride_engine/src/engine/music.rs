//! §11 — Music system.
//!
//! This module models the music playback state machine, audio focus
//! handling, network-loss behavior, and coaching-interop coordination.
//! The actual audio playback runs on the Kotlin/Android side (MediaSession,
//! ExoPlayer, audio focus). The Rust engine provides the **decision
//! logic**: playlist state transitions, audio focus priority, network-loss
//! recovery, coaching-interop pause/resume, blocked-content filtering,
//! and AI/manual mode selection.
//!
//! Key responsibilities:
//!
//! - **Playlist state machine**: Play → Pause → Resume → Stop transitions
//!   with validation (no resume from stopped, no pause from idle, etc.).
//! - **Audio focus handling**: the engine tracks audio focus state and
//!   decides whether to duck, pause, or resume based on focus changes
//!   (incoming calls, navigation prompts, other apps).
//! - **Coaching-interop**: the engine coordinates pausing music for
//!   coaching prompts and resuming after the coaching prompt finishes.
//! - **Network-loss behavior**: when streaming internet radio, the engine
//!   decides whether to buffer, reconnect, or switch to local media.
//! - **Blocked content**: the engine filters out blocked artists/genres
//!   from playback queues.
//! - **AI/manual mode**: the engine decides whether to use AI-curated
//!   playlists or the user's manual selection.

use serde::{Deserialize, Serialize};

// ─── Playback state machine ───────────────────────────────────────────────

/// The playback state of the music player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackState {
    /// No media loaded; player is idle.
    Idle,
    /// Media is loaded but not playing.
    Ready,
    /// Media is actively playing.
    Playing,
    /// Media is paused (user-initiated or focus loss).
    Paused,
    /// Media is buffering (streaming, waiting for data).
    Buffering,
    /// Media playback ended (reached end of track/playlist).
    Ended,
    /// Playback stopped and player released.
    Stopped,
    /// An error occurred (stream error, decode error, etc.).
    Error,
}

impl Default for PlaybackState {
    fn default() -> Self {
        PlaybackState::Idle
    }
}

impl PlaybackState {
    /// Returns `true` if the player is currently producing sound.
    pub fn is_playing(self) -> bool {
        matches!(self, PlaybackState::Playing)
    }

    /// Returns `true` if the player can be resumed (i.e. there is media
    /// loaded and it's not in a terminal state).
    pub fn can_resume(self) -> bool {
        matches!(
            self,
            PlaybackState::Paused | PlaybackState::Buffering | PlaybackState::Ready
        )
    }

    /// Returns `true` if the player can be paused (only from playing).
    pub fn can_pause(self) -> bool {
        matches!(self, PlaybackState::Playing)
    }

    /// Returns `true` if the state is terminal (cannot transition further
    /// without reloading).
    pub fn is_terminal(self) -> bool {
        matches!(self, PlaybackState::Stopped | PlaybackState::Ended | PlaybackState::Error)
    }

    /// Returns a human-readable label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            PlaybackState::Idle => "Idle",
            PlaybackState::Ready => "Ready",
            PlaybackState::Playing => "Playing",
            PlaybackState::Paused => "Paused",
            PlaybackState::Buffering => "Buffering",
            PlaybackState::Ended => "Ended",
            PlaybackState::Stopped => "Stopped",
            PlaybackState::Error => "Error",
        }
    }
}

/// A playback command that the user or system can issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackCommand {
    /// Start playing (from idle or ready).
    Play,
    /// Pause playback.
    Pause,
    /// Resume from pause.
    Resume,
    /// Skip to the next track.
    Skip,
    /// Go back to the previous track.
    Previous,
    /// Stop playback and release resources.
    Stop,
    /// Seek to a specific position.
    Seek,
}

/// The result of a state transition attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionResult {
    pub previous_state: PlaybackState,
    pub new_state: PlaybackState,
    pub accepted: bool,
    pub message: String,
}

/// Attempts to transition the playback state based on a command.
/// Returns a `TransitionResult` indicating whether the transition was
/// accepted and the new state.
///
/// Valid transitions:
/// - Play: Idle → Ready → Playing, Ready → Playing, Paused → Playing,
///   Buffering → Playing
/// - Pause: Playing → Paused
/// - Resume: Paused → Playing, Buffering → Playing, Ready → Playing
/// - Skip: Playing → Playing (next track), Paused → Playing (next track)
/// - Previous: Playing → Playing (prev track), Paused → Playing (prev)
/// - Stop: any → Stopped
/// - Seek: any non-terminal → same state (position changed)
pub fn transition(current: PlaybackState, command: PlaybackCommand) -> TransitionResult {
    let (new_state, accepted, message) = match (current, command) {
        // Play transitions
        (PlaybackState::Idle, PlaybackCommand::Play) => {
            (PlaybackState::Playing, true, "Starting playback".to_string())
        }
        (PlaybackState::Ready, PlaybackCommand::Play) => {
            (PlaybackState::Playing, true, "Starting playback".to_string())
        }
        (PlaybackState::Paused, PlaybackCommand::Play) => {
            (PlaybackState::Playing, true, "Resuming playback".to_string())
        }
        (PlaybackState::Buffering, PlaybackCommand::Play) => {
            (PlaybackState::Playing, true, "Resuming playback".to_string())
        }

        // Pause transitions
        (PlaybackState::Playing, PlaybackCommand::Pause) => {
            (PlaybackState::Paused, true, "Playback paused".to_string())
        }

        // Resume transitions
        (PlaybackState::Paused, PlaybackCommand::Resume) => {
            (PlaybackState::Playing, true, "Resuming playback".to_string())
        }
        (PlaybackState::Buffering, PlaybackCommand::Resume) => {
            (PlaybackState::Playing, true, "Resuming playback".to_string())
        }
        (PlaybackState::Ready, PlaybackCommand::Resume) => {
            (PlaybackState::Playing, true, "Starting playback".to_string())
        }

        // Skip transitions (advance to next track, resume if paused)
        (PlaybackState::Playing, PlaybackCommand::Skip) => {
            (PlaybackState::Playing, true, "Skipping to next track".to_string())
        }
        (PlaybackState::Paused, PlaybackCommand::Skip) => {
            (PlaybackState::Playing, true, "Skipping to next track".to_string())
        }
        (PlaybackState::Ended, PlaybackCommand::Skip) => {
            (PlaybackState::Playing, true, "Skipping to next track".to_string())
        }

        // Previous transitions
        (PlaybackState::Playing, PlaybackCommand::Previous) => {
            (PlaybackState::Playing, true, "Going to previous track".to_string())
        }
        (PlaybackState::Paused, PlaybackCommand::Previous) => {
            (PlaybackState::Playing, true, "Going to previous track".to_string())
        }

        // Stop — always valid from any state
        (_, PlaybackCommand::Stop) => {
            (PlaybackState::Stopped, true, "Playback stopped".to_string())
        }

        // Seek — valid from any non-terminal state
        (s, PlaybackCommand::Seek) if !s.is_terminal() => {
            (s, true, "Seeking".to_string())
        }

        // Invalid transitions
        (s, c) => {
            let msg = format!(
                "Invalid transition: cannot {} from {}",
                command_label(c),
                s.label()
            );
            (s, false, msg)
        }
    };

    TransitionResult {
        previous_state: current,
        new_state,
        accepted,
        message,
    }
}

fn command_label(c: PlaybackCommand) -> &'static str {
    match c {
        PlaybackCommand::Play => "play",
        PlaybackCommand::Pause => "pause",
        PlaybackCommand::Resume => "resume",
        PlaybackCommand::Skip => "skip",
        PlaybackCommand::Previous => "go to previous",
        PlaybackCommand::Stop => "stop",
        PlaybackCommand::Seek => "seek",
    }
}

// ─── Audio focus ──────────────────────────────────────────────────────────

/// The Android audio focus state. The app must request and manage audio
/// focus to play music; when another app takes focus, the app must
/// either duck (lower volume), pause, or stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFocusState {
    /// The app does not have audio focus.
    NoFocus,
    /// The app has full audio focus.
    Focused,
    /// Another app requested transient focus; we should duck (lower
    /// volume) temporarily.
    Ducking,
    /// Another app took transient focus; we should pause temporarily.
    TransientPause,
    /// Another app took permanent focus; we should stop.
    Lost,
}

impl Default for AudioFocusState {
    fn default() -> Self {
        AudioFocusState::NoFocus
    }
}

impl AudioFocusState {
    /// Returns `true` if the app can play audio in this focus state.
    pub fn can_play(self) -> bool {
        matches!(self, AudioFocusState::Focused | AudioFocusState::Ducking)
    }

    /// Returns the volume multiplier for this focus state (1.0 = full,
    /// 0.0 = silent, 0.2 = ducked).
    pub fn volume_multiplier(self) -> f64 {
        match self {
            AudioFocusState::Focused => 1.0,
            AudioFocusState::Ducking => 0.2,
            _ => 0.0,
        }
    }

    /// Returns a human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            AudioFocusState::NoFocus => "No audio focus",
            AudioFocusState::Focused => "Audio focus granted",
            AudioFocusState::Ducking => "Ducking (volume lowered)",
            AudioFocusState::TransientPause => "Paused (transient focus loss)",
            AudioFocusState::Lost => "Audio focus lost",
        }
    }
}

/// The type of audio focus change event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioFocusEvent {
    /// Audio focus was granted to the app.
    Gain,
    /// Another app requested transient ducking (e.g. navigation).
    Duck,
    /// Another app requested transient pause (e.g. incoming call).
    TransientPause,
    /// Another app took permanent focus.
    Loss,
}

/// Processes an audio focus event, returning the new focus state and
/// the recommended playback action.
pub fn handle_audio_focus_event(
    current_focus: AudioFocusState,
    event: AudioFocusEvent,
) -> (AudioFocusState, PlaybackAction) {
    match event {
        AudioFocusEvent::Gain => (AudioFocusState::Focused, PlaybackAction::Resume),
        AudioFocusEvent::Duck => (AudioFocusState::Ducking, PlaybackAction::Duck),
        AudioFocusEvent::TransientPause => {
            (AudioFocusState::TransientPause, PlaybackAction::Pause)
        }
        AudioFocusEvent::Loss => (AudioFocusState::Lost, PlaybackAction::Stop),
    }
}

/// The recommended playback action in response to an audio focus change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackAction {
    /// Keep playing as-is.
    Continue,
    /// Resume playback (focus gained or transient loss ended).
    Resume,
    /// Pause playback (transient focus loss).
    Pause,
    /// Duck (lower volume) temporarily.
    Duck,
    /// Stop playback entirely (permanent focus loss).
    Stop,
}

// ─── Coaching-interop ─────────────────────────────────────────────────────

/// The coaching-interop coordination state. When the AI coach delivers
/// a prompt (encouragement, form feedback, milestone), the music should
/// pause so the user can hear the coaching, then resume afterward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoachingInteropState {
    /// No coaching prompt active; music plays normally.
    Inactive,
    /// A coaching prompt is being delivered; music is paused.
    CoachingActive,
    /// The coaching prompt finished; music should resume.
    CoachingFinished,
}

impl Default for CoachingInteropState {
    fn default() -> Self {
        CoachingInteropState::Inactive
    }
}

/// A request to coordinate music with a coaching prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoachingRequest {
    /// A coaching prompt is about to start; pause music.
    PromptStarting,
    /// The coaching prompt finished; resume music.
    PromptFinished,
    /// The coaching prompt was cancelled; resume music.
    PromptCancelled,
}

/// The result of a coaching-interop coordination request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachingInteropResult {
    pub state: CoachingInteropState,
    pub music_action: PlaybackAction,
    pub was_playing_before: bool,
    pub message: String,
}

/// Processes a coaching-interop request, deciding whether to pause or
/// resume music. If music was not playing when the coaching prompt
/// starts, it should not resume afterward (avoiding unwanted playback).
pub fn handle_coaching_request(
    current_state: CoachingInteropState,
    request: CoachingRequest,
    music_is_playing: bool,
) -> CoachingInteropResult {
    match request {
        CoachingRequest::PromptStarting => {
            let action = if music_is_playing {
                PlaybackAction::Pause
            } else {
                PlaybackAction::Continue
            };
            CoachingInteropResult {
                state: CoachingInteropState::CoachingActive,
                music_action: action,
                was_playing_before: music_is_playing,
                message: "Pausing music for coaching prompt".to_string(),
            }
        }
        CoachingRequest::PromptFinished | CoachingRequest::PromptCancelled => {
            let should_resume = current_state == CoachingInteropState::CoachingActive
                && match request {
                    CoachingRequest::PromptFinished => true,
                    CoachingRequest::PromptCancelled => false, // cancelled → don't resume
                    _ => false,
                };
            let (state, action, msg) = if should_resume {
                (
                    CoachingInteropState::CoachingFinished,
                    PlaybackAction::Resume,
                    "Resuming music after coaching prompt",
                )
            } else if request == CoachingRequest::PromptCancelled {
                (
                    CoachingInteropState::Inactive,
                    PlaybackAction::Continue,
                    "Coaching cancelled, music unchanged",
                )
            } else {
                (
                    CoachingInteropState::Inactive,
                    PlaybackAction::Continue,
                    "No coaching was active",
                )
            };
            CoachingInteropResult {
                state,
                music_action: action,
                was_playing_before: music_is_playing,
                message: msg.to_string(),
            }
        }
    }
}

// ─── Music source / mode ──────────────────────────────────────────────────

/// The source of music content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MusicSource {
    /// Local files on the device.
    LocalDevice,
    /// Internet radio stream.
    InternetRadio,
    /// AI-curated playlist (server-side, per spec — AI runs server-side).
    AiCurated,
}

impl MusicSource {
    /// Returns `true` if this source requires a network connection.
    pub fn requires_network(self) -> bool {
        matches!(self, MusicSource::InternetRadio | MusicSource::AiCurated)
    }

    /// Returns a human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            MusicSource::LocalDevice => "Local device",
            MusicSource::InternetRadio => "Internet radio",
            MusicSource::AiCurated => "AI curated",
        }
    }
}

/// The music selection mode: AI-curated or manual user selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MusicMode {
    /// User manually selects tracks/playlists.
    Manual,
    /// AI curates the playlist based on workout context.
    Ai,
}

impl Default for MusicMode {
    fn default() -> Self {
        MusicMode::Manual
    }
}

impl MusicMode {
    /// Returns a human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            MusicMode::Manual => "Manual",
            MusicMode::Ai => "AI curated",
        }
    }
}

// ─── Network-loss behavior ────────────────────────────────────────────────

/// The state of the network connection for streaming music.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkState {
    /// Network is connected and streaming is possible.
    Connected,
    /// Network is weak; streaming may stutter.
    Weak,
    /// Network is lost; streaming is not possible.
    Lost,
}

/// The decision for what to do when the network is lost during streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLossDecision {
    pub should_switch_to_local: bool,
    pub should_buffer: bool,
    pub should_stop: bool,
    pub message: String,
    pub fallback_source: MusicSource,
}

/// Decides what to do when the network is lost during streaming. If
/// local media is available, the engine should switch to local. If not,
/// it should stop. Buffering is only useful for brief network hiccups.
pub fn handle_network_loss(
    network: NetworkState,
    current_source: MusicSource,
    has_local_media: bool,
    buffer_health_ms: i64,
) -> NetworkLossDecision {
    if !current_source.requires_network() {
        return NetworkLossDecision {
            should_switch_to_local: false,
            should_buffer: false,
            should_stop: false,
            message: "Local playback — not affected by network loss".to_string(),
            fallback_source: current_source,
        };
    }

    match network {
        NetworkState::Connected => NetworkLossDecision {
            should_switch_to_local: false,
            should_buffer: false,
            should_stop: false,
            message: "Network connected — streaming continues".to_string(),
            fallback_source: current_source,
        },
        NetworkState::Weak => {
            if buffer_health_ms > 5000 {
                NetworkLossDecision {
                    should_switch_to_local: false,
                    should_buffer: true,
                    should_stop: false,
                    message: "Weak network — buffering, continuing stream".to_string(),
                    fallback_source: current_source,
                }
            } else if has_local_media {
                NetworkLossDecision {
                    should_switch_to_local: true,
                    should_buffer: false,
                    should_stop: false,
                    message: "Weak network, low buffer — switching to local media".to_string(),
                    fallback_source: MusicSource::LocalDevice,
                }
            } else {
                NetworkLossDecision {
                    should_switch_to_local: false,
                    should_buffer: true,
                    should_stop: false,
                    message: "Weak network — buffering".to_string(),
                    fallback_source: current_source,
                }
            }
        }
        NetworkState::Lost => {
            if has_local_media {
                NetworkLossDecision {
                    should_switch_to_local: true,
                    should_buffer: false,
                    should_stop: false,
                    message: "Network lost — switching to local media".to_string(),
                    fallback_source: MusicSource::LocalDevice,
                }
            } else {
                NetworkLossDecision {
                    should_switch_to_local: false,
                    should_buffer: false,
                    should_stop: true,
                    message: "Network lost, no local media — stopping playback".to_string(),
                    fallback_source: current_source,
                }
            }
        }
    }
}

// ─── Blocked content ──────────────────────────────────────────────────────

/// A track in the playback queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub genre: String,
    pub duration_ms: i64,
    pub source: MusicSource,
}

/// A playlist with its tracks and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub tracks: Vec<Track>,
    pub source: MusicSource,
    pub mode: MusicMode,
}

impl Playlist {
    /// Returns the total duration of the playlist in ms.
    pub fn total_duration_ms(&self) -> i64 {
        self.tracks.iter().map(|t| t.duration_ms).sum()
    }

    /// Returns the number of tracks in the playlist.
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Returns `true` if the playlist is empty.
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
}

/// Filters a playlist, removing tracks by blocked artists or genres.
/// Returns a new playlist with only the unblocked tracks, and a count
/// of how many tracks were removed.
pub fn filter_blocked_content(
    playlist: &Playlist,
    blocked_artists: &[String],
    blocked_genres: &[String],
) -> (Playlist, usize) {
    let mut filtered_tracks = Vec::new();
    let mut removed_count = 0;

    for track in &playlist.tracks {
        let artist_blocked = blocked_artists
            .iter()
            .any(|a| a.eq_ignore_ascii_case(&track.artist));
        let genre_blocked = blocked_genres
            .iter()
            .any(|g| g.eq_ignore_ascii_case(&track.genre));

        if artist_blocked || genre_blocked {
            removed_count += 1;
        } else {
            filtered_tracks.push(track.clone());
        }
    }

    let filtered = Playlist {
        id: playlist.id.clone(),
        name: playlist.name.clone(),
        tracks: filtered_tracks,
        source: playlist.source,
        mode: playlist.mode,
    };

    (filtered, removed_count)
}

/// Returns `true` if a track should be blocked (by artist or genre).
pub fn is_track_blocked(
    track: &Track,
    blocked_artists: &[String],
    blocked_genres: &[String],
) -> bool {
    blocked_artists
        .iter()
        .any(|a| a.eq_ignore_ascii_case(&track.artist))
        || blocked_genres
            .iter()
            .any(|g| g.eq_ignore_ascii_case(&track.genre))
}

// ─── Likes and skips ──────────────────────────────────────────────────────

/// A user's feedback on a track (like/dislike/skip).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackFeedback {
    Liked,
    Disliked,
    Skipped,
    Completed,
}

impl TrackFeedback {
    /// Returns `true` if this feedback indicates a positive reaction.
    pub fn is_positive(self) -> bool {
        matches!(self, TrackFeedback::Liked | TrackFeedback::Completed)
    }

    /// Returns a human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            TrackFeedback::Liked => "Liked",
            TrackFeedback::Disliked => "Disliked",
            TrackFeedback::Skipped => "Skipped",
            TrackFeedback::Completed => "Completed",
        }
    }
}

/// Records of user feedback for tracks, used to inform AI curation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub track_id: String,
    pub track_artist: String,
    pub track_genre: String,
    pub feedback: TrackFeedback,
    pub recorded_at_ms: i64,
}

/// Decides whether a track should be recommended again based on the
/// user's feedback history. A disliked track should not be recommended;
/// a skipped track may be tried again after a cooldown.
pub fn should_recommend_track(
    feedback_history: &[FeedbackRecord],
    track_id: &str,
    now_ms: i64,
    skip_cooldown_ms: i64,
) -> bool {
    for record in feedback_history {
        if record.track_id == track_id {
            match record.feedback {
                TrackFeedback::Disliked => return false,
                TrackFeedback::Skipped => {
                    // Don't re-recommend a skipped track until the cooldown
                    // has passed.
                    if now_ms - record.recorded_at_ms < skip_cooldown_ms {
                        return false;
                    }
                }
                _ => {}
            }
        }
    }
    true
}

// ─── Aggregated music status ───────────────────────────────────────────────

/// A snapshot of the full music system state, suitable for returning via
/// FFI to the UI layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicStatus {
    pub playback_state: PlaybackState,
    pub audio_focus: AudioFocusState,
    pub coaching_interop: CoachingInteropState,
    pub music_source: MusicSource,
    pub music_mode: MusicMode,
    pub network: NetworkState,
    pub volume_multiplier: f64,
    pub current_track_index: Option<usize>,
    pub playlist_track_count: usize,
    pub playlist_total_duration_ms: i64,
}

impl Default for MusicStatus {
    fn default() -> Self {
        MusicStatus {
            playback_state: PlaybackState::Idle,
            audio_focus: AudioFocusState::NoFocus,
            coaching_interop: CoachingInteropState::Inactive,
            music_source: MusicSource::LocalDevice,
            music_mode: MusicMode::Manual,
            network: NetworkState::Connected,
            volume_multiplier: 1.0,
            current_track_index: None,
            playlist_track_count: 0,
            playlist_total_duration_ms: 0,
        }
    }
}

/// Builds a full music status snapshot from the current state.
pub fn build_status(
    playback: PlaybackState,
    focus: AudioFocusState,
    coaching: CoachingInteropState,
    source: MusicSource,
    mode: MusicMode,
    network: NetworkState,
    current_track_index: Option<usize>,
    playlist: Option<&Playlist>,
) -> MusicStatus {
    let (track_count, total_duration) = match playlist {
        Some(p) => (p.track_count(), p.total_duration_ms()),
        None => (0, 0),
    };

    // If coaching is active, the effective volume is 0 (music paused).
    let volume = if coaching == CoachingInteropState::CoachingActive {
        0.0
    } else {
        focus.volume_multiplier()
    };

    MusicStatus {
        playback_state: playback,
        audio_focus: focus,
        coaching_interop: coaching,
        music_source: source,
        music_mode: mode,
        network,
        volume_multiplier: volume,
        current_track_index,
        playlist_track_count: track_count,
        playlist_total_duration_ms: total_duration,
    }
}

// ─── Lock-screen / Bluetooth headset controls ──────────────────────────────

/// The type of remote control that issued a playback command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteControlSource {
    InApp,
    LockScreen,
    BluetoothHeadset,
    WearOs,
    Notification,
}

impl RemoteControlSource {
    /// Returns a human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            RemoteControlSource::InApp => "In-app",
            RemoteControlSource::LockScreen => "Lock screen",
            RemoteControlSource::BluetoothHeadset => "Bluetooth headset",
            RemoteControlSource::WearOs => "Wear OS",
            RemoteControlSource::Notification => "Notification",
        }
    }
}

/// Processes a remote control command from a lock screen, Bluetooth
/// headset, or other remote source. Returns the resulting transition.
pub fn handle_remote_control(
    current: PlaybackState,
    command: PlaybackCommand,
    _source: RemoteControlSource,
) -> TransitionResult {
    // Remote controls use the same transition logic as in-app controls.
    transition(current, command)
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn track(id: &str, artist: &str, genre: &str) -> Track {
        Track {
            id: id.to_string(),
            title: format!("Track {}", id),
            artist: artist.to_string(),
            genre: genre.to_string(),
            duration_ms: 180_000,
            source: MusicSource::LocalDevice,
        }
    }

    fn playlist(tracks: Vec<Track>) -> Playlist {
        Playlist {
            id: "pl1".to_string(),
            name: "Test Playlist".to_string(),
            tracks,
            source: MusicSource::LocalDevice,
            mode: MusicMode::Manual,
        }
    }

    // ── PlaybackState ──

    #[test]
    fn default_playback_state_is_idle() {
        assert_eq!(PlaybackState::default(), PlaybackState::Idle);
    }

    #[test]
    fn playing_state_is_playing() {
        assert!(PlaybackState::Playing.is_playing());
    }

    #[test]
    fn non_playing_states_are_not_playing() {
        assert!(!PlaybackState::Idle.is_playing());
        assert!(!PlaybackState::Paused.is_playing());
        assert!(!PlaybackState::Buffering.is_playing());
        assert!(!PlaybackState::Stopped.is_playing());
    }

    #[test]
    fn paused_can_resume() {
        assert!(PlaybackState::Paused.can_resume());
        assert!(PlaybackState::Buffering.can_resume());
        assert!(PlaybackState::Ready.can_resume());
    }

    #[test]
    fn idle_cannot_resume() {
        assert!(!PlaybackState::Idle.can_resume());
        assert!(!PlaybackState::Stopped.can_resume());
        assert!(!PlaybackState::Ended.can_resume());
    }

    #[test]
    fn only_playing_can_pause() {
        assert!(PlaybackState::Playing.can_pause());
        assert!(!PlaybackState::Paused.can_pause());
        assert!(!PlaybackState::Idle.can_pause());
    }

    #[test]
    fn terminal_states_are_terminal() {
        assert!(PlaybackState::Stopped.is_terminal());
        assert!(PlaybackState::Ended.is_terminal());
        assert!(PlaybackState::Error.is_terminal());
    }

    #[test]
    fn non_terminal_states_are_not_terminal() {
        assert!(!PlaybackState::Idle.is_terminal());
        assert!(!PlaybackState::Ready.is_terminal());
        assert!(!PlaybackState::Playing.is_terminal());
        assert!(!PlaybackState::Paused.is_terminal());
        assert!(!PlaybackState::Buffering.is_terminal());
    }

    #[test]
    fn playback_state_labels_are_non_empty() {
        for s in [
            PlaybackState::Idle,
            PlaybackState::Ready,
            PlaybackState::Playing,
            PlaybackState::Paused,
            PlaybackState::Buffering,
            PlaybackState::Ended,
            PlaybackState::Stopped,
            PlaybackState::Error,
        ] {
            assert!(!s.label().is_empty());
        }
    }

    // ── transition ──

    #[test]
    fn play_from_idle_starts_playing() {
        let r = transition(PlaybackState::Idle, PlaybackCommand::Play);
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Playing);
    }

    #[test]
    fn play_from_ready_starts_playing() {
        let r = transition(PlaybackState::Ready, PlaybackCommand::Play);
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Playing);
    }

    #[test]
    fn play_from_paused_resumes() {
        let r = transition(PlaybackState::Paused, PlaybackCommand::Play);
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Playing);
    }

    #[test]
    fn pause_from_playing_works() {
        let r = transition(PlaybackState::Playing, PlaybackCommand::Pause);
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Paused);
    }

    #[test]
    fn pause_from_idle_is_rejected() {
        let r = transition(PlaybackState::Idle, PlaybackCommand::Pause);
        assert!(!r.accepted);
        assert_eq!(r.new_state, PlaybackState::Idle);
    }

    #[test]
    fn pause_from_paused_is_rejected() {
        let r = transition(PlaybackState::Paused, PlaybackCommand::Pause);
        assert!(!r.accepted);
        assert_eq!(r.new_state, PlaybackState::Paused);
    }

    #[test]
    fn resume_from_paused_works() {
        let r = transition(PlaybackState::Paused, PlaybackCommand::Resume);
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Playing);
    }

    #[test]
    fn resume_from_stopped_is_rejected() {
        let r = transition(PlaybackState::Stopped, PlaybackCommand::Resume);
        assert!(!r.accepted);
        assert_eq!(r.new_state, PlaybackState::Stopped);
    }

    #[test]
    fn stop_from_any_state_works() {
        for s in [
            PlaybackState::Idle,
            PlaybackState::Ready,
            PlaybackState::Playing,
            PlaybackState::Paused,
            PlaybackState::Buffering,
            PlaybackState::Ended,
            PlaybackState::Error,
        ] {
            let r = transition(s, PlaybackCommand::Stop);
            assert!(r.accepted, "Stop from {:?} should be accepted", s);
            assert_eq!(r.new_state, PlaybackState::Stopped);
        }
    }

    #[test]
    fn seek_from_non_terminal_works() {
        let r = transition(PlaybackState::Playing, PlaybackCommand::Seek);
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Playing);
    }

    #[test]
    fn seek_from_terminal_is_rejected() {
        let r = transition(PlaybackState::Stopped, PlaybackCommand::Seek);
        assert!(!r.accepted);
    }

    #[test]
    fn skip_from_playing_works() {
        let r = transition(PlaybackState::Playing, PlaybackCommand::Skip);
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Playing);
    }

    #[test]
    fn skip_from_paused_resumes_and_skips() {
        let r = transition(PlaybackState::Paused, PlaybackCommand::Skip);
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Playing);
    }

    #[test]
    fn previous_from_playing_works() {
        let r = transition(PlaybackState::Playing, PlaybackCommand::Previous);
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Playing);
    }

    #[test]
    fn invalid_transition_keeps_state() {
        let r = transition(PlaybackState::Idle, PlaybackCommand::Resume);
        assert!(!r.accepted);
        assert_eq!(r.new_state, PlaybackState::Idle);
    }

    #[test]
    fn transition_result_has_message() {
        let r = transition(PlaybackState::Idle, PlaybackCommand::Play);
        assert!(!r.message.is_empty());
    }

    // ── AudioFocusState ──

    #[test]
    fn default_audio_focus_is_no_focus() {
        assert_eq!(AudioFocusState::default(), AudioFocusState::NoFocus);
    }

    #[test]
    fn focused_can_play() {
        assert!(AudioFocusState::Focused.can_play());
        assert!(AudioFocusState::Ducking.can_play());
    }

    #[test]
    fn no_focus_cannot_play() {
        assert!(!AudioFocusState::NoFocus.can_play());
        assert!(!AudioFocusState::TransientPause.can_play());
        assert!(!AudioFocusState::Lost.can_play());
    }

    #[test]
    fn focused_volume_is_full() {
        assert_eq!(AudioFocusState::Focused.volume_multiplier(), 1.0);
    }

    #[test]
    fn ducking_volume_is_reduced() {
        assert_eq!(AudioFocusState::Ducking.volume_multiplier(), 0.2);
    }

    #[test]
    fn no_focus_volume_is_zero() {
        assert_eq!(AudioFocusState::NoFocus.volume_multiplier(), 0.0);
        assert_eq!(AudioFocusState::Lost.volume_multiplier(), 0.0);
        assert_eq!(AudioFocusState::TransientPause.volume_multiplier(), 0.0);
    }

    #[test]
    fn audio_focus_labels_are_non_empty() {
        for s in [
            AudioFocusState::NoFocus,
            AudioFocusState::Focused,
            AudioFocusState::Ducking,
            AudioFocusState::TransientPause,
            AudioFocusState::Lost,
        ] {
            assert!(!s.label().is_empty());
        }
    }

    // ── handle_audio_focus_event ──

    #[test]
    fn gain_event_grants_focus_and_resumes() {
        let (new, action) =
            handle_audio_focus_event(AudioFocusState::NoFocus, AudioFocusEvent::Gain);
        assert_eq!(new, AudioFocusState::Focused);
        assert_eq!(action, PlaybackAction::Resume);
    }

    #[test]
    fn duck_event_ducks() {
        let (new, action) =
            handle_audio_focus_event(AudioFocusState::Focused, AudioFocusEvent::Duck);
        assert_eq!(new, AudioFocusState::Ducking);
        assert_eq!(action, PlaybackAction::Duck);
    }

    #[test]
    fn transient_pause_event_pauses() {
        let (new, action) =
            handle_audio_focus_event(AudioFocusState::Focused, AudioFocusEvent::TransientPause);
        assert_eq!(new, AudioFocusState::TransientPause);
        assert_eq!(action, PlaybackAction::Pause);
    }

    #[test]
    fn loss_event_stops() {
        let (new, action) =
            handle_audio_focus_event(AudioFocusState::Focused, AudioFocusEvent::Loss);
        assert_eq!(new, AudioFocusState::Lost);
        assert_eq!(action, PlaybackAction::Stop);
    }

    // ── Coaching-interop ──

    #[test]
    fn default_coaching_interop_is_inactive() {
        assert_eq!(
            CoachingInteropState::default(),
            CoachingInteropState::Inactive
        );
    }

    #[test]
    fn coaching_start_pauses_music() {
        let r = handle_coaching_request(
            CoachingInteropState::Inactive,
            CoachingRequest::PromptStarting,
            true,
        );
        assert_eq!(r.state, CoachingInteropState::CoachingActive);
        assert_eq!(r.music_action, PlaybackAction::Pause);
        assert!(r.was_playing_before);
    }

    #[test]
    fn coaching_start_when_not_playing_does_not_pause() {
        let r = handle_coaching_request(
            CoachingInteropState::Inactive,
            CoachingRequest::PromptStarting,
            false,
        );
        assert_eq!(r.music_action, PlaybackAction::Continue);
        assert!(!r.was_playing_before);
    }

    #[test]
    fn coaching_finish_resumes_music() {
        let r = handle_coaching_request(
            CoachingInteropState::CoachingActive,
            CoachingRequest::PromptFinished,
            true,
        );
        assert_eq!(r.state, CoachingInteropState::CoachingFinished);
        assert_eq!(r.music_action, PlaybackAction::Resume);
    }

    #[test]
    fn coaching_cancel_does_not_resume() {
        let r = handle_coaching_request(
            CoachingInteropState::CoachingActive,
            CoachingRequest::PromptCancelled,
            true,
        );
        assert_eq!(r.music_action, PlaybackAction::Continue);
    }

    #[test]
    fn coaching_finish_without_active_coaching_does_not_resume() {
        let r = handle_coaching_request(
            CoachingInteropState::Inactive,
            CoachingRequest::PromptFinished,
            true,
        );
        assert_eq!(r.music_action, PlaybackAction::Continue);
    }

    // ── MusicSource ──

    #[test]
    fn local_device_does_not_require_network() {
        assert!(!MusicSource::LocalDevice.requires_network());
    }

    #[test]
    fn internet_radio_requires_network() {
        assert!(MusicSource::InternetRadio.requires_network());
    }

    #[test]
    fn ai_curated_requires_network() {
        assert!(MusicSource::AiCurated.requires_network());
    }

    #[test]
    fn music_source_labels_are_non_empty() {
        for s in [
            MusicSource::LocalDevice,
            MusicSource::InternetRadio,
            MusicSource::AiCurated,
        ] {
            assert!(!s.label().is_empty());
        }
    }

    // ── MusicMode ──

    #[test]
    fn default_music_mode_is_manual() {
        assert_eq!(MusicMode::default(), MusicMode::Manual);
    }

    #[test]
    fn music_mode_labels_are_non_empty() {
        assert!(!MusicMode::Manual.label().is_empty());
        assert!(!MusicMode::Ai.label().is_empty());
    }

    // ── Network-loss ──

    #[test]
    fn network_loss_local_source_is_unaffected() {
        let d = handle_network_loss(
            NetworkState::Lost,
            MusicSource::LocalDevice,
            false,
            0,
        );
        assert!(!d.should_stop);
        assert!(!d.should_switch_to_local);
        assert!(!d.should_buffer);
    }

    #[test]
    fn network_connected_keeps_streaming() {
        let d = handle_network_loss(
            NetworkState::Connected,
            MusicSource::InternetRadio,
            false,
            0,
        );
        assert!(!d.should_stop);
        assert!(!d.should_switch_to_local);
    }

    #[test]
    fn network_lost_with_local_media_switches() {
        let d = handle_network_loss(
            NetworkState::Lost,
            MusicSource::InternetRadio,
            true,
            0,
        );
        assert!(d.should_switch_to_local);
        assert!(!d.should_stop);
        assert_eq!(d.fallback_source, MusicSource::LocalDevice);
    }

    #[test]
    fn network_lost_without_local_media_stops() {
        let d = handle_network_loss(
            NetworkState::Lost,
            MusicSource::InternetRadio,
            false,
            0,
        );
        assert!(d.should_stop);
        assert!(!d.should_switch_to_local);
    }

    #[test]
    fn network_weak_with_high_buffer_buffers() {
        let d = handle_network_loss(
            NetworkState::Weak,
            MusicSource::InternetRadio,
            false,
            10000,
        );
        assert!(d.should_buffer);
        assert!(!d.should_switch_to_local);
        assert!(!d.should_stop);
    }

    #[test]
    fn network_weak_with_low_buffer_and_local_switches() {
        let d = handle_network_loss(
            NetworkState::Weak,
            MusicSource::InternetRadio,
            true,
            1000,
        );
        assert!(d.should_switch_to_local);
        assert_eq!(d.fallback_source, MusicSource::LocalDevice);
    }

    #[test]
    fn network_weak_with_low_buffer_no_local_buffers() {
        let d = handle_network_loss(
            NetworkState::Weak,
            MusicSource::InternetRadio,
            false,
            1000,
        );
        assert!(d.should_buffer);
        assert!(!d.should_switch_to_local);
    }

    // ── Blocked content ──

    #[test]
    fn filter_removes_blocked_artist() {
        let pl = playlist(vec![
            track("1", "Good Artist", "Rock"),
            track("2", "Bad Artist", "Rock"),
        ]);
        let (filtered, removed) =
            filter_blocked_content(&pl, &["Bad Artist".to_string()], &[]);
        assert_eq!(removed, 1);
        assert_eq!(filtered.track_count(), 1);
        assert_eq!(filtered.tracks[0].id, "1");
    }

    #[test]
    fn filter_removes_blocked_genre() {
        let pl = playlist(vec![
            track("1", "Artist A", "Rock"),
            track("2", "Artist B", "Jazz"),
        ]);
        let (filtered, removed) =
            filter_blocked_content(&pl, &[], &["Jazz".to_string()]);
        assert_eq!(removed, 1);
        assert_eq!(filtered.track_count(), 1);
    }

    #[test]
    fn filter_case_insensitive_artist() {
        let pl = playlist(vec![track("1", "Bad Artist", "Rock")]);
        let (filtered, removed) =
            filter_blocked_content(&pl, &["BAD ARTIST".to_string()], &[]);
        assert_eq!(removed, 1);
        assert_eq!(filtered.track_count(), 0);
    }

    #[test]
    fn filter_no_blocked_returns_all() {
        let pl = playlist(vec![
            track("1", "Artist A", "Rock"),
            track("2", "Artist B", "Jazz"),
        ]);
        let (filtered, removed) = filter_blocked_content(&pl, &[], &[]);
        assert_eq!(removed, 0);
        assert_eq!(filtered.track_count(), 2);
    }

    #[test]
    fn is_track_blocked_checks_artist() {
        let t = track("1", "Bad Artist", "Rock");
        assert!(is_track_blocked(&t, &["Bad Artist".to_string()], &[]));
        assert!(!is_track_blocked(&t, &["Other Artist".to_string()], &[]));
    }

    #[test]
    fn is_track_blocked_checks_genre() {
        let t = track("1", "Artist A", "Jazz");
        assert!(is_track_blocked(&t, &[], &["Jazz".to_string()]));
        assert!(!is_track_blocked(&t, &[], &["Rock".to_string()]));
    }

    // ── Likes and skips ──

    #[test]
    fn liked_is_positive() {
        assert!(TrackFeedback::Liked.is_positive());
        assert!(TrackFeedback::Completed.is_positive());
        assert!(!TrackFeedback::Disliked.is_positive());
        assert!(!TrackFeedback::Skipped.is_positive());
    }

    #[test]
    fn track_feedback_labels_are_non_empty() {
        for f in [
            TrackFeedback::Liked,
            TrackFeedback::Disliked,
            TrackFeedback::Skipped,
            TrackFeedback::Completed,
        ] {
            assert!(!f.label().is_empty());
        }
    }

    #[test]
    fn disliked_track_not_recommended() {
        let history = vec![FeedbackRecord {
            track_id: "t1".to_string(),
            track_artist: "Artist A".to_string(),
            track_genre: "Rock".to_string(),
            feedback: TrackFeedback::Disliked,
            recorded_at_ms: 0,
        }];
        assert!(!should_recommend_track(&history, "t1", 100000, 60000));
    }

    #[test]
    fn liked_track_recommended() {
        let history = vec![FeedbackRecord {
            track_id: "t1".to_string(),
            track_artist: "Artist A".to_string(),
            track_genre: "Rock".to_string(),
            feedback: TrackFeedback::Liked,
            recorded_at_ms: 0,
        }];
        assert!(should_recommend_track(&history, "t1", 100000, 60000));
    }

    #[test]
    fn skipped_track_not_recommended_during_cooldown() {
        let history = vec![FeedbackRecord {
            track_id: "t1".to_string(),
            track_artist: "Artist A".to_string(),
            track_genre: "Rock".to_string(),
            feedback: TrackFeedback::Skipped,
            recorded_at_ms: 0,
        }];
        assert!(!should_recommend_track(&history, "t1", 30000, 60000));
    }

    #[test]
    fn skipped_track_recommended_after_cooldown() {
        let history = vec![FeedbackRecord {
            track_id: "t1".to_string(),
            track_artist: "Artist A".to_string(),
            track_genre: "Rock".to_string(),
            feedback: TrackFeedback::Skipped,
            recorded_at_ms: 0,
        }];
        assert!(should_recommend_track(&history, "t1", 70000, 60000));
    }

    #[test]
    fn unknown_track_recommended() {
        let history = vec![FeedbackRecord {
            track_id: "t1".to_string(),
            track_artist: "Artist A".to_string(),
            track_genre: "Rock".to_string(),
            feedback: TrackFeedback::Disliked,
            recorded_at_ms: 0,
        }];
        assert!(should_recommend_track(&history, "t2", 100000, 60000));
    }

    // ── Playlist ──

    #[test]
    fn playlist_total_duration_sums_tracks() {
        let pl = playlist(vec![
            track("1", "A", "Rock"),
            track("2", "B", "Jazz"),
        ]);
        assert_eq!(pl.total_duration_ms(), 360_000);
    }

    #[test]
    fn playlist_track_count() {
        let pl = playlist(vec![track("1", "A", "Rock"), track("2", "B", "Jazz")]);
        assert_eq!(pl.track_count(), 2);
    }

    #[test]
    fn empty_playlist_is_empty() {
        let pl = playlist(vec![]);
        assert!(pl.is_empty());
        assert_eq!(pl.track_count(), 0);
        assert_eq!(pl.total_duration_ms(), 0);
    }

    // ── MusicStatus ──

    #[test]
    fn default_music_status_is_idle() {
        let s = MusicStatus::default();
        assert_eq!(s.playback_state, PlaybackState::Idle);
        assert_eq!(s.audio_focus, AudioFocusState::NoFocus);
        assert_eq!(s.coaching_interop, CoachingInteropState::Inactive);
    }

    #[test]
    fn build_status_with_coaching_active_has_zero_volume() {
        let s = build_status(
            PlaybackState::Paused,
            AudioFocusState::Focused,
            CoachingInteropState::CoachingActive,
            MusicSource::LocalDevice,
            MusicMode::Manual,
            NetworkState::Connected,
            None,
            None,
        );
        assert_eq!(s.volume_multiplier, 0.0);
    }

    #[test]
    fn build_status_without_coaching_uses_focus_volume() {
        let s = build_status(
            PlaybackState::Playing,
            AudioFocusState::Focused,
            CoachingInteropState::Inactive,
            MusicSource::LocalDevice,
            MusicMode::Manual,
            NetworkState::Connected,
            None,
            None,
        );
        assert_eq!(s.volume_multiplier, 1.0);
    }

    #[test]
    fn build_status_with_playlist_has_track_count() {
        let pl = playlist(vec![track("1", "A", "Rock"), track("2", "B", "Jazz")]);
        let s = build_status(
            PlaybackState::Playing,
            AudioFocusState::Focused,
            CoachingInteropState::Inactive,
            MusicSource::LocalDevice,
            MusicMode::Manual,
            NetworkState::Connected,
            Some(0),
            Some(&pl),
        );
        assert_eq!(s.playlist_track_count, 2);
        assert_eq!(s.playlist_total_duration_ms, 360_000);
    }

    #[test]
    fn build_status_without_playlist_has_zero_count() {
        let s = build_status(
            PlaybackState::Playing,
            AudioFocusState::Focused,
            CoachingInteropState::Inactive,
            MusicSource::LocalDevice,
            MusicMode::Manual,
            NetworkState::Connected,
            None,
            None,
        );
        assert_eq!(s.playlist_track_count, 0);
        assert_eq!(s.playlist_total_duration_ms, 0);
    }

    // ── Remote control ──

    #[test]
    fn remote_control_play_from_idle() {
        let r = handle_remote_control(
            PlaybackState::Idle,
            PlaybackCommand::Play,
            RemoteControlSource::LockScreen,
        );
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Playing);
    }

    #[test]
    fn remote_control_pause_from_playing() {
        let r = handle_remote_control(
            PlaybackState::Playing,
            PlaybackCommand::Pause,
            RemoteControlSource::BluetoothHeadset,
        );
        assert!(r.accepted);
        assert_eq!(r.new_state, PlaybackState::Paused);
    }

    // ── Serialization ──

    #[test]
    fn playback_state_serializes_round_trip() {
        for s in [
            PlaybackState::Idle,
            PlaybackState::Ready,
            PlaybackState::Playing,
            PlaybackState::Paused,
            PlaybackState::Buffering,
            PlaybackState::Ended,
            PlaybackState::Stopped,
            PlaybackState::Error,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: PlaybackState = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn audio_focus_state_serializes_round_trip() {
        for s in [
            AudioFocusState::NoFocus,
            AudioFocusState::Focused,
            AudioFocusState::Ducking,
            AudioFocusState::TransientPause,
            AudioFocusState::Lost,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: AudioFocusState = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn transition_result_serializes() {
        let r = transition(PlaybackState::Idle, PlaybackCommand::Play);
        let json = serde_json::to_string(&r).unwrap();
        let back: TransitionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(r.accepted, back.accepted);
        assert_eq!(r.new_state, back.new_state);
    }

    #[test]
    fn network_loss_decision_serializes() {
        let d = handle_network_loss(
            NetworkState::Lost,
            MusicSource::InternetRadio,
            true,
            0,
        );
        let json = serde_json::to_string(&d).unwrap();
        let back: NetworkLossDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(d.should_stop, back.should_stop);
        assert_eq!(d.should_switch_to_local, back.should_switch_to_local);
    }

    #[test]
    fn music_status_serializes() {
        let s = MusicStatus::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: MusicStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(s.playback_state, back.playback_state);
        assert_eq!(s.volume_multiplier, back.volume_multiplier);
    }

    #[test]
    fn remote_control_source_labels_are_non_empty() {
        for s in [
            RemoteControlSource::InApp,
            RemoteControlSource::LockScreen,
            RemoteControlSource::BluetoothHeadset,
            RemoteControlSource::WearOs,
            RemoteControlSource::Notification,
        ] {
            assert!(!s.label().is_empty());
        }
    }
}

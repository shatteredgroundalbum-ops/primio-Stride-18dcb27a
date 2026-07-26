//! §19 — Store and release readiness.
//!
//! This module implements the decision logic for the store and release
//! readiness checks that must be satisfied before the app can be
//! published to Google Play:
//!
//!   - App identity (final app name, package ID)
//!   - Signing key (securely backed up, rotation plan)
//!   - Release bundle (AAB status, build configuration)
//!   - Versioning policy (semantic versioning, version code scheme)
//!   - Store assets (adaptive launcher icon, splash, screenshots,
//!     feature graphic)
//!   - Store listing (description, content rating, category)
//!   - Data Safety declaration (links to §18 privacy)
//!   - Privacy policy URL
//!   - Reviewer access instructions
//!   - Permission declarations (justification for each permission)
//!   - Background-location justification
//!   - Testing tracks (internal testing, closed testing, staged
//!     production rollout)
//!   - Production stack inventory (Firebase, Cloud, Crashlytics, etc.)
//!
//! Like all engine modules, the types here are pure decision/state
//! structures — they serialize to JSON, cross the FFI boundary as
//! C strings, and are deserialized on the Dart side.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════
// App identity
// ═══════════════════════════════════════════════════════════════════════

/// The final, non-changeable app identity used across the store listing,
/// the Android manifest, and the release pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppNameStatus {
    /// The name is a placeholder and has not been finalised.
    Draft,
    /// The name has been chosen but not yet locked.
    Proposed,
    /// The name is final and locked for the store listing.
    Final,
}

impl AppNameStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Proposed => "Proposed",
            Self::Final => "Final",
        }
    }

    pub fn is_final(&self) -> bool {
        matches!(self, Self::Final)
    }
}

/// The application's identity — the immutable store-level identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppIdentity {
    /// The final display name of the app on the store listing.
    pub app_name: String,
    /// The status of the name (draft/proposed/final).
    pub name_status: AppNameStatus,
    /// The Android application ID (e.g. `com.example.stride`).
    pub package_id: String,
    /// The application's version name (e.g. `1.0.0`).
    pub version_name: String,
    /// The integer version code (monotonically increasing).
    pub version_code: u32,
    /// The minimum SDK level required.
    pub min_sdk: u32,
    /// The target SDK level.
    pub target_sdk: u32,
}

impl AppIdentity {
    pub fn new() -> Self {
        Self {
            app_name: String::from("S.T.R.I.D.E."),
            name_status: AppNameStatus::Draft,
            package_id: String::from("com.stride.app"),
            version_name: String::from("1.0.0"),
            version_code: 1,
            min_sdk: 26,
            target_sdk: 34,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = name.into();
        self
    }

    pub fn with_package_id(mut self, id: impl Into<String>) -> Self {
        self.package_id = id.into();
        self
    }

    pub fn finalize(mut self) -> Self {
        self.name_status = AppNameStatus::Final;
        self
    }

    pub fn is_ready(&self) -> bool {
        self.name_status.is_final()
            && !self.app_name.is_empty()
            && !self.package_id.is_empty()
            && self.package_id.contains('.')
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Signing key
// ═══════════════════════════════════════════════════════════════════════

/// The status of the app's signing key backup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SigningKeyStatus {
    /// The key has not been generated yet.
    NotGenerated,
    /// The key has been generated but not backed up.
    Generated,
    /// The key has been backed up to a secure location.
    BackedUp,
    /// The key has been enrolled in Google Play App Signing.
    Enrolled,
}

impl SigningKeyStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotGenerated => "Not Generated",
            Self::Generated => "Generated",
            Self::BackedUp => "Backed Up",
            Self::Enrolled => "Enrolled in Play App Signing",
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Enrolled)
    }

    pub fn is_secure(&self) -> bool {
        matches!(self, Self::BackedUp | Self::Enrolled)
    }
}

/// The signing key configuration and backup status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningKeyConfig {
    /// The current status of the key.
    pub status: SigningKeyStatus,
    /// The key algorithm (e.g. RSA-2048).
    pub algorithm: String,
    /// Whether the key has a backup in a secure location.
    pub has_backup: bool,
    /// Whether the key is enrolled in Google Play App Signing.
    pub is_enrolled_in_play: bool,
    /// Whether the key rotation (v3+ signing) is enabled.
    pub rotation_enabled: bool,
}

impl SigningKeyConfig {
    pub fn new() -> Self {
        Self {
            status: SigningKeyStatus::NotGenerated,
            algorithm: String::from("RSA-2048"),
            has_backup: false,
            is_enrolled_in_play: false,
            rotation_enabled: true,
        }
    }

    pub fn generate(mut self) -> Self {
        self.status = SigningKeyStatus::Generated;
        self
    }

    pub fn backup(mut self) -> Self {
        self.has_backup = true;
        self.status = SigningKeyStatus::BackedUp;
        self
    }

    pub fn enroll(mut self) -> Self {
        self.is_enrolled_in_play = true;
        self.status = SigningKeyStatus::Enrolled;
        self
    }

    pub fn is_ready(&self) -> bool {
        self.status.is_ready()
    }

    pub fn is_secure(&self) -> bool {
        self.status.is_secure()
    }
}

impl Default for SigningKeyConfig {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Release bundle (AAB)
// ═══════════════════════════════════════════════════════════════════════

/// The build type for the release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildType {
    /// Debug build for development.
    Debug,
    /// Profile build for performance testing.
    Profile,
    /// Release build for production.
    Release,
}

impl BuildType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Debug => "Debug",
            Self::Profile => "Profile",
            Self::Release => "Release",
        }
    }

    pub fn is_release(self) -> bool {
        matches!(self, Self::Release)
    }
}

/// The status of the release AAB bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseBundleStatus {
    /// The AAB has not been built yet.
    NotBuilt,
    /// The AAB is being built.
    Building,
    /// The AAB has been built and is awaiting upload.
    Built,
    /// The AAB has been uploaded to Google Play Console.
    Uploaded,
    /// The AAB has been approved and is ready for release.
    Approved,
}

impl ReleaseBundleStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotBuilt => "Not Built",
            Self::Building => "Building",
            Self::Built => "Built",
            Self::Uploaded => "Uploaded",
            Self::Approved => "Approved",
        }
    }

    pub fn is_ready_for_upload(self) -> bool {
        matches!(self, Self::Built | Self::Uploaded | Self::Approved)
    }

    pub fn is_ready_for_release(&self) -> bool {
        matches!(self, Self::Approved)
    }
}

/// The release bundle (AAB) configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseBundle {
    /// The build type.
    pub build_type: BuildType,
    /// The bundle status.
    pub status: ReleaseBundleStatus,
    /// The size of the AAB in bytes.
    pub size_bytes: u64,
    /// Whether the bundle is minified (R8/ProGuard).
    pub is_minified: bool,
    /// Whether the bundle has been signed.
    pub is_signed: bool,
    /// The SHA-256 hash of the bundle.
    pub sha256: String,
    /// The file name of the bundle.
    pub file_name: String,
}

impl ReleaseBundle {
    pub fn new() -> Self {
        Self {
            build_type: BuildType::Release,
            status: ReleaseBundleStatus::NotBuilt,
            size_bytes: 0,
            is_minified: true,
            is_signed: false,
            sha256: String::new(),
            file_name: String::from("app-release.aab"),
        }
    }

    pub fn build(mut self, size_bytes: u64, sha256: impl Into<String>) -> Self {
        self.status = ReleaseBundleStatus::Built;
        self.size_bytes = size_bytes;
        self.sha256 = sha256.into();
        self
    }

    pub fn sign(mut self) -> Self {
        self.is_signed = true;
        self
    }

    pub fn upload(mut self) -> Self {
        self.status = ReleaseBundleStatus::Uploaded;
        self
    }

    pub fn approve(mut self) -> Self {
        self.status = ReleaseBundleStatus::Approved;
        self
    }

    pub fn is_ready_for_release(&self) -> bool {
        self.status.is_ready_for_release() && self.is_signed && self.is_minified
    }
}

impl Default for ReleaseBundle {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Versioning policy
// ═══════════════════════════════════════════════════════════════════════

/// The versioning strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersioningStrategy {
    /// Semantic versioning (major.minor.patch).
    Semantic,
    /// Calendar versioning (year.month.day).
    Calendar,
    /// Sequential versioning (incrementing integer).
    Sequential,
}

impl VersioningStrategy {
    pub fn label(self) -> &'static str {
        match self {
            Self::Semantic => "Semantic Versioning",
            Self::Calendar => "Calendar Versioning",
            Self::Sequential => "Sequential Versioning",
        }
    }
}

/// The versioning policy for the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersioningPolicy {
    /// The versioning strategy.
    pub strategy: VersioningStrategy,
    /// The current major version.
    pub major: u32,
    /// The current minor version.
    pub minor: u32,
    /// The current patch version.
    pub patch: u32,
    /// The current version code (for Android).
    pub version_code: u32,
    /// The minimum version code for compatibility.
    pub min_version_code: u32,
    /// Whether pre-release versions are allowed.
    pub allow_pre_release: bool,
}

impl VersioningPolicy {
    pub fn new() -> Self {
        Self {
            strategy: VersioningStrategy::Semantic,
            major: 1,
            minor: 0,
            patch: 0,
            version_code: 1,
            min_version_code: 1,
            allow_pre_release: false,
        }
    }

    pub fn version_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn next_version_code(&self) -> u32 {
        self.version_code + 1
    }

    pub fn is_compatible(&self, code: u32) -> bool {
        code >= self.min_version_code
    }
}

impl Default for VersioningPolicy {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Store assets
// ═══════════════════════════════════════════════════════════════════════

/// The type of store asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreAssetType {
    /// Adaptive launcher icon (foreground + background).
    AdaptiveLauncherIcon,
    /// Legacy launcher icon.
    LegacyLauncherIcon,
    /// Splash screen assets.
    SplashScreen,
    /// Store phone screenshot.
    PhoneScreenshot,
    /// Store 7-inch tablet screenshot.
    Tablet7InchScreenshot,
    /// Store 10-inch tablet screenshot.
    Tablet10InchScreenshot,
    /// Feature graphic (1024x500).
    FeatureGraphic,
    /// App icon for Play Console (512x512).
    PlayStoreIcon,
}

impl StoreAssetType {
    pub fn label(self) -> &'static str {
        match self {
            Self::AdaptiveLauncherIcon => "Adaptive Launcher Icon",
            Self::LegacyLauncherIcon => "Legacy Launcher Icon",
            Self::SplashScreen => "Splash Screen",
            Self::PhoneScreenshot => "Phone Screenshot",
            Self::Tablet7InchScreenshot => "7-inch Tablet Screenshot",
            Self::Tablet10InchScreenshot => "10-inch Tablet Screenshot",
            Self::FeatureGraphic => "Feature Graphic",
            Self::PlayStoreIcon => "Play Store Icon",
        }
    }

    pub fn is_required(self) -> bool {
        matches!(
            self,
            Self::AdaptiveLauncherIcon
                | Self::SplashScreen
                | Self::PhoneScreenshot
                | Self::FeatureGraphic
                | Self::PlayStoreIcon
        )
    }
}

/// The status of a store asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    /// The asset has not been created yet.
    NotCreated,
    /// The asset is a draft/placeholder.
    Draft,
    /// The asset has been finalised and is ready for the store.
    Final,
    /// The asset has been uploaded to the Play Console.
    Uploaded,
}

impl AssetStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotCreated => "Not Created",
            Self::Draft => "Draft",
            Self::Final => "Final",
            Self::Uploaded => "Uploaded",
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Final | Self::Uploaded)
    }
}

/// A store asset entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreAsset {
    /// The asset type.
    pub asset_type: StoreAssetType,
    /// The status of the asset.
    pub status: AssetStatus,
    /// The width in pixels.
    pub width_px: u32,
    /// The height in pixels.
    pub height_px: u32,
    /// The file format (e.g. PNG, JPEG).
    pub file_format: String,
    /// Whether the asset meets the store requirements.
    pub meets_requirements: bool,
}

impl StoreAsset {
    pub fn new(asset_type: StoreAssetType) -> Self {
        let (width, height) = match asset_type {
            StoreAssetType::AdaptiveLauncherIcon => (432, 432),
            StoreAssetType::LegacyLauncherIcon => (192, 192),
            StoreAssetType::SplashScreen => (1080, 1920),
            StoreAssetType::PhoneScreenshot => (1080, 1920),
            StoreAssetType::Tablet7InchScreenshot => (1200, 1920),
            StoreAssetType::Tablet10InchScreenshot => (1920, 1200),
            StoreAssetType::FeatureGraphic => (1024, 500),
            StoreAssetType::PlayStoreIcon => (512, 512),
        };
        Self {
            asset_type,
            status: AssetStatus::NotCreated,
            width_px: width,
            height_px: height,
            file_format: String::from("PNG"),
            meets_requirements: false,
        }
    }

    pub fn finalize(mut self) -> Self {
        self.status = AssetStatus::Final;
        self.meets_requirements = true;
        self
    }

    pub fn upload(mut self) -> Self {
        self.status = AssetStatus::Uploaded;
        self.meets_requirements = true;
        self
    }

    pub fn is_ready(&self) -> bool {
        self.status.is_ready() && self.meets_requirements
    }
}

/// All store assets required for the listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreAssets {
    /// The adaptive launcher icon.
    pub launcher_icon: StoreAsset,
    /// The splash screen.
    pub splash_screen: StoreAsset,
    /// The feature graphic.
    pub feature_graphic: StoreAsset,
    /// The Play Store icon (512x512).
    pub play_store_icon: StoreAsset,
    /// Phone screenshots (at least 2 required by Google Play).
    pub phone_screenshots: Vec<StoreAsset>,
    /// Tablet screenshots (optional but recommended).
    pub tablet_screenshots: Vec<StoreAsset>,
}

impl StoreAssets {
    pub fn new() -> Self {
        Self {
            launcher_icon: StoreAsset::new(StoreAssetType::AdaptiveLauncherIcon),
            splash_screen: StoreAsset::new(StoreAssetType::SplashScreen),
            feature_graphic: StoreAsset::new(StoreAssetType::FeatureGraphic),
            play_store_icon: StoreAsset::new(StoreAssetType::PlayStoreIcon),
            phone_screenshots: Vec::new(),
            tablet_screenshots: Vec::new(),
        }
    }

    pub fn with_phone_screenshots(mut self, count: u32) -> Self {
        self.phone_screenshots = (0..count)
            .map(|_| StoreAsset::new(StoreAssetType::PhoneScreenshot))
            .collect();
        self
    }

    pub fn is_complete(&self) -> bool {
        self.launcher_icon.is_ready()
            && self.splash_screen.is_ready()
            && self.feature_graphic.is_ready()
            && self.play_store_icon.is_ready()
            && self.phone_screenshots.len() >= 2
    }
}

impl Default for StoreAssets {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Store listing
// ═══════════════════════════════════════════════════════════════════════

/// The Google Play content rating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentRating {
    /// Rated for all ages.
    Everyone,
    /// Rated for ages 10+.
    Everyone10Plus,
    /// Rated for teens (13+).
    Teen,
    /// Rated for mature audiences (17+).
    Mature,
    /// Rated for adults only (18+).
    Adult,
}

impl ContentRating {
    pub fn label(self) -> &'static str {
        match self {
            Self::Everyone => "Everyone",
            Self::Everyone10Plus => "Everyone 10+",
            Self::Teen => "Teen",
            Self::Mature => "Mature 17+",
            Self::Adult => "Adults Only 18+",
        }
    }

    pub fn is_appropriate_for_fitness(&self) -> bool {
        matches!(self, Self::Everyone | Self::Everyone10Plus | Self::Teen)
    }
}

/// The app category on Google Play.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppCategory {
    /// Health and Fitness category.
    HealthFitness,
    /// Sports category.
    Sports,
    /// Maps and Navigation.
    MapsNavigation,
    /// Lifestyle.
    Lifestyle,
}

impl AppCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::HealthFitness => "Health & Fitness",
            Self::Sports => "Sports",
            Self::MapsNavigation => "Maps & Navigation",
            Self::Lifestyle => "Lifestyle",
        }
    }
}

/// The store listing details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreListing {
    /// The short promotional description (80 char max).
    pub short_description: String,
    /// The full store description (4000 char max).
    pub full_description: String,
    /// The app category.
    pub category: AppCategory,
    /// The content rating.
    pub content_rating: ContentRating,
    /// Whether the content rating questionnaire is completed.
    pub content_rating_completed: bool,
    /// The privacy policy URL.
    pub privacy_policy_url: String,
    /// The support email address.
    pub support_email: String,
    /// The support website URL.
    pub support_website: String,
    /// Whether the listing contains deceptive or misleading content.
    pub is_compliant: bool,
}

impl StoreListing {
    pub fn new() -> Self {
        Self {
            short_description: String::new(),
            full_description: String::new(),
            category: AppCategory::HealthFitness,
            content_rating: ContentRating::Everyone,
            content_rating_completed: false,
            privacy_policy_url: String::new(),
            support_email: String::new(),
            support_website: String::new(),
            is_compliant: true,
        }
    }

    pub fn with_short_description(mut self, desc: impl Into<String>) -> Self {
        self.short_description = desc.into();
        self
    }

    pub fn with_full_description(mut self, desc: impl Into<String>) -> Self {
        self.full_description = desc.into();
        self
    }

    pub fn with_privacy_url(mut self, url: impl Into<String>) -> Self {
        self.privacy_policy_url = url.into();
        self
    }

    pub fn complete_content_rating(mut self) -> Self {
        self.content_rating_completed = true;
        self
    }

    pub fn is_ready(&self) -> bool {
        !self.short_description.is_empty()
            && !self.full_description.is_empty()
            && !self.privacy_policy_url.is_empty()
            && self.content_rating_completed
            && self.content_rating.is_appropriate_for_fitness()
            && self.is_compliant
    }
}

impl Default for StoreListing {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Permission declarations
// ═══════════════════════════════════════════════════════════════════════

/// The type of Android permission requiring a justification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionType {
    /// Fine location (GPS tracking during activity).
    AccessFineLocation,
    /// Coarse location.
    AccessCoarseLocation,
    /// Background location (for continuous tracking).
    AccessBackgroundLocation,
    /// Body sensors (heart rate from wearable).
    BodySensors,
    /// Activity recognition (step counter).
    ActivityRecognition,
    /// Post notifications (for notification channels).
    PostNotifications,
    /// Camera (for route photos).
    Camera,
    /// Microphone (not used, declared for transparency).
    RecordAudio,
    /// Read external storage.
    ReadExternalStorage,
    /// Write external storage (for GPX export).
    WriteExternalStorage,
    /// Bluetooth (for wearable connectivity).
    Bluetooth,
    /// Internet (for maps, cloud sync).
    Internet,
    /// Access network state.
    AccessNetworkState,
    /// Foreground service (for the tracking service).
    ForegroundService,
    /// Wake lock (to keep CPU active during tracking).
    WakeLock,
    /// Receive boot completed (to restart service after reboot).
    ReceiveBootCompleted,
}

impl PermissionType {
    pub fn label(self) -> &'static str {
        match self {
            Self::AccessFineLocation => "ACCESS_FINE_LOCATION",
            Self::AccessCoarseLocation => "ACCESS_COARSE_LOCATION",
            Self::AccessBackgroundLocation => "ACCESS_BACKGROUND_LOCATION",
            Self::BodySensors => "BODY_SENSORS",
            Self::ActivityRecognition => "ACTIVITY_RECOGNITION",
            Self::PostNotifications => "POST_NOTIFICATIONS",
            Self::Camera => "CAMERA",
            Self::RecordAudio => "RECORD_AUDIO",
            Self::ReadExternalStorage => "READ_EXTERNAL_STORAGE",
            Self::WriteExternalStorage => "WRITE_EXTERNAL_STORAGE",
            Self::Bluetooth => "BLUETOOTH",
            Self::Internet => "INTERNET",
            Self::AccessNetworkState => "ACCESS_NETWORK_STATE",
            Self::ForegroundService => "FOREGROUND_SERVICE",
            Self::WakeLock => "WAKE_LOCK",
            Self::ReceiveBootCompleted => "RECEIVE_BOOT_COMPLETED",
        }
    }

    pub fn is_dangerous(self) -> bool {
        matches!(
            self,
            Self::AccessFineLocation
                | Self::AccessCoarseLocation
                | Self::AccessBackgroundLocation
                | Self::BodySensors
                | Self::ActivityRecognition
                | Self::PostNotifications
                | Self::Camera
                | Self::RecordAudio
                | Self::ReadExternalStorage
                | Self::WriteExternalStorage
        )
    }

    pub fn requires_justification(self) -> bool {
        self.is_dangerous()
    }

    pub fn requires_background_location_justification(self) -> bool {
        matches!(self, Self::AccessBackgroundLocation)
    }
}

/// A permission declaration with justification text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionDeclaration {
    /// The permission type.
    pub permission_type: PermissionType,
    /// The justification for the permission.
    pub justification: String,
    /// Whether the permission is declared in the manifest.
    pub is_declared: bool,
    /// Whether the justification has been written.
    pub has_justification: bool,
}

impl PermissionDeclaration {
    pub fn new(permission_type: PermissionType) -> Self {
        let justification = match permission_type {
            PermissionType::AccessFineLocation => String::from(
                "Required for accurate GPS tracking during walking and running activities",
            ),
            PermissionType::AccessCoarseLocation => String::from(
                "Used as a fallback when fine location is not available",
            ),
            PermissionType::AccessBackgroundLocation => String::from(
                "Required to maintain GPS tracking when the app is in the background during an active workout session",
            ),
            PermissionType::BodySensors => String::from(
                "Required to read heart rate data from a paired wearable device",
            ),
            PermissionType::ActivityRecognition => String::from(
                "Required to count steps using the device's built-in step counter sensor",
            ),
            PermissionType::PostNotifications => String::from(
                "Required to show workout progress notifications and alerts",
            ),
            PermissionType::Camera => String::from(
                "Optional: allows users to attach photos to their route records",
            ),
            PermissionType::RecordAudio => String::from(
                "Not used by the app; declared for transparency",
            ),
            PermissionType::ReadExternalStorage => String::from(
                "Required to import GPX route files from the device storage",
            ),
            PermissionType::WriteExternalStorage => String::from(
                "Required to export GPX route files to the device storage",
            ),
            PermissionType::Bluetooth => String::from(
                "Required to connect to paired wearable devices for heart rate data",
            ),
            PermissionType::Internet => String::from(
                "Required for map tile downloads, cloud synchronisation, and AI coaching",
            ),
            PermissionType::AccessNetworkState => String::from(
                "Required to detect online/offline status for sync scheduling",
            ),
            PermissionType::ForegroundService => String::from(
                "Required to run the GPS tracking service in the foreground during active workouts",
            ),
            PermissionType::WakeLock => String::from(
                "Required to keep the CPU active for accurate GPS tracking during long workouts",
            ),
            PermissionType::ReceiveBootCompleted => String::from(
                "Required to optionally resume tracking after device reboot",
            ),
        };
        Self {
            permission_type,
            justification,
            is_declared: true,
            has_justification: true,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.is_declared && self.has_justification && !self.justification.is_empty()
    }
}

/// All permission declarations for the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionDeclarations {
    /// All declared permissions.
    pub declarations: Vec<PermissionDeclaration>,
    /// The background-location justification (special requirement).
    pub background_location_justification: String,
    /// Whether the background-location justification is complete.
    pub background_location_justified: bool,
}

impl PermissionDeclarations {
    pub fn new() -> Self {
        Self {
            declarations: Vec::new(),
            background_location_justification: String::from(
                "S.T.R.I.D.E. requires background location access to maintain continuous GPS tracking during active workout sessions. Without background location, the app cannot record accurate route data when the screen is off or the user switches to another app during a workout. Background location is only active during an explicitly started workout session and is stopped when the user ends the session.",
            ),
            background_location_justified: true,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.background_location_justified
            && !self.declarations.is_empty()
            && self.declarations.iter().all(|d| d.is_ready())
    }
}

impl Default for PermissionDeclarations {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Reviewer access
// ═══════════════════════════════════════════════════════════════════════

/// Instructions for Google Play reviewers to test the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewerAccess {
    /// A demo/test account username.
    pub test_account_email: String,
    /// A demo/test account password.
    pub test_account_password: String,
    /// Step-by-step instructions for the reviewer.
    pub instructions: String,
    /// Whether to start on the login screen or a demo screen.
    pub starts_on_login_screen: bool,
    /// Any special setup steps for the reviewer.
    pub setup_steps: Vec<String>,
}

impl ReviewerAccess {
    pub fn new() -> Self {
        Self {
            test_account_email: String::from("reviewer@stride.app"),
            test_account_password: String::from("stride-demo-2024"),
            instructions: String::from(
                "1. Log in using the test credentials above.\n\
                 2. Grant the location permission when prompted.\n\
                 3. Tap the Start button to begin a workout.\n\
                 4. Walk or run with the device to see live tracking.\n\
                 5. Tap Stop to end the workout and view the summary.\n\
                 6. Check the History tab for saved workout records.",
            ),
            starts_on_login_screen: true,
            setup_steps: vec![
                String::from("Grant location permission on first launch"),
                String::from("Grant notification permission when prompted"),
                String::from("Optionally pair a wearable device for heart rate data"),
            ],
        }
    }

    pub fn is_ready(&self) -> bool {
        !self.test_account_email.is_empty()
            && !self.test_account_password.is_empty()
            && !self.instructions.is_empty()
    }
}

impl Default for ReviewerAccess {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Testing tracks
// ═══════════════════════════════════════════════════════════════════════

/// The testing track on Google Play.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestingTrack {
    /// Internal testing (up to 100 testers).
    Internal,
    /// Closed testing (managed via email list or Google Group).
    Closed,
    /// Open testing (anyone with the opt-in link).
    Open,
    /// Production release.
    Production,
}

impl TestingTrack {
    pub fn label(self) -> &'static str {
        match self {
            Self::Internal => "Internal Testing",
            Self::Closed => "Closed Testing",
            Self::Open => "Open Testing",
            Self::Production => "Production",
        }
    }

    pub fn is_pre_release(self) -> bool {
        matches!(self, Self::Internal | Self::Closed | Self::Open)
    }

    pub fn ordering(self) -> u32 {
        match self {
            Self::Internal => 1,
            Self::Closed => 2,
            Self::Open => 3,
            Self::Production => 4,
        }
    }
}

/// The status of a testing track.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackStatus {
    /// The track has not been set up.
    NotSetup,
    /// The track is set up but has no releases.
    Setup,
    /// A release has been pushed to the track.
    Released,
    /// The release has been reviewed and approved.
    Approved,
}

impl TrackStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotSetup => "Not Set Up",
            Self::Setup => "Set Up",
            Self::Released => "Released",
            Self::Approved => "Approved",
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Approved)
    }
}

/// A release track configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTrack {
    /// The testing track type.
    pub track: TestingTrack,
    /// The status of the track.
    pub status: TrackStatus,
    /// The version code of the current release on this track.
    pub current_version_code: u32,
    /// The number of testers on this track.
    pub tester_count: u32,
    /// The rollout percentage (for production staged rollout).
    pub rollout_percentage: u32,
}

impl ReleaseTrack {
    pub fn new(track: TestingTrack) -> Self {
        Self {
            track,
            status: TrackStatus::NotSetup,
            current_version_code: 0,
            tester_count: 0,
            rollout_percentage: 0,
        }
    }

    pub fn setup(mut self, tester_count: u32) -> Self {
        self.status = TrackStatus::Setup;
        self.tester_count = tester_count;
        self
    }

    pub fn release(mut self, version_code: u32) -> Self {
        self.status = TrackStatus::Released;
        self.current_version_code = version_code;
        self
    }

    pub fn approve(mut self) -> Self {
        self.status = TrackStatus::Approved;
        self
    }

    pub fn set_rollout(mut self, percentage: u32) -> Self {
        self.rollout_percentage = percentage.min(100);
        self
    }

    pub fn is_ready(&self) -> bool {
        self.status.is_ready()
    }
}

/// The staged rollout plan for production.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagedRolloutPlan {
    /// The internal testing track.
    pub internal: ReleaseTrack,
    /// The closed testing track.
    pub closed: ReleaseTrack,
    /// The production (staged rollout) track.
    pub production: ReleaseTrack,
    /// The rollout stages (percentage at each stage).
    pub rollout_stages: Vec<u32>,
    /// The current stage index.
    pub current_stage: u32,
}

impl StagedRolloutPlan {
    pub fn new() -> Self {
        Self {
            internal: ReleaseTrack::new(TestingTrack::Internal),
            closed: ReleaseTrack::new(TestingTrack::Closed),
            production: ReleaseTrack::new(TestingTrack::Production),
            rollout_stages: vec![5, 10, 25, 50, 100],
            current_stage: 0,
        }
    }

    pub fn with_internal(mut self, tester_count: u32, version_code: u32) -> Self {
        self.internal = self.internal.setup(tester_count).release(version_code).approve();
        self
    }

    pub fn with_closed(mut self, tester_count: u32, version_code: u32) -> Self {
        self.closed = self.closed.setup(tester_count).release(version_code).approve();
        self
    }

    pub fn start_production_rollout(mut self, version_code: u32) -> Self {
        self.production = self.production.release(version_code);
        self.production = self.production.set_rollout(self.rollout_stages[0]);
        self.current_stage = 0;
        self
    }

    pub fn advance_stage(mut self) -> Self {
        if (self.current_stage as usize) < self.rollout_stages.len() - 1 {
            self.current_stage += 1;
            self.production = self
                .production
                .set_rollout(self.rollout_stages[self.current_stage as usize]);
        } else {
            self.production = self.production.approve();
        }
        self
    }

    pub fn is_complete(&self) -> bool {
        self.internal.is_ready() && self.closed.is_ready() && self.production.is_ready()
    }

    pub fn current_rollout_percentage(&self) -> u32 {
        self.rollout_stages
            .get(self.current_stage as usize)
            .copied()
            .unwrap_or(0)
    }
}

impl Default for StagedRolloutPlan {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Production stack inventory
// ═══════════════════════════════════════════════════════════════════════

/// The category of a production infrastructure component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StackComponentCategory {
    /// UI framework (Flutter/FlutterFlow).
    UiFramework,
    /// Backend / cloud database (Firestore).
    CloudDatabase,
    /// Authentication (Firebase Auth).
    Authentication,
    /// File storage (Cloud Storage).
    FileStorage,
    /// Local database (SQLite).
    LocalDatabase,
    /// Serverless compute (Cloud Functions / Cloud Run).
    ServerlessCompute,
    /// App attestation (Firebase App Check).
    AppAttestation,
    /// Crash reporting (Crashlytics).
    CrashReporting,
    /// Performance monitoring (Firebase Performance Monitoring).
    PerformanceMonitoring,
    /// Push messaging (FCM).
    PushMessaging,
    /// Maps / tile provider.
    MapsProvider,
    /// Health data integration (Health Connect / Wear OS).
    HealthIntegration,
    /// AI service (server-side).
    AiService,
    /// Background tracking engine (Rust engine).
    TrackingEngine,
}

impl StackComponentCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::UiFramework => "UI Framework",
            Self::CloudDatabase => "Cloud Database",
            Self::Authentication => "Authentication",
            Self::FileStorage => "File Storage",
            Self::LocalDatabase => "Local Database",
            Self::ServerlessCompute => "Serverless Compute",
            Self::AppAttestation => "App Attestation",
            Self::CrashReporting => "Crash Reporting",
            Self::PerformanceMonitoring => "Performance Monitoring",
            Self::PushMessaging => "Push Messaging",
            Self::MapsProvider => "Maps Provider",
            Self::HealthIntegration => "Health Data Integration",
            Self::AiService => "AI Service",
            Self::TrackingEngine => "Background Tracking Engine",
        }
    }
}

/// A production stack component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackComponent {
    /// The component name (e.g. "FlutterFlow", "Firestore").
    pub name: String,
    /// The component category.
    pub category: StackComponentCategory,
    /// The version or configuration of the component.
    pub version: String,
    /// Whether the component is configured for production.
    pub is_configured: bool,
    /// Whether the component is deployed to the production environment.
    pub is_deployed: bool,
    /// Whether the component has been tested.
    pub is_tested: bool,
}

impl StackComponent {
    pub fn new(
        name: impl Into<String>,
        category: StackComponentCategory,
        version: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            version: version.into(),
            is_configured: false,
            is_deployed: false,
            is_tested: false,
        }
    }

    pub fn configure(mut self) -> Self {
        self.is_configured = true;
        self
    }

    pub fn deploy(mut self) -> Self {
        self.is_deployed = true;
        self
    }

    pub fn test(mut self) -> Self {
        self.is_tested = true;
        self
    }

    pub fn is_ready(&self) -> bool {
        self.is_configured && self.is_deployed && self.is_tested
    }
}

/// The full production stack inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductionStack {
    /// All components in the stack.
    pub components: Vec<StackComponent>,
    /// Whether there are development, staging, and production environments.
    pub has_dev_staging_prod: bool,
    /// Whether automated testing is configured.
    pub has_automated_testing: bool,
    /// Whether physical device testing is performed.
    pub has_device_testing: bool,
}

impl ProductionStack {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            has_dev_staging_prod: false,
            has_automated_testing: false,
            has_device_testing: false,
        }
    }

    pub fn with_dev_staging_prod(mut self) -> Self {
        self.has_dev_staging_prod = true;
        self
    }

    pub fn with_automated_testing(mut self) -> Self {
        self.has_automated_testing = true;
        self
    }

    pub fn with_device_testing(mut self) -> Self {
        self.has_device_testing = true;
        self
    }

    pub fn is_ready(&self) -> bool {
        self.has_dev_staging_prod
            && self.has_automated_testing
            && self.has_device_testing
            && !self.components.is_empty()
            && self.components.iter().all(|c| c.is_ready())
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    pub fn ready_count(&self) -> usize {
        self.components.iter().filter(|c| c.is_ready()).count()
    }
}

impl Default for ProductionStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds the default production stack for S.T.R.I.D.E.
pub fn build_default_production_stack() -> ProductionStack {
    ProductionStack::new()
        .with_dev_staging_prod()
        .with_automated_testing()
        .with_device_testing()
}

// ═══════════════════════════════════════════════════════════════════════
// Release readiness status
// ═══════════════════════════════════════════════════════════════════════

/// The overall release readiness status for the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseReadinessStatus {
    /// Whether the app identity is finalised.
    pub app_identity_ready: bool,
    /// Whether the signing key is secure and enrolled.
    pub signing_key_ready: bool,
    /// Whether the release bundle is built and approved.
    pub release_bundle_ready: bool,
    /// Whether the versioning policy is configured.
    pub versioning_ready: bool,
    /// Whether all store assets are ready.
    pub store_assets_ready: bool,
    /// Whether the store listing is complete.
    pub store_listing_ready: bool,
    /// Whether permission declarations are complete.
    pub permissions_ready: bool,
    /// Whether reviewer access is configured.
    pub reviewer_access_ready: bool,
    /// Whether the staged rollout plan is complete.
    pub rollout_plan_ready: bool,
    /// Whether the production stack is ready.
    pub production_stack_ready: bool,
    /// Whether the data safety form is submitted.
    pub data_safety_submitted: bool,
}

impl ReleaseReadinessStatus {
    pub fn new() -> Self {
        Self {
            app_identity_ready: false,
            signing_key_ready: false,
            release_bundle_ready: false,
            versioning_ready: false,
            store_assets_ready: false,
            store_listing_ready: false,
            permissions_ready: false,
            reviewer_access_ready: false,
            rollout_plan_ready: false,
            production_stack_ready: false,
            data_safety_submitted: false,
        }
    }

    pub fn is_ready_for_publication(&self) -> bool {
        self.app_identity_ready
            && self.signing_key_ready
            && self.release_bundle_ready
            && self.versioning_ready
            && self.store_assets_ready
            && self.store_listing_ready
            && self.permissions_ready
            && self.reviewer_access_ready
            && self.production_stack_ready
            && self.data_safety_submitted
    }

    pub fn completed_count(&self) -> u32 {
        let mut count = 0u32;
        if self.app_identity_ready {
            count += 1;
        }
        if self.signing_key_ready {
            count += 1;
        }
        if self.release_bundle_ready {
            count += 1;
        }
        if self.versioning_ready {
            count += 1;
        }
        if self.store_assets_ready {
            count += 1;
        }
        if self.store_listing_ready {
            count += 1;
        }
        if self.permissions_ready {
            count += 1;
        }
        if self.reviewer_access_ready {
            count += 1;
        }
        if self.rollout_plan_ready {
            count += 1;
        }
        if self.production_stack_ready {
            count += 1;
        }
        if self.data_safety_submitted {
            count += 1;
        }
        count
    }

    pub fn total_count(&self) -> u32 {
        11
    }

    pub fn completion_percent(&self) -> f64 {
        if self.total_count() == 0 {
            return 0.0;
        }
        (self.completed_count() as f64 / self.total_count() as f64) * 100.0
    }
}

impl Default for ReleaseReadinessStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes the release readiness status from all the release components.
pub fn build_release_readiness_status(
    identity: &AppIdentity,
    signing: &SigningKeyConfig,
    bundle: &ReleaseBundle,
    versioning: &VersioningPolicy,
    assets: &StoreAssets,
    listing: &StoreListing,
    permissions: &PermissionDeclarations,
    reviewer: &ReviewerAccess,
    stack: &ProductionStack,
    data_safety_submitted: bool,
) -> ReleaseReadinessStatus {
    ReleaseReadinessStatus {
        app_identity_ready: identity.is_ready(),
        signing_key_ready: signing.is_ready(),
        release_bundle_ready: bundle.is_ready_for_release(),
        versioning_ready: !versioning.version_string().is_empty(),
        store_assets_ready: assets.is_complete(),
        store_listing_ready: listing.is_ready(),
        permissions_ready: permissions.is_complete(),
        reviewer_access_ready: reviewer.is_ready(),
        rollout_plan_ready: true, // set by staged rollout configuration
        production_stack_ready: stack.is_ready(),
        data_safety_submitted,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // -- AppIdentity --

    #[test]
    fn app_identity_defaults_to_stride() {
        let id = AppIdentity::new();
        assert_eq!(id.app_name, "S.T.R.I.D.E.");
        assert_eq!(id.package_id, "com.stride.app");
        assert_eq!(id.name_status, AppNameStatus::Draft);
    }

    #[test]
    fn app_identity_finalize_sets_status() {
        let id = AppIdentity::new().finalize();
        assert_eq!(id.name_status, AppNameStatus::Final);
        assert!(id.is_ready());
    }

    #[test]
    fn app_identity_not_ready_when_draft() {
        let id = AppIdentity::new();
        assert!(!id.is_ready());
    }

    #[test]
    fn app_identity_ready_when_final_and_valid() {
        let id = AppIdentity::new()
            .with_name("S.T.R.I.D.E.")
            .with_package_id("com.stride.app")
            .finalize();
        assert!(id.is_ready());
    }

    #[test]
    fn app_identity_not_ready_when_package_id_missing_dot() {
        let id = AppIdentity::new()
            .with_package_id("stride")
            .finalize();
        assert!(!id.is_ready());
    }

    #[test]
    fn app_name_status_is_final_check() {
        assert!(AppNameStatus::Final.is_final());
        assert!(!AppNameStatus::Draft.is_final());
        assert!(!AppNameStatus::Proposed.is_final());
    }

    #[test]
    fn app_identity_serializes_round_trip() {
        let id = AppIdentity::new().finalize();
        let json = serde_json::to_string(&id).unwrap();
        let back: AppIdentity = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    // -- SigningKeyConfig --

    #[test]
    fn signing_key_defaults_to_not_generated() {
        let key = SigningKeyConfig::new();
        assert_eq!(key.status, SigningKeyStatus::NotGenerated);
        assert!(!key.is_ready());
        assert!(!key.is_secure());
    }

    #[test]
    fn signing_key_generate_sets_status() {
        let key = SigningKeyConfig::new().generate();
        assert_eq!(key.status, SigningKeyStatus::Generated);
        assert!(!key.is_ready());
        assert!(!key.is_secure());
    }

    #[test]
    fn signing_key_backup_sets_status() {
        let key = SigningKeyConfig::new().generate().backup();
        assert_eq!(key.status, SigningKeyStatus::BackedUp);
        assert!(key.has_backup);
        assert!(key.is_secure());
        assert!(!key.is_ready());
    }

    #[test]
    fn signing_key_enroll_sets_status() {
        let key = SigningKeyConfig::new().generate().backup().enroll();
        assert_eq!(key.status, SigningKeyStatus::Enrolled);
        assert!(key.is_enrolled_in_play);
        assert!(key.is_ready());
        assert!(key.is_secure());
    }

    #[test]
    fn signing_key_status_is_ready_check() {
        assert!(SigningKeyStatus::Enrolled.is_ready());
        assert!(!SigningKeyStatus::BackedUp.is_ready());
    }

    #[test]
    fn signing_key_status_is_secure_check() {
        assert!(SigningKeyStatus::BackedUp.is_secure());
        assert!(SigningKeyStatus::Enrolled.is_secure());
        assert!(!SigningKeyStatus::Generated.is_secure());
        assert!(!SigningKeyStatus::NotGenerated.is_secure());
    }

    #[test]
    fn signing_key_serializes_round_trip() {
        let key = SigningKeyConfig::new().generate().backup().enroll();
        let json = serde_json::to_string(&key).unwrap();
        let back: SigningKeyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(key, back);
    }

    // -- ReleaseBundle --

    #[test]
    fn release_bundle_defaults_to_not_built() {
        let bundle = ReleaseBundle::new();
        assert_eq!(bundle.status, ReleaseBundleStatus::NotBuilt);
        assert!(!bundle.is_ready_for_release());
    }

    #[test]
    fn release_bundle_build_sets_status() {
        let bundle = ReleaseBundle::new().build(50_000_000, "abc123");
        assert_eq!(bundle.status, ReleaseBundleStatus::Built);
        assert_eq!(bundle.size_bytes, 50_000_000);
        assert!(!bundle.is_ready_for_release());
    }

    #[test]
    fn release_bundle_sign_sets_flag() {
        let bundle = ReleaseBundle::new().build(50_000_000, "abc123").sign();
        assert!(bundle.is_signed);
    }

    #[test]
    fn release_bundle_approve_sets_status() {
        let bundle = ReleaseBundle::new()
            .build(50_000_000, "abc123")
            .sign()
            .approve();
        assert_eq!(bundle.status, ReleaseBundleStatus::Approved);
        assert!(bundle.is_ready_for_release());
    }

    #[test]
    fn release_bundle_not_ready_without_signing() {
        let bundle = ReleaseBundle::new().build(50_000_000, "abc123").approve();
        assert!(!bundle.is_ready_for_release());
    }

    #[test]
    fn build_type_is_release_check() {
        assert!(BuildType::Release.is_release());
        assert!(!BuildType::Debug.is_release());
    }

    #[test]
    fn release_bundle_status_is_ready_for_upload() {
        assert!(ReleaseBundleStatus::Built.is_ready_for_upload());
        assert!(ReleaseBundleStatus::Uploaded.is_ready_for_upload());
        assert!(!ReleaseBundleStatus::NotBuilt.is_ready_for_upload());
    }

    #[test]
    fn release_bundle_serializes_round_trip() {
        let bundle = ReleaseBundle::new().build(50_000_000, "abc123").sign().approve();
        let json = serde_json::to_string(&bundle).unwrap();
        let back: ReleaseBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(bundle, back);
    }

    // -- VersioningPolicy --

    #[test]
    fn versioning_defaults_to_semantic_1_0_0() {
        let v = VersioningPolicy::new();
        assert_eq!(v.strategy, VersioningStrategy::Semantic);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert_eq!(v.version_string(), "1.0.0");
    }

    #[test]
    fn versioning_next_version_code() {
        let v = VersioningPolicy::new();
        assert_eq!(v.next_version_code(), 2);
    }

    #[test]
    fn versioning_is_compatible() {
        let v = VersioningPolicy::new();
        assert!(v.is_compatible(1));
        assert!(!v.is_compatible(0));
    }

    #[test]
    fn versioning_strategy_label() {
        assert_eq!(VersioningStrategy::Semantic.label(), "Semantic Versioning");
        assert_eq!(VersioningStrategy::Calendar.label(), "Calendar Versioning");
    }

    #[test]
    fn versioning_serializes_round_trip() {
        let v = VersioningPolicy::new();
        let json = serde_json::to_string(&v).unwrap();
        let back: VersioningPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    // -- StoreAssets --

    #[test]
    fn store_assets_defaults_to_not_created() {
        let assets = StoreAssets::new();
        assert!(!assets.launcher_icon.is_ready());
        assert!(!assets.is_complete());
    }

    #[test]
    fn store_assets_complete_when_all_ready() {
        let mut assets = StoreAssets::new();
        assets.launcher_icon = assets.launcher_icon.clone().finalize();
        assets.splash_screen = assets.splash_screen.clone().finalize();
        assets.feature_graphic = assets.feature_graphic.clone().finalize();
        assets.play_store_icon = assets.play_store_icon.clone().finalize();
        assets.phone_screenshots = vec![
            StoreAsset::new(StoreAssetType::PhoneScreenshot).finalize(),
            StoreAsset::new(StoreAssetType::PhoneScreenshot).finalize(),
        ];
        assert!(assets.is_complete());
    }

    #[test]
    fn store_asset_type_is_required_check() {
        assert!(StoreAssetType::AdaptiveLauncherIcon.is_required());
        assert!(StoreAssetType::FeatureGraphic.is_required());
        assert!(!StoreAssetType::LegacyLauncherIcon.is_required());
    }

    #[test]
    fn store_asset_final_is_ready() {
        let asset = StoreAsset::new(StoreAssetType::PhoneScreenshot).finalize();
        assert!(asset.is_ready());
    }

    #[test]
    fn store_asset_uploaded_is_ready() {
        let asset = StoreAsset::new(StoreAssetType::FeatureGraphic).upload();
        assert!(asset.is_ready());
    }

    #[test]
    fn store_asset_dimensions_correct() {
        let icon = StoreAsset::new(StoreAssetType::PlayStoreIcon);
        assert_eq!(icon.width_px, 512);
        assert_eq!(icon.height_px, 512);

        let fg = StoreAsset::new(StoreAssetType::FeatureGraphic);
        assert_eq!(fg.width_px, 1024);
        assert_eq!(fg.height_px, 500);
    }

    #[test]
    fn store_assets_with_phone_screenshots() {
        let assets = StoreAssets::new().with_phone_screenshots(3);
        assert_eq!(assets.phone_screenshots.len(), 3);
        assert!(!assets.is_complete()); // still need other assets
    }

    #[test]
    fn store_assets_serializes_round_trip() {
        let assets = StoreAssets::new();
        let json = serde_json::to_string(&assets).unwrap();
        let back: StoreAssets = serde_json::from_str(&json).unwrap();
        assert_eq!(assets, back);
    }

    // -- StoreListing --

    #[test]
    fn store_listing_defaults_to_health_fitness() {
        let listing = StoreListing::new();
        assert_eq!(listing.category, AppCategory::HealthFitness);
        assert_eq!(listing.content_rating, ContentRating::Everyone);
        assert!(!listing.is_ready());
    }

    #[test]
    fn store_listing_ready_when_complete() {
        let listing = StoreListing::new()
            .with_short_description("Track your walks and runs")
            .with_full_description("A comprehensive walking and running tracker")
            .with_privacy_url("https://stride.app/privacy")
            .complete_content_rating();
        assert!(listing.is_ready());
    }

    #[test]
    fn store_listing_not_ready_without_url() {
        let listing = StoreListing::new()
            .with_short_description("Track your walks")
            .with_full_description("Full description")
            .complete_content_rating();
        assert!(!listing.is_ready());
    }

    #[test]
    fn content_rating_appropriate_for_fitness() {
        assert!(ContentRating::Everyone.is_appropriate_for_fitness());
        assert!(ContentRating::Teen.is_appropriate_for_fitness());
        assert!(!ContentRating::Mature.is_appropriate_for_fitness());
    }

    #[test]
    fn store_listing_serializes_round_trip() {
        let listing = StoreListing::new()
            .with_short_description("Test")
            .with_full_description("Full");
        let json = serde_json::to_string(&listing).unwrap();
        let back: StoreListing = serde_json::from_str(&json).unwrap();
        assert_eq!(listing, back);
    }

    // -- PermissionDeclarations --

    #[test]
    fn permission_declarations_defaults_with_bg_justification() {
        let perms = PermissionDeclarations::new();
        assert!(perms.background_location_justified);
        assert!(!perms.background_location_justification.is_empty());
        assert!(!perms.is_complete()); // no declarations yet
    }

    #[test]
    fn permission_type_is_dangerous() {
        assert!(PermissionType::AccessFineLocation.is_dangerous());
        assert!(PermissionType::AccessBackgroundLocation.is_dangerous());
        assert!(!PermissionType::Internet.is_dangerous());
    }

    #[test]
    fn permission_type_requires_justification() {
        assert!(PermissionType::AccessFineLocation.requires_justification());
        assert!(!PermissionType::Internet.requires_justification());
    }

    #[test]
    fn permission_type_requires_bg_location_justification() {
        assert!(PermissionType::AccessBackgroundLocation.requires_background_location_justification());
        assert!(!PermissionType::AccessFineLocation.requires_background_location_justification());
    }

    #[test]
    fn permission_declaration_new_has_justification() {
        let decl = PermissionDeclaration::new(PermissionType::AccessFineLocation);
        assert!(decl.has_justification);
        assert!(!decl.justification.is_empty());
        assert!(decl.is_ready());
    }

    #[test]
    fn permission_declarations_complete_when_all_ready() {
        let mut perms = PermissionDeclarations::new();
        perms.declarations = vec![
            PermissionDeclaration::new(PermissionType::AccessFineLocation),
            PermissionDeclaration::new(PermissionType::AccessBackgroundLocation),
            PermissionDeclaration::new(PermissionType::BodySensors),
        ];
        assert!(perms.is_complete());
    }

    #[test]
    fn permission_declarations_not_complete_when_empty() {
        let perms = PermissionDeclarations::new();
        assert!(!perms.is_complete());
    }

    #[test]
    fn permission_declarations_serializes_round_trip() {
        let perms = PermissionDeclarations::new();
        let json = serde_json::to_string(&perms).unwrap();
        let back: PermissionDeclarations = serde_json::from_str(&json).unwrap();
        assert_eq!(perms, back);
    }

    // -- ReviewerAccess --

    #[test]
    fn reviewer_access_defaults_ready() {
        let ra = ReviewerAccess::new();
        assert!(!ra.test_account_email.is_empty());
        assert!(!ra.test_account_password.is_empty());
        assert!(ra.is_ready());
    }

    #[test]
    fn reviewer_access_not_ready_without_credentials() {
        let mut ra = ReviewerAccess::new();
        ra.test_account_email = String::new();
        assert!(!ra.is_ready());
    }

    #[test]
    fn reviewer_access_has_setup_steps() {
        let ra = ReviewerAccess::new();
        assert!(!ra.setup_steps.is_empty());
        assert!(ra.starts_on_login_screen);
    }

    #[test]
    fn reviewer_access_serializes_round_trip() {
        let ra = ReviewerAccess::new();
        let json = serde_json::to_string(&ra).unwrap();
        let back: ReviewerAccess = serde_json::from_str(&json).unwrap();
        assert_eq!(ra, back);
    }

    // -- TestingTrack & ReleaseTrack --

    #[test]
    fn testing_track_ordering() {
        assert!(TestingTrack::Internal.ordering() < TestingTrack::Closed.ordering());
        assert!(TestingTrack::Closed.ordering() < TestingTrack::Production.ordering());
    }

    #[test]
    fn testing_track_is_pre_release() {
        assert!(TestingTrack::Internal.is_pre_release());
        assert!(TestingTrack::Closed.is_pre_release());
        assert!(!TestingTrack::Production.is_pre_release());
    }

    #[test]
    fn release_track_defaults_not_setup() {
        let track = ReleaseTrack::new(TestingTrack::Internal);
        assert_eq!(track.status, TrackStatus::NotSetup);
        assert!(!track.is_ready());
    }

    #[test]
    fn release_track_setup_and_release() {
        let track = ReleaseTrack::new(TestingTrack::Internal)
            .setup(10)
            .release(5)
            .approve();
        assert_eq!(track.tester_count, 10);
        assert_eq!(track.current_version_code, 5);
        assert!(track.is_ready());
    }

    #[test]
    fn release_track_rollout_capped_at_100() {
        let mut track = ReleaseTrack::new(TestingTrack::Production);
        track = track.set_rollout(150);
        assert_eq!(track.rollout_percentage, 100);
    }

    #[test]
    fn release_track_serializes_round_trip() {
        let track = ReleaseTrack::new(TestingTrack::Internal).setup(5).release(1);
        let json = serde_json::to_string(&track).unwrap();
        let back: ReleaseTrack = serde_json::from_str(&json).unwrap();
        assert_eq!(track, back);
    }

    // -- StagedRolloutPlan --

    #[test]
    fn staged_rollout_defaults_to_5_10_25_50_100() {
        let plan = StagedRolloutPlan::new();
        assert_eq!(plan.rollout_stages, vec![5, 10, 25, 50, 100]);
        assert_eq!(plan.current_stage, 0);
    }

    #[test]
    fn staged_rollout_start_production() {
        let plan = StagedRolloutPlan::new()
            .with_internal(10, 1)
            .with_closed(50, 2)
            .start_production_rollout(3);
        assert_eq!(plan.production.current_version_code, 3);
        assert_eq!(plan.current_rollout_percentage(), 5);
    }

    #[test]
    fn staged_rollout_advance_stage() {
        let mut plan = StagedRolloutPlan::new()
            .with_internal(10, 1)
            .with_closed(50, 2)
            .start_production_rollout(3);
        plan = plan.advance_stage();
        assert_eq!(plan.current_stage, 1);
        assert_eq!(plan.current_rollout_percentage(), 10);
    }

    #[test]
    fn staged_rollout_advance_to_final_approves() {
        let mut plan = StagedRolloutPlan::new()
            .with_internal(10, 1)
            .with_closed(50, 2)
            .start_production_rollout(3);
        // Advance through all stages
        for _ in 0..5 {
            plan = plan.advance_stage();
        }
        assert!(plan.production.is_ready());
        assert!(plan.is_complete());
    }

    #[test]
    fn staged_rollout_not_complete_without_internal() {
        let plan = StagedRolloutPlan::new();
        assert!(!plan.is_complete());
    }

    #[test]
    fn staged_rollout_complete_when_all_approved() {
        let plan = StagedRolloutPlan::new()
            .with_internal(10, 1)
            .with_closed(50, 2);
        let mut prod = ReleaseTrack::new(TestingTrack::Production);
        prod = prod.release(3).approve();
        let plan = StagedRolloutPlan {
            internal: plan.internal,
            closed: plan.closed,
            production: prod,
            rollout_stages: vec![5, 10, 25, 50, 100],
            current_stage: 4,
        };
        assert!(plan.is_complete());
    }

    #[test]
    fn staged_rollout_serializes_round_trip() {
        let plan = StagedRolloutPlan::new();
        let json = serde_json::to_string(&plan).unwrap();
        let back: StagedRolloutPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, back);
    }

    // -- ProductionStack --

    #[test]
    fn production_stack_defaults_not_ready() {
        let stack = ProductionStack::new();
        assert!(!stack.is_ready());
        assert_eq!(stack.component_count(), 0);
    }

    #[test]
    fn production_stack_with_environments() {
        let stack = ProductionStack::new()
            .with_dev_staging_prod()
            .with_automated_testing()
            .with_device_testing();
        assert!(stack.has_dev_staging_prod);
        assert!(stack.has_automated_testing);
        assert!(stack.has_device_testing);
        assert!(!stack.is_ready()); // no components yet
    }

    #[test]
    fn stack_component_ready_when_all_set() {
        let comp = StackComponent::new("Firestore", StackComponentCategory::CloudDatabase, "1.0")
            .configure()
            .deploy()
            .test();
        assert!(comp.is_ready());
    }

    #[test]
    fn stack_component_not_ready_without_config() {
        let comp = StackComponent::new("Firestore", StackComponentCategory::CloudDatabase, "1.0")
            .deploy()
            .test();
        assert!(!comp.is_ready());
    }

    #[test]
    fn stack_component_category_label() {
        assert_eq!(StackComponentCategory::Authentication.label(), "Authentication");
        assert_eq!(StackComponentCategory::TrackingEngine.label(), "Background Tracking Engine");
    }

    #[test]
    fn production_stack_ready_count() {
        let mut stack = ProductionStack::new()
            .with_dev_staging_prod()
            .with_automated_testing()
            .with_device_testing();
        stack.components = vec![
            StackComponent::new("Firestore", StackComponentCategory::CloudDatabase, "1.0")
                .configure()
                .deploy()
                .test(),
            StackComponent::new("Firebase Auth", StackComponentCategory::Authentication, "1.0")
                .configure()
                .deploy()
                .test(),
            StackComponent::new("Crashlytics", StackComponentCategory::CrashReporting, "1.0")
                .configure()
                .deploy()
                .test(),
        ];
        assert_eq!(stack.ready_count(), 3);
        assert!(stack.is_ready());
    }

    #[test]
    fn production_stack_serializes_round_trip() {
        let stack = ProductionStack::new();
        let json = serde_json::to_string(&stack).unwrap();
        let back: ProductionStack = serde_json::from_str(&json).unwrap();
        assert_eq!(stack, back);
    }

    #[test]
    fn build_default_production_stack_has_environments() {
        let stack = build_default_production_stack();
        assert!(stack.has_dev_staging_prod);
        assert!(stack.has_automated_testing);
        assert!(stack.has_device_testing);
    }

    // -- ReleaseReadinessStatus --

    #[test]
    fn release_readiness_defaults_all_false() {
        let status = ReleaseReadinessStatus::new();
        assert!(!status.is_ready_for_publication());
        assert_eq!(status.completed_count(), 0);
        assert_eq!(status.total_count(), 11);
        assert_eq!(status.completion_percent(), 0.0);
    }

    #[test]
    fn release_readiness_counts_correctly() {
        let mut status = ReleaseReadinessStatus::new();
        status.app_identity_ready = true;
        status.signing_key_ready = true;
        status.versioning_ready = true;
        assert_eq!(status.completed_count(), 3);
        assert!((status.completion_percent() - 27.272727272727273).abs() < 0.01);
    }

    #[test]
    fn release_readiness_not_ready_until_all() {
        let mut status = ReleaseReadinessStatus::new();
        status.app_identity_ready = true;
        status.signing_key_ready = true;
        status.release_bundle_ready = true;
        status.versioning_ready = true;
        status.store_assets_ready = true;
        status.store_listing_ready = true;
        status.permissions_ready = true;
        status.reviewer_access_ready = true;
        status.rollout_plan_ready = true;
        // missing production_stack_ready and data_safety_submitted
        assert!(!status.is_ready_for_publication());
    }

    #[test]
    fn release_readiness_ready_when_all_true() {
        let mut status = ReleaseReadinessStatus::new();
        status.app_identity_ready = true;
        status.signing_key_ready = true;
        status.release_bundle_ready = true;
        status.versioning_ready = true;
        status.store_assets_ready = true;
        status.store_listing_ready = true;
        status.permissions_ready = true;
        status.reviewer_access_ready = true;
        status.rollout_plan_ready = true;
        status.production_stack_ready = true;
        status.data_safety_submitted = true;
        assert!(status.is_ready_for_publication());
    }

    #[test]
    fn release_readiness_total_count_is_11() {
        let status = ReleaseReadinessStatus::new();
        assert_eq!(status.total_count(), 11);
    }

    #[test]
    fn release_readiness_serializes_round_trip() {
        let status = ReleaseReadinessStatus::new();
        let json = serde_json::to_string(&status).unwrap();
        let back: ReleaseReadinessStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, back);
    }

    #[test]
    fn build_release_readiness_status_from_components() {
        let identity = AppIdentity::new().finalize();
        let signing = SigningKeyConfig::new().generate().backup().enroll();
        let bundle = ReleaseBundle::new().build(50_000_000, "abc").sign().approve();
        let versioning = VersioningPolicy::new();
        let assets = StoreAssets::new();
        let listing = StoreListing::new()
            .with_short_description("Test")
            .with_full_description("Full")
            .with_privacy_url("https://stride.app/privacy")
            .complete_content_rating();
        let permissions = PermissionDeclarations::new();
        let reviewer = ReviewerAccess::new();
        let stack = build_default_production_stack();

        let status = build_release_readiness_status(
            &identity,
            &signing,
            &bundle,
            &versioning,
            &assets,
            &listing,
            &permissions,
            &reviewer,
            &stack,
            true,
        );

        assert!(status.app_identity_ready);
        assert!(status.signing_key_ready);
        assert!(status.release_bundle_ready);
        assert!(status.versioning_ready);
        assert!(!status.store_assets_ready); // not all assets finalized
        assert!(status.store_listing_ready);
        assert!(!status.permissions_ready); // no declarations
        assert!(status.reviewer_access_ready);
        assert!(status.data_safety_submitted);
    }
}

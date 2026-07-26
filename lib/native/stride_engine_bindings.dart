import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'stride_native_library.dart';

// ─── Native C function signatures ───────────────────────────────────────────
// These mirror the `extern "C"` functions in
// `rust/stride_engine/src/ffi.rs` one-to-one. Every function that returns
// `*mut c_char` hands Dart an owned, heap-allocated JSON string that MUST
// be freed via `stride_free_string` exactly once.

typedef _StrideEngineVersionNative = Pointer<Utf8> Function();
typedef _StrideEngineVersionDart = Pointer<Utf8> Function();

typedef _StrideCreateSessionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCreateSessionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDestroySessionNative = Pointer<Utf8> Function(Int64);
typedef _StrideDestroySessionDart = Pointer<Utf8> Function(int);

typedef _StrideStartNative = Pointer<Utf8> Function(Int64);
typedef _StrideStartDart = Pointer<Utf8> Function(int);

typedef _StridePauseNative = Pointer<Utf8> Function(Int64, Int64);
typedef _StridePauseDart = Pointer<Utf8> Function(int, int);

typedef _StrideResumeNative = Pointer<Utf8> Function(Int64);
typedef _StrideResumeDart = Pointer<Utf8> Function(int);

typedef _StrideDiscardNative = Pointer<Utf8> Function(Int64);
typedef _StrideDiscardDart = Pointer<Utf8> Function(int);

typedef _StrideSetGoalNative = Pointer<Utf8> Function(Int64, Pointer<Utf8>);
typedef _StrideSetGoalDart = Pointer<Utf8> Function(int, Pointer<Utf8>);

typedef _StrideAddLocationSampleNative = Pointer<Utf8> Function(
    Int64, Pointer<Utf8>, Int64);
typedef _StrideAddLocationSampleDart = Pointer<Utf8> Function(
    int, Pointer<Utf8>, int);

typedef _StrideAddHeartRateSampleNative = Pointer<Utf8> Function(
    Int64, Int64, Uint16);
typedef _StrideAddHeartRateSampleDart = Pointer<Utf8> Function(
    int, int, int);

typedef _StrideAddStepDeltaNative = Pointer<Utf8> Function(
    Int64, Int64, Uint32, Pointer<Utf8>);
typedef _StrideAddStepDeltaDart = Pointer<Utf8> Function(
    int, int, int, Pointer<Utf8>);

typedef _StrideTickNative = Pointer<Utf8> Function(Int64, Int64);
typedef _StrideTickDart = Pointer<Utf8> Function(int, int);

typedef _StrideManualLapNative = Pointer<Utf8> Function(Int64, Int64);
typedef _StrideManualLapDart = Pointer<Utf8> Function(int, int);

typedef _StrideBuildCheckpointNative = Pointer<Utf8> Function(Int64, Int64);
typedef _StrideBuildCheckpointDart = Pointer<Utf8> Function(int, int);

typedef _StrideFinishNative = Pointer<Utf8> Function(Int64, Int64);
typedef _StrideFinishDart = Pointer<Utf8> Function(int, int);

typedef _StrideGetSessionNative = Pointer<Utf8> Function(Int64);
typedef _StrideGetSessionDart = Pointer<Utf8> Function(int);

typedef _StrideEvaluateRecoveryNative = Pointer<Utf8> Function(
    Pointer<Utf8>, Int64);
typedef _StrideEvaluateRecoveryDart = Pointer<Utf8> Function(
    Pointer<Utf8>, int);

typedef _StrideRestoreSessionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideRestoreSessionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideSetPriorBestsNative = Pointer<Utf8> Function(
    Int64, Pointer<Utf8>);
typedef _StrideSetPriorBestsDart = Pointer<Utf8> Function(
    int, Pointer<Utf8>);

// Stateless decision-logic entry points (no session handle required).
typedef _StrideValidateNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideValidateDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDetectPersonalRecordsNative = Pointer<Utf8> Function(
    Pointer<Utf8>);
typedef _StrideDetectPersonalRecordsDart = Pointer<Utf8> Function(
    Pointer<Utf8>);

typedef _StrideRequiredPermissionActionNative = Pointer<Utf8> Function(
    Pointer<Utf8>);
typedef _StrideRequiredPermissionActionDart = Pointer<Utf8> Function(
    Pointer<Utf8>);

typedef _StrideEvaluateStartCapabilityNative = Pointer<Utf8> Function(
    Pointer<Utf8>);
typedef _StrideEvaluateStartCapabilityDart = Pointer<Utf8> Function(
    Pointer<Utf8>);

typedef _StrideChooseSamplingProfileNative = Pointer<Utf8> Function(
    Pointer<Utf8>);
typedef _StrideChooseSamplingProfileDart = Pointer<Utf8> Function(
    Pointer<Utf8>);

typedef _StrideConvertUnitNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideConvertUnitDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideFormatPaceNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideFormatPaceDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDetectAchievementsNative = Pointer<Utf8> Function(
    Pointer<Utf8>);
typedef _StrideDetectAchievementsDart = Pointer<Utf8> Function(Pointer<Utf8>);

// Cloud synchronization engine (spec section 3) — stateless decision logic.
typedef _StrideComputeSyncBackoffNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideComputeSyncBackoffDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDecideSyncRetryNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideDecideSyncRetryDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideResolveSyncConflictNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideResolveSyncConflictDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDetectSyncDuplicateNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideDetectSyncDuplicateDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDecideSyncUpsertNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideDecideSyncUpsertDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDecideDeviceSyncNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideDecideDeviceSyncDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideTombstoneShouldRetryNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideTombstoneShouldRetryDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideTombstoneShouldGcNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideTombstoneShouldGcDart = Pointer<Utf8> Function(Pointer<Utf8>);

// Route file format & storage layout (spec section 4) — stateless logic.
typedef _StrideDecideRouteFormatNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideDecideRouteFormatDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideGenerateRouteFilePathNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideGenerateRouteFilePathDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideEstimateRouteFileSizeNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideEstimateRouteFileSizeDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideShouldPreferWifiForUploadNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideShouldPreferWifiForUploadDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideSerializeGpxNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSerializeGpxDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBuildRouteFileMetadataNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBuildRouteFileMetadataDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideIsRouteSyncTerminalNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideIsRouteSyncTerminalDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideValidateRouteSummaryNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideValidateRouteSummaryDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDominantLocationSourceNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideDominantLocationSourceDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideLocationSourceLabelNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideLocationSourceLabelDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideProviderForViewNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideProviderForViewDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideAttributionForProviderNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideAttributionForProviderDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideAttributionForViewNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideAttributionForViewDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideTileUrlTemplateNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideTileUrlTemplateDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideClassifyGpsAccuracyNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideClassifyGpsAccuracyDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideGpsAccuracyDescriptionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideGpsAccuracyDescriptionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideGpsAccuracyColorNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideGpsAccuracyColorDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideLatLonToTileNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideLatLonToTileDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCountTilesInRegionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCountTilesInRegionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBuildOfflineRegionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBuildOfflineRegionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCheckStorageAvailabilityNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCheckStorageAvailabilityDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCanAddRegionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCanAddRegionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideSelectEvictionCandidateNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSelectEvictionCandidateDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideIsRegionStaleNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideIsRegionStaleDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideTouchRegionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideTouchRegionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideSimplificationEpsilonForZoomNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSimplificationEpsilonForZoomDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideShouldShowFullResolutionRouteNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideShouldShowFullResolutionRouteDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideShouldShowRecenterButtonNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideShouldShowRecenterButtonDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideShouldRotateWithHeadingNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideShouldRotateWithHeadingDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideValidateSavedRouteNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideValidateSavedRouteDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideTileCacheKeyNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideTileCacheKeyDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideTileProviderSlugNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideTileProviderSlugDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideFreeStringNative = Void Function(Pointer<Utf8>);
typedef _StrideFreeStringDart = void Function(Pointer<Utf8>);

/// Thin, allocation-safe wrapper around the raw native symbols. All public
/// methods return decoded `Map<String, dynamic>` (the `{"ok":..,"data"/
/// "error":..}` envelope) so callers never touch native memory directly —
/// every native string is read and freed inside this class.
class StrideEngineBindings {
  StrideEngineBindings._(DynamicLibrary lib) : _lib = lib {
    _version = _lib
        .lookupFunction<_StrideEngineVersionNative, _StrideEngineVersionDart>(
            'stride_engine_version');
    _createSession = _lib.lookupFunction<_StrideCreateSessionNative,
        _StrideCreateSessionDart>('stride_create_session');
    _destroySession = _lib.lookupFunction<_StrideDestroySessionNative,
        _StrideDestroySessionDart>('stride_destroy_session');
    _start =
        _lib.lookupFunction<_StrideStartNative, _StrideStartDart>('stride_start');
    _pause =
        _lib.lookupFunction<_StridePauseNative, _StridePauseDart>('stride_pause');
    _resume = _lib
        .lookupFunction<_StrideResumeNative, _StrideResumeDart>('stride_resume');
    _discard = _lib.lookupFunction<_StrideDiscardNative, _StrideDiscardDart>(
        'stride_discard');
    _setGoal = _lib.lookupFunction<_StrideSetGoalNative, _StrideSetGoalDart>(
        'stride_set_goal');
    _addLocationSample = _lib.lookupFunction<_StrideAddLocationSampleNative,
        _StrideAddLocationSampleDart>('stride_add_location_sample');
    _addHeartRateSample = _lib.lookupFunction<_StrideAddHeartRateSampleNative,
        _StrideAddHeartRateSampleDart>('stride_add_heart_rate_sample');
    _addStepDelta = _lib.lookupFunction<_StrideAddStepDeltaNative,
        _StrideAddStepDeltaDart>('stride_add_step_delta');
    _tick =
        _lib.lookupFunction<_StrideTickNative, _StrideTickDart>('stride_tick');
    _manualLap = _lib.lookupFunction<_StrideManualLapNative,
        _StrideManualLapDart>('stride_manual_lap');
    _buildCheckpoint = _lib.lookupFunction<_StrideBuildCheckpointNative,
        _StrideBuildCheckpointDart>('stride_build_checkpoint');
    _finish = _lib
        .lookupFunction<_StrideFinishNative, _StrideFinishDart>('stride_finish');
    _getSession = _lib.lookupFunction<_StrideGetSessionNative,
        _StrideGetSessionDart>('stride_get_session');
    _evaluateRecovery = _lib.lookupFunction<_StrideEvaluateRecoveryNative,
        _StrideEvaluateRecoveryDart>('stride_evaluate_recovery');
    _restoreSession = _lib.lookupFunction<_StrideRestoreSessionNative,
        _StrideRestoreSessionDart>('stride_restore_session');
    _setPriorBests = _lib.lookupFunction<_StrideSetPriorBestsNative,
        _StrideSetPriorBestsDart>('stride_set_prior_bests');
    _validate = _lib.lookupFunction<_StrideValidateNative,
        _StrideValidateDart>('stride_validate');
    _detectPersonalRecords = _lib.lookupFunction<
        _StrideDetectPersonalRecordsNative,
        _StrideDetectPersonalRecordsDart>('stride_detect_personal_records');
    _requiredPermissionAction = _lib.lookupFunction<
        _StrideRequiredPermissionActionNative,
        _StrideRequiredPermissionActionDart>(
        'stride_required_permission_action');
    _evaluateStartCapability = _lib.lookupFunction<
        _StrideEvaluateStartCapabilityNative,
        _StrideEvaluateStartCapabilityDart>('stride_evaluate_start_capability');
    _chooseSamplingProfile = _lib.lookupFunction<
        _StrideChooseSamplingProfileNative,
        _StrideChooseSamplingProfileDart>('stride_choose_sampling_profile');
    _convertUnit = _lib.lookupFunction<_StrideConvertUnitNative,
        _StrideConvertUnitDart>('stride_convert_unit');
    _formatPace = _lib.lookupFunction<_StrideFormatPaceNative,
        _StrideFormatPaceDart>('stride_format_pace');
    _detectAchievements = _lib.lookupFunction<_StrideDetectAchievementsNative,
        _StrideDetectAchievementsDart>('stride_detect_achievements');
    _computeSyncBackoff = _lib.lookupFunction<_StrideComputeSyncBackoffNative,
        _StrideComputeSyncBackoffDart>('stride_compute_sync_backoff');
    _decideSyncRetry = _lib.lookupFunction<_StrideDecideSyncRetryNative,
        _StrideDecideSyncRetryDart>('stride_decide_sync_retry');
    _resolveSyncConflict = _lib.lookupFunction<_StrideResolveSyncConflictNative,
        _StrideResolveSyncConflictDart>('stride_resolve_sync_conflict');
    _detectSyncDuplicate = _lib.lookupFunction<_StrideDetectSyncDuplicateNative,
        _StrideDetectSyncDuplicateDart>('stride_detect_sync_duplicate');
    _decideSyncUpsert = _lib.lookupFunction<_StrideDecideSyncUpsertNative,
        _StrideDecideSyncUpsertDart>('stride_decide_sync_upsert');
    _decideDeviceSync = _lib.lookupFunction<_StrideDecideDeviceSyncNative,
        _StrideDecideDeviceSyncDart>('stride_decide_device_sync');
    _tombstoneShouldRetry = _lib.lookupFunction<_StrideTombstoneShouldRetryNative,
        _StrideTombstoneShouldRetryDart>('stride_tombstone_should_retry');
    _tombstoneShouldGc = _lib.lookupFunction<_StrideTombstoneShouldGcNative,
        _StrideTombstoneShouldGcDart>('stride_tombstone_should_gc');
    _decideRouteFormat = _lib.lookupFunction<_StrideDecideRouteFormatNative,
        _StrideDecideRouteFormatDart>('stride_decide_route_format');
    _generateRouteFilePath = _lib.lookupFunction<_StrideGenerateRouteFilePathNative,
        _StrideGenerateRouteFilePathDart>('stride_generate_route_file_path');
    _estimateRouteFileSize = _lib.lookupFunction<_StrideEstimateRouteFileSizeNative,
        _StrideEstimateRouteFileSizeDart>('stride_estimate_route_file_size');
    _shouldPreferWifiForUpload = _lib.lookupFunction<_StrideShouldPreferWifiForUploadNative,
        _StrideShouldPreferWifiForUploadDart>('stride_should_prefer_wifi_for_upload');
    _serializeGpx = _lib.lookupFunction<_StrideSerializeGpxNative,
        _StrideSerializeGpxDart>('stride_serialize_gpx');
    _buildRouteFileMetadata = _lib.lookupFunction<_StrideBuildRouteFileMetadataNative,
        _StrideBuildRouteFileMetadataDart>('stride_build_route_file_metadata');
    _isRouteSyncTerminal = _lib.lookupFunction<_StrideIsRouteSyncTerminalNative,
        _StrideIsRouteSyncTerminalDart>('stride_is_route_sync_terminal');
    _validateRouteSummary = _lib.lookupFunction<_StrideValidateRouteSummaryNative,
        _StrideValidateRouteSummaryDart>('stride_validate_route_summary');
    _dominantLocationSource = _lib.lookupFunction<_StrideDominantLocationSourceNative,
        _StrideDominantLocationSourceDart>('stride_dominant_location_source');
    _locationSourceLabel = _lib.lookupFunction<_StrideLocationSourceLabelNative,
        _StrideLocationSourceLabelDart>('stride_location_source_label');
    _providerForView = _lib.lookupFunction<_StrideProviderForViewNative,
        _StrideProviderForViewDart>('stride_provider_for_view');
    _attributionForProvider = _lib.lookupFunction<_StrideAttributionForProviderNative,
        _StrideAttributionForProviderDart>('stride_attribution_for_provider');
    _attributionForView = _lib.lookupFunction<_StrideAttributionForViewNative,
        _StrideAttributionForViewDart>('stride_attribution_for_view');
    _tileUrlTemplate = _lib.lookupFunction<_StrideTileUrlTemplateNative,
        _StrideTileUrlTemplateDart>('stride_tile_url_template');
    _classifyGpsAccuracy = _lib.lookupFunction<_StrideClassifyGpsAccuracyNative,
        _StrideClassifyGpsAccuracyDart>('stride_classify_gps_accuracy');
    _gpsAccuracyDescription = _lib.lookupFunction<_StrideGpsAccuracyDescriptionNative,
        _StrideGpsAccuracyDescriptionDart>('stride_gps_accuracy_description');
    _gpsAccuracyColor = _lib.lookupFunction<_StrideGpsAccuracyColorNative,
        _StrideGpsAccuracyColorDart>('stride_gps_accuracy_color');
    _latLonToTile = _lib.lookupFunction<_StrideLatLonToTileNative,
        _StrideLatLonToTileDart>('stride_lat_lon_to_tile');
    _countTilesInRegion = _lib.lookupFunction<_StrideCountTilesInRegionNative,
        _StrideCountTilesInRegionDart>('stride_count_tiles_in_region');
    _buildOfflineRegion = _lib.lookupFunction<_StrideBuildOfflineRegionNative,
        _StrideBuildOfflineRegionDart>('stride_build_offline_region');
    _checkStorageAvailability = _lib.lookupFunction<_StrideCheckStorageAvailabilityNative,
        _StrideCheckStorageAvailabilityDart>('stride_check_storage_availability');
    _canAddRegion = _lib.lookupFunction<_StrideCanAddRegionNative,
        _StrideCanAddRegionDart>('stride_can_add_region');
    _selectEvictionCandidate = _lib.lookupFunction<_StrideSelectEvictionCandidateNative,
        _StrideSelectEvictionCandidateDart>('stride_select_eviction_candidate');
    _isRegionStale = _lib.lookupFunction<_StrideIsRegionStaleNative,
        _StrideIsRegionStaleDart>('stride_is_region_stale');
    _touchRegion = _lib.lookupFunction<_StrideTouchRegionNative,
        _StrideTouchRegionDart>('stride_touch_region');
    _simplificationEpsilonForZoom = _lib.lookupFunction<_StrideSimplificationEpsilonForZoomNative,
        _StrideSimplificationEpsilonForZoomDart>('stride_simplification_epsilon_for_zoom');
    _shouldShowFullResolutionRoute = _lib.lookupFunction<_StrideShouldShowFullResolutionRouteNative,
        _StrideShouldShowFullResolutionRouteDart>('stride_should_show_full_resolution_route');
    _shouldShowRecenterButton = _lib.lookupFunction<_StrideShouldShowRecenterButtonNative,
        _StrideShouldShowRecenterButtonDart>('stride_should_show_recenter_button');
    _shouldRotateWithHeading = _lib.lookupFunction<_StrideShouldRotateWithHeadingNative,
        _StrideShouldRotateWithHeadingDart>('stride_should_rotate_with_heading');
    _validateSavedRoute = _lib.lookupFunction<_StrideValidateSavedRouteNative,
        _StrideValidateSavedRouteDart>('stride_validate_saved_route');
    _tileCacheKey = _lib.lookupFunction<_StrideTileCacheKeyNative,
        _StrideTileCacheKeyDart>('stride_tile_cache_key');
    _tileProviderSlug = _lib.lookupFunction<_StrideTileProviderSlugNative,
        _StrideTileProviderSlugDart>('stride_tile_provider_slug');
    _freeString = _lib.lookupFunction<_StrideFreeStringNative,
        _StrideFreeStringDart>('stride_free_string');
  }

  static StrideEngineBindings? _instance;

  /// Lazily loads and looks up all native symbols exactly once per process.
  factory StrideEngineBindings() {
    return _instance ??= StrideEngineBindings._(loadStrideEngineLibrary());
  }

  final DynamicLibrary _lib;

  late final _StrideEngineVersionDart _version;
  late final _StrideCreateSessionDart _createSession;
  late final _StrideDestroySessionDart _destroySession;
  late final _StrideStartDart _start;
  late final _StridePauseDart _pause;
  late final _StrideResumeDart _resume;
  late final _StrideDiscardDart _discard;
  late final _StrideSetGoalDart _setGoal;
  late final _StrideAddLocationSampleDart _addLocationSample;
  late final _StrideAddHeartRateSampleDart _addHeartRateSample;
  late final _StrideAddStepDeltaDart _addStepDelta;
  late final _StrideTickDart _tick;
  late final _StrideManualLapDart _manualLap;
  late final _StrideBuildCheckpointDart _buildCheckpoint;
  late final _StrideFinishDart _finish;
  late final _StrideGetSessionDart _getSession;
  late final _StrideEvaluateRecoveryDart _evaluateRecovery;
  late final _StrideRestoreSessionDart _restoreSession;
  late final _StrideSetPriorBestsDart _setPriorBests;
  late final _StrideValidateDart _validate;
  late final _StrideDetectPersonalRecordsDart _detectPersonalRecords;
  late final _StrideRequiredPermissionActionDart _requiredPermissionAction;
  late final _StrideEvaluateStartCapabilityDart _evaluateStartCapability;
  late final _StrideChooseSamplingProfileDart _chooseSamplingProfile;
  late final _StrideConvertUnitDart _convertUnit;
  late final _StrideFormatPaceDart _formatPace;
  late final _StrideDetectAchievementsDart _detectAchievements;
  late final _StrideComputeSyncBackoffDart _computeSyncBackoff;
  late final _StrideDecideSyncRetryDart _decideSyncRetry;
  late final _StrideResolveSyncConflictDart _resolveSyncConflict;
  late final _StrideDetectSyncDuplicateDart _detectSyncDuplicate;
  late final _StrideDecideSyncUpsertDart _decideSyncUpsert;
  late final _StrideDecideDeviceSyncDart _decideDeviceSync;
  late final _StrideTombstoneShouldRetryDart _tombstoneShouldRetry;
  late final _StrideTombstoneShouldGcDart _tombstoneShouldGc;
  late final _StrideDecideRouteFormatDart _decideRouteFormat;
  late final _StrideGenerateRouteFilePathDart _generateRouteFilePath;
  late final _StrideEstimateRouteFileSizeDart _estimateRouteFileSize;
  late final _StrideShouldPreferWifiForUploadDart _shouldPreferWifiForUpload;
  late final _StrideSerializeGpxDart _serializeGpx;
  late final _StrideBuildRouteFileMetadataDart _buildRouteFileMetadata;
  late final _StrideIsRouteSyncTerminalDart _isRouteSyncTerminal;
  late final _StrideValidateRouteSummaryDart _validateRouteSummary;
  late final _StrideDominantLocationSourceDart _dominantLocationSource;
  late final _StrideLocationSourceLabelDart _locationSourceLabel;
  late final _StrideProviderForViewDart _providerForView;
  late final _StrideAttributionForProviderDart _attributionForProvider;
  late final _StrideAttributionForViewDart _attributionForView;
  late final _StrideTileUrlTemplateDart _tileUrlTemplate;
  late final _StrideClassifyGpsAccuracyDart _classifyGpsAccuracy;
  late final _StrideGpsAccuracyDescriptionDart _gpsAccuracyDescription;
  late final _StrideGpsAccuracyColorDart _gpsAccuracyColor;
  late final _StrideLatLonToTileDart _latLonToTile;
  late final _StrideCountTilesInRegionDart _countTilesInRegion;
  late final _StrideBuildOfflineRegionDart _buildOfflineRegion;
  late final _StrideCheckStorageAvailabilityDart _checkStorageAvailability;
  late final _StrideCanAddRegionDart _canAddRegion;
  late final _StrideSelectEvictionCandidateDart _selectEvictionCandidate;
  late final _StrideIsRegionStaleDart _isRegionStale;
  late final _StrideTouchRegionDart _touchRegion;
  late final _StrideSimplificationEpsilonForZoomDart _simplificationEpsilonForZoom;
  late final _StrideShouldShowFullResolutionRouteDart _shouldShowFullResolutionRoute;
  late final _StrideShouldShowRecenterButtonDart _shouldShowRecenterButton;
  late final _StrideShouldRotateWithHeadingDart _shouldRotateWithHeading;
  late final _StrideValidateSavedRouteDart _validateSavedRoute;
  late final _StrideTileCacheKeyDart _tileCacheKey;
  late final _StrideTileProviderSlugDart _tileProviderSlug;
  late final _StrideFreeStringDart _freeString;

  /// Reads, decodes, and frees a native JSON string pointer.
  Map<String, dynamic> _consume(Pointer<Utf8> ptr) {
    try {
      final jsonStr = ptr.toDartString();
      final decoded = jsonDecode(jsonStr);
      return decoded as Map<String, dynamic>;
    } finally {
      _freeString(ptr);
    }
  }

  Pointer<Utf8> _toNative(String s) => s.toNativeUtf8();

  String version() {
    final env = _consume(_version());
    return env['data'] as String? ?? 'unknown';
  }

  Map<String, dynamic> createSession(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_createSession(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  Map<String, dynamic> destroySession(int handle) =>
      _consume(_destroySession(handle));

  Map<String, dynamic> start(int handle) => _consume(_start(handle));

  Map<String, dynamic> pause(int handle, int nowMs) =>
      _consume(_pause(handle, nowMs));

  Map<String, dynamic> resume(int handle) => _consume(_resume(handle));

  Map<String, dynamic> discard(int handle) => _consume(_discard(handle));

  Map<String, dynamic> setGoal(int handle, Map<String, dynamic> goal) {
    final ptr = _toNative(jsonEncode(goal));
    try {
      return _consume(_setGoal(handle, ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  Map<String, dynamic> addLocationSample(
    int handle,
    Map<String, dynamic> point,
    int nowMs,
  ) {
    final ptr = _toNative(jsonEncode(point));
    try {
      return _consume(_addLocationSample(handle, ptr, nowMs));
    } finally {
      malloc.free(ptr);
    }
  }

  Map<String, dynamic> addHeartRateSample(int handle, int atMs, int bpm) =>
      _consume(_addHeartRateSample(handle, atMs, bpm));

  Map<String, dynamic> addStepDelta(
    int handle,
    int atMs,
    int delta,
    String source,
  ) {
    final ptr = _toNative(source);
    try {
      return _consume(_addStepDelta(handle, atMs, delta, ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  Map<String, dynamic> tick(int handle, int nowMs) =>
      _consume(_tick(handle, nowMs));

  Map<String, dynamic> manualLap(int handle, int nowMs) =>
      _consume(_manualLap(handle, nowMs));

  Map<String, dynamic> buildCheckpoint(int handle, int nowMs) =>
      _consume(_buildCheckpoint(handle, nowMs));

  Map<String, dynamic> finish(int handle, int nowMs) =>
      _consume(_finish(handle, nowMs));

  Map<String, dynamic> getSession(int handle) => _consume(_getSession(handle));

  /// Pure decision logic: given a persisted checkpoint JSON and the current
  /// wall-clock time, asks the Rust engine whether the workout should be
  /// resumed, finished-and-saved, or discarded (see
  /// `engine::recovery::evaluate_checkpoint`). Does not touch the session
  /// registry — safe to call before any handle exists.
  Map<String, dynamic> evaluateRecovery(
    Map<String, dynamic> checkpoint,
    int nowMs,
  ) {
    final ptr = _toNative(jsonEncode(checkpoint));
    try {
      return _consume(_evaluateRecovery(ptr, nowMs));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Rebuilds a live `WorkoutSessionController` from a persisted checkpoint
  /// and registers it under a new handle, returned in the response payload
  /// as `{"handle": .., "workout_id": ..}`. The restored controller always
  /// starts in the `Paused` state (see `restore_from_checkpoint` doc
  /// comments in `engine/controller.rs`).
  Map<String, dynamic> restoreSession(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_restoreSession(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Supplies prior personal bests (loaded from local/cloud history) so
  /// the engine can emit a live "personal record possible" coaching nudge.
  /// Optional; call right after `createSession` if history is available.
  Map<String, dynamic> setPriorBests(
    int handle,
    Map<String, dynamic> priorBests,
  ) {
    final ptr = _toNative(jsonEncode(priorBests));
    try {
      return _consume(_setPriorBests(handle, ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Re-runs full workout validation (spec section 26). Dart should call
  /// this after `finish()`, filling in `is_duplicate_of_existing_workout`
  /// from its own history lookup, and OR the `is_blocked` result with the
  /// one `finish()` already returned.
  Map<String, dynamic> validate(Map<String, dynamic> validationInput) {
    final ptr = _toNative(jsonEncode(validationInput));
    try {
      return _consume(_validate(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Detects personal records for a finalized `WorkoutSummary` against
  /// caller-supplied prior bests loaded from history.
  Map<String, dynamic> detectPersonalRecords(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_detectPersonalRecords(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Permissions decision logic (spec section 30). Pure/stateless.
  Map<String, dynamic> requiredPermissionAction(
    Map<String, dynamic> request,
  ) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_requiredPermissionAction(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Decides whether a workout can start full-featured, foreground-only,
  /// or blocked, given precise/background location permission states.
  Map<String, dynamic> evaluateStartCapability(
    Map<String, dynamic> request,
  ) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_evaluateStartCapability(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Battery/sampling manager (spec section 32). Call whenever activity
  /// level or battery state changes to get an updated `SamplingProfile`.
  Map<String, dynamic> chooseSamplingProfile(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_chooseSamplingProfile(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Unit system (spec section 34): applies a single named conversion.
  Map<String, dynamic> convertUnit(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_convertUnit(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Formats a canonical pace (sec/km) as "M:SS" for a display unit.
  Map<String, dynamic> formatPace(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_formatPace(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Achievement/badge engine (spec section 29). Dart owns loading history
  /// beforehand and persisting any newly-returned awards afterward.
  Map<String, dynamic> detectAchievements(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_detectAchievements(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  // ─── Cloud synchronization engine (spec section 3) ───────────────

  /// Computes the retry delay (ms) for a sync attempt using exponential
  /// backoff with jitter. `request`: `{"attempt": 2, "jitter_seed": 42}`.
  /// Returns `{"delay_ms": 4000}`.
  Map<String, dynamic> computeSyncBackoff(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_computeSyncBackoff(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Decides whether a failed sync attempt should be retried, and if so,
  /// after how long.
  /// `request`: `{"result": "retryable_failure", "current_retry_count": 2,
  /// "jitter_seed": 42}`.
  /// Returns `{"retry": true, "delay_ms": 4000}` or `{"retry": false}`.
  Map<String, dynamic> decideSyncRetry(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_decideSyncRetry(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Resolves a sync conflict between local and cloud versions.
  /// `request`: `{"workout_id": "w1", "local_updated_at": 2000,
  /// "cloud_updated_at": 1000, "cloud_device_id": "device-b",
  /// "local_device_id": "device-a", "strategy": "last_write_wins"}`.
  /// Returns `{"decision": "keep_local"}`.
  Map<String, dynamic> resolveSyncConflict(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_resolveSyncConflict(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Checks whether a local workout is a duplicate of an existing cloud
  /// workout. `request`: `{"local_workout_id": "w1",
  /// "local_updated_at": 1000, "cloud_workout_id": "w1",
  /// "cloud_updated_at": 1000}`.
  /// Returns `{"is_duplicate": true}`.
  Map<String, dynamic> detectSyncDuplicate(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_detectSyncDuplicate(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Decides whether to insert, update, or skip a workout upsert.
  /// Same request shape as `detectSyncDuplicate`.
  /// Returns `{"decision": "insert"}`.
  Map<String, dynamic> decideSyncUpsert(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_decideSyncUpsert(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Decides what action the current device should take for a workout in
  /// device-to-device sync. `request`: `{"recording_device_id": "device-a",
  /// "current_device_id": "device-a", "is_uploaded": false,
  /// "is_downloaded": false}`.
  /// Returns `{"action": "upload"}`.
  Map<String, dynamic> decideDeviceSync(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_decideDeviceSync(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Decides whether a deletion tombstone should be retried.
  /// `request`: `{"state": "failed", "retry_count": 3}`.
  /// Returns `{"should_retry": true}`.
  Map<String, dynamic> tombstoneShouldRetry(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_tombstoneShouldRetry(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Decides whether a synced tombstone is old enough to be GC'd.
  /// `request`: `{"state": "synced", "synced_at": 1000, "now_ms": 999999}`.
  /// Returns `{"should_gc": true}`.
  Map<String, dynamic> tombstoneShouldGc(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_tombstoneShouldGc(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  // --- Route file format & storage layout (spec section 4) ---------

  /// Decides which route file format to use for a workout.
  /// `request`: `{"point_count": 1000, "is_offline": false,
  /// "wants_gpx_export": false}`.
  /// Returns `{"format": "gpx"}`.
  Map<String, dynamic> decideRouteFormat(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_decideRouteFormat(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Generates the Cloud Storage object key for a route file.
  /// `request`: `{"user_id": "u1", "workout_id": "wk1",
  /// "format": "gpx"}`.
  /// Returns `{"path": "routes/u1/wk1.gpx"}` or `{"path": null}`.
  Map<String, dynamic> generateRouteFilePath(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_generateRouteFilePath(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Estimates the byte size of a route file before serialization.
  /// `request`: `{"format": "gpx", "point_count": 1000}`.
  /// Returns `{"size_bytes": 180512}`.
  Map<String, dynamic> estimateRouteFileSize(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_estimateRouteFileSize(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Decides whether a route file upload should wait for Wi-Fi.
  /// `request`: `{"format": "gpx", "point_count": 4000,
  /// "is_on_wifi": false}`.
  /// Returns `{"should_prefer_wifi": true}`.
  Map<String, dynamic> shouldPreferWifiForUpload(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_shouldPreferWifiForUpload(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Serializes WorkoutPoints into a GPX 1.1 XML document.
  /// `request`: `{"workout_id": "wk1", "started_at_ms": 1000,
  /// "points": [{...}, ...]}`.
  /// Returns `{"gpx": "<gpx ...>...</gpx>"}`.
  Map<String, dynamic> serializeGpx(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_serializeGpx(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Builds route file metadata at workout-finish time.
  /// `request`: `{"user_id": "u1", "workout_id": "wk1",
  /// "points": [...], "device_source": "pixel-8",
  /// "is_offline": false, "wants_gpx_export": false,
  /// "finished_at": 1900000}`.
  /// Returns the full RouteFileMetadata JSON object.
  Map<String, dynamic> buildRouteFileMetadata(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_buildRouteFileMetadata(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Checks whether a route file sync state is terminal.
  /// `request`: `{"state": "uploaded"}`.
  /// Returns `{"is_terminal": true}`.
  Map<String, dynamic> isRouteSyncTerminal(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_isRouteSyncTerminal(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Validates that a RouteSummary is safe to write to Firestore.
  /// `request`: the full RouteSummary JSON object.
  /// Returns `{"valid": true}` or `{"valid": false, "error": "..."}`.
  Map<String, dynamic> validateRouteSummary(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_validateRouteSummary(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Determines the dominant LocationSource of a route.
  /// `request`: `{"points": [{...}, ...]}`.
  /// Returns `{"source": "phone_gps"}` or `{"source": null}`.
  Map<String, dynamic> dominantLocationSource(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_dominantLocationSource(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns the human-readable label for a LocationSource.
  /// `request`: `{"source": "phone_gps"}`.
  /// Returns `{"label": "Phone GPS"}`.
  Map<String, dynamic> locationSourceLabel(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_locationSourceLabel(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  // ─────────────────────────────────────────────────────────────────────
  // §5 Maps and location services
  // ─────────────────────────────────────────────────────────────────────

  /// Determines the tile provider for a given map view type.
  Map<String, dynamic> providerForView(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_providerForView(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns the attribution text for a tile provider.
  Map<String, dynamic> attributionForProvider(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_attributionForProvider(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns the attribution text for a map view type.
  Map<String, dynamic> attributionForView(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_attributionForView(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns the tile URL template for a provider.
  Map<String, dynamic> tileUrlTemplate(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_tileUrlTemplate(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Classifies a GPS accuracy value into a quality level.
  Map<String, dynamic> classifyGpsAccuracy(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_classifyGpsAccuracy(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns the description for a GPS accuracy level.
  Map<String, dynamic> gpsAccuracyDescription(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_gpsAccuracyDescription(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns the hex color for a GPS accuracy level.
  Map<String, dynamic> gpsAccuracyColor(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_gpsAccuracyColor(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Converts (lat, lon) to a tile coordinate at a zoom level.
  Map<String, dynamic> latLonToTile(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_latLonToTile(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Counts tiles needed to cover a bounding box across zoom levels.
  Map<String, dynamic> countTilesInRegion(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_countTilesInRegion(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Builds an offline region manifest from download parameters.
  Map<String, dynamic> buildOfflineRegion(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_buildOfflineRegion(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Checks whether the device has enough free storage for a new region.
  Map<String, dynamic> checkStorageAvailability(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_checkStorageAvailability(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Checks whether the user can save a new offline region.
  Map<String, dynamic> canAddRegion(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_canAddRegion(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Selects the LRU eviction candidate from existing regions.
  Map<String, dynamic> selectEvictionCandidate(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_selectEvictionCandidate(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Checks whether a downloaded region is stale.
  Map<String, dynamic> isRegionStale(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_isRegionStale(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Updates a region's last_accessed_at timestamp.
  Map<String, dynamic> touchRegion(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_touchRegion(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Selects the simplification epsilon for a zoom level.
  Map<String, dynamic> simplificationEpsilonForZoom(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_simplificationEpsilonForZoom(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Whether to show full-resolution route at a zoom level.
  Map<String, dynamic> shouldShowFullResolutionRoute(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_shouldShowFullResolutionRoute(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Whether the recenter button should be visible.
  Map<String, dynamic> shouldShowRecenterButton(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_shouldShowRecenterButton(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Whether the map should rotate with the user's heading.
  Map<String, dynamic> shouldRotateWithHeading(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_shouldRotateWithHeading(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Validates a saved route before persisting.
  Map<String, dynamic> validateSavedRoute(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_validateSavedRoute(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Builds the cache key for a tile.
  Map<String, dynamic> tileCacheKey(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_tileCacheKey(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns the short slug for a tile provider.
  Map<String, dynamic> tileProviderSlug(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_tileProviderSlug(ptr));
    } finally {
      malloc.free(ptr);
    }
  }
}

/// Thrown when the native engine reports `{"ok": false, ...}` for a call
/// that the Dart layer expected to succeed unconditionally.
class StrideEngineException implements Exception {
  StrideEngineException(this.message);
  final String message;

  @override
  String toString() => 'StrideEngineException: $message';
}

/// Unwraps the `{"ok": bool, "data"/"error": ...}` envelope, throwing
/// [StrideEngineException] on failure and returning the `data` payload
/// (which may itself be `null`/any JSON value) on success.
dynamic unwrapEnvelope(Map<String, dynamic> envelope) {
  if (envelope['ok'] == true) {
    return envelope['data'];
  }
  throw StrideEngineException(envelope['error']?.toString() ?? 'unknown_error');
}

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

// §6 — Authentication / account lifecycle

typedef _StrideAuthProviderLabelNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideAuthProviderLabelDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideClassifySessionStateNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideClassifySessionStateDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideNeedsTokenRefreshNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideNeedsTokenRefreshDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideRequiresReloginNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideRequiresReloginDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideRequiresReauthenticationNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideRequiresReauthenticationDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideReauthThresholdMsNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideReauthThresholdMsDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideReauthReasonNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideReauthReasonDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideDecideVerificationActionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideDecideVerificationActionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCanResendVerificationNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCanResendVerificationDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideValidatePasswordNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideValidatePasswordDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StridePasswordStrengthScoreNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StridePasswordStrengthScoreDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StridePasswordStrengthLabelNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StridePasswordStrengthLabelDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideValidateEmailNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideValidateEmailDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideAccountStatusMessageNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideAccountStatusMessageDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCanSignInNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCanSignInDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideAnalyzeLoginAttemptNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideAnalyzeLoginAttemptDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideEnumerateUserDataCategoriesNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideEnumerateUserDataCategoriesDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBuildDeletionPlanNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBuildDeletionPlanDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBuildDeletionResultNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBuildDeletionResultDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideShouldAutoSignoutNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideShouldAutoSignoutDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCanUpgradeAnonymousNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCanUpgradeAnonymousDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityCollectionPathTemplateNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityCollectionPathTemplateDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityCollectionIsAdminOnlyNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityCollectionIsAdminOnlyDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityCollectionIsUserScopedNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityCollectionIsUserScopedDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityCheckAccessNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityCheckAccessDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityValidatePathOwnershipNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityValidatePathOwnershipDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityFieldRulesForCollectionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityFieldRulesForCollectionDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityAllowedFieldsForCollectionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityAllowedFieldsForCollectionDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityValidateDocumentNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityValidateDocumentDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecuritySanitizeStringNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecuritySanitizeStringDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityDetectInjectionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityDetectInjectionDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityIsSafeStringNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityIsSafeStringDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityValidateStoragePathNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityValidateStoragePathDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityAppCheckDecisionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityAppCheckDecisionDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityPlayIntegrityDecisionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityPlayIntegrityDecisionDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityCheckRateLimitNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityCheckRateLimitDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityRateLimitConfigNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityRateLimitConfigDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityCheckForSecretsNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityCheckForSecretsDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityIsSecretFreeNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityIsSecretFreeDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityValidateProjectIdNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityValidateProjectIdDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityEnvironmentFromProjectIdNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityEnvironmentFromProjectIdDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityGenerateFirestoreRulesNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityGenerateFirestoreRulesDart = Pointer<Utf8> Function(Pointer<Utf8>);


typedef _StrideSecurityGenerateStorageRulesNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideSecurityGenerateStorageRulesDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanExperienceCapsNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanExperienceCapsDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanValidateDayPlanNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanValidateDayPlanDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanValidateWeeklyPlanNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanValidateWeeklyPlanDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanGenerateFallbackNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanGenerateFallbackDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanRespondToPainNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanRespondToPainDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanContainsDiagnosisNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanContainsDiagnosisDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanContainsWeightLossPromiseNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanContainsWeightLossPromiseDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanValidateCoachingTextNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanValidateCoachingTextDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanEscalationMessageNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanEscalationMessageDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanProcessUserFeedbackNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanProcessUserFeedbackDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanDecideAiAvailabilityNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanDecideAiAvailabilityDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanShouldUseFallbackNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanShouldUseFallbackDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanEstimateAiCostNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanEstimateAiCostDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanCacheKeyNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanCacheKeyDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanSummarizeWorkoutNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanSummarizeWorkoutDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanGenerateEncouragementNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanGenerateEncouragementDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanAdjustPlanNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanAdjustPlanDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanRecommendProgressionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanRecommendProgressionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCoachingPlanModerateRequestNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCoachingPlanModerateRequestDart = Pointer<Utf8> Function(Pointer<Utf8>);

// ─── §9 — Calorie/fitness calculations ───

typedef _StrideCalorieEstimateNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCalorieEstimateDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCalorieClampNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideCalorieClampDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideCalorieSourcePriorityNative = Pointer<Utf8> Function();
typedef _StrideCalorieSourcePriorityDart = Pointer<Utf8> Function();

// ─── §10 — Wearable / Health Connect ───

typedef _StrideWearableDecideFallbackNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideWearableDecideFallbackDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideWearableBuildStatusNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideWearableBuildStatusDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideWearableSyncStatusNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideWearableSyncStatusDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideWearableDeduplicateSourceNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideWearableDeduplicateSourceDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideWearableConsentResultNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideWearableConsentResultDart = Pointer<Utf8> Function(Pointer<Utf8>);

// ─── §11 — Music system ───────────────────────────────────────────

typedef _StrideMusicTransitionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideMusicTransitionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideMusicAudioFocusNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideMusicAudioFocusDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideMusicCoachingInteropNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideMusicCoachingInteropDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideMusicNetworkLossNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideMusicNetworkLossDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideMusicFilterBlockedNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideMusicFilterBlockedDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideMusicShouldRecommendNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideMusicShouldRecommendDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideMusicBuildStatusNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideMusicBuildStatusDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideMusicRemoteControlNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideMusicRemoteControlDart = Pointer<Utf8> Function(Pointer<Utf8>);

// ─── §12 — Background execution ────────────────────────────────────
typedef _StrideBackgroundServiceTransitionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBackgroundServiceTransitionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBackgroundCheckpointIntervalNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBackgroundCheckpointIntervalDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBackgroundEvaluateNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBackgroundEvaluateDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBackgroundProcessKillNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBackgroundProcessKillDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBackgroundPowerModeNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBackgroundPowerModeDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBackgroundInterruptionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBackgroundInterruptionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBackgroundBatteryAssessmentNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBackgroundBatteryAssessmentDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBackgroundBuildStatusNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBackgroundBuildStatusDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideBackgroundExplanationNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideBackgroundExplanationDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideNotificationDecideNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideNotificationDecideDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideNotificationContentNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideNotificationContentDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideNotificationQuietHoursNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideNotificationQuietHoursDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideNotificationCoachingNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideNotificationCoachingDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideNotificationStatusNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideNotificationStatusDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideNotificationNextReminderNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideNotificationNextReminderDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideErrorDecideRecoveryNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideErrorDecideRecoveryDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideErrorRetryDelayNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideErrorRetryDelayDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideErrorTransitionNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideErrorTransitionDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideErrorHealthStatusNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideErrorHealthStatusDart = Pointer<Utf8> Function(Pointer<Utf8>);

typedef _StrideErrorIsRecoverableNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef _StrideErrorIsRecoverableDart = Pointer<Utf8> Function(Pointer<Utf8>);

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
    // §6 — Authentication / account lifecycle
    _authProviderLabel = _lib.lookupFunction<_StrideAuthProviderLabelNative,
        _StrideAuthProviderLabelDart>('stride_auth_provider_label');
    _classifySessionState = _lib.lookupFunction<_StrideClassifySessionStateNative,
        _StrideClassifySessionStateDart>('stride_classify_session_state');
    _needsTokenRefresh = _lib.lookupFunction<_StrideNeedsTokenRefreshNative,
        _StrideNeedsTokenRefreshDart>('stride_needs_token_refresh');
    _requiresRelogin = _lib.lookupFunction<_StrideRequiresReloginNative,
        _StrideRequiresReloginDart>('stride_requires_relogin');
    _requiresReauthentication = _lib.lookupFunction<_StrideRequiresReauthenticationNative,
        _StrideRequiresReauthenticationDart>('stride_requires_reauthentication');
    _reauthThresholdMs = _lib.lookupFunction<_StrideReauthThresholdMsNative,
        _StrideReauthThresholdMsDart>('stride_reauth_threshold_ms');
    _reauthReason = _lib.lookupFunction<_StrideReauthReasonNative,
        _StrideReauthReasonDart>('stride_reauth_reason');
    _decideVerificationAction = _lib.lookupFunction<_StrideDecideVerificationActionNative,
        _StrideDecideVerificationActionDart>('stride_decide_verification_action');
    _canResendVerification = _lib.lookupFunction<_StrideCanResendVerificationNative,
        _StrideCanResendVerificationDart>('stride_can_resend_verification');
    _validatePassword = _lib.lookupFunction<_StrideValidatePasswordNative,
        _StrideValidatePasswordDart>('stride_validate_password');
    _passwordStrengthScore = _lib.lookupFunction<_StridePasswordStrengthScoreNative,
        _StridePasswordStrengthScoreDart>('stride_password_strength_score');
    _passwordStrengthLabel = _lib.lookupFunction<_StridePasswordStrengthLabelNative,
        _StridePasswordStrengthLabelDart>('stride_password_strength_label');
    _validateEmail = _lib.lookupFunction<_StrideValidateEmailNative,
        _StrideValidateEmailDart>('stride_validate_email');
    _accountStatusMessage = _lib.lookupFunction<_StrideAccountStatusMessageNative,
        _StrideAccountStatusMessageDart>('stride_account_status_message');
    _canSignIn = _lib.lookupFunction<_StrideCanSignInNative,
        _StrideCanSignInDart>('stride_can_sign_in');
    _analyzeLoginAttempt = _lib.lookupFunction<_StrideAnalyzeLoginAttemptNative,
        _StrideAnalyzeLoginAttemptDart>('stride_analyze_login_attempt');
    _enumerateUserDataCategories = _lib.lookupFunction<_StrideEnumerateUserDataCategoriesNative,
        _StrideEnumerateUserDataCategoriesDart>('stride_enumerate_user_data_categories');
    _buildDeletionPlan = _lib.lookupFunction<_StrideBuildDeletionPlanNative,
        _StrideBuildDeletionPlanDart>('stride_build_deletion_plan');
    _buildDeletionResult = _lib.lookupFunction<_StrideBuildDeletionResultNative,
        _StrideBuildDeletionResultDart>('stride_build_deletion_result');
    _shouldAutoSignout = _lib.lookupFunction<_StrideShouldAutoSignoutNative,
        _StrideShouldAutoSignoutDart>('stride_should_auto_signout');
    _canUpgradeAnonymous = _lib.lookupFunction<_StrideCanUpgradeAnonymousNative,
        _StrideCanUpgradeAnonymousDart>('stride_can_upgrade_anonymous');
    _securityCollectionPathTemplate = _lib.lookupFunction<_StrideSecurityCollectionPathTemplateNative,
        _StrideSecurityCollectionPathTemplateDart>('stride_security_collection_path_template');
    _securityCollectionIsAdminOnly = _lib.lookupFunction<_StrideSecurityCollectionIsAdminOnlyNative,
        _StrideSecurityCollectionIsAdminOnlyDart>('stride_security_collection_is_admin_only');
    _securityCollectionIsUserScoped = _lib.lookupFunction<_StrideSecurityCollectionIsUserScopedNative,
        _StrideSecurityCollectionIsUserScopedDart>('stride_security_collection_is_user_scoped');
    _securityCheckAccess = _lib.lookupFunction<_StrideSecurityCheckAccessNative,
        _StrideSecurityCheckAccessDart>('stride_security_check_access');
    _securityValidatePathOwnership = _lib.lookupFunction<_StrideSecurityValidatePathOwnershipNative,
        _StrideSecurityValidatePathOwnershipDart>('stride_security_validate_path_ownership');
    _securityFieldRulesForCollection = _lib.lookupFunction<_StrideSecurityFieldRulesForCollectionNative,
        _StrideSecurityFieldRulesForCollectionDart>('stride_security_field_rules_for_collection');
    _securityAllowedFieldsForCollection = _lib.lookupFunction<_StrideSecurityAllowedFieldsForCollectionNative,
        _StrideSecurityAllowedFieldsForCollectionDart>('stride_security_allowed_fields_for_collection');
    _securityValidateDocument = _lib.lookupFunction<_StrideSecurityValidateDocumentNative,
        _StrideSecurityValidateDocumentDart>('stride_security_validate_document');
    _securitySanitizeString = _lib.lookupFunction<_StrideSecuritySanitizeStringNative,
        _StrideSecuritySanitizeStringDart>('stride_security_sanitize_string');
    _securityDetectInjection = _lib.lookupFunction<_StrideSecurityDetectInjectionNative,
        _StrideSecurityDetectInjectionDart>('stride_security_detect_injection');
    _securityIsSafeString = _lib.lookupFunction<_StrideSecurityIsSafeStringNative,
        _StrideSecurityIsSafeStringDart>('stride_security_is_safe_string');
    _securityValidateStoragePath = _lib.lookupFunction<_StrideSecurityValidateStoragePathNative,
        _StrideSecurityValidateStoragePathDart>('stride_security_validate_storage_path');
    _securityAppCheckDecision = _lib.lookupFunction<_StrideSecurityAppCheckDecisionNative,
        _StrideSecurityAppCheckDecisionDart>('stride_security_app_check_decision');
    _securityPlayIntegrityDecision = _lib.lookupFunction<_StrideSecurityPlayIntegrityDecisionNative,
        _StrideSecurityPlayIntegrityDecisionDart>('stride_security_play_integrity_decision');
    _securityCheckRateLimit = _lib.lookupFunction<_StrideSecurityCheckRateLimitNative,
        _StrideSecurityCheckRateLimitDart>('stride_security_check_rate_limit');
    _securityRateLimitConfig = _lib.lookupFunction<_StrideSecurityRateLimitConfigNative,
        _StrideSecurityRateLimitConfigDart>('stride_security_rate_limit_config');
    _securityCheckForSecrets = _lib.lookupFunction<_StrideSecurityCheckForSecretsNative,
        _StrideSecurityCheckForSecretsDart>('stride_security_check_for_secrets');
    _securityIsSecretFree = _lib.lookupFunction<_StrideSecurityIsSecretFreeNative,
        _StrideSecurityIsSecretFreeDart>('stride_security_is_secret_free');
    _securityValidateProjectId = _lib.lookupFunction<_StrideSecurityValidateProjectIdNative,
        _StrideSecurityValidateProjectIdDart>('stride_security_validate_project_id');
    _securityEnvironmentFromProjectId = _lib.lookupFunction<_StrideSecurityEnvironmentFromProjectIdNative,
        _StrideSecurityEnvironmentFromProjectIdDart>('stride_security_environment_from_project_id');
    _securityGenerateFirestoreRules = _lib.lookupFunction<_StrideSecurityGenerateFirestoreRulesNative,
        _StrideSecurityGenerateFirestoreRulesDart>('stride_security_generate_firestore_rules');
    _securityGenerateStorageRules = _lib.lookupFunction<_StrideSecurityGenerateStorageRulesNative,
        _StrideSecurityGenerateStorageRulesDart>('stride_security_generate_storage_rules');
    _CoachingPlanExperienceCaps = _lib.lookupFunction<_StrideCoachingPlanExperienceCapsNative,
        _StrideCoachingPlanExperienceCapsDart>('stride_coaching_plan_experience_caps');
    _CoachingPlanValidateDayPlan = _lib.lookupFunction<_StrideCoachingPlanValidateDayPlanNative,
        _StrideCoachingPlanValidateDayPlanDart>('stride_coaching_plan_validate_day_plan');
    _CoachingPlanValidateWeeklyPlan = _lib.lookupFunction<_StrideCoachingPlanValidateWeeklyPlanNative,
        _StrideCoachingPlanValidateWeeklyPlanDart>('stride_coaching_plan_validate_weekly_plan');
    _CoachingPlanGenerateFallback = _lib.lookupFunction<_StrideCoachingPlanGenerateFallbackNative,
        _StrideCoachingPlanGenerateFallbackDart>('stride_coaching_plan_generate_fallback');
    _CoachingPlanRespondToPain = _lib.lookupFunction<_StrideCoachingPlanRespondToPainNative,
        _StrideCoachingPlanRespondToPainDart>('stride_coaching_plan_respond_to_pain');
    _CoachingPlanContainsDiagnosis = _lib.lookupFunction<_StrideCoachingPlanContainsDiagnosisNative,
        _StrideCoachingPlanContainsDiagnosisDart>('stride_coaching_plan_contains_diagnosis');
    _CoachingPlanContainsWeightLossPromise = _lib.lookupFunction<_StrideCoachingPlanContainsWeightLossPromiseNative,
        _StrideCoachingPlanContainsWeightLossPromiseDart>('stride_coaching_plan_contains_weight_loss_promise');
    _CoachingPlanValidateCoachingText = _lib.lookupFunction<_StrideCoachingPlanValidateCoachingTextNative,
        _StrideCoachingPlanValidateCoachingTextDart>('stride_coaching_plan_validate_coaching_text');
    _CoachingPlanEscalationMessage = _lib.lookupFunction<_StrideCoachingPlanEscalationMessageNative,
        _StrideCoachingPlanEscalationMessageDart>('stride_coaching_plan_escalation_message');
    _CoachingPlanProcessUserFeedback = _lib.lookupFunction<_StrideCoachingPlanProcessUserFeedbackNative,
        _StrideCoachingPlanProcessUserFeedbackDart>('stride_coaching_plan_process_user_feedback');
    _CoachingPlanDecideAiAvailability = _lib.lookupFunction<_StrideCoachingPlanDecideAiAvailabilityNative,
        _StrideCoachingPlanDecideAiAvailabilityDart>('stride_coaching_plan_decide_ai_availability');
    _CoachingPlanShouldUseFallback = _lib.lookupFunction<_StrideCoachingPlanShouldUseFallbackNative,
        _StrideCoachingPlanShouldUseFallbackDart>('stride_coaching_plan_should_use_fallback');
    _CoachingPlanEstimateAiCost = _lib.lookupFunction<_StrideCoachingPlanEstimateAiCostNative,
        _StrideCoachingPlanEstimateAiCostDart>('stride_coaching_plan_estimate_ai_cost');
    _CoachingPlanCacheKey = _lib.lookupFunction<_StrideCoachingPlanCacheKeyNative,
        _StrideCoachingPlanCacheKeyDart>('stride_coaching_plan_cache_key');
    _CoachingPlanSummarizeWorkout = _lib.lookupFunction<_StrideCoachingPlanSummarizeWorkoutNative,
        _StrideCoachingPlanSummarizeWorkoutDart>('stride_coaching_plan_summarize_workout');
    _CoachingPlanGenerateEncouragement = _lib.lookupFunction<_StrideCoachingPlanGenerateEncouragementNative,
        _StrideCoachingPlanGenerateEncouragementDart>('stride_coaching_plan_generate_encouragement');
    _CoachingPlanAdjustPlan = _lib.lookupFunction<_StrideCoachingPlanAdjustPlanNative,
        _StrideCoachingPlanAdjustPlanDart>('stride_coaching_plan_adjust_plan');
    _CoachingPlanRecommendProgression = _lib.lookupFunction<_StrideCoachingPlanRecommendProgressionNative,
        _StrideCoachingPlanRecommendProgressionDart>('stride_coaching_plan_recommend_progression');
    _CoachingPlanModerateRequest = _lib.lookupFunction<_StrideCoachingPlanModerateRequestNative,
        _StrideCoachingPlanModerateRequestDart>('stride_coaching_plan_moderate_request');
    _CalorieEstimate = _lib.lookupFunction<_StrideCalorieEstimateNative,
        _StrideCalorieEstimateDart>('stride_calorie_estimate');
    _CalorieClamp = _lib.lookupFunction<_StrideCalorieClampNative,
        _StrideCalorieClampDart>('stride_calorie_clamp');
    _CalorieSourcePriority = _lib.lookupFunction<_StrideCalorieSourcePriorityNative,
        _StrideCalorieSourcePriorityDart>('stride_calorie_source_priority');
    _WearableDecideFallback = _lib.lookupFunction<_StrideWearableDecideFallbackNative,
        _StrideWearableDecideFallbackDart>('stride_wearable_decide_fallback');
    _WearableBuildStatus = _lib.lookupFunction<_StrideWearableBuildStatusNative,
        _StrideWearableBuildStatusDart>('stride_wearable_build_status');
    _WearableSyncStatus = _lib.lookupFunction<_StrideWearableSyncStatusNative,
        _StrideWearableSyncStatusDart>('stride_wearable_sync_status');
    _WearableDeduplicateSource = _lib.lookupFunction<_StrideWearableDeduplicateSourceNative,
        _StrideWearableDeduplicateSourceDart>('stride_wearable_deduplicate_source');
    _WearableConsentResult = _lib.lookupFunction<_StrideWearableConsentResultNative,
        _StrideWearableConsentResultDart>('stride_wearable_consent_result');
    // ─── §11 — Music system lookups ───────────────────────────────
    _MusicTransition = _lib.lookupFunction<_StrideMusicTransitionNative,
        _StrideMusicTransitionDart>('stride_music_transition');
    _MusicAudioFocus = _lib.lookupFunction<_StrideMusicAudioFocusNative,
        _StrideMusicAudioFocusDart>('stride_music_audio_focus');
    _MusicCoachingInterop = _lib.lookupFunction<_StrideMusicCoachingInteropNative,
        _StrideMusicCoachingInteropDart>('stride_music_coaching_interop');
    _MusicNetworkLoss = _lib.lookupFunction<_StrideMusicNetworkLossNative,
        _StrideMusicNetworkLossDart>('stride_music_network_loss');
    _MusicFilterBlocked = _lib.lookupFunction<_StrideMusicFilterBlockedNative,
        _StrideMusicFilterBlockedDart>('stride_music_filter_blocked');
    _MusicShouldRecommend = _lib.lookupFunction<_StrideMusicShouldRecommendNative,
        _StrideMusicShouldRecommendDart>('stride_music_should_recommend');
    _MusicBuildStatus = _lib.lookupFunction<_StrideMusicBuildStatusNative,
        _StrideMusicBuildStatusDart>('stride_music_build_status');
    _MusicRemoteControl = _lib.lookupFunction<_StrideMusicRemoteControlNative,
        _StrideMusicRemoteControlDart>('stride_music_remote_control');
    // ─── §12 — Background execution lookups ──────────────────────
    _BackgroundServiceTransition = _lib.lookupFunction<_StrideBackgroundServiceTransitionNative,
        _StrideBackgroundServiceTransitionDart>('stride_background_service_transition');
    _BackgroundCheckpointInterval = _lib.lookupFunction<_StrideBackgroundCheckpointIntervalNative,
        _StrideBackgroundCheckpointIntervalDart>('stride_background_checkpoint_interval');
    _BackgroundEvaluate = _lib.lookupFunction<_StrideBackgroundEvaluateNative,
        _StrideBackgroundEvaluateDart>('stride_background_evaluate');
    _BackgroundProcessKill = _lib.lookupFunction<_StrideBackgroundProcessKillNative,
        _StrideBackgroundProcessKillDart>('stride_background_process_kill');
    _BackgroundPowerMode = _lib.lookupFunction<_StrideBackgroundPowerModeNative,
        _StrideBackgroundPowerModeDart>('stride_background_power_mode');
    _BackgroundInterruption = _lib.lookupFunction<_StrideBackgroundInterruptionNative,
        _StrideBackgroundInterruptionDart>('stride_background_interruption');
    _BackgroundBatteryAssessment = _lib.lookupFunction<_StrideBackgroundBatteryAssessmentNative,
        _StrideBackgroundBatteryAssessmentDart>('stride_background_battery_assessment');
    _BackgroundBuildStatus = _lib.lookupFunction<_StrideBackgroundBuildStatusNative,
        _StrideBackgroundBuildStatusDart>('stride_background_build_status');
    _BackgroundExplanation = _lib.lookupFunction<_StrideBackgroundExplanationNative,
        _StrideBackgroundExplanationDart>('stride_background_explanation');
    // ── §13 — Notifications lookups ───────────────────────────
    _NotificationDecide = _lib.lookupFunction<_StrideNotificationDecideNative,
        _StrideNotificationDecideDart>('stride_notification_decide');
    _NotificationContent = _lib.lookupFunction<_StrideNotificationContentNative,
        _StrideNotificationContentDart>('stride_notification_content');
    _NotificationQuietHours = _lib.lookupFunction<_StrideNotificationQuietHoursNative,
        _StrideNotificationQuietHoursDart>('stride_notification_quiet_hours');
    _NotificationCoaching = _lib.lookupFunction<_StrideNotificationCoachingNative,
        _StrideNotificationCoachingDart>('stride_notification_coaching');
    _NotificationStatus = _lib.lookupFunction<_StrideNotificationStatusNative,
        _StrideNotificationStatusDart>('stride_notification_status');
    _NotificationNextReminder = _lib.lookupFunction<_StrideNotificationNextReminderNative,
        _StrideNotificationNextReminderDart>('stride_notification_next_reminder');

    // ── §14 — Error/Recovery States lookups ─────────────────────────
    _ErrorDecideRecovery = _lib.lookupFunction<_StrideErrorDecideRecoveryNative,
        _StrideErrorDecideRecoveryDart>('stride_error_decide_recovery');
    _ErrorRetryDelay = _lib.lookupFunction<_StrideErrorRetryDelayNative,
        _StrideErrorRetryDelayDart>('stride_error_retry_delay');
    _ErrorTransition = _lib.lookupFunction<_StrideErrorTransitionNative,
        _StrideErrorTransitionDart>('stride_error_transition');
    _ErrorHealthStatus = _lib.lookupFunction<_StrideErrorHealthStatusNative,
        _StrideErrorHealthStatusDart>('stride_error_health_status');
    _ErrorIsRecoverable = _lib.lookupFunction<_StrideErrorIsRecoverableNative,
        _StrideErrorIsRecoverableDart>('stride_error_is_recoverable');
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
  // §6 — Authentication / account lifecycle
  late final _StrideAuthProviderLabelDart _authProviderLabel;
  late final _StrideClassifySessionStateDart _classifySessionState;
  late final _StrideNeedsTokenRefreshDart _needsTokenRefresh;
  late final _StrideRequiresReloginDart _requiresRelogin;
  late final _StrideRequiresReauthenticationDart _requiresReauthentication;
  late final _StrideReauthThresholdMsDart _reauthThresholdMs;
  late final _StrideReauthReasonDart _reauthReason;
  late final _StrideDecideVerificationActionDart _decideVerificationAction;
  late final _StrideCanResendVerificationDart _canResendVerification;
  late final _StrideValidatePasswordDart _validatePassword;
  late final _StridePasswordStrengthScoreDart _passwordStrengthScore;
  late final _StridePasswordStrengthLabelDart _passwordStrengthLabel;
  late final _StrideValidateEmailDart _validateEmail;
  late final _StrideAccountStatusMessageDart _accountStatusMessage;
  late final _StrideCanSignInDart _canSignIn;
  late final _StrideAnalyzeLoginAttemptDart _analyzeLoginAttempt;
  late final _StrideEnumerateUserDataCategoriesDart _enumerateUserDataCategories;
  late final _StrideBuildDeletionPlanDart _buildDeletionPlan;
  late final _StrideBuildDeletionResultDart _buildDeletionResult;
  late final _StrideShouldAutoSignoutDart _shouldAutoSignout;
  late final _StrideCanUpgradeAnonymousDart _canUpgradeAnonymous;
  late final _StrideSecurityCollectionPathTemplateDart _securityCollectionPathTemplate;
  late final _StrideSecurityCollectionIsAdminOnlyDart _securityCollectionIsAdminOnly;
  late final _StrideSecurityCollectionIsUserScopedDart _securityCollectionIsUserScoped;
  late final _StrideSecurityCheckAccessDart _securityCheckAccess;
  late final _StrideSecurityValidatePathOwnershipDart _securityValidatePathOwnership;
  late final _StrideSecurityFieldRulesForCollectionDart _securityFieldRulesForCollection;
  late final _StrideSecurityAllowedFieldsForCollectionDart _securityAllowedFieldsForCollection;
  late final _StrideSecurityValidateDocumentDart _securityValidateDocument;
  late final _StrideSecuritySanitizeStringDart _securitySanitizeString;
  late final _StrideSecurityDetectInjectionDart _securityDetectInjection;
  late final _StrideSecurityIsSafeStringDart _securityIsSafeString;
  late final _StrideSecurityValidateStoragePathDart _securityValidateStoragePath;
  late final _StrideSecurityAppCheckDecisionDart _securityAppCheckDecision;
  late final _StrideSecurityPlayIntegrityDecisionDart _securityPlayIntegrityDecision;
  late final _StrideSecurityCheckRateLimitDart _securityCheckRateLimit;
  late final _StrideSecurityRateLimitConfigDart _securityRateLimitConfig;
  late final _StrideSecurityCheckForSecretsDart _securityCheckForSecrets;
  late final _StrideSecurityIsSecretFreeDart _securityIsSecretFree;
  late final _StrideSecurityValidateProjectIdDart _securityValidateProjectId;
  late final _StrideSecurityEnvironmentFromProjectIdDart _securityEnvironmentFromProjectId;
  late final _StrideSecurityGenerateFirestoreRulesDart _securityGenerateFirestoreRules;
  late final _StrideSecurityGenerateStorageRulesDart _securityGenerateStorageRules;
  late final _StrideCoachingPlanExperienceCapsDart _CoachingPlanExperienceCaps;
  late final _StrideCoachingPlanValidateDayPlanDart _CoachingPlanValidateDayPlan;
  late final _StrideCoachingPlanValidateWeeklyPlanDart _CoachingPlanValidateWeeklyPlan;
  late final _StrideCoachingPlanGenerateFallbackDart _CoachingPlanGenerateFallback;
  late final _StrideCoachingPlanRespondToPainDart _CoachingPlanRespondToPain;
  late final _StrideCoachingPlanContainsDiagnosisDart _CoachingPlanContainsDiagnosis;
  late final _StrideCoachingPlanContainsWeightLossPromiseDart _CoachingPlanContainsWeightLossPromise;
  late final _StrideCoachingPlanValidateCoachingTextDart _CoachingPlanValidateCoachingText;
  late final _StrideCoachingPlanEscalationMessageDart _CoachingPlanEscalationMessage;
  late final _StrideCoachingPlanProcessUserFeedbackDart _CoachingPlanProcessUserFeedback;
  late final _StrideCoachingPlanDecideAiAvailabilityDart _CoachingPlanDecideAiAvailability;
  late final _StrideCoachingPlanShouldUseFallbackDart _CoachingPlanShouldUseFallback;
  late final _StrideCoachingPlanEstimateAiCostDart _CoachingPlanEstimateAiCost;
  late final _StrideCoachingPlanCacheKeyDart _CoachingPlanCacheKey;
  late final _StrideCoachingPlanSummarizeWorkoutDart _CoachingPlanSummarizeWorkout;
  late final _StrideCoachingPlanGenerateEncouragementDart _CoachingPlanGenerateEncouragement;
  late final _StrideCoachingPlanAdjustPlanDart _CoachingPlanAdjustPlan;
  late final _StrideCoachingPlanRecommendProgressionDart _CoachingPlanRecommendProgression;
  late final _StrideCoachingPlanModerateRequestDart _CoachingPlanModerateRequest;
  late final _StrideCalorieEstimateDart _CalorieEstimate;
  late final _StrideCalorieClampDart _CalorieClamp;
  late final _StrideCalorieSourcePriorityDart _CalorieSourcePriority;
  late final _StrideWearableDecideFallbackDart _WearableDecideFallback;
  late final _StrideWearableBuildStatusDart _WearableBuildStatus;
  late final _StrideWearableSyncStatusDart _WearableSyncStatus;
  late final _StrideWearableDeduplicateSourceDart _WearableDeduplicateSource;
  late final _StrideWearableConsentResultDart _WearableConsentResult;
  // ─── §11 — Music system fields ────────────────────────────────
  late final _StrideMusicTransitionDart _MusicTransition;
  late final _StrideMusicAudioFocusDart _MusicAudioFocus;
  late final _StrideMusicCoachingInteropDart _MusicCoachingInterop;
  late final _StrideMusicNetworkLossDart _MusicNetworkLoss;
  late final _StrideMusicFilterBlockedDart _MusicFilterBlocked;
  late final _StrideMusicShouldRecommendDart _MusicShouldRecommend;
  late final _StrideMusicBuildStatusDart _MusicBuildStatus;
  late final _StrideMusicRemoteControlDart _MusicRemoteControl;
  // ─── §12 — Background execution fields ────────────────────────
  late final _StrideBackgroundServiceTransitionDart _BackgroundServiceTransition;
  late final _StrideBackgroundCheckpointIntervalDart _BackgroundCheckpointInterval;
  late final _StrideBackgroundEvaluateDart _BackgroundEvaluate;
  late final _StrideBackgroundProcessKillDart _BackgroundProcessKill;
  late final _StrideBackgroundPowerModeDart _BackgroundPowerMode;
  late final _StrideBackgroundInterruptionDart _BackgroundInterruption;
  late final _StrideBackgroundBatteryAssessmentDart _BackgroundBatteryAssessment;
  late final _StrideBackgroundBuildStatusDart _BackgroundBuildStatus;
  late final _StrideBackgroundExplanationDart _BackgroundExplanation;
  late final _StrideNotificationDecideDart _NotificationDecide;
  late final _StrideNotificationContentDart _NotificationContent;
  late final _StrideNotificationQuietHoursDart _NotificationQuietHours;
  late final _StrideNotificationCoachingDart _NotificationCoaching;
  late final _StrideNotificationStatusDart _NotificationStatus;
  late final _StrideNotificationNextReminderDart _NotificationNextReminder;

  // ── §14 — Error/Recovery States late final fields ───────────────
  late final _StrideErrorDecideRecoveryDart _ErrorDecideRecovery;
  late final _StrideErrorRetryDelayDart _ErrorRetryDelay;
  late final _StrideErrorTransitionDart _ErrorTransition;
  late final _StrideErrorHealthStatusDart _ErrorHealthStatus;
  late final _StrideErrorIsRecoverableDart _ErrorIsRecoverable;
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

  // §6 — Authentication / account lifecycle

  /// Returns a human-readable label for an auth provider.
  Map<String, dynamic> authProviderLabel(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_authProviderLabel(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Classifies the session state from token timestamps.
  Map<String, dynamic> classifySessionState(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_classifySessionState(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Whether the session needs a token refresh.
  Map<String, dynamic> needsTokenRefresh(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_needsTokenRefresh(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Whether the user must re-sign-in.
  Map<String, dynamic> requiresRelogin(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_requiresRelogin(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Whether a sensitive action requires reauthentication.
  Map<String, dynamic> requiresReauthentication(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_requiresReauthentication(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns the reauth threshold in milliseconds.
  Map<String, dynamic> reauthThresholdMs(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_reauthThresholdMs(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns the reason reauthentication is needed.
  Map<String, dynamic> reauthReason(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_reauthReason(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Decides what action to take for email verification.
  Map<String, dynamic> decideVerificationAction(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_decideVerificationAction(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Whether a verification email can be resent.
  Map<String, dynamic> canResendVerification(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_canResendVerification(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Validates a password against the password policy.
  Map<String, dynamic> validatePassword(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_validatePassword(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns the password strength score (0–4).
  Map<String, dynamic> passwordStrengthScore(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_passwordStrengthScore(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns a label for a password strength score.
  Map<String, dynamic> passwordStrengthLabel(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_passwordStrengthLabel(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Validates an email address.
  Map<String, dynamic> validateEmail(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_validateEmail(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Returns a message for an account status.
  Map<String, dynamic> accountStatusMessage(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_accountStatusMessage(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Whether the user can sign in given an account status.
  Map<String, dynamic> canSignIn(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_canSignIn(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Analyzes a login attempt for suspicious activity.
  Map<String, dynamic> analyzeLoginAttempt(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_analyzeLoginAttempt(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Enumerates all user-data categories for deletion.
  Map<String, dynamic> enumerateUserDataCategories(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_enumerateUserDataCategories(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Builds a deletion plan for a scope.
  Map<String, dynamic> buildDeletionPlan(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_buildDeletionPlan(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Builds a deletion result from deleted/failed categories.
  Map<String, dynamic> buildDeletionResult(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_buildDeletionResult(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Whether the app should auto sign-out.
  Map<String, dynamic> shouldAutoSignout(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_shouldAutoSignout(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// Whether an anonymous account can be upgraded.
  Map<String, dynamic> canUpgradeAnonymous(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_canUpgradeAnonymous(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// securityCollectionPathTemplate — §7 Secure Firebase security validation.
  Map<String, dynamic> securityCollectionPathTemplate(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityCollectionPathTemplate(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityCollectionIsAdminOnly — §7 Secure Firebase security validation.
  Map<String, dynamic> securityCollectionIsAdminOnly(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityCollectionIsAdminOnly(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityCollectionIsUserScoped — §7 Secure Firebase security validation.
  Map<String, dynamic> securityCollectionIsUserScoped(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityCollectionIsUserScoped(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityCheckAccess — §7 Secure Firebase security validation.
  Map<String, dynamic> securityCheckAccess(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityCheckAccess(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityValidatePathOwnership — §7 Secure Firebase security validation.
  Map<String, dynamic> securityValidatePathOwnership(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityValidatePathOwnership(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityFieldRulesForCollection — §7 Secure Firebase security validation.
  Map<String, dynamic> securityFieldRulesForCollection(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityFieldRulesForCollection(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityAllowedFieldsForCollection — §7 Secure Firebase security validation.
  Map<String, dynamic> securityAllowedFieldsForCollection(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityAllowedFieldsForCollection(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityValidateDocument — §7 Secure Firebase security validation.
  Map<String, dynamic> securityValidateDocument(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityValidateDocument(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securitySanitizeString — §7 Secure Firebase security validation.
  Map<String, dynamic> securitySanitizeString(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securitySanitizeString(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityDetectInjection — §7 Secure Firebase security validation.
  Map<String, dynamic> securityDetectInjection(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityDetectInjection(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityIsSafeString — §7 Secure Firebase security validation.
  Map<String, dynamic> securityIsSafeString(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityIsSafeString(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityValidateStoragePath — §7 Secure Firebase security validation.
  Map<String, dynamic> securityValidateStoragePath(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityValidateStoragePath(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityAppCheckDecision — §7 Secure Firebase security validation.
  Map<String, dynamic> securityAppCheckDecision(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityAppCheckDecision(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityPlayIntegrityDecision — §7 Secure Firebase security validation.
  Map<String, dynamic> securityPlayIntegrityDecision(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityPlayIntegrityDecision(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securitySecurityCheckRateLimit — §7 Secure Firebase security validation.
  Map<String, dynamic> securityCheckRateLimit(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityCheckRateLimit(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityRateLimitConfig — §7 Secure Firebase security validation.
  Map<String, dynamic> securityRateLimitConfig(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityRateLimitConfig(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityCheckForSecrets — §7 Secure Firebase security validation.
  Map<String, dynamic> securityCheckForSecrets(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityCheckForSecrets(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityIsSecretFree — §7 Secure Firebase security validation.
  Map<String, dynamic> securityIsSecretFree(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityIsSecretFree(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityValidateProjectId — §7 Secure Firebase security validation.
  Map<String, dynamic> securityValidateProjectId(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityValidateProjectId(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityEnvironmentFromProjectId — §7 Secure Firebase security validation.
  Map<String, dynamic> securityEnvironmentFromProjectId(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityEnvironmentFromProjectId(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityGenerateFirestoreRules — §7 Secure Firebase security validation.
  Map<String, dynamic> securityGenerateFirestoreRules(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityGenerateFirestoreRules(ptr));
    } finally {
      malloc.free(ptr);
    }
  }


  /// securityGenerateStorageRules — §7 Secure Firebase security validation.
  Map<String, dynamic> securityGenerateStorageRules(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_securityGenerateStorageRules(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanExperienceCaps — §8 coaching plan — experience-level caps.
  Map<String, dynamic> coachingPlanExperienceCaps(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanExperienceCaps(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanValidateDayPlan — §8 coaching plan — validate a single day plan.
  Map<String, dynamic> coachingPlanValidateDayPlan(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanValidateDayPlan(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanValidateWeeklyPlan — §8 coaching plan — validate a weekly plan.
  Map<String, dynamic> coachingPlanValidateWeeklyPlan(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanValidateWeeklyPlan(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanGenerateFallback — §8 coaching plan — generate a deterministic fallback weekly plan.
  Map<String, dynamic> coachingPlanGenerateFallback(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanGenerateFallback(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanRespondToPain — §8 coaching plan — non-diagnostic pain response.
  Map<String, dynamic> coachingPlanRespondToPain(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanRespondToPain(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanContainsDiagnosis — §8 coaching plan — detect medical diagnosis language.
  Map<String, dynamic> coachingPlanContainsDiagnosis(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanContainsDiagnosis(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanContainsWeightLossPromise — §8 coaching plan — detect weight-loss promise language.
  Map<String, dynamic> coachingPlanContainsWeightLossPromise(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanContainsWeightLossPromise(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanValidateCoachingText — §8 coaching plan — validate AI coaching text against content guards.
  Map<String, dynamic> coachingPlanValidateCoachingText(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanValidateCoachingText(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanEscalationMessage — §8 coaching plan — generate escalation message.
  Map<String, dynamic> coachingPlanEscalationMessage(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanEscalationMessage(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanProcessUserFeedback — §8 coaching plan — process user feedback on a plan.
  Map<String, dynamic> coachingPlanProcessUserFeedback(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanProcessUserFeedback(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanDecideAiAvailability — §8 coaching plan — decide whether AI should be called.
  Map<String, dynamic> coachingPlanDecideAiAvailability(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanDecideAiAvailability(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanShouldUseFallback — §8 coaching plan — whether to use rule-based fallback.
  Map<String, dynamic> coachingPlanShouldUseFallback(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanShouldUseFallback(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanEstimateAiCost — §8 coaching plan — estimate AI operation cost in cents.
  Map<String, dynamic> coachingPlanEstimateAiCost(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanEstimateAiCost(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanCacheKey — §8 coaching plan — deterministic cache key for an AI request.
  Map<String, dynamic> coachingPlanCacheKey(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanCacheKey(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanSummarizeWorkout — §8 coaching plan — rule-based workout summary.
  Map<String, dynamic> coachingPlanSummarizeWorkout(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanSummarizeWorkout(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanGenerateEncouragement — §8 coaching plan — rule-based encouragement message.
  Map<String, dynamic> coachingPlanGenerateEncouragement(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanGenerateEncouragement(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanAdjustPlan — §8 coaching plan — rule-based plan adjustment.
  Map<String, dynamic> coachingPlanAdjustPlan(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanAdjustPlan(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanRecommendProgression — §8 coaching plan — recommend weekly distance progression.
  Map<String, dynamic> coachingPlanRecommendProgression(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanRecommendProgression(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// coachingPlanModerateRequest — §8 coaching plan — safety moderation of user request.
  Map<String, dynamic> coachingPlanModerateRequest(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CoachingPlanModerateRequest(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  // ─── §9 — Calorie/fitness calculations ───

  /// calorieEstimate — §9 calorie — full calorie estimate with method, version, labels, source priority.
  Map<String, dynamic> calorieEstimate(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CalorieEstimate(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// calorieClamp — §9 calorie — validate & clamp a calorie value to a plausible range.
  Map<String, dynamic> calorieClamp(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_CalorieClamp(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// calorieSourcePriority — §9 calorie — source-priority order of estimation methods (no input args).
  Map<String, dynamic> calorieSourcePriority() {
    return _consume(_CalorieSourcePriority());
  }

  // ─── §10 — Wearable / Health Connect ───

  /// wearableDecideFallback — §10 wearable — decide operating mode from source availability.
  Map<String, dynamic> wearableDecideFallback(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_WearableDecideFallback(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// wearableBuildStatus — §10 wearable — full wearable status snapshot.
  Map<String, dynamic> wearableBuildStatus(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_WearableBuildStatus(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// wearableSyncStatus — §10 wearable — evaluate sync status from connection state + last sync time.
  Map<String, dynamic> wearableSyncStatus(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_WearableSyncStatus(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// wearableDeduplicateSource — §10 wearable — pick the winning source for a duplicate metric.
  Map<String, dynamic> wearableDeduplicateSource(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_WearableDeduplicateSource(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// wearableConsentResult — §10 wearable — process a Health Connect consent request result.
  Map<String, dynamic> wearableConsentResult(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_WearableConsentResult(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  // ─── §11 — Music system wrappers ──────────────────────────────

  /// musicTransition — §11 music — transition the playback state machine.
  Map<String, dynamic> musicTransition(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_MusicTransition(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// musicAudioFocus — §11 music — handle an audio focus event.
  Map<String, dynamic> musicAudioFocus(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_MusicAudioFocus(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// musicCoachingInterop — §11 music — coordinate music with coaching.
  Map<String, dynamic> musicCoachingInterop(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_MusicCoachingInterop(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// musicNetworkLoss — §11 music — decide what to do on network loss.
  Map<String, dynamic> musicNetworkLoss(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_MusicNetworkLoss(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// musicFilterBlocked — §11 music — filter blocked content from a playlist.
  Map<String, dynamic> musicFilterBlocked(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_MusicFilterBlocked(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// musicShouldRecommend — §11 music — decide whether to recommend a track.
  Map<String, dynamic> musicShouldRecommend(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_MusicShouldRecommend(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// musicBuildStatus — §11 music — build a full music status snapshot.
  Map<String, dynamic> musicBuildStatus(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_MusicBuildStatus(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// musicRemoteControl — §11 music — handle a remote control command.
  Map<String, dynamic> musicRemoteControl(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_MusicRemoteControl(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  // ─── §12 — Background execution wrappers ─────────────────────

  /// backgroundServiceTransition — §12 — transition the foreground service state machine.
  Map<String, dynamic> backgroundServiceTransition(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_BackgroundServiceTransition(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// backgroundCheckpointInterval — §12 — decide the checkpoint write interval.
  Map<String, dynamic> backgroundCheckpointInterval(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_BackgroundCheckpointInterval(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// backgroundEvaluate — §12 — evaluate whether background execution is permitted.
  Map<String, dynamic> backgroundEvaluate(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_BackgroundEvaluate(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// backgroundProcessKill — §12 — evaluate process-kill recovery.
  Map<String, dynamic> backgroundProcessKill(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_BackgroundProcessKill(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// backgroundPowerMode — §12 — decide power mode and notification interval.
  Map<String, dynamic> backgroundPowerMode(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_BackgroundPowerMode(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// backgroundInterruption — §12 — handle an interruption event.
  Map<String, dynamic> backgroundInterruption(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_BackgroundInterruption(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// backgroundBatteryAssessment — §12 — assess battery usage.
  Map<String, dynamic> backgroundBatteryAssessment(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_BackgroundBatteryAssessment(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// backgroundBuildStatus — §12 — build a full background status snapshot.
  Map<String, dynamic> backgroundBuildStatus(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_BackgroundBuildStatus(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// backgroundExplanation — §12 — get the background location explanation text.
  Map<String, dynamic> backgroundExplanation(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_BackgroundExplanation(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// notificationDecide — §13 — decide whether & how to deliver a
  /// notification given the full decision context (permission,
  /// preferences, quiet hours, voice coaching, timezone, etc.).
  Map<String, dynamic> notificationDecide(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_NotificationDecide(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// notificationContent — §13 — build the notification content
  /// (title, body, action label) for a given notification type and
  /// optional context params.
  Map<String, dynamic> notificationContent(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_NotificationContent(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// notificationQuietHours — §13 — evaluate whether a notification
  /// should be delivered now given the quiet hours configuration,
  /// notification type, local minute, and critical-bypass flag.
  Map<String, dynamic> notificationQuietHours(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_NotificationQuietHours(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// notificationCoaching — §13 — evaluate whether a voice coaching
  /// announcement should be made now given the coaching config, quiet
  /// hours, announcement kind, last distance/time, and local minute.
  Map<String, dynamic> notificationCoaching(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_NotificationCoaching(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// notificationStatus — §13 — build a full notification status
  /// snapshot for the UI / diagnostics given permission, preferences,
  /// quiet hours, voice coaching config, and timezone.
  Map<String, dynamic> notificationStatus(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_NotificationStatus(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// notificationNextReminder — §13 — compute the UTC timestamp
  /// (epoch milliseconds) for the next daily reminder at a given
  /// local-time minute.
  Map<String, dynamic> notificationNextReminder(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_NotificationNextReminder(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// errorDecideRecovery — §14 — decide what to do in response to an
  /// error given the error context and retry policy. Returns a
  /// RecoveryAction JSON object.
  Map<String, dynamic> errorDecideRecovery(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_ErrorDecideRecovery(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// errorRetryDelay — §14 — compute the retry delay in milliseconds
  /// for a given attempt number and retry policy using exponential
  /// backoff with jitter.
  Map<String, dynamic> errorRetryDelay(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_ErrorRetryDelay(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// errorTransition — §14 — validate and perform an error state
  /// transition. Returns an ErrorStateTransition JSON object with
  /// is_valid and message.
  Map<String, dynamic> errorTransition(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_ErrorTransition(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// errorHealthStatus — §14 — build an overall engine health status
  /// from an error registry. Returns an EngineHealthStatus JSON object.
  Map<String, dynamic> errorHealthStatus(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_ErrorHealthStatus(ptr));
    } finally {
      malloc.free(ptr);
    }
  }

  /// errorIsRecoverable — §14 — return whether errors of a given
  /// category are retryable.
  Map<String, dynamic> errorIsRecoverable(Map<String, dynamic> request) {
    final ptr = _toNative(jsonEncode(request));
    try {
      return _consume(_ErrorIsRecoverable(ptr));
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

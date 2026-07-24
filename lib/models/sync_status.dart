/// Tracks the synchronization state of a local record.
enum SyncState {
  /// Recorded locally, not yet uploaded.
  pending,

  /// Currently uploading to the cloud.
  syncing,

  /// Successfully synced to Firestore + Cloud Storage.
  synced,

  /// Upload attempted but failed; will retry.
  failed,
}

/// Metadata attached to any locally-stored record that must sync to the cloud.
class SyncMetadata {
  final SyncState state;
  final DateTime lastAttempt;
  final int retryCount;
  final String? errorMessage;

  const SyncMetadata({
    this.state = SyncState.pending,
    required this.lastAttempt,
    this.retryCount = 0,
    this.errorMessage,
  });

  Map<String, dynamic> toMap() => {
        'state': state.name,
        'lastAttempt': lastAttempt.toIso8601String(),
        'retryCount': retryCount,
        'errorMessage': errorMessage,
      };

  factory SyncMetadata.fromMap(Map<String, dynamic> map) => SyncMetadata(
        state: SyncState.values.firstWhere((e) => e.name == map['state']),
        lastAttempt: DateTime.parse(map['lastAttempt'] as String),
        retryCount: map['retryCount'] as int? ?? 0,
        errorMessage: map['errorMessage'] as String?,
      );

  SyncMetadata copyWith({
    SyncState? state,
    DateTime? lastAttempt,
    int? retryCount,
    String? errorMessage,
  }) =>
      SyncMetadata(
        state: state ?? this.state,
        lastAttempt: lastAttempt ?? this.lastAttempt,
        retryCount: retryCount ?? this.retryCount,
        errorMessage: errorMessage ?? this.errorMessage,
      );
}

import '../models/walking_session.dart';

/// In-memory session store — swap for Firestore collection 'WalkingSessions'.
class SessionRepository {
  final List<WalkingSession> _sessions = [];

  Future<List<WalkingSession>> getSessions(String userId) async {
    await Future<void>.delayed(const Duration(milliseconds: 300));
    return _sessions
        .where((s) => s.userId == userId)
        .toList()
      ..sort((a, b) => b.startTime.compareTo(a.startTime));
  }

  Future<void> saveSession(WalkingSession session) async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    final index = _sessions.indexWhere((s) => s.id == session.id);
    if (index >= 0) {
      _sessions[index] = session;
    } else {
      _sessions.add(session);
    }
  }

  Future<void> deleteSession(String sessionId) async {
    await Future<void>.delayed(const Duration(milliseconds: 200));
    _sessions.removeWhere((s) => s.id == sessionId);
  }

  Future<List<WalkingSession>> getSessionsInRange(
    String userId,
    DateTime start,
    DateTime end,
  ) async {
    await Future<void>.delayed(const Duration(milliseconds: 300));
    return _sessions
        .where((s) =>
            s.userId == userId &&
            s.startTime.isAfter(start) &&
            s.startTime.isBefore(end) &&
            s.status == SessionStatus.completed)
        .toList();
  }
}

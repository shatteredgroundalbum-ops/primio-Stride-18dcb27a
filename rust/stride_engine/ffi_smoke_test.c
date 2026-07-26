// Minimal smoke test for the stride_engine C-ABI, exercised before wiring
// up the Dart bindings. Build/run with:
//   gcc ffi_smoke_test.c -L target/release -lstride_engine -ldl -lpthread -lm -o /tmp/ffi_smoke && \
//   LD_LIBRARY_PATH=target/release /tmp/ffi_smoke
#include <stdio.h>
#include <string.h>

extern char* stride_engine_version();
extern char* stride_create_session(const char* request_json);
extern char* stride_start(long long handle);
extern char* stride_add_location_sample(long long handle, const char* point_json, long long now_ms);
extern char* stride_tick(long long handle, long long now_ms);
extern char* stride_pause(long long handle, long long now_ms);
extern char* stride_resume(long long handle);
extern char* stride_manual_lap(long long handle, long long now_ms);
extern char* stride_finish(long long handle, long long now_ms);
extern char* stride_get_session(long long handle);
extern char* stride_destroy_session(long long handle);
extern void stride_free_string(char* ptr);

static void show(const char* label, char* json) {
    printf("=== %s ===\n%s\n\n", label, json);
    stride_free_string(json);
}

int main() {
    show("version", stride_engine_version());

    const char* create_req =
        "{\"user_id\":\"u1\",\"activity_type\":\"walk\",\"weight_kg\":70.0,\"started_at\":1000}";
    char* create_resp = stride_create_session(create_req);
    printf("=== create_session ===\n%s\n\n", create_resp);

    // crude handle extraction for the smoke test only
    long long handle = 0;
    sscanf(strstr(create_resp, "\"handle\":"), "\"handle\":%lld", &handle);
    stride_free_string(create_resp);
    printf("parsed handle = %lld\n\n", handle);

    show("start", stride_start(handle));

    const char* pt1 =
        "{\"point_id\":\"p1\",\"workout_id\":\"\",\"latitude\":40.0,\"longitude\":-73.0,"
        "\"altitude_meters\":10.0,\"accuracy_meters\":6.0,\"altitude_accuracy_meters\":null,"
        "\"speed_meters_per_second\":1.3,\"bearing_degrees\":null,\"recorded_at\":1000,"
        "\"source\":\"phone_gps\",\"is_mock_location\":false,\"accepted\":false,\"rejection_reason\":null}";
    show("add_location_sample #1", stride_add_location_sample(handle, pt1, 1000));

    const char* pt2 =
        "{\"point_id\":\"p2\",\"workout_id\":\"\",\"latitude\":40.0000630,\"longitude\":-73.0,"
        "\"altitude_meters\":10.5,\"accuracy_meters\":6.0,\"altitude_accuracy_meters\":null,"
        "\"speed_meters_per_second\":1.4,\"bearing_degrees\":null,\"recorded_at\":6000,"
        "\"source\":\"phone_gps\",\"is_mock_location\":false,\"accepted\":false,\"rejection_reason\":null}";
    show("add_location_sample #2", stride_add_location_sample(handle, pt2, 6000));

    show("tick", stride_tick(handle, 7000));
    show("pause", stride_pause(handle, 8000));
    show("resume", stride_resume(handle));
    show("manual_lap", stride_manual_lap(handle, 9000));
    show("get_session", stride_get_session(handle));
    show("pause_before_finish", stride_pause(handle, 9500));
    show("finish", stride_finish(handle, 10000));
    show("destroy_session", stride_destroy_session(handle));

    // Unknown handle should produce a clean error envelope, not a crash.
    show("tick_after_destroy (expect error)", stride_tick(handle, 11000));

    printf("SMOKE TEST COMPLETE\n");
    return 0;
}

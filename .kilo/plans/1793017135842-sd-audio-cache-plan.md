# Plan: SD Card Audio Cache for Sidetrack M308

**Date:** 2026-10-26  
**Goal:** Persist librespot audio files to external microSD on Oilsky M308 (API 28) to reduce repeated streaming and CPU/bandwidth use. Discard passthrough and graphic equalizer — not feasible / out of scope.

## Constraints (from AGENTS.md / CONTEXT.md)
- API 28 (Android 9), ~2GB RAM, small screen
- No lossless, max 320 kbps OGG Vorbis
- GMS-free; auth via browser intent
- Preserve librespot core; no protocol reimplementation
- No destructive changes to existing functionality

## Technical feasibility (verified against code)
- `native/librespot-local/core/src/cache.rs` provides `Cache` + `FsSizeLimiter` (LRU by access time, size-limited)
- `native/src/session.rs:70` calls `Session::new(config, None)` — cache currently disabled
- `native/src/session.rs:19-30` already stores `TMP_DIR` from Android (`cacheDir.absolutePath`)
- `librespot_core::SessionConfig` has `tmp_dir` field only; no `cache` field — need `Cache::new(...)` passed as 2nd arg
- No audio cache setting exists in `SettingsManager.kt` or `NativeBridge.kt`

## Affected boundaries (files to modify)
1. `app/src/main/AndroidManifest.xml` — add `WRITE_EXTERNAL_STORAGE` permission
2. `app/src/main/java/com/sidetrack/SidetrackApp.kt` — resolve SD cache path at app init
3. `app/src/main/java/com/sidetrack/settings/SettingsManager.kt` — add `audioCacheEnabled` + `audioCacheLimitMb` to `SettingsState`, persist, push to native
4. `app/src/main/java/com/sidetrack/bridge/NativeBridge.kt` — add `setAudioCache(path, limitMb)` external fun
5. `native/src/bridge.rs` — add JNI export for `setAudioCache`
6. `native/src/session.rs` — store audio cache path/limit, construct `Cache`, pass `Some(cache)` to `Session::new`
7. `native/Cargo.toml` — no new deps needed (cache crate already part of librespot-local)

## Data flow
1. App starts → `SidetrackApp.onCreate()` resolves default SD cache path (`Environment.getExternalStorageDirectory()/Android/data/com.sidetrack.app/cache/audio/` or fallback `cacheDir/audio/`)
2. User toggles/enables cache in Settings → `SettingsManager.applyAudioSettings(...)` persists prefs + calls `NativeBridge.setAudioCache(path, limitMb)`
3. Rust stores path/limit; on next `connect_with_credentials()` constructs `Cache::new(cred_dir, volume_dir, audio_path, limit)` and calls `Session::new(config, Some(cache))`
4. librespot writes audio files under `<audio_path>/files/...`; `FsSizeLimiter` evicts LRU when limit exceeded

## Failure modes
- SD card absent / unmounted → fallback to internal `cacheDir/audio/`, log warning, keep streaming
- Permission denied at runtime → fallback to internal path, keep streaming
- Storage full → librespot `FsSizeLimiter` evicts oldest entries; session continues
- SD removed mid-session → cache reads fail → librespot re-fetches from network; no crash expected (verify)
- 2GB limit reached → LRU eviction; no crash

## Rollout / validation
- Debug build (`./gradlew assembleDebug`) passes on host
- Verify APK installs on API 28 emulator or M308 device
- Enable cache in Settings → playback proceeds, files appear under SD path
- Play same track twice → second play reads from cache (no network repeat)
- Disable cache → falls back to internal path / no cache
- Remove SD mid-session → playback continues via fallback/network

## Open decision (recommended answer)
- **Default cache size limit:** 2 GB (balances 16 GB device storage with low-consumption goal)
- **Default cache path:** `Environment.getExternalStorageDirectory()/Android/data/com.sidetrack.app/cache/audio/` with fallback to `cacheDir/audio/`
- **Permission:** `WRITE_EXTERNAL_STORAGE` in manifest + runtime request on API 28

---

Once confirmed, an implementation-capable agent should write the plan into `/var/home/wiliam/WorkArea/sidetrack/.kilo/plans/` and execute the ordered task list:
1. Add `WRITE_EXTERNAL_STORAGE` permission to manifest
2. Add cache settings (enabled, limitMb) to `SettingsManager.kt`
3. Add `setAudioCache(path, limitMb)` to `NativeBridge.kt` + `bridge.rs`
4. Wire `Cache` into `native/src/session.rs`
5. Resolve SD path + fallback in `SidetrackApp.kt`
6. Build debug APK and verify install + playback

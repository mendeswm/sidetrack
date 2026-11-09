# Plan: Integrate OpenEQ Equalizer into Sidetrack (Own Session Only)

**Date:** 2026-11-09
**Goal:** Add a per‑instance equalizer to Sidetrack that works only on Sidetrack’s audio playback, not a global listener.

## Current State Summary
- Sidetrack uses `AudioCallback` → `AudioTrack` (44.1 kHz stereo S16LE) to render PCM from native.
- No existing audio‑effect (Equalizer) usage; audio is played via plain Android `AudioTrack`.
- M308 adaptation targets Android 9 (API 28), small screen, limited RAM.
- External SD audio cache has been reverted to internal cache; permissions removed.

## Design Overview
- **Scope**: Equalizer operates on Sidetrack’s own audio session; no background service, no extra notifications.
- **Tech**: Use Android `Equalizer` (API 3+) attached to the `AudioTrack`’s session ID (`AudioTrack.audioSessionId`).
- **Lifecycle**: Create EQ after `AudioTrack` is created; update EQ levels when the track is recreated (e.g., after pause/resume, onAudioData recreation).
- **Persistence**: Equalizer settings (`eqEnabled`, `eqLevels` list) stored in `SettingsManager` (SharedPreferences), used to restore on next play.
- **UI**: Add an "Equalizer" section in `SettingsScreen`. For M308’s 480×640 screen, use vertical list of band sliders with band frequency labels (e.g., 60 Hz, 250 Hz, 500 Hz, 1 kHz, 2 kHz, 4 kHz, 8 kHz – whatever the device supports). Users can toggle EQ on/off and drag sliders.
- **Audio Focus**: EQ should be disabled when audio focus is lost (handled by existing `AudioFocusManager`). Note: M308 has no SIM/cellular radio, so phone-call audio focus loss is not possible; only other apps/notifications can steal focus.
- **No Global Mix**: OpenEQ’s foreground service and `AudioEffect` broadcast listeners are excluded; no extra manifest permissions.
- **Design Guardrails** (non-negotiable):
  1. **No global mix**: EQ must only attach to Sidetrack’s own audio session.
  2. **No new permissions**: No `FOREGROUND_SERVICE`, `POST_NOTIFICATIONS`, `WRITE_EXTERNAL_STORAGE`, or any runtime permission.
  3. **No cross-app leakage**: EQ must not apply to other apps’ audio sessions.
  4. **Graceful degradation**: If `Equalizer` is unsupported, disable EQ without crashing; default off.
  5. **No audio quality regression**: EQ must not change sample rate, channel count, or encoding; PCM stays 44.1 kHz stereo S16LE.
  6. **No blocking**: EQ setup/updates must not run on main thread or block playback.
  7. **No persistence beyond settings**: No file I/O beyond settings persistence.
  8. **Small-screen safe**: UI must fit 480×640, touch targets ≥48dp, no horizontal scrolling.
  9. **No protocol changes**: EQ is local DSP only; no librespot/Rust protocol changes.

## Files to Modify
1. **`app/src/main/java/com/sidetrack/audio/AudioCallback.kt`**
   - Add an `Equalizer` field and a method `setEqLevels(List<Float>)`.
   - Create EQ after `AudioTrack` is created, using `AudioTrack.audioSessionId`.
   - When `onAudioData` recreates the track (`audioTrack = null`), release EQ and recreate after new track.
   - Provide a function to release EQ when the service stops.
2. **`app/src/main/java/com/sidetrack/settings/SettingsManager.kt`**
   - Add `eqEnabled: Boolean = false` and `eqLevels: List<Float> = ...` to `SettingsState`.
   - Add `loadFromPrefs` and `applyEqSettings(normalization, ...)` similar to audio settings; store/load `eqEnabled` and `eqLevels` from `SharedPreferences`.
   - Add `setEqLevels(levels: List<Float>)` that persists and pushes levels to `AudioCallback`.
3. **`app/src/main/java/com/sidetrack/ui/SettingsScreen.kt`**
   - Add an "Equalizer" section after the audio controls.
   - Display a switch `Enable EQ` and a scrollable column of `Slider`s for each band.
   - Use `EQUtils.getEqBands(context)` and `EQUtils.getEqRange(context)` (copy from OpenEQ) to populate band list and range.
   - When the user changes a slider, update `SettingsManager.setEqLevels` and call `NativeBridge.playerRecreate()` (or send command to native to update EQ).
   - UI limited to vertical layout, large touch targets (≥48dp), simple labels.
4. **`app/src/main/java/com/sidetrack/viewmodel/PlayerViewModel.kt`**
   - Add an API for EQ level changes; call `NativeBridge.playerConfigure` to send EQ JSON (similar to audio settings) so the native layer can apply the EQ immediately after player recreation.
5. **`app/src/main/AndroidManifest.xml`**
   - No new permissions required (no FOREGROUND_SERVICE, no WRITE_EXTERNAL_STORAGE).
   - Ensure `android:usesCleartextTraffic="true"` if needed (already present?).
6. **`app/src/main/java/com/sidetrack/bridge/NativeBridge.kt`**
   - Add `setEqLevels(levelsJson: String): String?` external fun; expose to Kotlin.
   - Implement JNI in native (`native/src/bridge.rs`, `native/src/session.rs`) to parse EQ levels JSON and store in a global slot; apply EQ to current `Equalizer` instance (create if not present, update levels).
   - Ensure EQ is applied after `playerCreate` or `playerRecreate`.
   - If the EQ is disabled, clear the EQ instance (or release).

## Data Flow
1. **Startup**: `SidetrackApp.onCreate()` → `NativeBridge.init()` → native initializes global EQ state.
2. **Settings UI**: User toggles EQ switch and adjusts sliders → `SettingsManager.setEqLevels(levels)`:
   - Persist `eqEnabled` + `eqLevels` to `SharedPreferences`.
   - Call `NativeBridge.setEqLevels(JSON)`; native stores levels and applies to the current `Equalizer`.
3. **Audio Playback**:
   - When a track starts, `AudioCallback.createAudioTrack` creates an Android `AudioTrack` with a session ID.
   - Immediately after track creation (or after `PlayerViewModel.recreatePlayer()`), native uses the session ID to attach/update the `Equalizer` instance.
   - `AudioCallback.onAudioData` writes PCM to the `AudioTrack`; the EQ is applied by Android hardware if the device supports it.
4. **Pause/Resume / Track recreation**:
   - If `AudioTrack` is released (`audioTrack = null`), release the `Equalizer` (via JNI).
   - When playback resumes, `AudioCallback` recreates the `AudioTrack` and native re‑attaches the EQ.

## Failure Modes & Risks
- **Device Support**: Some Android devices may not expose a usable `Equalizer` for a custom audio session; `Equalizer(0, sessionId)` may throw `RuntimeException`. Handle gracefully with a fallback (EQ disabled).
- **AudioFocus**: When audio focus is lost, the EQ should be disabled to avoid audio leakage; `AudioFocusManager` can trigger `NativeBridge.setEqEnabled(false)`.
- **Screen Size**: M308’s 480×640 may not fit many bands. Show only available bands (`EQUtils.getEqBands`).
- **Performance**: `Equalizer` may introduce CPU overhead; keep EQ off by default, let user enable.
- **Persistence**: EQ levels should survive app process death? `SharedPreferences` persists across restarts.
- **Compatibility**: Equalizer may affect audio track’s buffer size; no impact expected.
- **Permission Creep**: No new permissions needed (unlike OpenEQ’s foreground service).

## Validation Steps
1. **Static Analysis**: Lint, Kotlin compiler, build (`:app:compileDebugSources`, `:app:assembleDebug`).
2. **Native Build**: `cargo check` in `native/`; ensure JNI function compiles.
3. **Device Simulation** (where possible): Build debug APK, install on API 28 emulator, enable EQ, verify slider UI appears.
4. **Playback Test**: Start playback, enable EQ, change levels, verify no crashes.
5. **Audio Focus Test**: Simulate another app forcing audio focus loss (e.g., launching a video player), check EQ disabled.
6. **EQ Support Test**: On a device with limited EQ support, verify fallback (EQ disabled) works.

## Rollout Plan
- **Phase 1 (Current)**: Implement core EQ attachment in `AudioCallback` and native bridge.
- **Phase 2**: Add `SettingsManager` state and UI in `SettingsScreen`.
- **Phase 3**: Test on physical M308 (if available) and emulator; tune UI for small screen.
- **Phase 4**: Release as a stable feature; document in `README.md` and `wiki/`.

## Open Decisions (Out of Scope)
- Global‑mix equalizer (as in OpenEQ’s foreground service) is excluded.
- Persistent presets (like OpenEQ’s saved EQ profiles) are not implemented; only current live EQ levels.
- Cross‑app EQ synchronization (e.g., EQ set in one app affecting Sidetrack) is not included.

## Next Immediate Action
Write the core EQ attachment in `AudioCallback` and the JNI bridge functions (`setEqLevels`). Then implement SettingsManager and SettingsScreen UI.

---
**Plan saved to:** `.kilo/plans/1794235697034-openeq-integration-plan.md`

**Next step:** Approve plan and switch to an implementation agent to execute the ordered tasks.
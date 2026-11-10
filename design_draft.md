# Equalizer for Sidetrack (M308 audio-only device)

* **Author:** mendeswm
* **Status:** Draft
* **Date:** 2026-11-10

--------------------------------------------------------------------------------

## Overview

Sidetrack is an unofficial Spotify client built on librespot (Rust) with a Jetpack Compose UI, adapted for the Oilsky M308: Android 9 (API 28), 480x640 touch screen, ~2 GB RAM, audio-only use. This design adds an equalizer so a listener can fine-tune audio on a device whose sole purpose is playback. The equalizer attaches only to Sidetrack's own `AudioTrack` audio session via `android.media.audiofx.Equalizer`, applies level changes live without recreating the player, defaults off, and lives entirely in Kotlin. No new permissions, no native (Rust/JNI) changes, no protocol changes, no offline playback.

## The Problem

Sidetrack currently renders PCM through a plain `AudioTrack` with no DSP. A listener on the M308 cannot adjust bass, midrange, or treble for the speaker/headphones in use, for the room, or for personal taste. The fork's stated mission is stable online playback on a constrained device, so any audio feature must not regress that stability or add CPU/RAM overhead when disabled.

The existing plan at `.kilo/plans/1794235697034-openeq-integration-plan.md` proposed routing the equalizer through `NativeBridge`/Rust and through `playerRecreate()` on slider changes. That is infeasible (`Equalizer` is a Java/Kotlin framework API that native code cannot instantiate, and native code never sees the `AudioTrack` session ID) and would interrupt playback on every slider drag. It also referenced an `EQUtils` helper that does not exist and hardcoded band counts that vary per device.

## Goals

* Provide a per-device, runtime-queried equalizer (band count, center frequencies, millibel range) attached only to Sidetrack's own `AudioTrack` session.
* Apply level changes live to the active `Equalizer` instance without recreating the player or interrupting playback.
* Default to off; create the `Equalizer` lazily only when enabled; release it on disable, audio-focus loss, or playback stop.
* Persist `eqEnabled` and per-band levels in `SettingsManager` (SharedPreferences) and restore them on the next play.
* Keep all equalizer logic in Kotlin (`com.sidetrack.audio`); do not modify `NativeBridge`, Rust, or the Spotify protocol.
* Add no new permissions and no new dependencies.
* Fit a 480x640 touch screen: vertical list of full-width horizontal sliders, touch targets >= 48 dp, no horizontal scroll for essentials.
* Degrade gracefully on devices where `Equalizer` construction throws: disable the toggle, show "not supported", never crash playback.

## Non-Goals

* Presets (flat/rock/pop/jazz/classical) in v1; deferred to a follow-up.
* OpenEQ code copy, foreground service, broadcast listeners, global mix, or cross-app leakage.
* Persistent EQ profiles beyond the current live levels.
* Native-side equalizer state or `NativeBridge` additions.
* Offline audio caching or cross-app EQ synchronization.
* Lossless/HiFi audio (librespot max is 320 kbps OGG Vorbis).

## Open Questions

* **Q1**: Whether the M308's audio framework exposes a usable `Equalizer` for a custom session at all (likely only verifiable on physical device; API 28 emulator often reports unsupported).
* **Q2**: Whether a persistent session ID via `AudioTrack.Builder.setAudioSessionId` survives track recreation on this OEM's Android 9 build, or whether per-track attach/release is required.
* **Q3**: CPU/RAM delta with the equalizer enabled on the M308 (to be measured; acceptance criterion 7).
* **Q4**: Whether 3 coarse bands (bass/mid/treble) would be more usable than the device's full band count on a 480 px screen (inventive-catalyst suggestion; deferred decision).

## Critical User Journeys

### CUJ 1: Enable and adjust equalizer

* **Given**: Sidetrack is playing a track on a device that supports `Equalizer`; the equalizer is off.
* **When**: The user opens Settings, toggles "Enable equalizer" on, and drags a band slider.
* **Then**: The `Equalizer` is created on the active session, the level applies to the playing audio within one slider update, no playback interruption or glitch occurs, and the setting persists across app restart.

### CUJ 2: Unsupported device

* **Given**: The device's audio framework cannot create an `Equalizer` for the session.
* **When**: The user opens Settings and views the equalizer section.
* **Then**: The toggle is disabled, a "Equalizer not supported on this device" message is shown, no exception escapes, and playback continues normally.

### CUJ 3: Audio focus loss

* **Given**: The equalizer is enabled and playing.
* **When**: Another app takes audio focus.
* **Then**: The equalizer is disabled (released) while focus is lost; on focus regain the previous enabled state and levels are restored and re-applied.

### CUJ 4: Playback stop and app destroy

* **Given**: The equalizer is enabled.
* **When**: Playback stops or the app process is destroyed.
* **Then**: The `Equalizer` instance is released; no audio session leak remains.

--------------------------------------------------------------------------------

## Technical Plan (Architecture)

### Components

* **`AudioEffectsManager`** (new, `com.sidetrack.audio`): owns the `android.media.audiofx.Equalizer` instance, the stable audio session ID, the `supported` flag, and the apply/release logic. Exposes `onTrackCreated(track: AudioTrack)`, `onTrackReleased()`, `setEnabled(enabled: Boolean)`, `setLevels(levels: List<Int>)`, and `release()`. All `Equalizer` access is guarded by a lock; construction failures set `supported = false` and never propagate to the audio thread.
* **`AudioCallback`** (modify): notifies `AudioEffectsManager` after `createAudioTrack()` in both code paths (`onAudioData` lazy create and the `IllegalStateException` recreate path) and on `release()`. Does not own the `Equalizer`.
* **`SettingsManager`** (modify): add `eqEnabled: Boolean = false` and `eqLevels: List<Int>` to `SettingsState`, with prefs keys `KEY_EQ_ENABLED` and `KEY_EQ_LEVELS` (stored as a comma-separated millibel string or JSON array). `loadFromPrefs` pads/truncates levels to the runtime band count and clamps to the range. Add `setEqEnabled(Boolean)` and `setEqLevels(List<Int>)` that persist + update `_state` only (no native push, no player recreate). Expose `StateFlow<SettingsState>`.
* **`SettingsScreen`** (modify): add an "Equalizer" section after the audio controls: a switch bound to `eqEnabled` and a vertical column of `Slider`s, one per runtime-discovered band, with frequency + dB labels, `fillMaxWidth`, >= 48 dp touch targets, no horizontal scroll. Disabled/hidden with an "Equalizer not supported on this device" message when `supported == false`.
* **`PlayerViewModel`** (modify): wire `AudioEffectsManager` to `AudioCallback` and to `SettingsManager.state`; on audio-focus loss disable EQ and restore on gain; re-apply levels after `play()` recreates the track; release in `stop()` and `onCleared()`.
* **`AudioFocusManager`** (no change): existing listener interface is reused by `PlayerViewModel` to drive EQ disable/restore.
* **`NativeBridge` / Rust** (no change): untouched. No JNI, no `setEqLevels`, no `playerConfigure` EQ JSON, no `playerRecreate` on slider change.

### Data flow

```text
SettingsScreen (UI thread)
   | toggle / slider drag
   v
SettingsManager.setEqEnabled / setEqLevels  (persist + StateFlow update)
   |
   v
PlayerViewModel observes SettingsState
   |
   v
AudioEffectsManager.setEnabled / setLevels  (lock-guarded)
   |  if enabled && supported && track exists
   v
Equalizer.setBandLevel(band, millibels)   (live, no player recreate)

AudioCallback.onAudioData (audio thread)
   | track == null -> createAudioTrack() -> onTrackCreated(track)
   v
AudioEffectsManager.onTrackCreated(track)
   | capture track.audioSessionId (stable, set at build time)
   | if enabled && supported: Equalizer(0, sessionId) [try/catch] + apply persisted levels

AudioCallback.release() / stop() / focus loss
   |
   v
AudioEffectsManager.onTrackReleased() / setEnabled(false)  (release instance)
```

### Stable session ID

`AudioCallback.createAudioTrack()` must call `.setSessionId(sessionId)` on the `AudioTrack.Builder`, using one session ID per app session (generated once, e.g. `AudioManager.generateAudioSessionId()` or a fixed app constant, retained across track recreations). This lets the `Equalizer` survive track recreation instead of being released and re-created per track. The session ID is released when the app process is destroyed.

### Threading

* `Equalizer` construction and `setBandLevel` are binder calls. They are never invoked on the audio thread from `onAudioData`: construction happens on a coroutine/IO dispatcher triggered by `onTrackCreated`; slider updates are debounced and dispatched off the main thread.
* A single lock (or single-thread dispatcher) serializes all `Equalizer` access to avoid torn references between UI, audio, and focus paths.
* Construction failures are caught (`RuntimeException`, `IllegalArgumentException`, `UnsupportedOperationException`) and recorded as `supported = false`; they never propagate to the PCM write path.

### Persistence format

* `eqEnabled`: boolean in SharedPreferences.
* `eqLevels`: comma-separated millibels (e.g. `"0,1200,-300,..."`) or a JSON array of ints. On load, if the stored length differs from the runtime band count, pad with 0 or truncate, then clamp each value to `getBandLevelRange()`. Keys are additive so older installs load defaults.

### Non-functional

* **Performance**: zero DSP overhead when disabled (no `Equalizer` instance exists); lazy creation only when enabled and a track exists; release on disable.
* **Rollout/rollback**: default off; no feature flag needed beyond the toggle; reverting is removing the section (additive changes only).
* **Security/privacy**: no new permissions; no secrets; no logging of auth material.

--------------------------------------------------------------------------------

## Alternatives Considered

### Alternative A: Native/Rust owns the Equalizer via JNI

* **Pros**: Keeps DSP state in the same layer as playback; could in theory share state across components.
* **Cons**: Infeasible. `android.media.audiofx.Equalizer` is a Java/Kotlin framework class; Rust cannot instantiate or configure it without an undefined JNI-to-Java callback layer. Native code never sees `AudioTrack` or its session ID. Violates "no JNI / no protocol changes."
* **Reason for Rejection**: Rejected by author: impossible given the Android API surface; adds cross-language state for no benefit.

### Alternative B: Slider changes routed through `playerRecreate()` / `playerConfigure` JSON

* **Pros**: Reuses existing native player lifecycle and config-push path.
* **Cons**: Interrupts playback on every slider drag; contradicts "apply live without recreating the player"; risks glitches/stalls on a 2 GB device whose acceptance criteria require sustained glitch-free playback.
* **Reason for Rejection**: Rejected by author: EQ is an Android effect on the session, not a librespot config; recreating the player is unnecessary and harmful.

### Alternative C: Hardcoded 5 bands (60 Hz / 250 Hz / 500 Hz / 1 kHz / 2 kHz / 4 kHz / 8 kHz)

* **Pros**: Simple UI; matches the plan's original sketch.
* **Cons**: Device band count and center frequencies vary; hardcoding produces wrong labels or out-of-range calls. OpenEQ's `EQUtils` (cited as "copy from OpenEQ") does not exist in this repo and is external licensed code.
* **Reason for Rejection**: Rejected by author: band metadata must come from `Equalizer` at runtime; no external code copy.

### Alternative D: `Equalizer` stored inside `AudioCallback`

* **Pros**: Co-located with the `AudioTrack` it attaches to; fewer classes.
* **Cons**: Couples PCM rendering to DSP; harder to test and lifecycle-manage; construction on the audio thread risks blocking the PCM write path.
* **Reason for Rejection**: Rejected by author: a dedicated `AudioEffectsManager` separates concerns and keeps `AudioCallback` focused on PCM delivery.

### Alternative E: `Equalizer` stored inside `PlayerViewModel`

* **Pros**: UI and audio state in one place.
* **Cons**: `PlayerViewModel` does not own `AudioTrack` session timing; fragile attach/release ordering; UI layer gains audio-hardware responsibility.
* **Reason for Rejection**: Rejected by author: manager owned by `com.sidetrack.audio`, wired by `PlayerViewModel`.

### Alternative F: Always-created `Equalizer` even when disabled

* **Pros**: Simpler state (no lazy creation).
* **Cons**: Wastes CPU/RAM on a 2 GB device; construction can fail on unsupported devices even when the user never enabled EQ.
* **Reason for Rejection**: Rejected by author: lazy creation only when enabled; release on disable.

### Alternative G: Presets (flat / rock / pop / jazz / classical) as primary UI

* **Pros**: One-tap adjustment; less screen crowding; lower cognitive load on a 480 px screen.
* **Cons**: Not in v1 scope; adds preset definitions and mapping; deferred to follow-up.
* **Reason for Rejection**: Rejected by author for v1: start with live per-band sliders; revisit presets if usability testing shows the slider panel is too dense.

--------------------------------------------------------------------------------

## Detailed Implementation Plan

### Target Files to Create/Modify

#### 1. `app/src/main/java/com/sidetrack/audio/AudioEffectsManager.kt` (Create)

* **Type / Scope**: Kotlin class in `com.sidetrack.audio`.
* **Proposed Changes**: New manager owning `@Volatile var equalizer: Equalizer?`, `supported: Boolean`, `sessionId: Int`, a lock, and persisted levels. Methods: `onTrackCreated(track: AudioTrack)`, `onTrackReleased()`, `setEnabled(enabled: Boolean)`, `setLevels(levels: List<Int>)`, `release()`. `onTrackCreated` captures `track.audioSessionId` (stable, set by `AudioCallback`), constructs `Equalizer(0, sessionId)` inside try/catch, applies persisted levels if enabled. `setLevels` updates the live instance under the lock and stores levels. `setEnabled(false)` releases the instance. `release()` releases instance and resets state. Never throws to callers.

#### 2. `app/src/main/java/com/sidetrack/audio/AudioCallback.kt` (Modify)

* **Type / Scope**: Kotlin audio sink.
* **Proposed Changes**: Add a constructor/field reference to `AudioEffectsManager` (or a callback interface). In `createAudioTrack()`, add `.setSessionId(sessionId)` to the `AudioTrack.Builder` using a session ID generated once per app session and retained. After both `createAudioTrack()` call sites in `onAudioData()`, call `audioEffectsManager.onTrackCreated(track)`. In `release()`, call `audioEffectsManager.onTrackReleased()` before releasing the `AudioTrack`. No `Equalizer` field here.

#### 3. `app/src/main/java/com/sidetrack/settings/SettingsManager.kt` (Modify)

* **Type / Scope**: Kotlin settings/persistence.
* **Proposed Changes**: Add `eqEnabled: Boolean = false` and `eqLevels: List<Int> = emptyList()` to `SettingsState`. Add prefs keys `KEY_EQ_ENABLED`, `KEY_EQ_LEVELS`. `loadFromPrefs` reads them (levels parsed from comma-separated string; pad/truncate to a stored or default band count; clamp). Add `setEqEnabled(Boolean)` and `setEqLevels(List<Int>)` that persist + update `_state` only. No `pushConfigToNative` or `playerRecreate` calls for EQ. Additive keys so old installs load defaults.

#### 4. `app/src/main/java/com/sidetrack/ui/SettingsScreen.kt` (Modify)

* **Type / Scope**: Jetpack Compose UI.
* **Proposed Changes**: Add an "Equalizer" section after the audio controls: a `Switch` bound to `eqEnabled`, and a vertical `Column` of `Slider`s (one per runtime-discovered band) with frequency (Hz) and current level (dB/millibels) labels. `fillMaxWidth`, touch targets >= 48 dp, no horizontal scroll. When `supported == false`, disable the switch and show "Equalizer not supported on this device." Slider changes call `SettingsManager.setEqLevels` immediately (live apply via `AudioEffectsManager`).

#### 5. `app/src/main/java/com/sidetrack/viewmodel/PlayerViewModel.kt` (Modify)

* **Type / Scope**: Kotlin ViewModel.
* **Proposed Changes**: Instantiate `AudioEffectsManager`, pass to `AudioCallback`, and observe `SettingsManager.state` to drive `setEnabled`/`setLevels`. In `initPlatform`'s `AudioFocusManager` listener, disable EQ on duck/loss and restore on gain. In `play()`, re-apply levels after track recreation. In `stop()` and `onCleared()`, call `audioEffectsManager.release()`. Do not call `recreatePlayer()` for EQ changes.

#### 6. `app/src/main/AndroidManifest.xml` (No change)

* **Type / Scope**: Manifest.
* **Proposed Changes**: None. Guardrail satisfied (no new permissions; `FOREGROUND_SERVICE` already present for media playback is unrelated).

#### 7. `app/build.gradle.kts` (No change)

* **Type / Scope**: Gradle build.
* **Proposed Changes**: None. `minSdk` 28 supports `Equalizer` (API 3+); no new dependencies.

#### 8. `README.md` (Modify, later)

* **Type / Scope**: Documentation.
* **Proposed Changes**: Add a short "Equalizer" note under features/limitations once the feature ships; record M308 EQ behavior as unverified.

#### 9. `wiki/project/sidetrack-m308-adaptation.md` (Modify, later)

* **Type / Scope**: Wiki.
* **Proposed Changes**: Record EQ implementation status, CPU/RAM delta, and unsupported-device findings.

### Verification Plan

* **Compilation/Build**: `./gradlew :app:compileDebugSources` and `./gradlew :app:assembleDebug`; `cargo check` in `native/` (unchanged, must still pass).
* **Static Analysis & Linting**: Kotlin compiler warnings; no new lint suppressions.
* **Unit/Manual Tests**:
  * API 28 emulator: install debug APK; verify EQ section shows "not supported" or disabled when `Equalizer` construction fails; no crash; playback works.
  * Physical M308: toggle EQ on, drag each band, confirm live audio change with no glitches/stalls; sustained playback >= 10 minutes with EQ on.
  * Audio focus loss: simulate (launch video player) -> EQ disables; on regain -> EQ restores previous state.
  * Persistence: set levels, kill process, relaunch -> levels restored.
  * CPU/RAM: measure with EQ off vs on (acceptance criterion 7); explain any regression.
* **Test Coverage Requirement**: No automated unit tests exist in repo; manual matrix above is the gate. Add tests if a test harness is introduced.
* **E2E/Integration**: Not applicable (no E2E harness present).

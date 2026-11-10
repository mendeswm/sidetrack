# 5-band Equalizer for Sidetrack (M308 audio-only device)

* **Author:** mendeswm
* **Date:** 2026-11-10

## Overview

Add a 5-band (device-dependent, queried at runtime) equalizer to Sidetrack so users can fine-tune audio on the Oilsky M308, an Android 9/API 28, 480x640 touch, ~2 GB RAM audio-only player. The EQ attaches to the app's own `AudioTrack` session via `android.media.audiofx.Equalizer` (API 3+), applies live on slider changes without recreating the player, defaults off, and owns nothing native — no JNI changes, no new permissions, no protocol changes. Settings (`eqEnabled`, band levels) persist in `SettingsManager` (SharedPreferences) and restore on next play.

Scope is deliberately narrow: settings-only, default off, no presets initially, no OpenEQ code copied, no foreground service, no global mix, no cross-app leakage.

## Analysis

Seven Goldfish lenses (design, codebase-details, zoom-out, comprehension, implementation-readiness, structural, inventive-catalyst) reviewed the plan at `.kilo/plans/1794235697034-openeq-integration-plan.md` against the actual code.

Key objections / open risks:

1. **Native/JNI EQ ownership is infeasible (blocker).** `android.media.audiofx.Equalizer` is a Java/Kotlin framework class. Rust can never instantiate or configure it, and the native layer never sees `AudioTrack` or its session ID. The plan's `NativeBridge.setEqLevels` + Rust storage path is a dead end and contradicts "no protocol changes." EQ must live entirely in Kotlin under `com.sidetrack.audio`.

2. **Live slider changes routed through `playerRecreate()`/`playerConfigure` JSON interrupts playback (blocker).** This contradicts the stated requirement "applied live on slider changes without recreating the player" and would glitch/stal on a 2 GB device whose acceptance criteria demand sustained glitch-free playback. EQ updates must call `Equalizer.setBandLevel` directly on the existing instance.

3. **`EQUtils.getEqBands`/`getEqRange` do not exist; hardcoding 5 bands (or 7 frequencies) is wrong (blocker).** OpenEQ code is external/licensed and must not be copied. Device band count, center frequencies, and millibel range vary per device; the UI must query `Equalizer.getNumberOfBands()`, `getBandFreqRange(band)`, and `getBandLevelRange()` at runtime and render only supported bands.

4. **No error handling for `Equalizer(0, sessionId)` throwing `RuntimeException` (blocker).** Unsupported devices throw on construction. Zero try/catch exists anywhere for `Equalizer`. Failure must degrade silently to "EQ unavailable / default off" with no playback crash.

5. **No stable audio session ID (major).** `AudioCallback.createAudioTrack()` never calls `AudioTrack.Builder.setSessionId(...)`, so every track/pause-resume gets a fresh session. EQ would need release+re-attach each time — a stalls risk on M308. Fix: pin one session ID per app session via `setAudioSessionId`, attach EQ once, release it on app destroy/disable.

6. **`List<Float>` is the wrong unit (major).** Android EQ levels are integer millibels (`getBandLevelRange()`, typically -15000 to +15000), not floats. Model levels as `List<Int>`/`List<Short>` clamped to the range; slider maps position to millibels; persist as ints (comma-separated or JSON).

7. **SettingsManager should persist only; AudioEffectsManager applies live (major).** The plan conflates DSP with player config (`PlayerViewModel.recreatePlayer()`) and mixes EQ into `SettingsManager.pushConfigToNative`/JSON. `SettingsManager` stores `eqEnabled` + band levels in its `StateFlow`; a dedicated `AudioEffectsManager` owns the `Equalizer`, attaches/releases it, and applies levels live. UI updates `SettingsManager`; manager applies to audio.

8. **Threading is unspecified (major).** `onAudioData` runs on the Rust-attached audio thread; slider updates run on the UI thread; `Equalizer` calls are binder IPC. Need a lock / single-thread dispatcher, construction never propagated to the audio thread, UI updates off main thread with debouncing.

9. **EQ is premature complexity (contrarian/YAGNI).** Project priorities are stable online playback, API 28 runtime verification, ABI confirmation, and RAM/CPU discipline — EQ mitigates none of these core risks (ABI and density are still unverified; no emulator testing). If shipped, gate behind default-off toggle, measure CPU/RAM with EQ on vs off, and record M308 EQ behavior as unverified per acceptance criterion 10.

10. **Preferred structure (structural):** a dedicated `AudioEffectsManager` in `com.sidetrack.audio` owning the `Equalizer`; `AudioCallback` only notifies `onTrackCreated`/`onTrackReleased`. `PlayerViewModel` wires it; no EQ field in `AudioCallback` itself.

11. **Inventive-catalyst IFR (no compromise):** presets primary (flat/rock/pop/jazz/classical), optional 3 coarse bands (bass/mid/treble) in advanced mode — not 5 sliders on a 480px screen; lazy `Equalizer` created only when enabled; released on disable/focus loss; stable session ID; zero native changes; `AudioTrack.Builder.setAudioSessionId` for continuity across track recreation.

## Goals vs. Non-Goals

**In-scope goals:**
- 5-band (device-queried) equalizer attached to Sidetrack's own `AudioTrack` session.
- Live slider apply, default off, no player recreation.
- Settings persisted in `SettingsManager`; restore on next play.
- No new permissions; no JNI/Rust changes; no protocol changes.
- Compact vertical slider panel in `SettingsScreen` fitting 480x640, touch targets >=48dp, no horizontal scroll.
- Graceful fallback when `Equalizer` unsupported (disable toggle, no crash).
- Audio focus loss disables EQ; restore on gain.

**Out-of-scope non-goals:**
- Presets (v1) — considered but deferred; kept as a candidate advanced simplification.
- OpenEQ code copy / foreground service / broadcast listeners / global mix.
- Persistent EQ profiles beyond current live levels.
- Native-side EQ state or `NativeBridge` additions.
- Offline audio caching or cross-app EQ synchronization.

## Alternatives Weighed

| Alternative | Reason rejected |
|---|---|
| Native/Rust owns `Equalizer` via JNI | Infeasible: `Equalizer` is Java-only; native has no session ID access. |
| 5 hardcoded bands (60Hz–8kHz) | Device band count/frequencies vary; hardcoding contradicts API. |
| `EQUtils` copied from OpenEQ | External license; no copy needed (`Equalizer` exposes all metadata). |
| Slider changes via `playerRecreate()` / `playerConfigure` JSON | Interrupts playback; EQ is an Android effect on the session, not librespot config. |
| Always-created `Equalizer` even when disabled | Wastes CPU/RAM on a 2 GB device; create lazily only when enabled. |
| `Equalizer` inside `AudioCallback` | Couples PCM rendering to DSP; harder to test and lifecycle-manage. |
| `Equalizer` inside `PlayerViewModel` | ViewModel does not own `AudioTrack` session timing; fragile. |

## Key Decisions & Staging for Next Steps

**Decisions:**
- EQ owned by new `AudioEffectsManager` in `com.sidetrack.audio`; `AudioCallback` notifies track lifecycle.
- Levels stored as millibels (`List<Int>`/`List<Short>`), clamped to `getBandLevelRange()`.
- No JNI/Rust changes; `NativeBridge` untouched.
- Default off; lazy creation; release on disable/focus loss/app destroy.
- Stable session ID via `AudioTrack.Builder.setAudioSessionId`.
- UI: vertical full-width horizontal sliders (≥48dp), labels (frequency + dB), disabled/hidden state when unsupported.
- No presets in v1; defer to follow-up.

**Staging:**
1. Revise `.kilo/plans/1794235697034-openeq-integration-plan.md` to reflect the corrected scope (drop JNI/native EQ, drop `playerRecreate` on slider change, runtime band query, millibel levels, `AudioEffectsManager`).
2. Implement `AudioEffectsManager` + wire into `AudioCallback` track lifecycle + `SettingsManager` eq fields.
3. Add EQ section to `SettingsScreen`.
4. Build `:app:assembleDebug` + `cargo check`; test API 28 emulator (unsupported fallback) and physical M308 (happy path, focus loss, persistence, CPU/RAM delta).
5. Document in `README.md` and `wiki/`; record M308 EQ behavior as unverified.

**Handoff options:** Publish `concepts_brief.md` as RFC / proceed to `egm-draft` / continue refining / conclude.

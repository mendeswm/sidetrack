---
name: android-resource-optimization
description: Profile and optimize Android memory, CPU, battery, startup time, UI responsiveness, images, Jetpack Compose, JNI, and Rust audio paths. Use for resource work on the Sidetrack Oilsky M308 fork, especially Android 9 and constrained hardware.
---

# Android Resource Optimization

## Purpose and target

Improve resource efficiency on the Oilsky M308 without sacrificing playback stability, audio quality, Android 9 (API 28) compatibility, UI responsiveness, security, or maintainability.

Owner-reported device specifications:
- Android 9 / API 28
- 2 GB RAM
- 2.8-inch touch display, reported 480 × 640 physical pixels
- 8-core 1.5 GHz processor
- 16 GB internal storage

Verify actual Android build, ABI, display density, and profiling support when they affect a decision. Do not infer architecture from marketing specifications.

## When to use

Use this skill when investigating or changing RAM, CPU, battery, startup time, UI jank, image loading/caching, Compose recomposition, services, background work, audio buffers, Kotlin/JNI/Rust boundaries, R8, APK size, or performance-related dependencies.

Do not use this skill as a reason to rewrite working code preemptively.

## Mandatory rules

1. **Measure before optimizing.** Establish a baseline before changing behavior whenever practical.
2. **Change one logical thing at a time.** State a hypothesis, make a focused change, and compare results.
3. **Protect playback.** Do not knowingly introduce audible glitches, persistent underruns, lost playback state, or UI stalls.
4. **Separate metrics.** APK size, install size, runtime RAM, CPU, battery, and responsiveness are different measurements.
5. **Avoid speculative micro-optimizations.** Added complexity must have a measurable or well-supported benefit.
6. **Preserve API 28 compatibility.** Newer APIs or dependencies need a compatible fallback and testing.
7. **Report evidence honestly.** Distinguish measurements from estimates and emulator results from physical-device results.
8. **Protect secrets.** Never log passwords, tokens, cookies, keys, or authentication payloads.
9. **Avoid destructive changes.** Explain risks and obtain confirmation before destructive or irreversible actions.
10. **Preserve the architecture** unless evidence supports a change.

## Workflow

### 1. Inspect

Before editing, identify:
- `minSdk`, `targetSdk`, `compileSdk`, Android Gradle Plugin, Gradle Wrapper, Kotlin, Compose, JDK, and NDK versions.
- Image-loading and caching implementation.
- Playback service lifecycle and background-work strategy.
- Audio path from librespot/Rust through JNI to Android audio output.
- Existing tests, logs, tracing, profiling facilities, and native ABIs packaged in the APK.

Read relevant source and build files; do not assume a library is present.

### 2. Establish a baseline

Where possible, measure:
- Cold and warm startup time.
- Process memory at idle, during browsing, and during playback.
- Managed Java/Kotlin heap and native memory separately.
- CPU during idle, browsing, and sustained playback.
- UI frame timing, jank, and main-thread stalls.
- Audio underruns, glitches, and recovery from network interruptions.
- Battery/energy impact when a reliable method is available.
- Release APK and native-library sizes.

Record device, Android version, build type, workload, test duration, and method. Use repeatable before/after workloads. If the M308 is unavailable, label other-device or emulator results as interim and unverified on the target device.

### 3. Form a testable hypothesis

Examples:
- Album artwork retains oversized bitmaps.
- A frequently recomposed screen performs unnecessary work.
- PCM buffers are copied excessively across JNI.
- Polling wakes the CPU too often.
- A cache grows without a bound.
- A service retains references beyond its useful lifetime.

State the metric expected to improve and behavior that must remain unchanged.

### 4. Make the smallest effective change

Keep unrelated refactoring separate. If results do not support the hypothesis, revert or reconsider the change rather than stacking speculative optimizations.

### 5. Validate and report

Run relevant Rust/Android tests, lint/static checks, debug builds, and release builds where available. After changes to audio, services, scheduling, or memory handling, test sustained playback.

Report the hypothesis, changed files, measurements and conditions, UI/audio regression checks, limitations, and whether the change should be kept, revised, or reverted.

## Memory and allocation

- Investigate retained objects, oversized images, unbounded caches, duplicate data, activity/view references, and allocations in hot paths.
- Decode artwork near its actual display size.
- Bound image caches and avoid retaining screens or bitmaps unnecessarily.
- Release resources according to lifecycle.
- Inspect both managed heap and native memory; Rust/JNI allocations may not appear in Java heap metrics.
- Do not add object pools automatically: they may increase retained memory and synchronization costs.
- Do not force garbage collection as a routine fix.
- Do not remove useful caches without checking CPU, network, and latency effects.

## CPU and battery

- Keep network operations, expensive decoding, and blocking work off the main thread.
- Avoid unnecessary polling, tight loops, excessive timers, and wakeups.
- Stop background work when it is no longer needed.
- Reuse computations only when correctness and lifecycle are clear.
- Measure before removing animations or UI updates.
- Low average CPU is not enough if long stalls occur.

## Jetpack Compose

If Compose is present:
- Find recomposition hotspots with supported tools before changing state structure.
- Keep state local where practical; avoid broad state changes that invalidate unrelated UI.
- Move expensive work out of frequently recomposed composables.
- Use memoization only when it reduces measured work and does not create stale state.
- Avoid heavy object creation and unbounded collections during recomposition.
- Do not add complex stability annotations or patterns without understanding correctness.
- Recheck scrolling, navigation, touch response, and playback controls.

## Images and small-screen UI

- Load artwork on demand and decode at an appropriate size.
- Bound memory caches and avoid duplicate image copies.
- Do not load all artwork for a large library at once.
- Fit essential controls to the 480 × 640 physical-pixel screen, accounting for density and insets.
- Preserve legibility and usable touch targets; do not trade them for tiny CPU savings.

## Audio, Rust, and JNI

Audio stability is a release-blocking concern. Inspect PCM buffer size/lifetime/ownership, copies across Rust/JNI/Kotlin/audio output, format conversions, allocation frequency, blocking, lock contention, backpressure, underruns, and network-stall recovery.

- Do not perform blocking network or UI work on the latency-sensitive audio path.
- Avoid unbounded queues and uncontrolled buffer accumulation.
- Do not reduce buffers just to lower RAM without testing underruns and latency.
- Do not change unsafe JNI/Rust ownership without a safety argument and tests.
- Prefer measured reduction of redundant copies over speculative rewrites.
- Test sustained playback, track changes, pause/resume, network interruption, and returning to the app.

## Services and background work

- Limit background work to playback and user-visible functionality.
- Follow Android 9 service/notification behavior; do not blindly apply guidance requiring newer Android versions.
- Avoid repeated wakeups and background polling.
- Preserve playback controls and session behavior when navigating away.
- Test lifecycle transitions and process recreation where relevant.

## Dependencies, R8, and release builds

- Check actual dependency use and weight before adding/removing libraries.
- Prefer existing project facilities over adding a framework for a small task.
- Evaluate R8 code shrinking and resource shrinking for release builds when supported.
- Review keep rules carefully; JNI, reflection, serialization, or service discovery may depend on them.
- Verify required Rust libraries and resources remain in release artifacts.
- Treat APK-size improvements separately from runtime RAM/CPU improvements.

## Tooling

Use compatible tools already available, such as Android Studio Profiler, Perfetto, Simpleperf, `adb`, `dumpsys meminfo`, `top`, logcat, Rust tests, native profilers, Gradle reports, and APK inspection tools.

Tool support varies by Android version, firmware, host, and build configuration. Verify a method before relying on it. Do not add a profiling dependency to the production app if the same measurement can be collected externally.

## Decision criteria

Keep an optimization only if it measurably improves the target metric, fixes a confirmed defect, or removes clearly unnecessary work with low regression risk.

Revisit or reject changes when results are within measurement noise, playback/UI/compatibility regresses, a small CPU gain substantially increases retained memory, complexity grows without clear benefit, or APK size is used as evidence for an unrelated runtime metric.

## Definition of success

An optimization succeeds when the target metric improves or a confirmed defect is fixed, test conditions are documented, Android 9 compatibility is preserved, and playback remains stable. State what was measured and on which device; never claim unmeasured performance gains.

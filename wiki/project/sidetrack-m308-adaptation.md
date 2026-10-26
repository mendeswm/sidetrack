# Sidetrack M308 Adaptation

> Sources: AGENTS.md, 2026-10-26; CONTEXT.md, 2026-10-26; DEPENDENCIAS-UBUNTU-26.04.md, 2026-10-26; DESIGN.md, 2026-10-26; README.md, 2026-10-26; app/build.gradle.kts, 2026-10-26
> Raw: [project context](../../raw/project/2026-10-26-context.md); [ubuntu dependencies](../../raw/dev-environment/2026-10-26-ubuntu-dependencies.md); [design doc](../../raw/project/2026-10-26-design-doc.md); [readme](../../raw/project/2026-10-26-readme-sidetrack.md)
> Updated: 2026-10-26

## Overview

Sidetrack is a fork of qfertig/sidetrack (itself a fork of jtaekman/sidespot) adapted for the Oilsky M308 portable player. The adaptation targets Android 9 (API 28), a 480×640 touch screen, and ~2GB RAM, prioritizing stable online playback, compact touch UI, and memory efficiency.

## Target Device

- OS: Android 9.0 (API 28)
- CPU: 8 cores, 1.5 GHz
- RAM: 2 GB
- Storage: 16 GB internal + microSD
- Screen: 2.8" LCD touch, 480×640 pixels
- ABI: unconfirmed (likely arm64-v8a or armeabi-v7a)

## Build Configuration

- AGP 8.7.3, Kotlin 2.1.0, Gradle 8.11.1, Java 17
- compileSdk 34, targetSdk 30, minSdk 28 (lowered from 30)
- NDK 26.1.10909125
- ABIs: arm64-v8a, armeabi-v7a (universal APK enabled)
- Native built via cargo-ndk into app/src/main/jniLibs/

## Architecture

- UI: Jetpack Compose, MVVM, Navigation Compose
- Native: Rust (librespot fork) with JNI bridge, tokio async runtime
- Audio: librespot decodes OGG Vorbis → PCM, delivered via JNI to AudioTrack
- Max quality: 320 kbps OGG Vorbis (no lossless support via librespot)

## Memory Optimizations Applied

1. **Coil image loader** configured in SidetrackApp with 10% memory cache and 20MB disk cache; crossfade disabled
2. **NowPlayingScreen** album art reduced from 720px to 320px (both sharp and blurred variants)
3. **DynamicTheme** palette cache reduced from 20 to 8 entries; color count from 16 to 8
4. **QueueManager** metadata cache cap reduced from 200 to 50 entries; eviction window narrowed
5. **PlayerViewModel** metadata resolution reduced from 20 to 8 upcoming tracks; preload from 3 to 2
6. **NowPlayingScreen** horizontal padding reduced from 24dp to 16dp for 480px width

## Optimization Experiment: `allowHardware(true)` on Album Art (2026-10-26)

**Hypothesis:** Enabling hardware bitmaps on the sharp album-art bitmap in NowPlayingScreen moves ~400 KB from Java heap to native GPU memory (ashmem), reducing Java heap pressure on the 2 GB M308.

**Change made:**
- `NowPlayingScreen.kt:93` — sharp bitmap: `.allowHardware(false)` → `.allowHardware(true)`
- `NowPlayingScreen.kt:102` — blur bitmap: kept `.allowHardware(false)` (blur transform uses `getPixels()`, incompatible with hardware bitmaps on API 28)
- `DynamicTheme.kt:93` — kept `.allowHardware(false)` (draws to software Canvas)

**Device:** Oilsky M308, MT6750V/C (MediaTek), Mali-T860 MP2 GPU, Android 9 (API 28), 2 GB RAM (~1.8 GB reported).

**Measurement results (not comparable — different workload states):**

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| Java Heap | 11,316 KB | 12,624 KB | +1,308 KB |
| Native Heap | 22,944 KB | 20,476 KB | -2,468 KB |
| Graphics | 8,488 KB | 3,216 KB | -5,272 KB |
| Ashmem | 36 KB | 36 KB | 0 KB |
| TOTAL | 83,356 KB | 119,023 KB | +35,667 KB (driven by .apk mmap) |

**Assessment:**
- Ashmem unchanged at 36 KB suggests the hardware bitmap is either not being retained or `Drawable.toBitmap()` is copying to software.
- The 5.3 MB graphics drop strongly indicates different screens were active during each measurement (NowPlaying with album art vs. a list screen).
- Measurements are **not apples-to-apples**; no definitive conclusion possible.

**Decision:** Kept as-is for further testing. The change is low-risk (Compose renders hardware bitmaps on hardware-accelerated canvases, which the Mali-T860 supports), but the ~400 KB theoretical benefit is unproven. Revert if runtime crashes occur or a controlled test shows no benefit.

**SoC note:** MT6750V/C fully supports hardware bitmaps — this is an Android framework feature (API 26+), not a SoC-specific decoding capability. Mali-T860 MP2 is capable of rendering hardware bitmaps.

## UI Adaptations

- Dark theme with dynamic album-art colors (Palette extraction)
- Touch targets minimum 48dp
- Compact typography for 480×640
- Bottom navigation: Queue, Library, Search
- Now Playing as full-screen overlay

## Known Limitations

- ABI unconfirmed on actual M308 device
- Screen density/insets unverified
- No emulator testing performed
- Foreground service behavior on API 28 may differ from API 30
- librespot submodule is dev branch (cca88d6)

## See Also

- [Ubuntu 26.04 Dependencies](../dev-environment/ubuntu-26.04-dependencies.md)

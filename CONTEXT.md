# CONTEXT.md — Oilsky M308 project context

## Overview
This repository is a fork of Sidetrack: https://github.com/qfertig/sidetrack. It uses librespot as a streaming component. The goal is to adapt the existing app for the Oilsky M308, whose Android 9 installation is below the upstream project's stated minimum Android version and whose small display makes the conventional Spotify app difficult to use. Inspect the current checkout to confirm the actual blockers before changing code.

## Target device (owner-reported)
- Android 9.0 / API 28
- 8-core 1.5 GHz processor
- 2 GB RAM, 16 GB internal storage
- microSD expansion up to 1 TB
- 2.8-inch touch LCD, reported 480 × 640 physical pixels
- FAT32, exFAT, and NTFS support reported

These details have not been independently verified. Confirm ABI, display density, firmware behavior, and available profiling support on the actual device.

## Objectives
1. Make the app compatible with Android 9/API 28.
2. Install and run on the Oilsky M308.
3. Provide a compact touch interface suitable for the small display.
4. Deliver stable online playback.
5. Use the best streaming quality actually available to the integration and account.
6. Reduce unnecessary RAM, CPU, allocations, and UI work.
7. Keep the UI responsive while audio plays.
8. Preserve the existing architecture and librespot core where feasible.
9. Document build, installation, tests, and known limitations.

## Initial feature priorities
- Existing supported login/session flow.
- Play/pause, previous/next, title, artist, album artwork.
- Progress and seeking when supported.
- Volume and background playback controls where supported by the existing implementation and Android 9.
- Simple access to library, search, playlists, favorites, and queue where feasible.
- Clear connecting/loading/playing/paused/network-error/session-expired states.

The first release is online-only. Do not add offline downloads or persistent audio-track caching.

## UI requirements
Design for a small portable touch screen, not a scaled-down phone UI. Keep essential controls visible, touch targets large, text legible, navigation shallow, artwork appropriately sized, and caches bounded. Avoid unnecessary animations and horizontal scrolling. Account for actual display density and insets; 480 × 640 physical pixels is not necessarily 480 × 640 dp.

## Audio requirements
Prioritize stable streaming at the best quality actually offered by the implementation and account. Upstream documentation reports up to 320 kbps Ogg Vorbis; verify against the checked-out code. This is not lossless audio. Do not bypass DRM, access controls, or service restrictions.

Inspect the current audio path, JNI bridge, Android output, buffers, and error handling. Stability and responsiveness take priority over speculative micro-optimizations.

## Technical constraints
- Android 9 corresponds to API 28.
- Lowering `minSdk` alone does not fix newer API calls or incompatible dependencies.
- `minSdk`, `targetSdk`, and `compileSdk` have different purposes.
- Do not infer CPU ABI from core count or marketing data.
- Rust targets and JNI libraries must match the device ABI.
- Gradle, AGP, Kotlin, Compose, JDK, and NDK versions must be compatible.
- A successful build does not prove physical-device compatibility.

## Success criteria
- APK installs and starts on the M308.
- Essential controls fit and work by touch.
- Online playback starts with an eligible account.
- Playback controls function.
- Sustained playback has no persistent glitches or stalls.
- UI remains responsive.
- RAM/CPU are measured where possible and regressions documented.
- Build steps are reproducible and require no committed secrets.
- Emulator and physical-device results are clearly distinguished.
- Untested device-specific behavior is explicitly identified.

## Risks and out of scope
The upstream may use APIs unavailable on Android 9; dependencies or manufacturer firmware may impose additional constraints. Third-party integration may break when Spotify changes its service. Unofficial clients may be subject to service terms and account risk; do not claim Spotify endorsement.

Do not create a new streaming client, reimplement the protocol without demonstrated need, add offline playback, bypass access controls, rewrite the architecture without evidence, or add complexity that does not improve the essential M308 experience.

## Overall direction
Audit first, fix actual compatibility blockers, then adapt the UI and optimize based on measurements. Validate on the physical player whenever possible.

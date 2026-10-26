# AGENTS.md — Development rules

## Mission
Adapt this fork of `qfertig/sidetrack` for the Oilsky M308: Android 9 (API 28), reported 480 × 640 touch display, 2 GB RAM, and an ABI that must be verified. Prioritize stable online playback at the best quality actually available through the integration and account. Offline playback is out of scope.

## Mandatory workflow
1. Inspect before editing: read `README.md`, `DESIGN.md`, Gradle files, `app/`, `native/`, submodules, and build scripts.
2. First report a concise audit of architecture, SDK levels, Gradle/AGP/Kotlin/Java versions, API 28 blockers, native ABI, and audio path.
3. Work incrementally and test after relevant changes. Preserve existing behavior and architecture unless evidence supports changing them.
4. Never invent build/test/device results. Distinguish source inspection, emulator testing, and physical-device testing.
5. Explain destructive or irreversible changes and obtain confirmation before proceeding.
6. For resource work, follow `.kilo/skills/android-resource-optimization/SKILL.md`.

## Initial audit
- Inspect `.gitmodules`; verify the librespot submodule and commit.
- Identify `minSdk`, `targetSdk`, `compileSdk`, Gradle Wrapper, Android Gradle Plugin, Kotlin, Java, Compose, and NDK versions.
- Locate JNI, Rust, audio output, and playback service.
- Identify APIs/dependencies incompatible with API 28.
- Confirm the device ABI rather than inferring it from marketing specifications.
- Review session/credential storage; never log secrets.
- Record baseline APK size, startup, memory, and CPU where possible.

## Android 9
Android 9 is API 28. Changing `minSdk` alone does not guarantee compatibility. Check AndroidX, Compose, image libraries, services, notifications, and runtime API calls. Keep `minSdk`, `targetSdk`, and `compileSdk` conceptually distinct. Do not add mandatory Google Play Services. Validate on the M308; a successful build is not proof of runtime compatibility.

## Small touch display
Design for 480 × 640 physical pixels while respecting density, insets, and dp. Prioritize artwork, title, artist, progress, previous, play/pause, and next. Use large touch targets, readable text, simple navigation, bounded image caches, few animations, and no essential controls clipped or requiring horizontal scrolling.

## Audio and performance
- Preserve librespot unless an incompatibility is demonstrated.
- Inspect the Rust/JNI/Android audio path for redundant PCM copies, allocations, blocking, and buffer issues.
- Keep network and decoding work off the UI thread.
- Avoid unbounded queues and unnecessary polling.
- Do not reduce audio buffers without testing underruns and latency.
- Use the best quality actually supported. Upstream documentation reports up to 320 kbps Ogg Vorbis; do not call that lossless.
- Do not bypass DRM, access controls, or Spotify restrictions.
- Online streaming is sufficient; do not add offline downloads or persistent audio caching.

## Security
Never log or commit passwords, tokens, cookies, keys, authentication payloads, or signing keystores. Use private app storage for session data. Identify the app as an unofficial third-party client; do not claim Spotify endorsement.

## Build quality
Use the repository Gradle Wrapper and compatible JDK/AGP/Kotlin versions. Use stable Rust unless the project requires otherwise. Preserve dependency versions initially and change only for a demonstrated reason. Run relevant Rust tests, Android tests, lint, debug build, and unsigned release build when available. Verify native ABIs in the APK.

## Acceptance criteria
1. APK installs and launches on Android 9/API 28.
2. Essential controls fit and work on the touch display.
3. An eligible account can start online playback.
4. Playback controls work as implemented.
5. Sustained playback has no persistent glitches or stalls.
6. UI remains responsive.
7. CPU/RAM are measured when possible and regressions explained.
8. Build instructions do not require committed secrets.
9. Emulator and physical-device results are distinguished.
10. Unverified M308 behavior is clearly stated.

## Suggested order
1. Audit and baseline.
2. Fix build blockers and API 28 incompatibilities.
3. Build for confirmed ABI; validate JNI/audio.
4. Adapt touch UI.
5. Optimize from measurements.
6. Test streaming, interruptions, resume, and sustained playback.
7. Document build, tests, limitations, and troubleshooting.

Do not rewrite the Spotify protocol, replace the streaming core without evidence, add offline playback, or remove upstream functionality without justification.

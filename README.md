<div align="center">

# Sidetrack

**A lightweight, GMS-free Spotify client for Android, powered by librespot and Jetpack Compose.**

[![Android](https://img.shields.io/badge/Android-9%2B%20%28API%2028%2B%29-3ddc84?style=flat-square&logo=android&logoColor=white)](https://developer.android.com/about/versions/pie)
[![Kotlin](https://img.shields.io/badge/Kotlin-2.1.0-7f52ff?style=flat-square&logo=kotlin&logoColor=white)](https://kotlinlang.org)
[![Rust](https://img.shields.io/badge/Rust-librespot-000000?style=flat-square&logo=rust&logoColor=white)](https://github.com/librespot-org/librespot)
[![License](https://img.shields.io/badge/License-GPL--3.0-blue?style=flat-square)](LICENSE)

[Features](#features) &nbsp;•&nbsp; [Install](#install) &nbsp;•&nbsp; [Build from source](#build-from-source) &nbsp;•&nbsp; [Architecture](#architecture) &nbsp;•&nbsp; [Limitations](#limitations)

</div>

Sidetrack runs the Spotify Connect stack locally on a small Android device: no Google Play Services, no web view, no official Spotify app required. The interface is a native Jetpack Compose UI designed around a 480 × 640 touch screen.

This repository is the [`mendeswm/sidetrack`](https://github.com/mendeswm/sidetrack) fork, retargeted at the **Oilsky M308** (Android 9 / API 28, 2 GB RAM). It builds on [Sidespot](https://github.com/jtaekman/sidespot) by [@jtaekman](https://github.com/jtaekman), which in turn builds on upstream [`qfertig/sidetrack`](https://github.com/qfertig/sidetrack).

## Screenshots

| | |
|---|---|
| ![Library](screenshots/library.png) | ![Now playing](screenshots/now_playing.png) |
| ![Liked songs](screenshots/liked_songs.png) | ![Search](screenshots/search.png) |
| ![Queue](screenshots/queue.png) | ![Sign in](screenshots/sign_in.png) |

More screens: `screenshots/new_episodes.png`, `screenshots/library_top.png`.

## Features

- **No Google Play Services** — runs on degoogled and minimal Android builds
- **Small-screen UI** — dark theme, compact typography, 48 dp touch targets, no horizontal scrolling for essential controls
- **Full playback** — play, pause, seek, skip, shuffle, repeat, queue management
- **Spotify Connect pairing** — pair with the official Spotify app over Wi-Fi via Zeroconf, or sign in on another device
- **Library browsing** — playlists, liked songs, saved albums, followed artists, saved podcasts
- **Library management** — save/remove albums and playlists, like tracks, create playlists
- **Search** — tracks, artists, albums, playlists, and podcasts
- **Artist pages** — followed artists, popular tracks, jump-to-artist
- **Podcast support** — saved shows, episode lists, playback, and a New Episodes screen
- **Discover** — track suggestions seeded from liked songs, with Play/Shuffle actions
- **Background playback** — foreground service with media notification and lock-screen controls
- **Dynamic theming** — album art colors tint the UI
- **Audio focus** — pauses for calls, ducks for notifications, resumes automatically
- **Play history** — local plus official Spotify history
- **E-ink display mode** — high-contrast monochrome UI for reflective screens
- **Settings** — audio quality (Low 96 / Normal 160 / High 320 kbps), normalization, gapless, autoplay, e-ink mode, key mapping, and audio cache
- **Key mapping** — remap physical soft keys per device

## Requirements

- A **Spotify Premium** account (free-tier accounts are not supported by librespot)
- Android 9+ (API 28+); this fork targets API 28
- On the M308: arm64-v8a

## Install

### From releases

Download an APK from the [Releases](https://github.com/mendeswm/sidetrack/releases) page and sideload it:

```sh
adb install sidetrack-v*.apk
```

### From source

See [Build from source](#build-from-source).

## Build from source

Verified configuration in this checkout:

| Component | Version |
|---|---|
| Android Gradle Plugin | 8.7.3 |
| Kotlin | 2.1.0 |
| Gradle | 8.11.1 |
| Java | 17 |
| compileSdk / targetSdk / minSdk | 34 / 30 / 28 |
| NDK | 26.1.10909125 |
| ABI | arm64-v8a |
| librespot submodule | [`jtaekman/librespot`](https://github.com/jtaekman/librespot) @ `cca88d6` |
| versionName / versionCode | 0.4.1 / 12 |

### Prerequisites (Ubuntu 26.04)

```sh
sudo apt update
sudo apt install \
  git curl unzip zip xz-utils file \
  build-essential pkg-config cmake ninja-build clang lld \
  openjdk-17-jdk adb
```

Then install Rust and the Android targets:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install stable
rustup default stable
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-ndk
```

Install the Android SDK (API 34, Build-Tools 34.0.0) and NDK `26.1.10909125` using [Android command-line tools](https://developer.android.com/studio#command-tools) or Android Studio. See [`DEPENDENCIES-UBUNTU-26.04.md`](DEPENDENCIES-UBUNTU-26.04.md) for details.

### Clone

```sh
git clone --recurse-submodules https://github.com/mendeswm/sidetrack.git
cd sidetrack
```

### Build and install

The Gradle build compiles the Rust native library via cargo-ndk before assembling the APK.

```sh
# Debug build + install to a connected device
./gradlew installDebug

# Debug APK only
./gradlew assembleDebug

# Release APK (requires release-keystore.properties)
./gradlew assembleRelease
```

If the Android SDK is not at the default location, point the build at it explicitly:

```sh
ANDROID_HOME=/path/to/android-sdk \
ANDROID_NDK_HOME=/path/to/android-sdk/ndk/26.1.10909125 \
./gradlew assembleDebug
```

Output: `app/build/outputs/apk/debug/app-arm64-v8a-debug.apk` (debug) or `app/build/outputs/apk/release/app-arm64-v8a-release.apk` (release).

### Release signing

Create `release-keystore.properties` in the project root (git-ignored):

```properties
storeFile=path/to/your/keystore.jks
storePassword=your-store-password
keyAlias=your-key-alias
keyPassword=your-key-password
```

> [!WARNING]
> Never commit the keystore or `release-keystore.properties`. Build instructions in this repository do not require any committed secrets.

## Architecture

```text
┌─────────────────────────────────┐
│  Jetpack Compose UI             │  Kotlin / MVVM
│  Library · Search · Player      │
├─────────────────────────────────┤
│  JNI Bridge                     │  Kotlin external fun  ⇄  Rust extern "C"
├─────────────────────────────────┤
│  librespot (Rust)               │  session · playback · metadata · decoding
│  + Tokio async runtime          │
└─────────────────────────────────┘
```

- **Native core** — librespot handles the Spotify protocol, authentication, audio streaming, and decryption.
- **JNI bridge** — JSON payloads between Kotlin and Rust (`app/.../bridge/NativeBridge.kt` ↔ `native/src/bridge.rs`).
- **Audio pipeline** — librespot decodes OGG Vorbis to 24-bit PCM, delivered through a JNI callback to Android `AudioTrack`. Since API 28 does not expose a 24-bit `AudioTrack` encoding, the sink downconverts to S16LE at 44.1 kHz stereo. See [wiki/audio/24bit-pcm-investigation.md](wiki/audio/24bit-pcm-investigation.md).
- **UI** — Jetpack Compose with Navigation, Material 3, and Coil for album art.
- **Playback service** — foreground `Service` with a `MediaSession` for background playback and notification controls.

## Audio behavior

- **Quality** — Low (96 kbps), Normal (160 kbps), or High (320 kbps) OGG Vorbis. Spotify's lossless tier uses DRM that librespot cannot and will not circumvent.
- **Volume** — the librespot player volume is fixed at 100%; hardware volume keys control the Android system volume like any other app.
- **Audio focus** — playback pauses for calls and resumes afterward.
- **Audio cache** — optional internal-storage cache (enabled by default, 1 GB limit). Caching to external SD was removed due to intermittent distortion on the M308.

## Limitations

> [!IMPORTANT]
> Sidetrack is an unofficial third-party client. Spotify has not sanctioned it, and librespot is a reverse-engineered implementation. Use at your own risk.

- **Spotify Premium required** — free-tier accounts are not supported.
- **ARM only** — the native library is built for arm64-v8a (the `armeabi-v7a` library is built into `jniLibs/` by cargo-ndk but not packaged).
- **No lossless / HiFi** — maximum quality is 320 kbps OGG Vorbis.
- **Spotify Connect is receive-only** — pairs via Zeroconf so the official Spotify app can hand off playback; it does not advertise or control other Connect targets.
- **No crossfade**, **no offline downloads** — streaming only.
- **M308 verification** — the app builds and runs on the M308, but sustained playback, display insets, and thermal behavior have not been exhaustively verified. See [wiki/project/sidetrack-m308-adaptation.md](wiki/project/sidetrack-m308-adaptation.md).

## M308 notes

- Target: Android 9 (API 28), 480 × 640 touch display, ~2 GB RAM, arm64-v8a
- Touch targets ≥ 48 dp; compact typography; no horizontal scrolling for essentials
- Coil memory cache at 10% of app memory with a 20 MB disk cache; crossfade disabled
- No on-screen back arrow — the M308 has physical navigation keys
- Volume keys adjust system volume; the internal librespot volume stays at 100%

## Project knowledge

| Path | Contents |
|---|---|
| [`AGENTS.md`](AGENTS.md) | Development rules and M308 constraints |
| [`CONTEXT.md`](CONTEXT.md) | Project context and success criteria |
| [`DESIGN.md`](DESIGN.md) | Design notes |
| [`wiki/`](wiki/) | Synthesized project knowledge |
| [`raw/`](raw/) | Immutable source and reference material |

## Disclaimer

Sidetrack is not affiliated with, endorsed by, or sponsored by Spotify. It uses [librespot](https://github.com/librespot-org/librespot), a reverse-engineered implementation of the Spotify protocol. A Spotify Premium subscription is required. Use at your own risk.
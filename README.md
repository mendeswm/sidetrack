# Sidetrack

A lightweight, GMS-free Spotify client for Android, built on
[librespot](https://github.com/librespot-org/librespot) (Rust) with a
[Jetpack Compose](https://developer.android.com/jetpack/compose) UI. This fork
targets the Oilsky M308 (Android 9 / API 28, 480x640 touch screen, ~2 GB RAM)
as its primary device while remaining usable on other small-screen Android
devices.

## Credit

Sidetrack is a fork of [Sidespot](https://github.com/jtaekman/sidespot) by
[@jtaekman](https://github.com/jtaekman), originally built for the
[Sidephone SP-01](https://sidephone.com). Sidetrack retargets that same
librespot/Compose foundation at small-screen Android devices. None of this
would exist without jtaekman's original work on the JNI bridge, the Compose UI,
and the Spotify integration itself.

This repository is the `mendeswm/sidetrack` fork, which diverges from upstream
`qfertig/sidetrack` for the Oilsky M308 adaptation.

## Screenshots

See `screenshots/` for sign-in, library, liked songs, search, queue, and
now-playing screens. Original upstream screenshots are retained.

## Features

- **No Google Play Services required** - runs on degoogled and minimal Android devices
- **Small-screen friendly** - dark theme, compact typography, 480x640 targets
- **Touch UI** - touch targets >= 48dp; no essential controls clipped or horizontally scrolled
- **Full playback** - play, pause, seek, skip, shuffle, repeat, queue management
- **Spotify Connect pairing** - pairs with the official Spotify app over Wi-Fi via Zeroconf
- **Library browsing** - playlists, liked songs, saved albums, followed artists, saved podcasts
- **Library management** - save/remove albums, playlists, podcasts; like tracks; create playlists
- **Search** - tracks, artists, albums, playlists, podcasts
- **Artist pages** - followed artists, popular tracks, jump-to-artist
- **Podcast support** - saved shows, episode lists, playback, New Episodes screen
- **Background playback** - foreground service with media notification controls
- **Dynamic theming** - album art colors tint the UI
- **Hardware volume keys** - physical button integration
- **Audio focus** - pauses for calls, ducks for notifications, resumes automatically
- **Play history** - local + official Spotify history view
- **E-ink display mode** - high-contrast monochrome UI
- **Settings** - audio quality (160/320 kbps), normalization, gapless, autoplay, e-ink, key mapping, audio cache
- **Key mapping** - remap physical soft keys per device
- **Audio cache** - optional internal-storage cache (default off, 1 GB limit)

## Requirements

- Spotify Premium account (free-tier accounts are not supported by librespot)
- Android 9+ (API 28+); this fork targets API 28

## Install

### From Releases

Download an APK from [Releases](https://github.com/mendeswm/sidetrack/releases) and
sideload:

```sh
adb install sidetrack-v*.apk
```

### From Source

See "Build from Source" below.

## Build from Source

This fork targets the Oilsky M308. Build configuration (verified from
`app/build.gradle.kts` and `gradle/wrapper/gradle-wrapper.properties`):

- AGP 8.7.3, Kotlin 2.1.0, Gradle 8.11.1, Java 17
- compileSdk 34, targetSdk 30, minSdk 28
- NDK 26.1.10909125
- ABIs: arm64-v8a only (universal APK disabled; `armeabi-v7a` .so is also
  built into `jniLibs/` by cargo-ndk but not packaged)
- librespot submodule: `jtaekman/librespot` at `cca88d6` (heads/master)
- versionName 0.4.1 (versionCode 12)

### Prerequisites (Ubuntu 26.04)

- Java 17 (`openjdk-17-jdk`)
- Android SDK API 34 with Build-Tools 34.0.0
- Android NDK 26.1.10909125
- Rust toolchain via rustup with `aarch64-linux-android` and `armv7-linux-androideabi`
- cargo-ndk (`cargo install cargo-ndk`)
- build-essential, cmake, ninja-build, clang, lld

See `wiki/dev-environment/ubuntu-26.04-dependencies.md` for environment variables.

### Setup

```sh
# Install Rust Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi

# Clone with submodules (librespot)
git clone --recurse-submodules https://github.com/mendeswm/sidetrack.git
cd sidetrack
```

### Build & Install

The Gradle build compiles the Rust native library via cargo-ndk before assembling.

```sh
# Debug build + install to connected device
export JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64
./gradlew installDebug

# Release APK (requires release-keystore.properties)
./gradlew assembleRelease
```

Release APK output: `app/build/outputs/apk/release/app-release.apk`.

### Release Signing

Create `release-keystore.properties` in the project root:

```properties
storeFile=path/to/your/keystore.jks
storePassword=your-store-password
keyAlias=your-key-alias
keyPassword=your-key-password
```

## Architecture

```
┌─────────────────────────────┐
│   Jetpack Compose UI        │  Kotlin / MVVM
│   (Library, Search, Player) │
├─────────────────────────────┤
│   JNI Bridge                │  Kotlin external fun <-> Rust extern "C"
├─────────────────────────────┤
│   librespot (Rust)          │  Session, playback, metadata, audio decoding
│   + Tokio async runtime     │
└─────────────────────────────┘
```

- **Native core**: librespot handles Spotify protocol, authentication, audio streaming, decryption
- **JNI bridge**: JSON payloads between Kotlin and Rust
- **Audio pipeline**: librespot decodes OGG Vorbis to PCM, delivered via JNI callback to Android AudioTrack (44.1 kHz stereo S16LE)
- **UI**: Jetpack Compose with Navigation, Material3, Coil for album art
- **Audio quality**: Normal 160 kbps or High 320 kbps OGG Vorbis (no lossless)

## Current Limitations

- **Spotify Premium required** - free-tier accounts are not supported by librespot
- **M308 adaptation in progress** - ABI, display density, insets, and sustained-playback
  behavior on the actual Oilsky M308 are not yet fully verified; see
  `wiki/project/sidetrack-m308-adaptation.md`
- **ARM only** - native library built for arm64-v8a (and armeabi-v7a in jniLibs)
- **No lossless/HiFi** - max quality is 320 kbps OGG Vorbis; Spotify's lossless tier
  uses DRM that librespot cannot and will not circumvent
- **Spotify Connect (receive-only)** - pairs via Zeroconf so the official Spotify app can
  hand off playback; does not advertise/control other Connect targets
- **No crossfade** - not supported
- **No offline mode** - streaming only; optional internal audio cache (default off)
- **Account risk** - Spotify has not sanctioned third-party clients. Use at your own risk
- **Audio cache note** - cache lives in app internal storage (`cache/audio_cache`),
  default 1 GB limit; external-SD caching was removed due to intermittent distorted
  playback on the M308
- **allowHardware(true) on album art** - applied to the sharp album-art bitmap in
  NowPlayingScreen; unverified on device (see wiki adaptation doc)

## M308 / small-screen notes

- Target: Android 9 (API 28), 480x640 touch, ~2 GB RAM
- Touch targets >= 48dp; compact typography; no horizontal scrolling for essentials
- Coil memory cache 10%, 20 MB disk cache; crossfade disabled
- See `AGENTS.md`, `CONTEXT.md`, and `wiki/project/sidetrack-m308-adaptation.md`

## Disclaimer

Sidetrack is not affiliated with or endorsed by Spotify. It uses a
reverse-engineered protocol implementation ([librespot](https://github.com/librespot-org/librespot)).
Use at your own risk. A Spotify Premium subscription is required.

## License

GPLv3. See [LICENSE](LICENSE) for details.

## Project knowledge

- `AGENTS.md` - development rules and M308 constraints
- `CONTEXT.md` - project context and success criteria
- `DESIGN.md` - design notes
- `wiki/` - persistent synthesized project knowledge
- `raw/` - immutable source/reference material

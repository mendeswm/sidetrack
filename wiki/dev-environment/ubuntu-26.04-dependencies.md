# Ubuntu 26.04 Dependencies

> Sources: DEPENDENCIAS-UBUNTU-26.04.md, 2026-10-26
> Raw: [ubuntu dependencies](../../raw/dev-environment/2026-10-26-ubuntu-dependencies.md)
> Updated: 2026-10-26

## Overview

Development environment for building the Sidetrack M308 fork on Ubuntu 26.04 LTS.

## Required Tools

- Java 17 (openjdk-17-jdk)
- Android SDK API 34 with Build-Tools 34.0.0
- Android NDK 26.1.10909125
- Rust toolchain via rustup with aarch64-linux-android and armv7-linux-androideabi targets
- cargo-ndk for cross-compilation
- build-essential, cmake, ninja-build, clang, lld

## Environment Variables

- ANDROID_HOME / ANDROID_SDK_ROOT: SDK location
- ANDROID_NDK_HOME: NDK location
- JAVA_HOME: Java 17 installation

## Build Commands

```sh
./gradlew assembleDebug
```

Native library is built automatically via cargo-ndk before APK assembly.

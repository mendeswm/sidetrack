# Development dependencies — Ubuntu 26.04 LTS

Confirm exact requirements against the checked-out Gradle files and native build configuration before pinning tool versions.

## Ubuntu packages

```bash
sudo apt update
sudo apt install \
  git git-lfs curl wget unzip zip xz-utils file \
  build-essential pkg-config cmake ninja-build clang lld \
  openjdk-17-jdk \
  adb
```

Use the JDK required by the repository's Gradle Wrapper and Android Gradle Plugin. The upstream documents Java 17, but verify the current checkout. CMake/Ninja/Clang/LLD are useful for native dependencies. Install additional `*-dev` packages only when build diagnostics require them. `git-lfs` is optional.

## Rust

Install Rust with rustup: https://rustup.rs/. Avoid mixing APT Rust packages with rustup unless needed.

```bash
rustup toolchain install stable
rustup default stable
rustup component add rustfmt clippy
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-ndk
```

`aarch64-linux-android` targets Android ARM64 (`arm64-v8a`); `armv7-linux-androideabi` targets 32-bit ARM (`armeabi-v7a`). Confirm the M308 ABI first and install only the required targets.

## Android SDK and NDK

Install Android SDK Command-line Tools, Platform Tools, Build Tools, the required SDK Platform, and a compatible NDK using official Android tools. Android Studio is optional; SDK components can also be managed with `sdkmanager`.

The upstream documentation mentions Android SDK API 34 and NDK `27.0.12077973`. Treat these as initial references, not guarantees for the current checkout. Confirm the actual build files and keep versions reproducible. Multiple NDK versions can coexist.

Set `ANDROID_SDK_ROOT` (or `ANDROID_HOME` if the project requires it) and `ANDROID_NDK_HOME` to the selected installations. Do not commit personal absolute paths.

## Gradle and Kotlin

Use `./gradlew` and the version in `gradle/wrapper/gradle-wrapper.properties`; a global Gradle installation is unnecessary. Keep AGP, Kotlin, Compose, and JDK versions compatible. Do not update all dependencies at once to address Android 9 compatibility; identify the actual blocker first.

## Optional tools

```bash
sudo apt install gdb lldb jq ripgrep
```

GDB/LLDB help diagnose native crashes; `jq` inspects JSON; `ripgrep` (`rg`) searches source. Android Studio is useful for Logcat and profiling but is optional.

## Verify the environment

Verify tool versions and do not consider the setup validated until a debug build succeeds. If ADB is supported by the device, connect it over USB and query Android version and ABI. Avoid publishing identifying device properties unnecessarily.

## References
- Sidetrack: https://github.com/qfertig/sidetrack
- Rust on Ubuntu: https://ubuntu.com/developers/docs/howto/rust-setup/
- Rust Android targets: https://doc.rust-lang.org/rustc/platform-support/android.html
- Android NDK: https://developer.android.com/studio/projects/install-ndk
- Rust installation: https://rustup.rs/
- Android performance measurement: https://developer.android.com/topic/performance/measuring-performance
- Android memory management: https://developer.android.com/topic/performance/memory

Ubuntu 26.04 provides recent tools, but the newest version is not necessarily compatible with an existing Android project. Prefer versions required by the project's Gradle Wrapper, AGP, and NDK configuration.

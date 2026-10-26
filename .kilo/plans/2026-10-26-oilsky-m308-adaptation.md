# Plan: Adapt Sidetrack for Oilsky M308 (Android 9 / API 28)

**Date:** 2026-10-26  
**Goal:** Make Sidetrack installable and usable on Oilsky M308 (Android 9/API 28, 480×640 touch, ~2GB RAM). Preserve librespot, focus on stable online playback, fit UI to small screen, avoid offline/DRM bypass.

## Current state summary (verified)
- Repo: fork of qfertig/sidetrack, branch `main`, clean. Submodule `native/librespot-local` at `cca88d6...` (librespot dev fork).
- Build: AGP 8.7.3, Kotlin 2.1.0, Gradle 8.11.1, Java 17. `app/build.gradle.kts`: `compileSdk 34`, `targetSdk 30`, `minSdk 30`, `ndkVersion 26.1.10909125`, ABI splits for `arm64-v8a` and `armeabi-v7a` (universal APK enabled). Native built via `cargo ndk ... build --release` into `app/src/main/jniLibs/`.
- Native: `native/Cargo.toml` uses `librespot-*` from local submodule with `rustls-tls-webpki-roots`, `jni 0.21`, tokio, serde, etc. Release profile optimized for size (`opt-level="s"`, LTO, strip).
- App: Jetpack Compose, MVVM, JNI bridge (`bridge/NativeBridge.kt`, `bridge/Models.kt`), playback service, auth, UI screens for library/search/now playing/queue/etc. Android 11+ targeted (minSdk 30).
- M308 target: Android 9/API 28, 480×640. Current minSdk 30 is too high; many deps/SDKs may assume API 30+. ABI unknown.

## Blockers to fix for API 28
1. **minSdk/targetSdk**: Lower to 28 after audit. `targetSdk 30` is fine if APIs used exist on 28; need to check for post-28 APIs (notifications, foreground services, permissions, etc.). `compileSdk 34` is usually okay if using compat libs.
2. **Dependencies**: Compose BOM 2024.12.01, Activity 1.9.3, Lifecycle 2.8.7, Navigation 2.8.5, Coroutines 1.9.0, Serialization 1.7.3, Coil 2.7.0, Palette 1.0.0 - verify all compatible with API 28/minSdk 28. Some may require higher minSdk.
3. **Android APIs**: Check for APIs added >28 (e.g., `PendingIntent` mutability changes, notification channels behavior, foreground service types, media notifications, `setForegroundServiceBehavior`, etc.). Also `windowSoftInputMode`, edge-to-edge considerations.
4. **NDK/Tooling**: NDK 26.1.10909125 is fine; cargo-ndk works. Ensure Rust targets installed for device ABI.
5. **ABI**: Must determine actual ABI of M308 (likely `arm64-v8a` or `armeabi-v7a`). Build for confirmed ABI first.

## Plan (incremental, testable)

### Phase 0: Discovery/verification (done)
- [x] Read AGENTS.md, CONTEXT.md, DEPENDENCIAS-UBUNTU-26.04.md, DESIGN.md, README.md
- [x] Git state clean, submodule at cca88d6
- [x] Build files inspected, app structure mapped
- [x] Wiki initialized

### Phase 1: API 28 compatibility audit
- [ ] **Build files**: Review `app/build.gradle.kts` for any API-gated code. Check if `minSdk` can be lowered to 28 safely.
- [ ] **Source grep**: Search for APIs introduced after API 28 (e.g., `Context.getSystemService` usages, notification channels, `startForegroundService`, `FOREGROUND_SERVICE_*` types, `MediaStyle`, `setOngoing`, etc.). Also check for `Build.VERSION_CODES.*` checks.
- [ ] **Dependencies**: Check each dependency's minSdk requirements. May need to downgrade or add compat.
- [ ] **Manifest**: Review permissions - `FOREGROUND_SERVICE_MEDIA_PLAYBACK` is API 34+, but declared with `tools:targetApi`? Or just declared. On API 28, foreground service media playback type handling differs; need to ensure compatibility.
- [ ] **Test compile**: Try to compile with `minSdk 28` (if possible) or run lint to catch issues.

### Phase 2: Lower minSdk and fix blockers
- [ ] **Change minSdk**: Set `minSdk = 28` in `app/build.gradle.kts`. Keep `targetSdk 30` initially (or lower if needed). `compileSdk 34` is okay.
- [ ] **Fix API issues**: Add version checks where needed. For foreground service, handle API differences. For notifications, ensure channel creation works on API 28.
- [ ] **Dependencies**: If any dependency requires higher minSdk, either find compatible version or remove/replace. Check Compose BOM compatibility.
- [ ] **Proguard/R8**: Keep existing rules; may need adjustments.
- [ ] **NDK**: NDK 26 should work for API 28.

### Phase 3: ABI confirmation and native build
- [ ] **Determine ABI**: If device available via ADB, run `adb shell getprop ro.product.cpu.abi` and `ro.product.cpu.abilist`. If not, start with `arm64-v8a` (common for modern ARM64) but be ready to try `armeabi-v7a`.
- [ ] **Build native**: Ensure Rust toolchain has targets. Run `./gradlew buildNativeRelease` or let Gradle do it. Verify `.so` files in `app/src/main/jniLibs/`.
- [ ] **JNI compatibility**: JNI should be fine across APIs.

### Phase 4: UI adaptation for 480×640 touch
- [ ] **Audit layouts**: Review key screens (NowPlaying, Library, Search, Queue) for fixed sizes, large paddings, overflow. Use dp-aware layouts.
- [ ] **Touch targets**: Ensure minimum 48dp. Current codebase is D-pad focused but needs touch support.
- [ ] **Screen fit**: Test layouts on 480×640 (emulator or device). Adjust spacing, text sizes, image sizes.
- [ ] **Images**: Ensure Coil loads appropriately sized images, cache limited.
- [ ] **Navigation**: Keep simple, few levels.

### Phase 5: Performance/memory optimizations
- [ ] **Image loading**: Review Coil usage, add size constraints.
- [ ] **Recompositions**: Check for unnecessary recompositions.
- [ ] **Lists**: Use LazyColumn efficiently.
- [ ] **Background work**: Ensure no heavy work on main thread.

### Phase 6: Testing and validation
- [ ] **Build**: `./gradlew assembleDebug` and/or `assembleRelease` (if keystore configured)
- [ ] **Install**: Test on emulator API 28 or device
- [ ] **Playback**: Test login, play/pause, skip, seek, volume
- [ ] **Stability**: Long playback test, screen navigation during playback
- [ ] **Memory/CPU**: Basic profiling if possible

## Risks
- Some Compose/AndroidX versions may not support API 28 - may need to downgrade BOM/dependencies
- Foreground service behavior differs on API 28 vs 30+
- librespot submodule is dev branch - may have issues
- ABI unknown - need device access

## Next immediate actions
1. Try to compile with current setup to see baseline errors
2. Lower minSdk to 28 and fix compilation errors iteratively
3. Check dependencies compatibility
4. Build native library if possible

## Files to modify
- `app/build.gradle.kts` (minSdk, possibly dependencies)
- Source files with API-gated issues
- Possibly `AndroidManifest.xml`
- Gradle files if needed

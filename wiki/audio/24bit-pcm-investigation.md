# 24-bit PCM Audio Output Investigation

> Sources: Sidetrack source investigation, 2026-11-12
> Raw: [2026-11-12-24bit-pcm-investigation](../../raw/audio/2026-11-12-24bit-pcm-investigation.md)
> Updated: 2026-11-12

## Overview

Investigated whether Sidetrack can deliver 24-bit PCM audio to the Oilsky M308 DAC without unnecessary precision loss. The conclusion: native 24-bit PCM output via the public Android AudioTrack API on API 28 is **NOT FEASIBLE** — `ENCODING_PCM_24BIT` does not exist on API 28 (added in API 29). The recommended approach is to keep librespot processing at S24 internally and downconvert to S16 for AudioTrack output.

## Device and environment

| Property | Value | Source |
|----------|-------|--------|
| Model | Oilsky M308 | `getprop ro.product.model` |
| Hardware | mt6755 | `getprop ro.hardware` |
| Board | k50v1_64_bsp | `getprop ro.product.board` |
| CPU ABI | arm64-v8a | `getprop ro.product.cpu.abi` |
| Android SDK | 28 | `getprop ro.build.version.sdk` |
| ALSA card | mtsndcard | `/proc/asound/cards` |

## Audio HAL capabilities

Verified via `dumpsys media.audio_flinger`:

| Output thread | Sample rate | HAL format | Processing format |
|---------------|-------------|------------|-------------------|
| Primary (AudioOut_D) | 48000 Hz | AUDIO_FORMAT_PCM_32_BIT | AUDIO_FORMAT_PCM_32_BIT |
| 2D (AudioOut_2D) | 44100 Hz | AUDIO_FORMAT_PCM_16_BIT | AUDIO_FORMAT_PCM_16_BIT |

Verified via `dumpsys media.audio_policy` — all output devices support PCM_16_BIT and PCM_32_BIT at 44100/48000 Hz.

Audio policy configuration (`/system/etc/audio_policy_configuration.xml`) — primary output mixPort uses `AUDIO_FORMAT_PCM_16_BIT` at 48000 Hz stereo.

## Android API limitations

`ENCODING_PCM_24BIT` does not exist on API 28 (added in API 29). Attempting it produces:

```
java.lang.IllegalArgumentException: Invalid encoding 22
	at android.media.AudioFormat$Builder.setEncoding(AudioFormat.java:862)
	at com.sidetrack.audio.AudioCallback.createAudioTrack(AudioCallback.kt:53)
```

Confirmed via `adb logcat` crash trace. `ENCODING_PCM_32BIT` is also not exposed publicly on API 28.

## Librespot format support

Verified via `native/librespot-local/playback/src/config.rs` — `AudioFormat` enum supports `F64`, `F32`, `S32`, `S24`, `S24_3`, `S16`.

Verified via `native/librespot-local/playback/src/convert.rs` — conversion methods `f64_to_s16()`, `f64_to_s24()`, `f64_to_s24_3()`, `f64_to_s32()`, `f64_to_f32()` all available. Dithering: `TriangularDitherer` default for S16/S24/S24_3.

## Current pipeline state

Verified via source inspection:

| Stage | File | Line | Value |
|-------|------|------|-------|
| Librespot output format | `native/src/player.rs` | 137 | `AudioFormat::S24` |
| Sink conversion | `native/src/audio_sink.rs` | 70-75 | S24→S16 downconversion |
| AudioTrack encoding | `app/src/main/java/com/sidetrack/audio/AudioCallback.kt` | 42 | `ENCODING_PCM_16BIT` |

## Conclusion

**Native 24-bit PCM output via public AudioTrack API on API 28: NOT FEASIBLE.**

Direct ALSA access is theoretically possible but bypasses Android audio focus/mixing and introduces significant stability risks without verified benefit on this device.

**Recommended approach:** Keep librespot at S24 internal processing, downconvert S24→S16 for AudioTrack output on API 28. This provides 24-bit processing precision (useful for volume control and normalisation headroom) without breaking API 28 compatibility.

## See Also

- [librespot CLI Options](../librespot/librespot-cli-options.md)
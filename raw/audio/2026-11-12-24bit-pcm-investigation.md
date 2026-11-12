# 24-bit PCM Audio Output Investigation

> Source: Investigation on Oilsky M308 (API 28) device connected via adb
> Collected: 2026-11-12
> Published: Unknown

{Original content below. Preserve the source text faithfully. Clean up formatting noise (extra whitespace, broken HTML artifacts, navigation chrome), but do not rewrite opinions or alter meaning.}

## Investigation Summary

Investigated whether Sidetrack can deliver 24-bit PCM audio to the Oilsky M308 DAC without unnecessary precision loss.

### Crash reproduced

Attempting `AudioFormat.ENCODING_PCM_24BIT` on API 28 causes:

```
FATAL EXCEPTION: Thread-19
Process: com.sidetrack.app, PID: 6730
java.lang.IllegalArgumentException: Invalid encoding 22
	at android.media.AudioFormat$Builder.setEncoding(AudioFormat.java:862)
	at com.sidetrack.audio.AudioCallback.createAudioTrack(AudioCallback.kt:53)
	at com.sidetrack.audio.AudioCallback.onAudioData(AudioCallback.kt:22)
```

Encoding value 22 (`ENCODING_PCM_32BIT` / `ENCODING_PCM_24BIT`) does not exist on API 28.

### Device properties (verified via adb)

- Model: Oilsky M308
- Manufacturer: alps
- Board: k50v1_64_bsp
- Hardware: mt6755
- CPU ABI: arm64-v8a
- Android SDK: 28

### Audio HAL capabilities (verified via `dumpsys media.audio_flinger`)

Primary output thread:
- Sample rate: 48000 Hz
- HAL format: 0x3 (AUDIO_FORMAT_PCM_32_BIT)
- Channel count: 2

2D output thread:
- Sample rate: 44100 Hz
- HAL format: 0x1 (AUDIO_FORMAT_PCM_16_BIT)
- Channel count: 2

### Audio policy capabilities (verified via `dumpsys media.audio_policy`)

All output devices (Speaker, Earpiece, Wired Headset, Wired Headphone):
- AUDIO_FORMAT_PCM_16_BIT at 44100, 48000 Hz
- AUDIO_FORMAT_PCM_32_BIT at 44100, 48000 Hz

Primary output mixPort (`/system/etc/audio_policy_configuration.xml`):
- format: AUDIO_FORMAT_PCM_16_BIT
- samplingRates: 48000
- channelMasks: AUDIO_CHANNEL_OUT_STEREO

Deep buffer mixPort:
- format: AUDIO_FORMAT_PCM_16_BIT
- samplingRates: 48000
- channelMasks: AUDIO_CHANNEL_OUT_STEREO

### ALSA

ALSA card `mtsndcard` (mt-snd-card) exists at `/dev/snd/`. `pcmC0D0p` present but closed at time of query; hw_params/format not readable while idle.

### Librespot format support (verified via source inspection)

`native/librespot-local/playback/src/config.rs`:
- AudioFormat enum: F64, F32, S32, S24, S24_3, S16 (default S16)
- All supported via FromStr

`native/librespot-local/playback/src/convert.rs`:
- f64_to_s16(), f64_to_s24(), f64_to_s24_3(), f64_to_s32(), f64_to_f32() all available
- Dithering: TriangularDitherer default for S16/S24/S24_3

### Current pipeline state (verified via source)

- `native/src/player.rs:137`: `let format = AudioFormat::S24;` — librespot processes at S24
- `native/src/audio_sink.rs:70-75`: converts S24→S16 (`s >> 8`) before JNI write
- `app/src/main/java/com/sidetrack/audio/AudioCallback.kt:42`: `ENCODING_PCM_16BIT` — AudioTrack uses S16LE on API 28

### Conclusion

Native 24-bit PCM output via public AudioTrack API on API 28 is NOT FEASIBLE — `ENCODING_PCM_24BIT` was added in API 29 and does not exist on API 28. Direct ALSA access is theoretically possible but introduces significant risks (bypasses audio focus, mixing, permissions) without verified benefit on this device.

Recommended approach: Keep librespot at S24 internal processing, downconvert S24→S16 for AudioTrack output on API 28. This provides 24-bit processing precision (useful for volume control and normalisation headroom) without breaking API 28 compatibility.
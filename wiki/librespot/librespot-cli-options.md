# librespot CLI Options

> Sources: librespot-org/librespot Wiki, 2026-02-04
> Raw: [2026-11-12-options](../../raw/librespot/2026-11-12-options.md)
> Updated: 2026-11-12

## Overview

The `librespot` CLI provides a comprehensive set of command-line options for connecting to Spotify and configuring audio output. Key options include cache configuration for audio data and credentials, bitrate selection, device naming, audio format, mixer settings, dithering, and authentication methods (OAuth or access token).

## Core Options

| Option | Short | Long | Type | Description |
|--------|-------|------|------|-------------|
| `--help` | `-h` | | Flag | Print list of options and flags |
| `--cache` | `-c` | | PATH | Path to directory where audio files are cached |
| `--system-cache` | | | PATH | Path for system files (credentials, volume) - may differ from cache |
| `--cache-size-limit` | | | SIZE | Maximum cache size (supports K, M, G suffixes) |
| `--disable-audio-cache` | | | | Disable audio data caching |
| `--disable-credential-cache` | | | | Disable credential caching (not advised with OAuth) |
| `--quiet` | `-q` | | Flag | Log only warnings and errors |
| `--name` | `-n` | | NAME | Device name for Spotify Connect |
| `--device-type` | | | TYPE | Device type display (default: `speaker`) |
| `--bitrate` | `-b` | | BITRATE | Audio bitrate: 96, 160, or 320 kbps (default: 160) |
| `--onevent` | | | PROGRAM | Script to run when librespot events trigger |
| `--emit-sink-events` | | | Flag | Run `--onevent` script before/after sink opens |
| `--version` | `-V` | | Flag | Display version string |
| `--verbose` | `-v` | | Flag | Enable verbose output |

## Authentication Options

### OAuth
- `--enable-oauth`: Interactive OAuth sign-in (requires web browser)
- `--oauth-port`: OAuth redirect port (default 5588, use 0 for headless)

### Access Token
- `--access-token`: Spotify token with 'streaming' scope (use `--cache`/`--system-cache` for persistence)

### Discovery
- Skip authentication via Spotify Connect - select librespot instance in Spotify client

## Audio Configuration

### Output Format
- `--format`: `F64`, `F32`, `S32`, `S24`, `S24_3`, `S16` (default: `S16`)

### Mixer
- `--mixer`: `softvol` (default) or `alsa`
- `--alsa-mixer-control`: ALSA control (default: `PCM`)
- `--alsa-mixer-device`: ALSA device (default: `default`)
- `--alsa-mixer-index`: Mixer card index (default: 0)
- `--initial-volume`: Initial volume 0-100% (default: 50)

### Volume Control
- `--volume-ctrl`: `cubic`, `fixed`, `linear`, `log` (default: `log`)

## Network & Discovery

- `--proxy`: HTTP proxy (`http://ip:port`)
- `--ap-port`: Spotify API port (80, 443, 4070)
- `--disable-discovery`: Disable zeroconf discovery
- `--zeroconf-backend`: `avahi`, `dns-sd`, `libmdns` (requires compiled feature flags)
- `--zeroconf-port`: Zeroconf server port (1-65535, may need root)
- `--zeroconf-interface`: Bound interface IPs (comma-separated)

## Volume Normalization

- `--enable-volume-normalisation`: Enable ReplayGain
- `--normalisation-method`: `basic` or `dynamic` (default: `dynamic`)
- `--normalisation-gain-type`: `track`, `album`, `auto` (default: `auto`)
- `--normalisation-pregain`: Pregain in dB (default: 0)
- `--normalisation-threshold`: Clipping threshold dBFS (default: -2.0)
- `--normalisation-attack`: Attack time ms (default: 5)
- `--normalisation-release`: Release time ms (default: 100)
- `--normalisation-knee`: Limiter knee steepness (default: 1.0)

## Dithering

- `--dither`: `none`, `gpdf`, `tpdf`, `tpdf_hp` (default varies by format)
- Triangle dithering default for S16, S24, S24_3 formats

## Local Files

- `-l` / `--local-file-dir`: Directory to search for local files (multiple allowed)
- Supports recursive directory search
- Requires matching audio tags/metadata with Spotify tracks
- Sample rate must be 44,100 Hz

## Autoplay & Gapless

- `--autoplay`: autoplay similar songs when music ends
- `--disable-gapless`: disable gapless playback (forces sink close between tracks)
- `--passthrough`: pass raw stream to output (pipe/subprocess backends only)

## Environment Variables

All CLI options can be passed as environment variables using `LIBRESPOT_` prefix:
- `--bitrate` → `LIBRESPOT_BITRATE`
- `--cache-size-limit` → `LIBRESPOT_CACHE_SIZE_LIMIT`
- etc.

## Removed Options

- `mixer-card` → `alsa-mixer-device`
- `mixer-index` → `alsa-mixer-index`
- `mixer-name` → `alsa-mixer-control`
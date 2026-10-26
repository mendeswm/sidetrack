# DESIGN.md — sidespot Design Document

> Source: native/librespot-local / upstream sidetrack
> Collected: 2026-10-26
> Published: Unknown

Project design for Sidetrack / sidespot: lightweight Spotify client using librespot (Rust) for authentication, metadata, and audio streaming. Architecture layers: Compose UI → Kotlin ViewModel → JNI bridge → librespot core (Rust/Tokio) → Android AudioTrack. Audio quality supported: 96, 160, 320 kbps Ogg Vorbis (no lossless/FLAC). Device targets include small-screen Android devices (480x640, 2GB RAM).

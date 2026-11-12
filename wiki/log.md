# Wiki Log

## [2026-10-26] ingest | Sidetrack M308 Adaptation
- Disposition: New
- Raw: raw/project/2026-10-26-context.md
- Updated: ubuntu-26.04-dependencies.md

## [2026-11-12] ingest | librespot CLI Options
- Disposition: New
- Raw: raw/librespot/2026-11-12-options.md
- Updated: librespot/librespot-cli-options.md

## [2026-10-26] optimization | allowHardware experiment on album art
- Disposition: Kept as-is, pending further testing
- Files: app/src/main/java/com/sidetrack/ui/NowPlayingScreen.kt (line 93)
- Hypothesis: Hardware bitmaps reduce Java heap by ~400 KB on M308
- Result: Inconclusive — measurements not comparable (different workload states)
- Ashmem unchanged at 36 KB; hardware bitmap may not be retained
- SoC: MT6750V/C (Mali-T860) supports hardware bitmaps (API 26+)
- Next: Controlled before/after test with identical workload; verify bitmap.isHardware at runtime

## [2026-11-12] ingest | 24-bit PCM Audio Output Investigation
- Disposition: New
- Raw: raw/audio/2026-11-12-24bit-pcm-investigation.md
- Updated: audio/24bit-pcm-investigation.md

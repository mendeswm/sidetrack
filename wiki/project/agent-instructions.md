# Agent Instructions — Sidetrack M308 Fork

> Sources: AGENTS.md (local), 2026-10-26
> Raw: [agents-instructions](../../raw/project/2026-10-26-agents-instructions.md)
> Updated: 2026-10-26

## Overview
Project-specific working rules for the Sidetrack fork on Oilsky M308. Covers inspection-before-change rules, incremental verifiable modifications, preservation of existing architecture/librespot, compatibility with Android 9/API 28, touch UI for 480x640, no GMS, security (no secrets in repo), and documentation requirements.

## Key Rules
- Inspect before altering (read README, DESIGN, Gradle)
- Small incremental changes with verification after each
- Preserve librespot core; don't replace without justification
- Don't invent build/test/device results
- Document limitations explicitly
- Use Brazilian Portuguese for UI if appropriate

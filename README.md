<p align="center"><img src="ps5/icon0.png" width="96" alt="TITAN CORE"></p>
<h1 align="center">TITAN CORE</h1>
<p align="center">A PS5 game workspace with a native FPKG engine, external-drive installer and live diagnostics.</p>

Developed and maintained by **[devchmod](https://github.com/devchmod)** following six months of independent research, TITAN CORE brings game browsing, USB installation and diagnostics into one PS5 workspace. It integrates the native mounting workflow from Kraken Mount, kstuff-lite 1.11 and ShadowMountPlus 1.7beta10, with TITAN CORE interface, installer and integration work. It targets jailbroken PS5 consoles with elfldr. The development console uses firmware 13.60; wider hardware compatibility has not been established. This project does not include a console exploit.

[Releases](https://github.com/devchmod/TITAN-CORE/releases) · [Source](https://github.com/devchmod/TITAN-CORE) · [Credits](CREDITS.md)

## Features

- Dashboard and game library for supported debug FPKGs, folders and image sources, including exFAT.
- USB/extended-drive browser, internal FPKG copy with progress, byte-for-byte verification and cancellation.
- Rename-only import of supported installed packages from `/user/app` into the homebrew workflow.
- Reported system/payload CPU and memory, storage, live logs, kernel-log viewer and current alerts.
- Browser-local CPU history, read-only reported health checks, JSON/CSV exports and print/Save PDF reports.
- 31 console-language/regional choices, flag cues, Arabic RTL, themes and controller-friendly navigation.

![TITAN CORE diagnostics](docs/screenshots/monitor.png)
*Interface preview with synthetic data. Unsupported sensors remain unavailable on the actual console.*

## Use

1. Close games before loading a new payload.
2. Send the release ELF through your existing elfldr workflow.
3. Open `http://<PS5-IP>:8097` from the console or a browser on your Mac/PC/phone.
4. Browse Library or open Installer to choose a supported PS5 debug FPKG from a mounted USB/extended drive.

Installer copies one complete PKG internally and keeps the external original. The native engine then builds its cache, registers the game and mounts its content at launch. This is the homebrew/debug-FPKG workflow, not a general retail/encrypted PKG installer. Cache takes extra space and can be substantial for incompatible block layouts. Moving a supported existing `/user/app` package uses rename only; if direct movement is unavailable, its source stays in place without creating another full package.

Do not disconnect a source drive during installation. Ordinary cancellation/errors clean the transaction's partial copy; power loss may leave a reserved `.part` directory that needs inspection before retrying. Resume and update/DLC merging are not implemented.

## Diagnostics and localization

Metrics are reported by the console, with payload-only fallbacks labeled explicitly. Temperature/GPU/I/O rate are not exposed by this backend and remain unknown. CPU history starts when the page opens; it is not historical console telemetry. Diagnostic checks inspect reported status only and are not hardware stress tests. Reports/history are handled in the browser.

English, French, Arabic, Simplified Chinese and German have full current catalogs. Additional languages currently translate 50 core labels and use English fallback for remaining technical text. See [coverage](Localizations/coverage.json).

## Technology

[![Rust](https://img.shields.io/badge/Rust-native_backend-DEA584?logo=rust)](https://github.com/devchmod/TITAN-CORE/search?l=rust) [![HTML](https://img.shields.io/badge/HTML-bundled_interface-E34F26?logo=html5&logoColor=white)](https://github.com/devchmod/TITAN-CORE/search?l=html) [![C](https://img.shields.io/badge/C-PS5_shims-555555?logo=c)](https://github.com/devchmod/TITAN-CORE/tree/main/ps5/shim) [![Shell](https://img.shields.io/badge/Shell-build_scripts-89E051?logo=gnubash&logoColor=white)](https://github.com/devchmod/TITAN-CORE/tree/main/ps5) [![Assembly](https://img.shields.io/badge/Assembly-startup-6E4C13)](https://github.com/devchmod/TITAN-CORE/tree/main/ps5) [![Docker](https://img.shields.io/badge/Docker-build_environment-2496ED?logo=docker&logoColor=white)](https://github.com/devchmod/TITAN-CORE/blob/main/ps5/Dockerfile) [![Python](https://img.shields.io/badge/Python-tooling-3572A5?logo=python&logoColor=white)](https://github.com/devchmod/TITAN-CORE/tree/main/review)

| Layer | Implementation |
|---|---|
| Console backend | Rust: package discovery, cache, registration, mounting, installer and HTTP service |
| Console integration | C and Assembly: PS5 shims, startup and linking support |
| Browser interface | HTML, CSS and JavaScript: library, settings, localization and diagnostics |
| Build and tooling | Shell, Dockerfile and Python: payload builds, UI embedding and review tools |

GitHub's language percentages measure classified source bytes; they do not measure engineering effort or ownership. See the repository's Languages panel for its current calculation.

## Build

The source uses Rust 1.99.0 and PS5 payload SDK 0.43 in the Docker build. Keep compiler parallelism at two jobs.

```sh
TITAN_UNIFIED_STACK=1 bash ps5/build.sh
```

The build verifies bundled runtime/service hashes and builds both ELF stages. Internal crate/toolchain names retain upstream compatibility; the user-facing application is TITAN CORE. See [architecture](DESIGN.md), [validation](docs/VALIDATION.md) and [credits](CREDITS.md).

## License and credits

GPL-3.0-or-later. TITAN CORE is a derivative/aggregate project, with original authorship and upstream notices retained. The display name `chmod v0.1` applies to a custom AMPR 0.4.0 launch-notice build; it does not claim a new upstream runtime or authorship of that engine. Bundled component source snapshots, modified runtime source and notices are included under `vendor/`.

See [CREDITS.md](CREDITS.md), [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES).

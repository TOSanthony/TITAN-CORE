# TITAN CORE architecture

## Runtime

TITAN CORE is a Rust PS5 ELF with a bundled, offline web interface on8097. The unified-stack feature coordinates pinned kstuff-lite1.11 and ShadowMountPlus1.7beta10 as separate resident processes while preserving the native FPKG/folder engine. Readiness, process/boot guards and per-title ownership avoid duplicate component launches and competing overlays.

App data lives in /data/titan-core. The tile starts the saved /data/titan-core/titan-core.elf. The build has two stages: stage2 embeds/saves stage1. Upstream-compatible internal crate, protocol, index and sandbox names remain intentional.

## Native sources

src/storage.rs discovers mounted internal/USB/extended drives. src/scan.rs finds supported debug PKGs/folders, validates metadata and resolves duplicates. src/cache.rs and src/ampr_index.rs prepare per-title caches and streaming indexes. src/register.rs registers game tiles and source mounts; src/launch.rs handles sandbox overlays at launch. Native work pauses while a game runs; no force unmount is used. Rebuild preserves game-written files; confirmed source Delete is a separate operation.

## Installed and external packages

Supported /user/app packages may be imported to /data/homebrew/titan-core/TID using rename or a verified /user/data alias only. If direct movement fails, the original stays; no full-copy backup fallback. Receipts recover completed interrupted moves. Per-title ShadowMount policy coordination preserves native ownership.

External installation copies a selected supported debug PKG once into /data/homebrew/titan-core/installed/TID, with progress/cancel, space/collision checks, full byte comparison, source-size/mtime verification, reparse and final rename. The external original is never deleted. Owned install roots are prioritized for native scanning. The worker shares the native work mutex. Ordinary error/cancel cleans the partial transaction; power-loss resume is not implemented. Package plus variable cache needs extra storage. No generic retail/encrypted or update/DLC merging support is implied.

## Web interface

Six destinations: Dashboard, Library, Installer, Storage, Diagnostics, Settings. One image filter covers exFAT/mixed formats without duplicate categories. Settings groups existing values; flags/native language selector, RTL, themes, controller and reduced-motion support remain. The public Stop control is removed; the guarded internal takeover endpoint remains.

GET /api/state provides reported sources/drives/cache/telemetry/services. Other existing routes manage settings, rescan/rebuild/source deletion and source metadata under their normal guards. GET /api/klog proxies at most64KiB of the pinned service's kernel-log tail. Installer browse/start/cancel/status routes validate mounted canonical external paths. No arbitrary browser-provided path is accepted for source deletion.

Diagnostics uses system metrics when exposed and explicitly labeled payload/peak fallbacks otherwise. Unsupported temperature/GPU/I/O stay unknown. CPU samples are browser-session only, bounded to one hour/1801points, with outage/null gaps and uptime-reset handling. Read-only health checks inspect a fresh status snapshot, persisting at most20browser-local results. JSON/CSV/browser-print reports remain client-side. CSV guards formula prefixes; console strings are rendered literally.

Header/About source links target this project; credits remain in CREDITS.md with upstream provenance. Background launcher removal only unregisters the exact recognized ShadowMount web tile after service readiness/no game, without stopping its process. The custom chmod v0.1 display runtime is AMPR0.4.0 with only its launch notice changed; attribution, ABI and engine source stay compatible.

## Build and evidence

Run TITAN_UNIFIED_STACK=1 bash ps5/build.sh with the existing Rust1.99/SDK0.43 Docker toolchain and at most2compiler jobs. Verify vendor hashes and both stages. Edit review/monitor-client.js then embed-monitor.py; regenerate catalogs with embed-localization.py and check script syntax. Existing browser/host/installer tests use synthetic fixtures and are not evidence of hardware gameplay. See docs/VALIDATION.md and CREDITS.md.

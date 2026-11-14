# Credits and licenses

TITAN CORE contains original interface/integration work alongside upstream engines and payloads. The project preserves their authorship, source and license notices.

| Component | Upstream | Role |
|---|---|---|
| Kraken Mount | [quer3q/kraken-mount](https://github.com/quer3q/kraken-mount) | Original native FPKG/folder mounting, cache, registration and launch foundation |
| ampr-emu-ng0.4.0 | [quer3q/ampr-emu-ng](https://github.com/quer3q/ampr-emu-ng/tree/v0.4.0) | Packed-game runtime; custom launch-notice display label `chmod v0.1` |
| kstuff-lite1.11 | [EchoStretch/kstuff-lite](https://github.com/EchoStretch/kstuff-lite/releases/tag/v1.11) | Pinned resident payload, embedded unchanged |
| ShadowMountPlus1.7beta10 | [drakmor/ShadowMountPlus](https://github.com/drakmor/ShadowMountPlus/releases/tag/1.7beta10) | Pinned image service, embedded unchanged |
| PS5 Dump Forge | [quer3q/ps5-dump-forge](https://github.com/quer3q/ps5-dump-forge) | Toolchain/workflow references and attributed sparse-format reader reference |
| ps5-payload-sdk | [ps5-payload-dev/sdk](https://github.com/ps5-payload-dev/sdk) | Payload build toolchain and PS5 libraries |

## Source and notices

- Root license: GPL-3.0-or-later; see [LICENSE](LICENSE).
- Existing dependency notices: [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES).
- Pinned service versions, original source snapshots, component licenses and hashes: [vendor/unified](vendor/unified).
- Original AMPR0.4.0 binary/source and custom display-only runtime source: [vendor/ampr-emu-ng](vendor/ampr-emu-ng), including [branding explanation](vendor/ampr-emu-ng/TITAN_BRANDING.md).
- Sparse-format port attribution and exact source reference: [vendor/format-reference](vendor/format-reference).
- TITAN CORE artwork provenance: [docs/branding/PROVENANCE.md](docs/branding/PROVENANCE.md).

The custom AMPR source changes only the launch-notice text. Compatible filenames, exported runtime identity and underlying engine version remain upstream0.4.0. Upstream credits in source archives are preserved. Additional third-party code retains its own license terms; consult the included notices/source for each component.

# chmod v0.1 display branding

Based on quer3q/ampr-emu-ng tag v0.4.0 (upstream engine version remains0.4.0). The owner requested the launch toast name chmod v0.1. Only load_notice in crates/ampr-sys/src/rt/exports.rs changes its two prefix slices from ampr-emu-ng: v plus VERSION to chmod v0.1 plus empty slice. Mode and running suffix, ABI identity, format paths, exported functions, hooks, cache and read engine remain unchanged in source. GPL-3.0-or-later attribution preserved. Original binary and source ZIP are included, alongside modified source ZIP.

Built with the existing Rust1.99/SDK0.43 Docker image: make -j2 verify. Upstream PRX checker passed137 exports,141 definitions,103 imports, FSELF0x02000001. This is a custom notice build, not an unmodified upstream release, and has not been run on hardware by the agent.

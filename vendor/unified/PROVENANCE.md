# Unified payload provenance — 2026-11-14

The owner explicitly requested kstuff v1.11 and ShadowMount 1.7beta10 inside TITAN CORE, with firmware 13.60 as the device target and a version gate through 13.60. These are required runtime components of the new isolated unified build, not game assets. No payload was executed on a PS5 by the agent.

Unmodified official release ELFs were fetched from EchoStretch/kstuff-lite tag v1.11 and drakmor/ShadowMountPlus tag 1.7beta10. Their exact SHA-256 values match the GitHub release API asset digest fields. URLs, commit pins and hashes are in vendor/unified/MANIFEST.json; build.sh verifies vendor/unified/SHA256SUMS when unified mode is enabled. No latest-tag auto-update is used.

Upstream source snapshots are included for both exact commits. The kstuff source's GPL-3.0-or-later file headers and the ShadowMount GPL-3.0 LICENSE are preserved in those source archives. Full GPL text and upstream ShadowMount third-party notices are included in vendor/unified/notices. Exact pinned BearSSL, libtomcrypt and ISA-L Crypto submodule source snapshots are included too; their own license files remain in those archives. Upstream SDK/static-library dependencies remain documented by upstream source/build files and notices. The combined launcher embeds the official release binaries rather than rebuilding or altering their patch/mount code. It does not claim the upstream release toolchain has been independently reproduced.

The readiness probe in src/stack.rs implements the KEKCALL_CHECK calling contract from ShadowMountPlus commit a8e400487effa1a63c81943ad16fef2d9ea83cfb (src/sm_kstuff.c and include/sm_kstuff.h). That port is distributed under the project's GPL-compatible license with this attribution. No new crates or dependency versions were added to the production Cargo manifest/lockfile.

Original project acknowledgments and THIRD_PARTY_NOTICES are retained. This is an aggregate test build for the owner, not a claim of authorship over upstream code, trademarks or support guarantees. The generated TITAN CORE icon retains its prior provenance in docs/branding/PROVENANCE.md.

The read-only kinfo process-record parsing also follows the pinned ShadowMountPlus src/main.c layout contract (record size prefix, name offset 447). It is used to avoid restarting already-running dependencies when their API/readiness is not yet available. No signaling or process termination code was copied or added.

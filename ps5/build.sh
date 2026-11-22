#!/usr/bin/env bash
# Builds target/ps5/titan-core.elf, the PS5 payload, in Docker (toolchain from ps5-dump-forge).
set -euo pipefail
cd "$(dirname "$0")/.."

if [[ "${1:-}" != --inside ]]; then
  docker build -t titan-core-ps5 ps5
  exec docker run --rm --user "$(id -u):$(id -g)" -e HOME=/tmp \
    -e CARGO_BUILD_JOBS=2 -e TITAN_UNIFIED_STACK="${TITAN_UNIFIED_STACK:-0}" -e CARGO_HOME=/work/target/ps5/cargo -v "$PWD:/work" titan-core-ps5 bash ps5/build.sh --inside
fi

# The bundled ampr-emu-ng must be the released file (SHA256SUMS copied from its GitHub release).
(cd vendor/ampr-emu-ng && sha256sum -c --ignore-missing --strict SHA256SUMS)

features=()
if [[ "${TITAN_UNIFIED_STACK:-0}" == 1 ]]; then
  # vendor/unified/PAYLOADS.conf: "<file>=1" embeds a payload, "<file>=0" leaves it out.
  declare -A want=([kstuff-lite-v1.11.elf]= [shadowmountplus-1.7beta10.elf]= [etaHEN.elf]=)
  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%%#*}"; line="${line//[[:space:]]/}"
    [[ -z "$line" ]] && continue
    name="${line%%=*}"; value="${line#*=}"
    [[ "$line" == *=* && -v "want[$name]" ]] || { echo "PAYLOADS.conf: unknown entry '$line'" >&2; exit 1; }
    [[ "$value" == 0 || "$value" == 1 ]] || { echo "PAYLOADS.conf: '$name' must be 0 or 1" >&2; exit 1; }
    want[$name]=$value
  done < vendor/unified/PAYLOADS.conf
  for name in "${!want[@]}"; do
    [[ -n "${want[$name]}" ]] || { echo "PAYLOADS.conf: '$name' is not listed" >&2; exit 1; }
  done
  [[ "${want[shadowmountplus-1.7beta10.elf]}" == 1 ]] || { echo "PAYLOADS.conf: shadowmountplus-1.7beta10.elf must be 1" >&2; exit 1; }
  for name in "${!want[@]}"; do
    if [[ "${want[$name]}" == 1 ]]; then
      (cd vendor/unified && grep -F -- "  $name" SHA256SUMS | sha256sum -c --strict -)
    fi
  done
  features=(--features unified-stack)
  [[ "${want[kstuff-lite-v1.11.elf]}" == 1 ]] && features+=(--features payload-kstuff)
  [[ "${want[etaHEN.elf]}" == 1 ]] && features+=(--features payload-etahen)
  echo "Embedded payloads: kstuff=${want[kstuff-lite-v1.11.elf]} shadowmountplus=1 etaHEN=${want[etaHEN.elf]}"
fi
out=target/ps5
cc="$PS5_PAYLOAD_SDK/bin/prospero-clang"
mkdir -p "$out/shim"
# Web-uploaded sources may not preserve Unix executable bits.
chmod +x ps5/linker.sh
for src in ps5/shim/freebsd11.c ps5/shim/syscalls.S ps5/entry.c ps5/compat.c; do
  name=$(basename "${src%.*}")
  "$cc" -O2 -Wall -Wextra -Werror -c "$src" -o "$out/shim/$name.o"
done

export RUSTC_BOOTSTRAP=1
# The PS5 kernel fills FreeBSD 11 structs (stat 120 bytes, dirent 264, old statfs); libc and the
# std built here default to FreeBSD 12's. libc >= 0.2.187 reads this cfg; rustflags reach std
# too under -Z build-std.
export CARGO_TARGET_X86_64_PS5_FREEBSD_RUSTFLAGS='--cfg libc_unstable_freebsd_version="11"'
export CARGO_TARGET_DIR="$out"
export CARGO_TARGET_X86_64_PS5_FREEBSD_LINKER=/work/ps5/linker.sh
export CC_x86_64_ps5_freebsd="$cc" AR_x86_64_ps5_freebsd="$PS5_PAYLOAD_SDK/bin/prospero-ar"
# Cargo doesn't see the C objects or linker.sh: drop the linked output so a shim change relinks.
build() {
  rm -f "$out"/x86_64-ps5-freebsd/release/deps/titan_core-*
  cargo build --release --locked \
    --target ps5/x86_64-ps5-freebsd.json -Z json-target-spec \
    -Z build-std=std,panic_abort -Z build-std-features=panic-unwind "$@"
}
# Stage 1 is the copy the tile starts (/data/titan-core/titan-core.elf); stage 2, the ELF you
# push, embeds it and saves it there (as Dump Forge's self copy).
build "${features[@]}"
cp "$out/x86_64-ps5-freebsd/release/titan-core" "$out/titan-core-stage1.elf"
TITAN_SELF_ELF="/work/$out/titan-core-stage1.elf" build "${features[@]}" --features self-copy
cp "$out/x86_64-ps5-freebsd/release/titan-core" "$out/titan-core.elf"
llvm-readelf -h -d "$out/titan-core.elf" > "$out/titan-core.elf.txt"
ls -l "$out/titan-core.elf"

#!/bin/sh
# Keel release: one script, the same for the workflow and for a person.
#
#   sh release.sh [--out <dir>] [--tag <tag>]
#
# Builds the tool in `tool/` (cargo, release), asks the binary its
# version and rustc the target, and writes into the out directory
# (`tool/target/dist/` by default -- inside the ignored build tree, so
# a release never lands in `git status`):
#
#   keel-<version>-<target>.tar.gz     one file inside: `keel`
#   keel-<version>-<target>.tar.gz.sha256
#
# One checksum file per archive, not a shared SHA256SUMS: the release
# workflow builds several targets in parallel, and a shared file is
# what they would overwrite. The launcher and the installer fetch the
# archive and its .sha256 side by side and check before unpacking.
#
# What is measured aloud rather than assumed: the version is what the
# BUILT binary answers, never a number copied out of a manifest; the
# target is rustc's own word for this machine.
#
# `--tag <tag>` is what the workflow passes: the tag being released.
# The archive is named by the binary's version, and the launcher asks
# for `<tag>/keel-<version>-<target>.tar.gz` -- so a tag whose tree
# answers another number would publish a release no pin ever finds
# (review 0048 R-1). With `--tag`, `v<version>` must equal the tag,
# or this refuses and writes nothing.
set -eu

out=""
tag=""
while [ $# -gt 0 ]; do
    case "$1" in
        --out)
            [ $# -ge 2 ] || { echo "release.sh: --out needs a directory" >&2; exit 2; }
            out="$2"; shift 2 ;;
        --tag)
            [ $# -ge 2 ] || { echo "release.sh: --tag needs a tag" >&2; exit 2; }
            tag="$2"; shift 2 ;;
        *) echo "release.sh: unknown argument $1" >&2; exit 2 ;;
    esac
done

here="$(cd "$(dirname "$0")" && pwd)"
manifest="$here/tool/Cargo.toml"
[ -f "$manifest" ] || { echo "release.sh: no tool/Cargo.toml beside this script" >&2; exit 1; }
[ -n "$out" ] || out="$here/tool/target/dist"

for tool in cargo rustc tar; do
    command -v "$tool" >/dev/null 2>&1 || { echo "release.sh: $tool is required and was not found" >&2; exit 1; }
done

checksum() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        echo ""
    fi
}

target="$(rustc -vV | sed -n 's/^host: //p')"
[ -n "$target" ] || { echo "release.sh: rustc names no host target" >&2; exit 1; }

# The tree's own target directory, as install.sh builds: an inherited
# CARGO_TARGET_DIR would put the binary somewhere else entirely.
echo "release.sh: building the tool (cargo, release) for $target"
CARGO_TARGET_DIR="$here/tool/target" cargo build --release --quiet --manifest-path "$manifest"
binary="$here/tool/target/release/keel"
[ -x "$binary" ] || { echo "release.sh: cargo built nothing at $binary" >&2; exit 1; }

version="$("$binary" --version | head -1 | awk '{print $2}')"
[ -n "$version" ] || { echo "release.sh: the binary answers no version" >&2; exit 1; }
if [ -n "$tag" ] && [ "$tag" != "v$version" ]; then
    echo "release.sh: the tree at $tag answers keel $version -- a release under $tag would" >&2
    echo "release.sh: never be found by a pin; tag the commit whose crate says ${tag#v}" >&2
    exit 1
fi

mkdir -p "$out"
name="keel-$version-$target.tar.gz"
tar -czf "$out/$name" -C "$(dirname "$binary")" keel
sum="$(checksum "$out/$name")"
if [ -z "$sum" ]; then
    echo "release.sh: this machine has neither sha256sum nor shasum, so no checksum" >&2
    echo "release.sh: can be written -- a release without one is not a release" >&2
    rm -f "$out/$name"
    exit 1
fi
printf '%s  %s\n' "$sum" "$name" > "$out/$name.sha256"

echo "release.sh: wrote $out/$name  (keel $version, $target)"
echo "release.sh: wrote $out/$name.sha256  ($sum)"

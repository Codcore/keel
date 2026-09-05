#!/bin/sh
# Keel installer.
#
# Clones the method into ~/.keel and puts a `keel` command on PATH.
#
#   curl -fsSL https://raw.githubusercontent.com/Codcore/keel/main/install.sh | sh
#
# The tool is a Rust crate in `tool/`; this clones the repository and builds it,
# because a release binary is not published yet. Updating is one `git pull` and
# one rebuild, which is what a second run does.
#
# git and cargo are needed. Nothing else -- but cargo writes its own registry
# and cache into CARGO_HOME (~/.cargo by default, tens of megabytes on a first
# build). That is cargo's home, not keel's, and this script does not move it:
# set CARGO_HOME yourself if it must live elsewhere (review 0039 R-9).
#
# A VERSION may be named -- as the first argument or as KEEL_REF -- and then
# exactly that git ref is installed:
#
#   KEEL_REF="v0.8.9" sh install.sh
#   curl -fsSL .../install.sh | sh -s -- v0.8.9
#
# `keel version` prints that very line when keel.toml pins a version this
# binary is not. Named because the courts refuse while the two differ, and
# advice with no hand behind it is not advice.
#
# The border, said rather than hidden: this fetches a git ref BY NAME. It is
# not a verified checksum, and there is no ~/.keel/versions/ holding several
# releases side by side -- that rung of the concept is not built.
#
# Override with KEEL_REPO, KEEL_HOME, KEEL_BIN, KEEL_REF.
set -eu

REPO="${KEEL_REPO:-https://github.com/Codcore/keel.git}"
KEEL_HOME="${KEEL_HOME:-$HOME/.keel}"
KEEL_BIN="${KEEL_BIN:-$HOME/.local/bin}"
KEEL_REF="${KEEL_REF:-${1:-}}"

# One clone to fetch with, and a home per version beside it (wave
# 0041). Before this, everything went into $KEEL_HOME and one binary
# went on PATH, so installing a second version overwrote the first and
# two projects on two pins could not work at the same time at all.
SOURCE="$KEEL_HOME/source"
VERSIONS="$KEEL_HOME/versions"

# The checksum this distribution has: sha256 of the binary itself,
# by whichever tool the machine carries. Where it carries neither, the
# integrity check is skipped -- and said aloud rather than faked.
checksum() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d' ' -f1
    else
        echo ""
    fi
}

# The launcher: the ONE `keel` on PATH, which reads a project's pin
# and hands over to that version (NEW-CONCEPT, "Distribution"). The
# operator's line is kept: there is no shim inside a project.
install_launcher() {
    cat > "$1" <<'LAUNCHER'
#!/bin/sh
# keel launcher -- written by install.sh, do not edit.
#
# It reads the `version` a project pins in keel.toml and runs exactly
# that version out of ~/.keel/versions/. It NEVER runs a different
# one: the wrong binary in silence is worse than a refusal.
set -eu

KEEL_HOME="${KEEL_HOME:-$HOME/.keel}"
VERSIONS="$KEEL_HOME/versions"

# Where the project is: `-C <dir>` if it was given, else here. The
# launcher must find the same project the tool will judge.
where="$PWD"
prev=""
for word in "$@"; do
    if [ "$prev" = "-C" ]; then
        where="$word"
        break
    fi
    prev="$word"
done

# The pin, from the nearest keel.toml at or above that directory.
pin=""
dir="$(cd "$where" 2>/dev/null && pwd || echo "$PWD")"
while :; do
    if [ -f "$dir/keel.toml" ]; then
        pin="$(sed -n 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$dir/keel.toml" | head -1)"
        break
    fi
    [ "$dir" = "/" ] && break
    dir="$(dirname "$dir")"
done

run() {
    home="$1"
    shift
    # The integrity check, before handing over: a binary that is not
    # the one that was installed is named, never run.
    if [ -f "$home/.keel-sum" ]; then
        want="$(cat "$home/.keel-sum")"
        if [ -n "$want" ]; then
            if command -v sha256sum >/dev/null 2>&1; then
                have="$(sha256sum "$home/keel" | cut -d' ' -f1)"
            elif command -v shasum >/dev/null 2>&1; then
                have="$(shasum -a 256 "$home/keel" | cut -d' ' -f1)"
            else
                have="$want"
            fi
            if [ "$have" != "$want" ]; then
                echo "keel: the binary at $home/keel is not the one that was installed" >&2
                echo "keel:   recorded $want" >&2
                echo "keel:   found    $have" >&2
                echo "keel: instead: reinstall that version, or delete $home" >&2
                exit 2
            fi
        fi
    fi
    exec "$home/keel" "$@"
}

if [ -n "$pin" ]; then
    for home in "$VERSIONS"/*; do
        [ -f "$home/.keel-version" ] || continue
        if [ "$(cat "$home/.keel-version")" = "$pin" ]; then
            run "$home" "$@"
        fi
    done
    echo "keel: keel.toml pins version \"$pin\", and it is not installed here" >&2
    echo "keel: installed:" >&2
    for home in "$VERSIONS"/*; do
        [ -f "$home/.keel-version" ] || continue
        echo "keel:   $(cat "$home/.keel-version")  ($(basename "$home"))" >&2
    done
    echo "keel: instead: install exactly that version --" >&2
    echo "keel:   KEEL_REF=\"<the tag or commit of $pin>\" sh install.sh" >&2
    exit 2
fi

current=""
[ -f "$KEEL_HOME/.keel-current" ] && current="$(cat "$KEEL_HOME/.keel-current")"
if [ -n "$current" ] && [ -x "$VERSIONS/$current/keel" ]; then
    run "$VERSIONS/$current" "$@"
fi
echo "keel: no version is installed in $VERSIONS" >&2
echo "keel: instead: sh install.sh" >&2
exit 2
LAUNCHER
    chmod +x "$1"
}

for tool in git cargo; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "keel: $tool is required and was not found" >&2
        exit 1
    }
done

if [ -d "$SOURCE/.git" ]; then
    echo "keel: updating $SOURCE"
    git -C "$SOURCE" fetch --quiet --tags origin
    # A checkout of a named ref is not on a branch, so pull would have
    # nothing to fast-forward. Only the unpinned road pulls -- and it
    # comes back to a branch first, by the remote's own head.
    #
    # Review 0039 R-2: `checkout -` stood here, and `@{-1}` does not
    # exist in a clone that was never moved, so the SECOND ordinary
    # run died -- the very run this script's own head calls updating.
    if [ -z "$KEEL_REF" ]; then
        if ! git -C "$SOURCE" symbolic-ref -q HEAD >/dev/null 2>&1; then
            head="$(git -C "$SOURCE" symbolic-ref --short -q refs/remotes/origin/HEAD 2>/dev/null || true)"
            branch="${head#origin/}"
            [ -n "$branch" ] || branch="main"
            echo "keel: back to $branch from a pinned checkout"
            git -C "$SOURCE" checkout --quiet "$branch"
        fi
        git -C "$SOURCE" pull --ff-only --quiet
    fi
else
    echo "keel: cloning into $SOURCE"
    mkdir -p "$KEEL_HOME"
    git clone --quiet "$REPO" "$SOURCE"
fi

if [ -n "$KEEL_REF" ]; then
    # The named version, and nothing else: a ref that is not there is
    # a refusal by name, never a silent build of whatever main is.
    if ! git -C "$SOURCE" rev-parse --verify --quiet "$KEEL_REF^{commit}" >/dev/null; then
        echo "keel: no such version \"$KEEL_REF\" in $REPO" >&2
        echo "keel: the versions this clone knows:" >&2
        git -C "$SOURCE" tag | tail -10 >&2
        exit 1
    fi
    echo "keel: checking out $KEEL_REF"
    git -C "$SOURCE" checkout --quiet --detach "$KEEL_REF"
fi

# A ref may predate the layout this installer builds -- keel v1 kept the
# crate elsewhere. Said by name, rather than left to cargo's "manifest
# path does not exist" a screen later.
if [ ! -f "$SOURCE/tool/Cargo.toml" ]; then
    echo "keel: ${KEEL_REF:-main} carries no tool/Cargo.toml -- this installer builds" >&2
    echo "keel: the crate in tool/, which older versions of keel did not have" >&2
    exit 1
fi

# The name of this version's home: the ref that was asked for, or the
# branch the remote leads with when none was.
name="$KEEL_REF"
if [ -z "$name" ]; then
    name="$(git -C "$SOURCE" symbolic-ref --short -q HEAD 2>/dev/null || echo main)"
fi
home="$VERSIONS/$name"
sha="$(git -C "$SOURCE" rev-parse HEAD)"

echo "keel: building the tool (cargo, release)"
mkdir -p "$home"
# The version's own tree, so its build cannot be moved by the next
# install. An inherited CARGO_TARGET_DIR would put the binary
# somewhere else entirely and the copy below would miss it.
CARGO_TARGET_DIR="$SOURCE/tool/target" \
    cargo build --release --quiet --manifest-path "$SOURCE/tool/Cargo.toml"

cp "$SOURCE/tool/target/release/keel" "$home/keel"
chmod +x "$home/keel"

# What it answers for itself, and the commit it was built from. The
# first is what a project's pin names; the second is the checksum this
# distribution has -- git's own. Said plainly: a sha proves the tree is
# the one the ref named, NOT that the ref is worth trusting.
version="$("$home/keel" --version | head -1 | awk '{print $2}')"
printf '%s\n' "$version" > "$home/.keel-version"
printf '%s\n' "$sha" > "$home/.keel-sha"
printf '%s\n' "$(checksum "$home/keel")" > "$home/.keel-sum"
printf '%s\n' "$name" > "$KEEL_HOME/.keel-current"

mkdir -p "$KEEL_BIN"
install_launcher "$KEEL_BIN/keel"

echo "keel: keel $version installed at $home"
echo "keel: the launcher at $KEEL_BIN/keel runs the version a project pins"

case ":${PATH}:" in
    *":$KEEL_BIN:"*) ;;
    *)
        echo
        echo "$KEEL_BIN is not on your PATH. Add it:"
        echo "  export PATH=\"$KEEL_BIN:\$PATH\""
        ;;
esac

echo
echo "Next, in the project you want to work in:"
echo "  keel init"

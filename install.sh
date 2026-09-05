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

for tool in git cargo; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "keel: $tool is required and was not found" >&2
        exit 1
    }
done

if [ -d "$KEEL_HOME/.git" ]; then
    echo "keel: updating $KEEL_HOME"
    git -C "$KEEL_HOME" fetch --quiet --tags origin
    # A checkout of a named ref is not on a branch, so pull would have
    # nothing to fast-forward. Only the unpinned road pulls -- and it
    # comes back to a branch first, by the remote's own head.
    #
    # Review 0039 R-2: `checkout -` stood here, and `@{-1}` does not
    # exist in a clone that was never moved, so the SECOND ordinary
    # run died -- the very run this script's own head calls updating.
    if [ -z "$KEEL_REF" ]; then
        if ! git -C "$KEEL_HOME" symbolic-ref -q HEAD >/dev/null 2>&1; then
            head="$(git -C "$KEEL_HOME" symbolic-ref --short -q refs/remotes/origin/HEAD 2>/dev/null || true)"
            branch="${head#origin/}"
            [ -n "$branch" ] || branch="main"
            echo "keel: back to $branch from a pinned checkout"
            git -C "$KEEL_HOME" checkout --quiet "$branch"
        fi
        git -C "$KEEL_HOME" pull --ff-only --quiet
    fi
else
    echo "keel: cloning into $KEEL_HOME"
    git clone --quiet "$REPO" "$KEEL_HOME"
fi

if [ -n "$KEEL_REF" ]; then
    # The named version, and nothing else: a ref that is not there is
    # a refusal by name, never a silent build of whatever main is.
    if ! git -C "$KEEL_HOME" rev-parse --verify --quiet "$KEEL_REF^{commit}" >/dev/null; then
        echo "keel: no such version \"$KEEL_REF\" in $REPO" >&2
        echo "keel: the versions this clone knows:" >&2
        git -C "$KEEL_HOME" tag | tail -10 >&2
        exit 1
    fi
    echo "keel: checking out $KEEL_REF"
    git -C "$KEEL_HOME" checkout --quiet --detach "$KEEL_REF"
fi

# A ref may predate the layout this installer builds -- keel v1 kept the
# crate elsewhere. Said by name, rather than left to cargo's "manifest
# path does not exist" a screen later.
if [ ! -f "$KEEL_HOME/tool/Cargo.toml" ]; then
    echo "keel: ${KEEL_REF:-main} carries no tool/Cargo.toml -- this installer builds" >&2
    echo "keel: the crate in tool/, which older versions of keel did not have" >&2
    exit 1
fi

echo "keel: building the tool (cargo, release)"
# Its own target directory: an inherited CARGO_TARGET_DIR would put
# the binary somewhere else entirely and the copy below would miss it.
CARGO_TARGET_DIR="$KEEL_HOME/tool/target" \
    cargo build --release --quiet --manifest-path "$KEEL_HOME/tool/Cargo.toml"

mkdir -p "$KEEL_BIN"
cp "$KEEL_HOME/tool/target/release/keel" "$KEEL_BIN/keel"
chmod +x "$KEEL_BIN/keel"

version=$("$KEEL_BIN/keel" --version | head -1)
echo "keel: $version installed at $KEEL_BIN/keel"

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

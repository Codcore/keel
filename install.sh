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
# git and cargo are needed. Nothing else.
#
# Override with KEEL_REPO, KEEL_HOME, KEEL_BIN.
set -eu

REPO="${KEEL_REPO:-https://github.com/Codcore/keel.git}"
KEEL_HOME="${KEEL_HOME:-$HOME/.keel}"
KEEL_BIN="${KEEL_BIN:-$HOME/.local/bin}"

for tool in git cargo; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "keel: $tool is required and was not found" >&2
        exit 1
    }
done

if [ -d "$KEEL_HOME/.git" ]; then
    echo "keel: updating $KEEL_HOME"
    git -C "$KEEL_HOME" pull --ff-only --quiet
else
    echo "keel: cloning into $KEEL_HOME"
    git clone --quiet "$REPO" "$KEEL_HOME"
fi

echo "keel: building the tool (cargo, release)"
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

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
# Each version gets a home of its own under ~/.keel/versions/, and the `keel`
# put on PATH is a LAUNCHER: it reads the version a project pins and runs
# exactly that one. Two projects on two pins work at the same time.
#
# The border, said rather than hidden: this fetches a git ref BY NAME, and the
# sha it records proves which tree arrived -- not that the ref is worth
# trusting. A signed, published release with a checksum of its own is not
# built, and the launcher does not fetch a missing version by itself: it
# refuses with the command.
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
    # The home this installer used is baked in as the default, so a
    # person who moved KEEL_HOME does not have to export it forever
    # (review 0041 R-8).
    printf '%s\n' "#!/bin/sh" > "$1"
    printf '%s\n' "# keel launcher -- written by install.sh, do not edit." >> "$1"
    printf '%s\n' "KEEL_HOME=\"\${KEEL_HOME:-$KEEL_HOME}\"" >> "$1"
    cat >> "$1" <<'LAUNCHER'
#
# It reads the `version` a project pins in keel.toml and runs exactly
# that version out of $KEEL_HOME/versions/. It NEVER runs a different
# one: the wrong binary in silence is worse than a refusal.
set -eu

VERSIONS="$KEEL_HOME/versions"

# Where the project is. `-C <dir>` if it was given -- the FIRST one,
# which is the one the tool itself takes -- else a plain argument that
# is a directory, else here. Review 0041 R-5: a positional path is the
# form keel's own probes use, and the launcher used to ignore it, so
# it could hand over a version the tool would then refuse.
#
# It does not have to be perfect: where the launcher picks wrong, the
# tool's own pin court refuses aloud. What it must never do is pick
# wrong in silence.
where="$PWD"
prev=""
found_c=""
for word in "$@"; do
    if [ -n "$found_c" ]; then break; fi
    if [ "$prev" = "-C" ]; then
        where="$word"
        found_c="yes"
    fi
    prev="$word"
done
if [ -z "$found_c" ]; then
    for word in "$@"; do
        case "$word" in
            -*) ;;
            *) if [ -d "$word" ]; then where="$word"; fi ;;
        esac
    done
fi

# The pin, from the nearest keel.toml at or above that directory.
# Both TOML string forms, because both are legal and the tool reads
# both: reading only one made a pin in single quotes look like no pin
# at all, and the launcher then ran another version in silence
# (review 0041 R-2).
pin=""
pinned_line=""
dir="$(cd "$where" 2>/dev/null && pwd || echo "$PWD")"
while :; do
    if [ -f "$dir/keel.toml" ]; then
        pinned_line="$(grep -E '^[[:space:]]*version[[:space:]]*=' "$dir/keel.toml" | head -1 || true)"
        pin="$(printf '%s' "$pinned_line" | sed -n "s/^[[:space:]]*version[[:space:]]*=[[:space:]]*[\"']\([^\"']*\)[\"'].*/\1/p")"
        break
    fi
    [ "$dir" = "/" ] && break
    dir="$(dirname "$dir")"
done

# A `version` line that is there and cannot be read is a refusal, not
# a shrug: silently treating it as "no pin" is how R-2 ran the wrong
# version.
if [ -n "$pinned_line" ] && [ -z "$pin" ]; then
    echo "keel: $dir/keel.toml has a version line the launcher cannot read:" >&2
    echo "keel:   $pinned_line" >&2
    echo "keel: instead: write it as version = \"<the version or ref>\"" >&2
    exit 2
fi

# Every version standing here, deepest first: a ref may carry a slash
# (`plan/0041-...` is a branch of this very repository), and the home
# is named by an encoded form of it. Review 0041 R-4: a flat glob saw
# none of those and told a person nothing was installed.
homes() {
    find "$VERSIONS" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | sort
}

name_of() {
    if [ -f "$1/.keel-ref" ]; then cat "$1/.keel-ref"; else basename "$1"; fi
}

say_installed() {
    for home in $(homes); do
        [ -f "$home/.keel-version" ] || continue
        echo "keel:   $(cat "$home/.keel-version")  (ref $(name_of "$home"))" >&2
    done
}

run() {
    home="$1"
    shift
    # The integrity check, before handing over. A recorded checksum
    # that is empty, or a record that is gone, is NOT a pass: on a
    # machine with no sha256 tool the installer wrote an empty file
    # and the check quietly turned off (review 0041 R-3).
    if [ ! -f "$home/.keel-sum" ]; then
        echo "keel: $home has no recorded checksum -- it was not installed by this installer" >&2
        echo "keel: instead: reinstall that version, or delete $home" >&2
        exit 2
    fi
    want="$(cat "$home/.keel-sum")"
    if [ "$want" = "none" ]; then
        echo "keel: warning: $(name_of "$home") was installed on a machine with no sha256 tool," >&2
        echo "keel: warning: so the binary is run unchecked" >&2
    else
        if command -v sha256sum >/dev/null 2>&1; then
            have="$(sha256sum "$home/keel" | cut -d' ' -f1)"
        elif command -v shasum >/dev/null 2>&1; then
            have="$(shasum -a 256 "$home/keel" | cut -d' ' -f1)"
        else
            have=""
        fi
        if [ -z "$have" ]; then
            echo "keel: warning: no sha256 tool here, so the binary is run unchecked" >&2
        elif [ "$have" != "$want" ]; then
            echo "keel: the binary at $home/keel is not the one that was installed" >&2
            echo "keel:   recorded $want" >&2
            echo "keel:   found    $have" >&2
            echo "keel: instead: reinstall that version, or delete $home" >&2
            exit 2
        fi
    fi
    # The binary knows its crate version and not the ref it was built
    # from; the launcher does, and tells it, so a pin may name either.
    KEEL_RUNNING_REF="$(name_of "$home")" export KEEL_RUNNING_REF
    exec "$home/keel" "$@"
}

if [ -n "$pin" ]; then
    # A pin may name the ref or the version. Two homes can answer for
    # one crate version -- on keel itself EVERY ref answers 0.1.0 --
    # and picking one of them by glob order is exactly the silent
    # wrong binary this launcher exists to prevent (review 0041 R-1).
    matched=""
    count=0
    for home in $(homes); do
        [ -f "$home/.keel-version" ] || continue
        if [ "$(name_of "$home")" = "$pin" ]; then
            matched="$home"
            count=1
            break
        fi
        if [ "$(cat "$home/.keel-version")" = "$pin" ]; then
            matched="$home"
            count=$((count + 1))
        fi
    done
    if [ "$count" -gt 1 ]; then
        echo "keel: keel.toml pins \"$pin\", and more than one version here answers to it:" >&2
        say_installed
        echo "keel: instead: pin the ref instead of the version -- it is unique," >&2
        echo "keel:   and the refs are named above" >&2
        exit 2
    fi
    if [ -n "$matched" ]; then
        run "$matched" "$@"
    fi
    echo "keel: keel.toml pins \"$pin\", and it is not installed here" >&2
    echo "keel: installed:" >&2
    say_installed
    echo "keel: instead: install exactly that one --" >&2
    echo "keel:   KEEL_REF=\"$pin\" sh install.sh" >&2
    echo "keel:   curl -fsSL https://raw.githubusercontent.com/Codcore/keel/main/install.sh | sh -s -- \"$pin\"" >&2
    exit 2
fi

current=""
[ -f "$KEEL_HOME/.keel-current" ] && current="$(cat "$KEEL_HOME/.keel-current")"
if [ -n "$current" ] && [ -x "$VERSIONS/$current/keel" ]; then
    run "$VERSIONS/$current" "$@"
fi
# Nothing to run -- and which of the two reasons it is, said plainly.
# Review 0041 R-10: this refusal claimed the whole directory was empty
# while two versions stood in it.
if [ -n "$(homes)" ]; then
    echo "keel: no version is marked current in $KEEL_HOME, though these stand here:" >&2
    say_installed
    echo "keel: instead: reinstall the one you want -- it becomes current" >&2
else
    echo "keel: no version is installed in $VERSIONS" >&2
    echo "keel: instead: sh install.sh" >&2
fi
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
# A ref may carry a slash -- `plan/0041-...` is a branch of this very
# repository -- and a home named with one is a directory a level
# deeper that neither the launcher nor the lamp could see (review 0041
# R-4). The home is named by an encoded form; the true ref is written
# beside it and is what a person and a pin see.
encoded="$(printf '%s' "$name" | tr '/' '~')"
home="$VERSIONS/$encoded"
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
printf '%s\n' "$name" > "$home/.keel-ref"
printf '%s\n' "$sha" > "$home/.keel-sha"
# A checksum this machine cannot compute is recorded as "none", not as
# an empty line: an empty record read as "nothing to check" and the
# integrity gate turned itself off in silence (review 0041 R-3).
sum="$(checksum "$home/keel")"
if [ -z "$sum" ]; then
    sum="none"
    echo "keel: this machine has neither sha256sum nor shasum, so no checksum" >&2
    echo "keel: was recorded -- the launcher will say so on every run" >&2
fi
printf '%s\n' "$sum" > "$home/.keel-sum"
printf '%s\n' "$encoded" > "$KEEL_HOME/.keel-current"

mkdir -p "$KEEL_BIN"
# A keel that is not ours is said aloud before it is replaced (review
# 0041 R-7: the decision claimed it was never written over).
if [ -f "$KEEL_BIN/keel" ] && ! head -2 "$KEEL_BIN/keel" | grep -q "keel launcher"; then
    echo "keel: $KEEL_BIN/keel is not this launcher; replacing it" >&2
    echo "keel: (a copy is kept at $KEEL_BIN/keel.before-launcher)" >&2
    cp "$KEEL_BIN/keel" "$KEEL_BIN/keel.before-launcher"
fi
install_launcher "$KEEL_BIN/keel"

if [ -d "$KEEL_HOME/.git" ]; then
    echo "keel: $KEEL_HOME/.git is the old single-tree layout, left from an" >&2
    echo "keel: earlier release -- nothing here uses it now; you may delete" >&2
    echo "keel: $KEEL_HOME/.git and $KEEL_HOME/tool" >&2
fi

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

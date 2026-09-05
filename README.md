<p align="center">
  <img src="assets/logo.png" alt="Keel" width="340">
</p>

# keel

The tool of the Keel method. It computes state; it never writes prose.

The method itself — what has to be true — is [METHODOLOGY.md](METHODOLOGY.md):
chapters and numbered paragraphs, cited as `§4.2`. Every refusal this tool
prints names the paragraph it comes from. MIT licence.

## Why

An agent writes fast and convincingly, which is exactly why it is hard to watch.
It says "done" and did something else; it adds a file nobody planned; it writes
a test that is green and proves nothing. Every one of those failures is quiet.

Keel makes the work **derivable** and the state **underivable by hand**: a
scenario becomes a test, a transform becomes a commit, a file list becomes a
boundary — and no status is ever typed by a person, it is computed from git and
from the documents, so it cannot lie.

Two entities carry it. A **wave** is what we are doing now. A **contract** is a
promise that outlives the wave. Everything else is derived from those two.

## A wave is one file

The header is machine-read, so it is English; the prose is the project's
language, because a person reads it.

`keel/waves/0007-session-loop.md`

```markdown
---
depends_on: [0006-graph-tools]

scenarios:
  finishes-when-no-tool-called:
    proves: session-run@7c40de
    covers: [functional.correctness]

transforms:
  drive-turns-on-the-model:
    implements: [finishes-when-no-tool-called]
    contracts:  [session-run@7c40de]
    files:
      - src/session.rs
      - tests/session_test.rs

decisions:
  performance.capacity: "not applicable"
  # ... every one of the forty cuts gets exactly one answer (§10.3)
---

## Why

One conversation with a model against a toolset handed in from outside.

## scenario: finishes-when-no-tool-called

**Given** an opening context and an empty toolset,
**When** the model answers with text and calls nothing,
**Then** the conversation ends in `finished`, and the trace holds the turn.

## transform: drive-turns-on-the-model

Drive turns while the model keeps calling tools. There is no attempt counter — a
tool's refusal is an answer, and the next turn is the retry.
```

The test names the promise it proves, and **which revision** of it:

```rust
/// proves: finishes-when-no-tool-called@a3f1c0
#[test]
fn the_conversation_finishes_when_no_tool_is_called() { … }
```

Those hashes are the point. Reword the scenario and `keel check` says the proof
has gone stale instead of leaving it green (§5.5). Reword the contract and every
transform that leaned on the old text is named (§5.7).

## The cycle

A **full** wave rides two branches and two pull requests; a **light** one rides
a single branch. The weight is derived from the file, never typed: light means
one transform, no contract created or changed, nothing withdrawn (§6.8).

```
PLAN                     branch plan/0007-session-loop
  keel plan <slug>         the scaffolding; the content is written by hand
  keel check               the plan judged: forty cuts, links, shape
  PR → a person reads → merge          approval IS the merge (§6.6)

WORK                     branch 0007-session-loop
  keel next                one step: transform, files, boundaries, scenarios
  git commit -m "red: <scenario>"      the test is seen failing first (§6.3)
  git commit -m "<transform>: …"       the work
  keel check               what is wrong right now
  ⟳ while keel next hands out another

  keel review              the package for a fresh reviewer (§9.9)
  keel close               the closing court: the battery, three times
  PR
```

Nothing about the stage has to be remembered. `keel next` hands over a package
holding everything needed for one move and nothing beyond it, and the
session-start hook puts the current state into the agent's context by itself.

## Installing

The tool is a Rust crate in `tool/`; git and cargo are all it needs.

```bash
curl -fsSL https://raw.githubusercontent.com/Codcore/keel/main/install.sh | sh
```

That clones the repository into `~/.keel`, builds the release binary and puts
`keel` in `~/.local/bin`. Run it again to update. `KEEL_REPO`, `KEEL_HOME` and
`KEEL_BIN` override all three. By hand it is the same three lines:

```bash
git clone https://github.com/Codcore/keel ~/.keel
cargo build --release --manifest-path ~/.keel/tool/Cargo.toml
install -m755 ~/.keel/tool/target/release/keel ~/.local/bin/keel
```

Then, in the project you want to work in:

```bash
keel init
```

`init` asks a handful of questions — language, adapter, mode, agents, CI command
— and writes `keel.toml` plus the integrations below. `keel setup` changes any
answer later; `keel update` refreshes what a newer release generates.

The methodology and the checklist travel **inside the binary**, so a project
carries no copy that can drift.

## Commands

| command | what it does |
|---|---|
| `keel check [dir]` | judges the documents and the branch |
| `keel status [dir]` | the state of every wave |
| `keel next [--for <agent>] [dir]` | the one next step, as a package |
| `keel plan <slug> [dir]` | lays a wave's scaffolding |
| `keel new contract <slug> [dir]` | a contract's scaffolding |
| `keel rev [--write] [dir]` | revisions of scenarios and contracts |
| `keel review [dir]` | the package and briefing for a reviewer |
| `keel close [dir]` | the closing court — runs the battery three times |
| `keel map [dir]` | the quality map, forty cuts |
| `keel cuts [dir]` | the cuts themselves, with their questions |
| `keel method [§N.M \| chapter] [dir]` | the methodology this binary carries |
| `keel concept [dir]` | the concept this project leans on |
| `keel init [flags] [dir]` | the method's frame in a project |
| `keel setup [flags] [dir]` | change what `init` asked |
| `keel update [dir]` | refresh the generated integrations |
| `keel trust [dir]` | record trust for `verify` / `ci` commands (§7.16) |
| `keel hook [dir]` | install the commit-msg hook |
| `keel gate <message-file> [dir]` | the court over one commit |
| `keel version [dir]` | the version and what it holds |

Exit codes: `0` green, `1` findings, `2` a refusal. `--help` and `--version` are
answered anywhere; an unknown flag or a second path is refused, never read as a
directory.

## What the courts actually check

`keel check` is the one you run constantly. It judges, and says aloud what it
could **not** judge rather than painting green over it:

- headers against the methodology's vocabulary, and header ↔ body both ways;
- every link: cuts, `implements`, `depends_on`, successors — and silence is a
  finding, since each of the forty cuts needs exactly one answer (§10.3);
- **scope**: the branch's diff against the merge-base, both ways — a file
  touched but not declared, and a file declared but untouched (chapter 4);
- **revisions**: a test tag whose hash no longer matches the scenario's text,
  and a tag that vanished since the fork point (§5.5, §7.15);
- **the red birth**: work committed for a promise that was never seen failing
  (§6.3, §7.12);
- **contract form**: the promised signatures, found in the module's own source
  (§7.6) — and a module the code does not have is a finding, not silence;
- **trust**: the `verify` and `ci` commands against their recorded fingerprints,
  so a command cannot be swapped underneath the courts (§7.16).

`keel close` is the heavier one: it runs the project's battery three times in
its own target directory, because an inherited cache shifts verdicts.

## Two languages

`lang` in `keel.toml` picks the tool's own language — `uk` or `en`. It decides
every message, every refusal, and which normative text `keel method` and
`keel cuts` serve. The Ukrainian text is the source of truth (§1.8); the English
one records the revision it was translated from, and a stale record is a finding.

## Adapters — and the honest state of them

Two adapters exist: **`rust`** (`"cargo"` accepted) and **`ruby`** (minitest).
Both run the language-shaped courts — the `proves:` tags are read from the
project's test files, and a contract's `exports` are compared against the
module's own source, wherever that language keeps it.

Name a language this release does not know and you get a finding with the list
of the ones it does — never a silent skip. Name none at all and every other
court still runs: documents, links, scope, revisions, and the tool says which
ones it skipped instead of leaving them green.

The concept's starting set is **Elixir, Ruby, Python, TypeScript/JavaScript**.
Ruby is built; the other three are not, and RSpec is not read yet either — the
ruby adapter is minitest. That is the largest remaining gap, and it is named
here rather than left for a reader to discover.

| language | tests | one test | module source |
|---|---|---|---|
| `rust` | `tests/*.rs` | `cargo test --test <file> <fn> -- --exact` | `src/<name>.rs`, `src/<name>/mod.rs` |
| `ruby` | `test/**/*_test.rb` | `ruby -Itest <file> -n <method>` | `lib/<name>.rb` for `Name`, `lib/a/b.rb` for `A::B` |

An honest limit of the ruby adapter, and §7.12 foresaw it: ruby does not tell
"failed" from "did not load" by its exit code — both are 1. The adapter reads
the text (`SyntaxError`, `LoadError`), and where the text does not say, it takes
a failure as a failure: the direction that cannot turn red into green.

What an adapter has to answer is small and written down:

| question | why |
|---|---|
| where do the test files live | the `proves:` tags are read from them (§5.5) |
| how to run exactly one test | the red birth is judged by watching it fail (§7.12) |
| how to run the whole battery | `keel close` runs it three times (§7.13) |
| how to read a module's source | a contract's `exports` are compared against it (§7.6) |
| can it tell "failed" from "did not build" | §7.12; where it cannot, it accepts any failure and says so |

A contract may also carry `verify: <command>`, and that road is
language-independent: `keel close` runs the command and records its fingerprint,
so it cannot be swapped later (§7.16). Until your language has an adapter,
`verify` is how a contract can still be held by machine.

## What `init` puts in a project

| file | what it is |
|---|---|
| `keel.toml` | the answers, the version pin, trust and generated digests |
| `keel/waves/`, `keel/contracts/`, `keel/reviews/` | the documents |
| `AGENTS.md` | a keel block, appended; text above it untouched |
| `.claude/skills/keel/SKILL.md`, `.agents/skills/keel/SKILL.md` | the skill |
| `.claude/settings.json`, `.cursor/hooks.json` | the agent hooks |
| `.github/workflows/keel.yml` | `keel check`, `keel close`, the battery |
| `.git/hooks/commit-msg` | the commit court (§8.4, §7.12) |

Every generated file is recorded by digest. Edit one by hand and `keel update`
refuses to overwrite it, saying so — it never touches what it did not write.

## Modes

`keel init --mode` answers who may start a procedure and whether anything
watches while it runs:

| mode | who starts a procedure | agent hooks |
|---|---|---|
| `strict` (default) | the agent, on its own judgement | installed |
| `soft` | the agent, on its own judgement | none |
| `manual` | only you, by typing the slash command | none |

The agent hooks read what the agent is about to write and refuse a file the
current wave does not declare. The git hook is separate and is always installed:
it holds the commit grammar and the red birth.

## State

Nothing here is a status field. The stage of a wave is derived from git and from
the documents every time it is asked, which is why `keel status` cannot be wrong
and cannot be edited. A session that dies loses nothing: the next agent runs
`keel next` and continues from the same place (§9.10).

## The honest border

A green check means *the test exists, its revision matches, and it passes* — not
that the promise is proven in essence. No mechanism closes that gap; a fresh
reviewer does, with the four questions of §9.9. The tool says this in its own
verdict rather than letting a green line imply more than it holds (§7.8).

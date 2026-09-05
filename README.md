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

That clones the repository into `~/.keel/source`, builds a release binary into
`~/.keel/versions/<ref>/`, and puts a **launcher** at `~/.local/bin/keel`. Run
it again to update. `KEEL_REPO`, `KEEL_HOME` and `KEEL_BIN` override all three.

**Versions stand side by side.** Each installed ref gets its own home, so two
projects on two different pins work at the same time. The launcher reads the
`version` a project pins in `keel.toml` (honouring `-C`) and runs exactly that
one; a pin nobody installed is a refusal naming what *is* here and the command
that brings what is not. It never runs a different version — the wrong binary
in silence is worse than a refusal. Before it hands over it checks the binary
is the one that was installed. `keel version` lists what stands here.

A **version** may be named — first argument or `KEEL_REF` — and then exactly
that git ref is installed:

```bash
KEEL_REF="<tag or commit>" sh install.sh
curl -fsSL https://raw.githubusercontent.com/Codcore/keel/main/install.sh | sh -s -- <tag or commit>
```

A ref that is not there refuses and lists what the clone knows. This is what
`keel version` prints when `keel.toml` pins a version the running binary is
not — the advice names the command with the pin already in it, and the border
with it: `KEEL_REF` takes a **git ref by name**, while `version` holds a number,
and the two are only the same word once a tag carries that name. Where
`keel.toml` pins a version, the generated CI step carries that pin too.

`cargo` writes its own registry and cache into `CARGO_HOME` (`~/.cargo` by
default, tens of megabytes on a first build) — that is cargo's home, not keel's,
and the installer does not move it.

By hand it is the same three lines:

```bash
git clone https://github.com/Codcore/keel ~/.keel
cargo build --release --manifest-path ~/.keel/tool/Cargo.toml
install -m755 ~/.keel/tool/target/release/keel ~/.local/bin/keel
```

Then, in the project you want to work in:

```bash
keel init
```

**What is not built, said here rather than discovered later:** a keel release is
a **git ref fetched by name**. The commit sha is recorded and the binary's own
sha256 is checked before every run, so you always know *which tree* you got and
that nobody swapped the file — but nothing proves the *ref itself* is
trustworthy. A signed, published release with a checksum of its own is not
built. Nor does the launcher fetch a missing version by itself: it refuses with
the command instead.

**No published tag carries the current layout** — keel v1 kept the crate outside
`tool/`, so `KEEL_REF=v0.8.9` refuses by name and only a commit or a branch
works until a v2 release is tagged. The installer the generated CI step fetches
comes from `main`, unpinned: a project pinned to an older keel still runs
today's script.

## For scripts

Every command takes `-C <dir>` (where to work) and `--branch <name>` (which
branch to believe **where git does not know it** — a CI checkout with a detached
HEAD). `--branch` never overrules git: where git has a branch, git is the fact,
and the tool says aloud that the flag was not used rather than dropping the word
in silence.

The reading commands — `check`, `close`, `status`, `next`, `map`, `review`,
`version`, `cuts`, `rev` — also take `--json`, and then print one JSON object
and nothing else:

```json
{"keel":1,"command":"check","ok":false,"exit":1,"root":"…","lang":"uk",
 "structured":true,
 "findings":[{"file":"keel/waves/0001-a-wave.md","reason":"…","instead":"…"}],
 "limits":["…"],"summary":{"documents":62,"findings":1,"limits":1},
 "report":"…the prose verdict, byte for byte…"}
```

A refusal is the same envelope with `refusal` carrying `file`, `reason` and
`instead`. Without the flag the output is byte-for-byte what it always was.

Two borders, said here rather than found later: the package carries the
structure the courts already computed and the whole prose in `report` — it does
not turn every sentence into a typed field, and `structured` says so. And the
commands that *write* (`init`, `setup`, `plan`, `new`, `update`, `gate`,
`hook`, `trust`) have no `--json`: they tell a person what they did in their
project, and a harness that wants the outcome asks `check` afterwards.
`concept` and `method` do take it — they write nothing, they read the norm.

`report` is what a person sees on stdout, to the byte. One thing is not in it:
the price line `keel close` prints on **stderr before** it starts work, which is
a warning ahead of the verdict rather than part of it.

## What `init` asks

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

**The generated CI runs where keel itself runs the battery.** The question goes
to the adapter, because only the adapter knows which directory it works from:
for rust that is the crate's root, and for **ruby and elixir it is always the
repository root**, since both adapters run from there. A `working-directory` is
written only when that differs from the repository root; where the adapter
cannot say — no crate, several, or one deeper than a level — the file says so
in a comment rather than leaving a step to fail on a runner without a reason.

Until wave 0044 the step was `cargo test` at the root, so a project whose crate
sits in a subdirectory got `could not find Cargo.toml` from a file keel had
written for it. keel's own repository is that shape, and its own CI had been
saying so. The first cut of the fix looked for `Gemfile` and `mix.exs` itself,
and that was worse: CI then ran a *different* battery from the one the courts
judge, and a red tree came out green.

**And it names the toolchain it judged with**, for a tongue that has one: a
project carrying `rust-toolchain.toml` gets its channel installed by name, and
one carrying none gets the truth in the file — the courts take whatever the
runner has that day, which is repeatable only by accident. keel does not pin on
your behalf; the pin is your project's decision. It made that decision for
itself after `clippy -D warnings` came out clean on 1.94 and red on 1.98, on the
same tree, because a lint had been added in between.

The channel is read with a TOML reader and must be a channel **name** —
letters, digits, dot, dash, underscore, and the whole of the value. The first
cut took the text after `channel` by hand and put it straight into a `run:`
line, so a pin could write any command at all into the workflow keel generates.
A value this release cannot vouch for is treated as no pin: the file then says
none is named, which is true and harmless.

**A test it watched fail holds the wave open** — whoever claims it. That reads
obvious and was not true until wave 0043: blockers were counted only from the
uncovered promises of the branch's own wave, so a red test no scenario named
was printed by name and then closed over, with exit 0. Measured that way in
rust, ruby and elixir alike, which is why the fix is in the court and not in
any adapter. A flaky test blocks the same way: three runs exist precisely so
flakiness is visible, and a flaky test is not a green one. Waves closed in
earlier generations keep their verdict — their promises were proven at their
time, and today's red is not their lack.

The same wave took a second false green out of `keel check`: a declaration
written inside a multi-line text — an elixir `@moduledoc`, a ruby heredoc, a
rust `r#"..."#` — used to hold a contract's `exports` as if it were live code.
Comments were already not code; text is not code either, in all three tongues,
and the finding now names the file it looked in.

That rule is a **reader per tongue, not a rule per mark** — and the first cut of
it, which was three passes over three marks, is why the reviewer sent the wave
back. Rust is read in one pass the way rustc reads it (nested `/* */`, raw
strings with their hash count, ordinary strings with their escapes, a char
literal told from a lifetime); before that, `ident.strip_prefix("r#")` in `syn`
opened a raw string that was never open and **17 of the 3419 crates** in the
local registry lost a live declaration — with the same trap already standing in
keel's own source. Ruby and Elixir are read **line by line, deliberately**:
ruby writes `$'` for the post-match and `?'` for a character, and carrying quote
state across lines to catch a multi-line string cost **513 live `def`s across 87
files** of ruby's own library. A heredoc opens only when its word really stands
alone on a line below, so a shovel, an example inside a string, and any heredoc
shape this reader does not know all leave the file alone.

The direction is chosen and stated: this court may let a ghost through and say
so — the borders are listed in `BACKLOG.md` — but it must not refuse a promise
that is alive. A court that refuses live code is not a stricter court; it is a
broken one. Measured after the rewrite: zero live declarations lost across both
corpora.

## Two languages

`lang` in `keel.toml` picks the tool's own language — `uk` or `en`. It decides
every message, every refusal, and which normative text `keel method` and
`keel cuts` serve. The Ukrainian text is the source of truth (§1.8); the English
one records the revision it was translated from, and a stale record is a finding.

## Adapters — and the honest state of them

Three adapters exist: **`rust`** (`"cargo"` accepted), **`ruby`** (minitest) and
**`elixir`** (`"mix"` accepted, ExUnit).
All three run the language-shaped courts — the `proves:` tags are read from the
project's test files, and a contract's `exports` are compared against the
module's own source, wherever that language keeps it.

Name a language this release does not know and you get a finding with the list
of the ones it does — never a silent skip. Name none at all and every other
court still runs: documents, links, scope, revisions, and the tool says which
ones it skipped instead of leaving them green.

The concept's starting set is **Elixir, Ruby, Python, TypeScript/JavaScript**.
Ruby and Elixir are built; Python and TypeScript/JS are not, and RSpec is not
read yet either — the ruby adapter is minitest. That is the largest remaining
gap, and it is named here rather than left for a reader to discover.

| language | tests | one test | module source |
|---|---|---|---|
| `rust` | `tests/*.rs` | `cargo test --test <file> <fn> -- --exact` | `src/<name>.rs`, `src/<name>/mod.rs` |
| `ruby` | `test/**/*_test.rb` | `ruby -Itest <file> -n <method>` | `lib/<name>.rb`, `lib/<name>/init.rb`, `app/<name>.rb` — `A::B` is `a/b.rb`, and an acronym stays one word (`HTTPServer` → `http_server`) |
| `elixir` | `test/**/*_test.exs` | `mix test --only 'test:test <name>'` | `lib/<name>.ex` — `A.B` is `a/b.ex`, acronyms as above |

The ruby battery reads minitest's own verbose voice, so a test file that does
not load is a refusal aloud rather than a page of green: without a run there is
no verdict for anyone. A `.rb` file in `test/` that is not named `*_test.rb` is
not read, and the check says which ones those were.

**Elixir tells the two apart, and the tool says so.** `mix test` leaves with 0
green, **2 on a failure and 1 on a compilation error**, so a broken build is
judged a broken build rather than a red test — and `keel check` prints that,
not ruby's border. A border that is not about your project is as untrue as one
left unsaid. Two smaller things measured there: an ExUnit test is named by a
*string*, and inside a `describe` block ExUnit puts the block's name in front,
so the tool builds the full name rather than calling it a limit.

The border Elixir does share with Ruby is named too: neither writes types in a
`def`, so §7.6 compares a name and its parameters and nothing more. The other
half of that border is gone — a `def` written inside a `@moduledoc` used to pass
for a live one, and wave 0043 took it away in all three tongues at once.

An honest limit of the ruby adapter, and §7.12 foresaw it: ruby does not tell
"failed" from "did not load" by its exit code — both are 1. The adapter reads
the text (`SyntaxError`, `LoadError`), and where the text does not say, it takes
a failure as a failure: the direction that cannot turn red into green. `keel
check` prints that border itself, next to a second one: ruby writes no types, so
the §7.6 form court compares a method name and its parameters and nothing more.

Adding a language is a module, a row in `Language::NAMES`, the dictionary in
both tongues, and **fourteen** places where something branches on the tongue —
counted off the source rather than guessed, because the number that stood here
before (six, beside a list of seven) was neither:

`adapter::builds_heavily`, `build_dir`, `tests_dir`, `run_line`, `test_files`,
`run_test`, `run_all`; `config::battery_command`; `holding::comparability` (the
module layout) and `holding::strip_comments` (the comment shape);
`tags::scan_text` (the declaration shape), `tags::marks` and `tags::declares`
(these three keyed by the file's extension, never by the project's config — see
below); and `check` for the tongue's own limits.

Not "one file". Wave 0042 paid exactly that price for Elixir.

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
| `.github/workflows/keel.yml` | a step that installs keel, then `keel check`, `keel close`, the battery |
| `.git/hooks/commit-msg` | the commit court (§8.4, §7.12) |

Every generated file is recorded by digest. Edit one by hand and `keel update`
refuses to overwrite it, saying so — it never touches what it did not write.

## Modes

`keel init --mode` answers **who may start a procedure**, and nothing else:

| mode | who starts a procedure |
|---|---|
| `strict` (default) | the agent, on its own judgement |
| `soft` | the agent, on its own judgement, and the commit court is advisory |
| `manual` | only you, by typing the slash command |

Two things this table used to claim and does not:

- **The agent hooks are not switched by `--mode`.** They are written whenever
  `hooks` is on, in every mode; `--no-hooks` (or `hooks = false`) is the switch,
  and it turns off the git hook too.
- **The agent hooks do not read what the agent is about to write.** What is
  generated today is a *session-start* hook that runs `keel next` and puts the
  current step into the agent's context. A hook that judges a write before it
  happens is not built.

The git hook is separate: it holds the commit grammar and the red birth (§8.4,
§7.12) — and it is installed unless you answered `hooks = false`, in which case
nothing holds those two but your own care. The keel block in `AGENTS.md` and
the skill say which of the two your project **asked for**: where `hooks` is off
they say plainly that no commit judgement runs and name what still judges
(`keel close`, `keel check`).

Whether a hook really stands on *this* machine is a different question, because
**git does not clone hooks**: a fresh clone and a CI runner have none. The block
cannot know that — it is compared by digest across every machine — so `keel
check` says it instead, as a limit, on any clone whose block promises a machine
and where no hook of ours is installed. `keel hook` puts one back.

## State

Nothing here is a status field. The stage of a wave is derived from git and from
the documents every time it is asked, which is why `keel status` cannot be wrong
and cannot be edited. A session that dies loses nothing: the next agent runs
`keel next` and continues from the same place (§9.10).

## The honest border

A green check means *the test exists, its revision matches, and it passes* — not
that the promise is proven in essence. No mechanism closes that gap; a fresh
reviewer does, with the four questions of §9.9. `keel check` prints that border
in its own verdict rather than letting a green line imply more than it holds
(§7.8) — `keel close`, the heavier court, does not yet, and says "every live
scenario proven" where it means the same narrower thing.

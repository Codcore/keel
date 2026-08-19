<p align="center">
  <img src="assets/logo.png" alt="Keel" width="340">
</p>

# keel

The tool of the Keel method. It knows the state and writes no prose.

The method — what has to be true — lives in [METHODOLOGY.md](METHODOLOGY.md), in
chapters and numbered paragraphs, cited as `§4.2`. This file is what runs it.

MIT licence.

## Contents

- [What it solves](#what-it-solves)
- [What it looks like](#what-it-looks-like)
- [The cycle](#the-cycle)
- [Installing](#installing)
- [Commands](#commands)
- [CI is a command the project names](#ci-is-a-command-the-project-names)
- [Two languages, and they are independent](#two-languages-and-they-are-independent)
- [Language adapters](#language-adapters)
  - [Exports, short and long](#exports-short-and-long)
  - [A contract with `verify`](#a-contract-with-verify)
- [How Keel gets into a project](#how-keel-gets-into-a-project)
- [Git hooks](#git-hooks)
- [Agent hooks](#agent-hooks)
  - [The session hook points at a skill](#the-session-hook-points-at-a-skill)
  - [The hook before a write](#the-hook-before-a-write)
- [Three modes](#three-modes)
- [Skills](#skills)
- [State](#state)
- [Open](#open)

---

## What it solves

An agent writes fast and convincingly, which is exactly why it is hard to watch.
It says "done" and did something else; it adds a file nobody planned; it writes a
test that is green and proves nothing. Every one of those failures is quiet:
everything looks fine.

Keel does two things. **Predictable** — what was agreed is written so that the
work is derived from it: a scenario becomes a test, a transform becomes a commit,
a file list becomes a boundary. **Traceable** — no status is ever written by
hand; it is computed from git and from the documents, so it cannot lie.

There are two entities: the **wave**, which is what we are doing now, and the
**contract**, a promise that outlives the wave. Everything else is derived.

## What it looks like

One wave, one file. The header is English, because its fields become code; the
prose is the project's language, because a person reads it.

`keel/waves/0007-session-loop.md`

```markdown
---
depends_on: [0005-flat-tools, 0006-graph-tools]

scenarios:
  finishes-when-no-tool-called:   {proves: session-run@7c40de}
  only-handed-tools-are-callable: {proves: session-run@7c40de}

transforms:
  drive-turns-on-reqllm:
    implements: [finishes-when-no-tool-called, only-handed-tools-are-callable]
    contracts:  [session-run@7c40de]
    files:
      - lib/keel_agent/session.ex
      - test/keel_agent/session_test.exs
---

## Why

One conversation with a model against a toolset handed in from outside.

## scenario: finishes-when-no-tool-called

**Given** an opening context and an empty toolset,
**When** the model answers with text and calls nothing,
**Then** the conversation ends in `:finished`, and the trace holds the turn.

## transform: drive-turns-on-reqllm

Drive turns while the model keeps calling tools.

Boundaries: there is no attempt counter — a tool's refusal is an answer,
and the next turn is the retry.
```

The contract sits in its own file, because it outlives the wave that created it:

```markdown
---
module: KeelAgent.Session
exports:
  - "run(Context.t(), [Tool.t()], Config.t()) :: Outcome.t()"
---

One conversation with one model.
```

And the test names the scenario it proves — and which revision of it:

```elixir
@tag proves: :finishes_when_no_tool_called, rev: "a3f1c0"
test "the conversation finishes when the model calls no tool" do
  outcome = Session.run(opening(), [], config())

  assert outcome.stop == :finished
end
```

The three hashes in that example are not decoration. `session-run@7c40de` says
**which revision** of the promise the transform implements, and `rev: "a3f1c0"`
which revision of the scenario the test proves. Reword either text and the check
says the proof has gone stale, instead of leaving it green.

## The cycle

Two pull requests per wave: the plan separately, the work separately. The
operator starts each stage with a single command; the tool drives the rest.

```
PLAN                                branch plan/0007-session-loop

  /keel-plan session-loop           the operator calls it
      keel new wave                   the file skeleton
      the agent writes                why → scenarios → transforms with files
      the agent asks                  what is unsettled — as a question, not a guess
      keel gaps                       what is missing
  PR → a person reads → merge       approval = the plan is on the main branch

WORK                                branch 0007-session-loop

  /keel-work                        one invocation, one transform
      keel next                       transform, files, boundaries, scenarios, contracts
      the agent works                 strictly inside the named files
      keel check                      what is wrong right now
      git commit                      the transform slug in the message
  ⟳ while keel next hands out another

  /keel-review                      the full check and the "what did we stay silent about" question
  PR
```

`keel next` hands over a **package**: everything needed for one move and nothing
beyond it. The agent opens no documents around it — it gets a slice and works
with that.

The stage does not have to be remembered. The session-start hook asks the tool
which state the wave is in, and puts the answer into the context along with the
name of the skill to take.

## Installing

```bash
curl -fsSL https://raw.githubusercontent.com/Codcore/keel/main/install.sh | sh
```

It clones the repository into `~/.keel` and puts a `keel` command in
`~/.local/bin`. Run it again to update — the second run is a `git pull`.
`KEEL_REPO`, `KEEL_HOME` and `KEEL_BIN` override all three.

A single downloaded file would not do: `keel init` copies the references into the
project beside the tool, so the whole repository has to be somewhere on disk.

**Hence two levels, and they are easy to confuse.** `~/.keel` is the method's
home, your clone of the repository; `keel/` inside a project holds the copies
`init` put there. Different commands move them:

| what is updated | with | from where |
|---|---|---|
| the home `~/.keel` | the same `curl` line again | GitHub |
| the project's copies | `keel update` | the home |

`keel update` **fetches nothing from the network** — it only carries into the
project what already sits at home. So when the method moves on, the home is
updated first and the project second.

Then, in the project you want to work in:

```bash
keel init
```

git and Python 3 are needed. Nothing else.

**Standard library only, and this is not asceticism.** The method does not depend
on the project's language, so the tool should not either; Python is on every
machine and in every CI. The YAML here is a narrow subset — keys, lists, two
levels of nesting — and a hundred-line reader of one's own is cheaper than a
`pip install` at the start of every project. A tool that holds a method together
should not itself be a dependency.

Tests:

```bash
python3 -m unittest discover -s tests -t .
```

They are laid out by what they check: `test_yamlish`, `test_documents`,
`test_scope`, `test_adapters`, `test_commands`, `test_git_hooks`,
`test_agent_hooks`, `test_skills`, `test_setup`, `test_ci`, `test_modes`,
`test_language`, `test_install`. Each runs on its own
(`python3 -m unittest tests.test_scope`); the shared fixture is in
`tests/support.py`. Tests are copied nowhere, so splitting them costs nothing —
unlike the tool itself.

One test is optional: if PyYAML is installed, it reads the generated skill
headers with a real parser. Our own reader is forgiving, and the headers will be
read by Claude and Cursor, not by it.

## Commands

| Command | What it does |
|---|---|
| `keel new wave <slug>` | the file skeleton: a header with empty fields and stub sections |
| `keel new contract <slug>` | the same for a contract: `module` with `exports`, or `verify` |
| `keel gaps [wave]` | what is missing from a wave's description: slugs without sections, transforms without files, scenarios without `proves`. And it asks about a forgotten edge: the wave declares a file another wave declares too while `depends_on` does not name it |
| `keel next` | the **package** for the next move: the transform, its files and boundaries, the scenarios it brings closer, the bodies of the contracts it leans on — and nothing beyond. On the main branch it names instead the wave that is ready to be worked and the branch to take. Markdown; `--json` for scripts |
| `keel check` | every check — the full gate. `--fast` leaves those that run nothing, `--no-tests` skips the test run and the CI command, `--branch` names the branch where git does not know it, `--json` for scripts |
| `keel rev` | shows revisions that have drifted apart; `--write` records the new ones |
| `keel show` | a wave as a person reads it: the why, the scenarios and the transforms, with what is closed and what is not |
| `keel hooks` | shows the state of `pre-commit` and `pre-push`; `--install` installs them, `--force` overwrites somebody else's |
| `keel skills` | regenerates the skills from the method |
| `keel init` | installs Keel into a project: directories, a copy of the tool, the references, `AGENTS.md`, CI, hooks. `--docs` and `--lang` set the languages, `--adapter` names the project's language, `--mode` how much installs itself, `--ci` the project's own gate, `--force` overwrites somebody else's hook |
| `keel hook <event> --agent` | answers an agent hook; called by a config, not by hand |
| `keel update` | brings the project's copies up to date. `--diff` shows the difference, `--force` overwrites even hand-edited files |

`gaps` and `check` are separate precisely because a scenario without a test is
fine while planning and not fine before a PR. One command with two modes would
confuse both. And neither of them plans: `gaps` says what is missing, `check`
says what is wrong.

## CI is a command the project names

Keel checks documents against facts. Whether the project builds, passes its own
linter and its own suite is the project's business, and only the project knows
the command. So it **names** one, and the condition is simply that the command
succeeds — the same shape a contract's `verify` already has, for the same
reason: who makes the promise does not matter, that it can be checked does.

The `ci` key in `keel/keel.json`, and three states:

| value | what it means | what `check` does |
|---|---|---|
| `"mix ci"` | a command is named | runs it; red when it fails |
| `""` | **nobody has decided** | says so on every run, without turning red |
| `"none"` | a refusal out loud | silent |

The middle state is the whole point. A merge going through with nothing of the
project's own run, and nobody knowing it, is the same silence the rest of this
tool stands against. A project with no CI stays green — but only once that has
been said out loud. Refusal is not silence; the quality cuts already live by
that rule.

The adapter **proposes** a command where the language has a convention for one:
for Elixir that is `mix ci`. Python has no such convention, and inventing one
would hand the operator a command that was never true — so it is left empty and
said. `keel init --ci "<command>"` names it at install time.

It runs in the full `check`, not in `--fast`: a commit may be half-written, a
push and a merge may not. Which means pre-push, the CI workflow and
`/keel-review` all get it without any wiring of their own.

On a `plan/*` branch `check` knows where it is. Checks 5 and 6 do not run there —
a plan branch carries no code by design — and `gaps` runs in their place. Without
that a plan branch could be neither pushed nor merged: pre-push and CI run the
full `check`, those two would fail every time, and red on a plan PR would soon
stop meaning anything. `--fast` leaves it out deliberately: a commit on a plan
branch may be half-written, a push and a merge may not.

The `-C DIR` flag says where to work. The project root is searched upwards: the
first directory with `keel/waves/`, or the first with `.git`.

## Two languages, and they are independent

`keel init` records two of them in `keel/keel.json` — the third is the mode —
and the two should not be conflated:

| Key | What it decides | Flag |
|---|---|---|
| `docs` | which language the references arrive in | `--docs uk\|en` |
| `lang` | what the agent writes waves and commits in, and which phrases the skills catch | `--lang uk\|en` |
| `ci` | the project's own gate — a command, `""` for undecided, `"none"` for a refusal out loud | `--ci "<command>"` |

They are separate because wanting an English reference with Ukrainian triggers is
a reasonable thing to want. A skill's body depends on neither: it is read by the
model, and it is always English. So is what neither of them reads: the CI
workflow and the git hook scripts, down to the message somebody meets on a
failing push.

Those two, the mode, the adapter, the agent-hooks override and the CI command
are what Keel keeps in that settings file, along with the digests of every file
it generated,
but not the whole of what may live there: the file is yours, it sits in your
repository, and keys Keel does not know survive a rewrite. Neither language can
be guessed: the language of a project's prose is a team's decision, not a
property of the code.

`lang` also decides what this tool says, and what language `keel new` writes a
wave or contract skeleton in. The header fields are the same either way — they
become code — while the hints and the Why heading follow the project; the reader
accepts both spellings, so a project may change language without its existing
waves becoming unreadable. The tool's messages are keyed by their English
text, so a missing translation degrades to readable English rather than to an
error. The command line itself stays English in both: flag names, metavars and
`--help` are the interface's own vocabulary, like `--force` is.

**Ukrainian is the source, English leans on it.** The source lives in `docs/uk/`,
and the English translation sits at the root of the repository, because that is
what a first-time visitor opens. Which one is the source and which one is on
display are separate questions.

The revision each translation leans on is recorded in `docs/revisions.json`
rather than in the file's own header: the root `README.md` is the repository's
front page, and a bookkeeping header would render as a table above its first
line. `keel update` turns red when the source has changed and the translation has
not. It is the same rule that holds tests and contracts together: whoever leans
on a text holds its revision. Bilingual documents die precisely from a
translation quietly falling behind.

## Language adapters

Two of the checks depend on the language: what runs the tests (§7.5) and where a
module's exports come from (6). The adapter is chosen by a marker in the project
root.

| | Elixir | Python |
|---|---|---|
| marker | `mix.exs` | `pyproject.toml`, `setup.py`, `setup.cfg` |
| tests | `mix test` | unittest over every `test_*.py` and `*_test.py`, loaded by path |
| where tests live | `test/**/*_test.exs`, `apps/*/test/**` | `tests/**/test_*.py`, `*_test.py` |
| scenario tag | `@tag proves: :slug, rev: "a3f1c0"` | `# proves: slug, rev: "a3f1c0"` |
| exports | `mix run --no-start` asks `__info__(:functions)` | import the module, `__all__` or the public names |
| types | `Code.Typespec.fetch_specs` — a whole `@spec` is checkable | none: a promised shape is reported as unchecked |
| CI steps | `erlef/setup-beam` plus `mix deps.get` | `actions/setup-python` |

A scenario's slug and the name in the tag are compared after normalising, so
`finishes-when-no-tool` in the wave and `:finishes_when_no_tool` in the test are
the same thing.

CI versions come from the machine: `keel init` asks `elixir --version` where it
was run and writes down what it found. Where there is no Elixir to ask — which
is exactly the case `--adapter elixir` in an empty repository — it falls back to
a pinned pair, and that pair goes into the workflow. Change it there if your
project stands on different ground.

**The language's CI steps stand under a `hashFiles` condition, for a reason.** A
language may be named before its marker exists — an adapter written into
`keel.json` ahead of the work, or `init --adapter` in an empty repository. Then
`mix deps.get` would run on a branch with no `mix.exs`, which is every plan
branch by design, and CI would be red for a reason that has nothing to do with
the branch. With the condition the waves simply do not run until the marker is
there.

**When more than one marker sits in the root, Keel does not guess.** A project
holding both `mix.exs` and `pyproject.toml` is a question, not a first line of a
list: the answer decides whose tests are run and who is asked for exports. Checks
5 and 6 name both languages and the one that was taken, and say how to settle it:

```json
{ "adapter": "elixir" }
```

`keel init --adapter elixir` does the same. The choice is written into
`keel/keel.json`, and the CI generator reads it too — otherwise a project would
settle on one language while its workflow installed another.

A language without an adapter does not break the whole of `check`, only checks 5
and 6, and it says so plainly. A new adapter is a class inheriting from
`Adapter`, with a marker, a test command, a tag pattern and a way to get the
exports. The inheritance is not a formality: the base class carries
`supports_specs = False`, and that is what makes the check say "this language
cannot be asked about types" instead of putting a green mark over something
nobody compared. A language that can sets it to `True`, and then `exports()`
returns the specs as well.

### Exports, short and long

Which forms may be written — §2.9. Here is how the check reads them.

```yaml
exports:
  - "run(binary(), keyword()) :: {:ok, term()} | {:error, term()}"
  - "halt/1"
```

Both are checked for what they say. `run/3` says the function exists with that
arity. The signature says that too — the arity is counted from the arguments —
and, where the adapter can read types, that the module declares this `@spec`. On
a mismatch the check prints what the module declares, in the form you would write
it, so the fix is a copy. A function with no `@spec` at all is reported rather
than passed, and so is a shape promised in a language whose adapter cannot read
types — Python today.

**Arguments may be named.** `run(text :: binary(), opts :: keyword()) :: :ok` is
exactly how the compiler hands them back, and exactly how people write them. The
separator is the `::` outside any bracket, so names inside break nothing.

**Several `@spec` lines on one function are several honest promises.** A contract
may name any of them, and the check is green if one matches. On a mismatch it
prints them all, not an arbitrary first:

```
✗ the promised shape of pick/1 is not what the module declares:
      pick(integer()) :: :small
      pick(binary()) :: :big
```

**A type is a difference; a layout is not.** Both sides are squeezed the same way
before comparison: line breaks go, spaces hugging a bracket go, exactly one space
follows a comma, exactly one sits either side of `|`. The compiler writes
`run( binary() )` and a person writes `run(binary())`, and those are the same
thing. So are `{:ok,term()}` and `{:ok, term()}`.

### A contract with `verify`

It needs no adapter — it carries a command. That command has to be a string, or
the check turns red rather than passing it over in silence, and it is bounded at
30 seconds: a promise is a probe, not a build, and a hung probe would hold
`pre-push` and CI for as long as they are allowed to run. Its stdin is closed, so
a command that prompts fails at once instead of waiting. `--no-tests` runs
neither it nor the export probe: the probe imports the project's modules, and an
import executes whatever a module does at load — so the run-nothing flag is now
honest all the way down.

**Everything that executes the project's code is bounded**, not only `verify`.
The test run gets ten minutes, the export probe two. A fixed command from an
adapter is no safer than a command from a file: `mix test` and the script that
asks a module for its exports both execute somebody else's code, and check 6 is
what `pre-push` runs. A timeout is a red check naming the command, not silence
and not a traceback; so is a missing interpreter.

**Two commands now come out of files in the repository, and that is worth
knowing.** A contract's `verify` and the `ci` key in `keel/keel.json`. The
second is the wider of the two: `verify` runs only for contracts that declare
one, while `ci` runs for any project that names one — and `init` proposes
`mix ci` to every Elixir project it installs into. `keel check` runs it through a shell, and `pre-push` runs `keel check`.
So cloning somebody's project, or checking out a branch from a pull request and
running `git push`, means executing whatever its contracts say. Until now every
subprocess Keel launched was a fixed adapter command; these are the ones set
by whoever wrote the contract. The practical conclusion is simple: read a
contract in somebody else's PR as carefully as you read code.

## How Keel gets into a project

The `keel` repository is the method's home, not what an agent reads. The agent
reads files **in the project it works on**, and `keel init` puts them there:

```
keel/waves/                     empty directories
keel/contracts/
keel/keel.json                  the settings: two languages and the mode
keel/keel.py                    the tool itself, as a copy
keel/METHODOLOGY.md             the method, as a copy
keel/README.md                  this file, as a copy
keel/QUALITY.md                 the quality cuts — walked point by point
AGENTS.md                       a block between markers: principles and pointers
.github/workflows/keel.yml      CI, generated
.claude/skills/keel-*/SKILL.md  generated
.cursor/skills/keel-*/SKILL.md  generated — the same format
.claude/settings.json           agent hooks in strict mode; anything else stays
.cursor/hooks.json              the same for Cursor
.git/hooks/                     pre-commit and pre-push, which call keel
```

**The tool travels as a copy**, and that is exactly why it is one file. CI raises
a clean machine on every push; it fetches only the project's repository, so
`keel.py` has to be in it or there is nothing to check with. It also settles the
question of whether whoever cloned the repo has keel installed.

A hook looks for the tool in order: the `KEEL` variable, then `keel` on PATH,
then the copy in the project, and last the absolute path of whatever installed
the hook. The copy in the project is the one always there; the baked path is a
fallback for a checkout that has none, and it is why `.git/hooks` is not worth
sharing between machines.

**The references travel as copies, and `AGENTS.md` points at each.** A file makes
nobody read it: four things do — `AGENTS.md`, which is always read, the
session-start hook, the hook before a file write, and the skill loaded while
planning. You can only point at what sits in the same repository. A file nothing
points at is dead.

They are read rarely, and that is fine: at work the agent needs the package from
`keel next`, while planning it needs the skill. But "rarely" is not "never". When
it is unclear what goes in a transform's header or how a revision is computed,
there has to be somewhere to look.

**The principles are not a separate file.** Seven statements are ten lines, and
they go into `AGENTS.md`, which the agent always reads. The text is taken from
`PRINCIPLES.md`, so there is no way for them to drift apart.

`AGENTS.md` is **appended to, not created**: a block between `<!-- keel:start -->`
and `<!-- keel:end -->`, the rest of the file belongs to the project, and updates
do not touch it. A project has its own `AGENTS.md` almost always, and overwriting
it would mean taking something away.

`.claude/settings.json` is merged the same way: other settings stay, our entry is
not duplicated on a repeated `init`, and if the file is broken the tool leaves it
alone and says so.

The skills are installed **as both sets at once**: they are cheap, and which
agent you will be working with tomorrow is unknown.

**`init` commits what it wrote** — as its own commit, staging only its own
files, so anything of yours sitting uncommitted next to it stays yours.
`--no-commit` if you would rather do it. Without git it refuses outright: Keel
reads all of its state from git, and creating somebody's repository is a bigger
decision than installing a method.

**The agent is started in the project directory itself.** Skills are taken from
the starting directory and its parents; ones that sit below it are only picked up
once the agent reads a file there.

Right after the install the first call may answer "Unknown skill" — the skills
have not been picked up yet. `/reload-skills` fixes it, and so does simply
calling again; a session opened before the install has to be restarted, because
`/clear` does not register the directory.

**A plan branch may carry Keel's own files.** The rule "a plan touches no code"
is about the project's code, not about what `init` put there: otherwise it would
wall off the first wave every time `init` or `update` refreshed something.

**Updating does not clobber work.** `keel init` records the digest of every
generated file in `keel/keel.json`, and that is what lets `keel update` tell "the
method has moved on" apart from "this file was edited by hand". The first is
updated silently, the second is not: `update` names such a file, leaves it as it
is, and exits with a non-zero code.

It deliberately does not ask. A question stops an autonomous run, a silent
overwrite destroys work; refusing one file at a time does neither. `--diff` shows
the difference, `--force` overwrites.

`AGENTS.md` and `.claude/settings.json` take no part in that bookkeeping: Keel
owns a block inside them, not the file, so they are merged rather than replaced.

The translation check lives here too: `update` turns red when the English
reference has fallen behind the Ukrainian source. It is deliberately not in
`check` — the checks are about the graph of waves in a project, and this one
is about the method's copies of itself.

## Git hooks

What everything rests on. They work independently of the agent and of whether it
read anything.

| Hook | What it does |
|---|---|
| `pre-commit` | the fast checks: references, cycles, revisions, scope, headings |
| `pre-push` | the full `keel check`, tests and exports included |
| CI | the same as `pre-push` — in case the hook was bypassed |

Fast on commit, slow on push: the agent commits often and must not wait minutes,
and red will not reach the main branch either way.

Somebody else's hook is not overwritten without `--force`.

CI differs in two small ways, and without them it is quietly green. The head
there is detached, so the branch name is passed as `--branch`; the main branch is
not local, it exists as `origin/main`. The whole history has to be fetched —
scope is compared against the point where the branch left.

## Agent hooks

Faster feedback. They do not replace the git hooks: an agent hook makes a mistake
cheaper, `pre-commit` makes it impossible.

| Event | What it does | Claude | Cursor | Codex |
|---|---|---|---|---|
| session start | injects the state and names the skill | `SessionStart` | `sessionStart` | `SessionStart` |
| before a file write | refuses if the file is not declared | `PreToolUse` | `preToolUse` | `PreToolUse` |

The second event is the valuable one: drift is caught the moment the agent is
about to write in the wrong place, rather than at the commit.

The event names match, the configs are all project-level and all committed. What
differs is what a hook answers with:

| What | Claude | Cursor | Codex |
|---|---|---|---|
| refusal | `hookSpecificOutput.permissionDecision: "deny"` | `permission: "deny"` | `decision: "block"` |
| text into context | `additionalContext` | `additional_context` | `additionalContext` |
| wrapper | `matcher` and a nested `hooks` list | a flat list, `"version": 1` | a flat list |

Exiting with code 2 blocks the action in all three — that is the only thing they
share.

Hence the shape of what is generated: **one script, several configs**. The
configs call the same `keel hook` with a flag saying whose dialect to answer in.
Otherwise the scripts would drift apart: what is written separately also ages
separately.

### The session hook points at a skill

A skill is a file of instructions the agent picks up itself when it decides that
it fits. A hook is a program Claude or Cursor runs, and the agent has no say in
it.

A hook cannot load a skill: all it can do is **put text into the context**. So
`keel hook session` looks into git and prints a sentence naming the skill to take:

| State | What it says |
|---|---|
| a `plan/*` branch | take `keel-plan` |
| the main branch | the wave that is ready and the branch to take, or that it is time to plan |
| the wave file does not parse | the reason, and nothing else |
| the plan is not on the main branch | there is no work: approval has not happened |
| an open transform | take `keel-work`, here is the `next` package |
| every transform closed | take `keel-review` |
| the branch is not a wave | take `keel-plan`, here is how to start |

The point is that git knows the stage, not the agent. Without this the agent
would have to guess.

In `manual` mode the sentence addresses not the agent but, through it, the
person: the agent is not allowed to take the skill there, and telling it to would
produce a blocked call. So the text says "ask them to type /keel-plan".

### The hook before a write

It compares the file against those declared in the wave — deliberately the same
ones check 4 compares, so that the hook is not stricter than the gate.

**On the main branch it refuses a write to code.** Not through scope — there is
nothing there to compare against: check 4 compares a branch **with** `main`, and
standing on `main` itself it honestly returns nothing. Which is why the one
branch where no work is planned stayed the one branch where anything could be
written. Now the hook says that `main` is where finished work arrives, and sends
you to a wave's branch. Keel's own furniture is always free, and a project with
no wave yet is not walled in: there is no plan there to work from.

The path sits in different places in the incoming JSON, and the documentation is
uneven. In Claude it is `tool_input.file_path` for Write, Edit and NotebookEdit,
and that is documented. Cursor documents nothing for its write tools; `file_path`
is there in the neighbouring `beforeReadFile` and `afterFileEdit`, and a bug
report says the same key applies to Write. So the path is looked for under
several names, and `tool_input` is parsed both as an object and as a string with
JSON inside — for some tools Cursor hands it over exactly that way.

**When no path is found, the hook says so out loud and lists the declared files.**
Passing in silence would look like a check that never happened.

Codex has no hooks yet: writes go through `apply_patch`, and the path sits inside
the patch text rather than in a field. Its hooks also have to be switched on in
`config.toml` with `[features] hooks = true`, and they do not work on Windows.

## Three modes

A skill file and a slash command are the same object in both agents. One line in
the header decides who may reach it: with `disable-model-invocation: true` only a
person typing `/keel-plan` can start the procedure, and the description leaves
the model's context entirely; without it the model reads the description and may
take the procedure when it judges the moment right. Cursor's own migration
converts old slash commands to skills with exactly that line.

So the question is not skills or commands. It is who starts a procedure, and
whether anything watches while it runs. `keel init --mode` answers both in one
word:

| Mode | Who starts a procedure | Agent hooks |
|---|---|---|
| `strict` (default) | the agent, on its own judgement | installed |
| `soft` | the agent, on its own judgement | none |
| `manual` | only you, by typing `/keel-plan` | none |

The default is `strict` because a method nobody starts is not a method.

**The hooks in that column are the agent hooks** — the ones that read what the
agent is about to write and refuse a file the wave does not declare. Git hooks
are a different animal and go in whatever the mode: they guard the repository
against a broken wave reaching the remote, and they guard it against you too.

Three words cover three positions, and lose a fourth: starting every procedure
by hand while keeping the guard that refuses an undeclared write. `--agent-hooks`
and `--no-agent-hooks` overrule the mode, so that combination stays reachable:

```bash
keel init --mode manual --agent-hooks
```

When `keel/keel.json` does not parse, `init`, `skills` and `update` refuse to
work until it is fixed: acting on the defaults would rewrite a Ukrainian project
in English and label its pristine copies hand-edited. The hooks still answer on
a broken config — silence is the one thing they are not allowed.

The mode is written into `keel/keel.json` and read back by `keel skills` and
`keel update`, so regenerating never quietly hands back either the procedures to
the model or the hooks the mode did not want. Edit that mode by hand and the next
`update` takes the hooks back rather than merely declining to add them. The
`--agent-hooks` and `--no-agent-hooks` flags are written there too: the override
outlives updates instead of lasting until the first one.

**Narrowing the mode also takes back what a wider one installed.** `keel init
--mode manual` over a strict install does not merely stop writing hooks: it
removes `.cursor/hooks.json` and takes our entries out of `.claude/settings.json`,
leaving everything else in that file untouched. Otherwise the output would say
"no agent hooks" while the hooks went on refusing writes — and a report the
filesystem contradicts is the one thing this whole tool exists against.

A file we did not write is not ours to take away: if `.cursor/hooks.json` no
longer matches what Keel put there, it is named and left — the same answer
`update` gives a file somebody edited by hand.

## Skills

A skill is needed where there is judgement. In Keel there is one such place —
**planning**: how to walk the quality cuts, how to tell a scenario from a
boundary, how to see that a transform is not atomic yet. None of that fits in a
command.

The work drives itself: `next` says the action, `check` says the problem. A skill
there would be a retelling of what the tool already prints. So: one substantial
skill, `keel-plan`, and two thin ones, `keel-work` and `keel-review`.

| Agent | Where it goes |
|---|---|
| Claude Code | `.claude/skills/<name>/SKILL.md` |
| Cursor | `.cursor/skills/<name>/SKILL.md` |
| Codex and the rest | `AGENTS.md`, ten lines of pointers |

The format is the same for both: `name`, `description`, an optional `paths`.
Cursor also has rules in `.cursor/rules/*.mdc`, but those are for short standing
constraints, and multi-step procedures are skills.

**A skill is not only for the model to pick up.** The directory name is the
command, so the operator starts a stage of the cycle directly — and in `manual`
mode the operator is the only one who starts it:

| Skill | When | What runs underneath |
|---|---|---|
| `/keel-plan <slug>` | new work begins | `keel new wave`, `keel rev --write`, `keel gaps` |
| `/keel-work` | the plan is merged, the code gets written | `keel next`, `keel check` |
| `/keel-review` | before the PR | `keel check` |

The words on the left and the words on the right are deliberately different. On
the left are the **stages** of the cycle, named exactly as `METHODOLOGY.md` names them;
on the right are the tool's **actions**. One stage is several actions, so they do
not collapse into one word: `/keel-plan` is not `new wave`, it is three commands
and all the judgement in between. The operator knows the three words on the left;
the rest is the agent's vocabulary.

The description is the main triggering mechanism, so **all the information about
when to take a skill** lives there, and the body holds only what to do. It is
written deliberately insistently: models tend to *under*-trigger and skip a skill
where it would have helped. The description is quoted because it contains a
colon, and it is truncated at 1536 characters in Claude's skill listing.

**None of the three carries `paths`, and that is deliberate.** The field scopes a
skill to the files it is about — but while nothing matches the pattern at the
start of a session, the skill disappears from the listing altogether, rather than
merely losing its automatic pickup. Verified live: with `keel/waves/` empty,
`/keel-plan` did not exist, while its two siblings without `paths` were there.
And binding the skill that writes the **first** wave to wave files hides it at
exactly the moment it is needed.

**These files are generated by `keel skills`, not maintained by hand.** The body
is identical in both; only the fields the other one does not know differ, and a
test guards that.

## State

Done: every check, `new`, `gaps`, `next`, `rev`, `check`, `hooks`, `skills`,
`init`, `update`, `hook`; the git hooks and CI; three skills in two dialects;
agent hooks for Claude and Cursor; references in two languages with the revision
lock on them.

Left:

1. **Hooks for Codex** — postponed: writes go through `apply_patch`, and the path
   sits inside the patch text rather than in a field.
2. **The first real wave on `keel-agent`** — until then the method is verified by
   the tool and the tests, but not by work.

## Open

- **Whether Cursor really calls that field `file_path`.** In Claude it is
  documented; in Cursor it is not — the evidence is indirect, from a bug report
  and from the neighbouring events. The hook looks under several names and never
  passes in silence, so the first run in Cursor will tell the truth; the list can
  be narrowed then.
- **Whether `tool_input` arrives as a string for the built-in tools too.** For MCP
  that is visible in captured examples; the reader parses both shapes.
- **What a wave actually costs.** Forty cuts on a small transform may turn out to
  be too dear, and the write hook may get in the way more than it helps. That is
  visible only from the first real wave.
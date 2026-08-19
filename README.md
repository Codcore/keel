<p align="center">
  <img src="assets/logo.png" alt="Keel" width="340">
</p>

# keel

The tool behind the Keel method. It knows the state and writes no prose.

The method itself — why steps, contracts and revisions exist — is in
[KEEL.md](KEEL.md). This file is what runs it. Either may point at the other
freely: KEEL.md says what has to be true, README says which command checks it.

## Installing

```bash
curl -fsSL https://raw.githubusercontent.com/Codcore/keel/main/install.sh | sh
```

It clones the repository into `~/.keel` and puts a `keel` command in
`~/.local/bin`. Run it again to update — the second run is a `git pull`.
`KEEL_REPO`, `KEEL_HOME` and `KEEL_BIN` override all three.

A single downloaded file would not do: `keel init` copies the references into the
project beside the tool, so the whole repository has to be somewhere on disk.

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
`test_agent_hooks`, `test_skills`, `test_setup`. Each runs on its own
(`python3 -m unittest tests.test_scope`); the shared fixture is in
`tests/support.py`. Tests are copied nowhere, so splitting them costs nothing —
unlike the tool itself.

One test is optional: if PyYAML is installed, it reads the generated skill
headers with a real parser. Our own reader is forgiving, and the headers will be
read by Claude and Cursor, not by it.

## Commands

| Command | What it does |
|---|---|
| `keel new step <slug>` | the file skeleton: a header with empty fields and stub sections |
| `keel new contract <slug>` | the same for a contract: `module` with `exports`, or `verify` |
| `keel gaps [step]` | what is missing from a step's description: slugs without sections, transforms without files, scenarios without `proves` |
| `keel next` | the **package** for the next move: the transform, its files and boundaries, the scenarios it brings closer, the bodies of the contracts it leans on — and nothing beyond. Markdown; `--json` for scripts |
| `keel check` | the six checks — the full gate. `--fast` leaves those that run nothing, `--no-tests` skips the run, `--branch` names the branch where git does not know it, `--json` for scripts |
| `keel rev` | shows revisions that have drifted apart; `--write` records the new ones |
| `keel hooks` | shows the state of `pre-commit` and `pre-push`; `--install` installs them, `--force` overwrites somebody else's |
| `keel skills` | regenerates the skills from the method |
| `keel init` | installs Keel into a project: directories, a copy of the tool, the references, `AGENTS.md`, CI, hooks. `--docs` and `--lang` set the languages, `--force` overwrites somebody else's hook |
| `keel hook <event> --agent` | answers an agent hook; called by a config, not by hand |
| `keel update` | brings the project's copies up to date. `--diff` shows the difference, `--force` overwrites even hand-edited files |

`gaps` and `check` are separate precisely because a scenario without a test is
fine while planning and not fine before a PR. One command with two modes would
confuse both. And neither of them plans: `gaps` says what is missing, `check`
says what is wrong.

The `-C DIR` flag says where to work. The project root is searched upwards: the
first directory with `keel/steps/`, or the first with `.git`.

## Two languages, and they are independent

`keel init` records two of them in `keel/keel.json` — the third is the mode —
and the two should not be conflated:

| Key | What it decides | Flag |
|---|---|---|
| `docs` | which language the references arrive in | `--docs uk\|en` |
| `lang` | what the agent writes steps and commits in, and which phrases the skills catch | `--lang uk\|en` |

They are separate because wanting an English reference with Ukrainian triggers is
a reasonable thing to want. A skill's body depends on neither: it is read by the
model, and it is always English. So is what neither of them reads: the CI
workflow and the git hook scripts, down to the message somebody meets on a
failing push.

Those two and the mode are the whole of what Keel keeps in that settings file,
but not the whole of what may live there: the file is yours, it sits in your
repository, and keys Keel does not know survive a rewrite. Neither language can
be guessed: the language of a project's prose is a team's decision, not a
property of the code.

`lang` also decides what this tool says, and what language `keel new` writes a
step or contract skeleton in. The header fields are the same either way — they
become code — while the hints and the Why heading follow the project; the reader
accepts both spellings, so a project may change language without its existing
steps becoming unreadable. The tool's messages are keyed by their English
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

Two of the six checks depend on the language: what runs the tests (5) and where a
module's exports come from (6). The adapter is chosen by a marker in the project
root.

| | Elixir | Python |
|---|---|---|
| marker | `mix.exs` | `pyproject.toml`, `setup.py`, `setup.cfg` |
| tests | `mix test` | `python3 -m unittest discover -s tests -p "*test*.py" -t .` |
| where tests live | `test/**/*_test.exs` | `tests/**/test_*.py`, `*_test.py` |
| scenario tag | `@tag proves: :slug, rev: "a3f1c0"` | `# proves: slug, rev: "a3f1c0"` |
| exports | `mix run --no-start` asks `__info__(:functions)` | import the module, `__all__` or the public names |
| types | `Code.Typespec.fetch_specs` — a whole `@spec` is checkable | none: a promised shape is reported as unchecked |
| CI steps | `erlef/setup-beam` plus `mix deps.get` | `actions/setup-python` |

A scenario's slug and the name in the tag are compared after normalising, so
`finishes-when-no-tool` in the step and `:finishes_when_no_tool` in the test are
the same thing.

CI versions are not guessed: `keel init` asks `elixir --version` on the machine
it was run on and writes down what it found.

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

An `exports:` entry is either `run/3` or a whole signature:

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

**This executes a command out of a file in the repository, and that is worth
knowing.** `keel check` runs it through a shell, and `pre-push` runs `keel check`.
So cloning somebody's project, or checking out a branch from a pull request and
running `git push`, means executing whatever its contracts say. Until now every
subprocess Keel launched was a fixed adapter command; this is the first one set
by whoever wrote the contract. The practical conclusion is simple: read a
contract in somebody else's PR as carefully as you read code.

## How Keel gets into a project

The `keel` repository is the method's home, not what an agent reads. The agent
reads files **in the project it works on**, and `keel init` puts them there:

```
keel/steps/                     empty directories
keel/contracts/
keel/keel.json                  the settings: two languages and the mode
keel/keel.py                    the tool itself, as a copy
keel/KEEL.md                    the method, as a copy
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
then the copy in the project. The copy is last, and it is the only one always
there.

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
wall off the first step every time `init` or `update` refreshed something.

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
`check` — the six checks are about the graph of steps in a project, and this one
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
| an open transform | take `keel-work`, here is the `next` package |
| every transform closed | take `keel-review` |
| the branch is not a step | take `keel-plan`, here is how to start |

The point is that git knows the stage, not the agent. Without this the agent
would have to guess.

In `manual` mode the sentence addresses not the agent but, through it, the
person: the agent is not allowed to take the skill there, and telling it to would
produce a blocked call. So the text says "ask them to type /keel-plan".

### The hook before a write

It compares the file against those declared in the step — deliberately the same
ones check 4 compares, so that the hook is not stricter than the gate.

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
agent is about to write and refuse a file the step does not declare. Git hooks
are a different animal and go in whatever the mode: they guard the repository
against a broken step reaching the remote, and they guard it against you too.

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

| Command | When |
|---|---|
| `/keel-plan <slug>` | start a new step: the `plan/` branch, the skeleton, the scenarios |
| `/keel-work` | do the next transform; one invocation, one transform |
| `/keel-review` | check before the PR |

The description is the main triggering mechanism, so **all the information about
when to take a skill** lives there, and the body holds only what to do. It is
written deliberately insistently: models tend to *under*-trigger and skip a skill
where it would have helped. The description is quoted because it contains a
colon, and it is truncated at 1536 characters in Claude's skill listing.

`keel-plan` is also bound to `keel/steps/*.md` through `paths`, so it is picked up
on its own when a step file is being edited.

**These files are generated by `keel skills`, not maintained by hand.** The body
is identical in both; only the fields the other one does not know differ, and a
test guards that.

## State

Done: the six checks, `new`, `gaps`, `next`, `rev`, `check`, `hooks`, `skills`,
`init`, `update`, `hook`; the git hooks and CI; three skills in two dialects;
agent hooks for Claude and Cursor; references in two languages with the revision
lock on them.

Left:

1. **Hooks for Codex** — postponed: writes go through `apply_patch`, and the path
   sits inside the patch text rather than in a field.
2. **The first real step on `keel-agent`** — until then the method is verified by
   the tool and the tests, but not by work.

## Open

- **Whether Cursor really calls that field `file_path`.** In Claude it is
  documented; in Cursor it is not — the evidence is indirect, from a bug report
  and from the neighbouring events. The hook looks under several names and never
  passes in silence, so the first run in Cursor will tell the truth; the list can
  be narrowed then.
- **Whether `tool_input` arrives as a string for the built-in tools too.** For MCP
  that is visible in captured examples; the reader parses both shapes.
- **What a step actually costs.** Forty cuts on a small transform may turn out to
  be too dear, and the write hook may get in the way more than it helps. That is
  visible only from the first real step.
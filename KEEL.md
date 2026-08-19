# Keel

A method for developing with an agent: **two kinds of document, six checks**.

This file is **what has to be true, and why**. What runs it, how it is installed
into a project, which commands, hooks and skills exist — that is [README.md](README.md).
The two may point at each other freely; mixing them in one file, no.

The principles are one page. In this repository they are `PRINCIPLES.md`; in
a project they arrive as the block `init` writes into `AGENTS.md`, which is
what an agent reads at every start. This file is a reference, read when needed.

## Language and format

A document's header is English: its fields become code, test tags and file names.
The prose is the project's own language, because a person reads and approves it.

The header is YAML between three dashes. The body is ordinary Markdown.

## Where things live

```
keel/steps/              steps — one file per branch
keel/contracts/          contracts — one file per contract
keel/QUALITY.md          the quality cuts — the checklist used while writing a step
keel/KEEL.md             this file, as a copy
keel/README.md           the tool reference, as a copy
AGENTS.md                seven principles and the pointers, as a block between markers
```

That is what **a project living by Keel** looks like. In the method's own home —
the `keel` repository — these files sit at the root, because there they are the
source rather than a copy. What exactly is placed into a project, and why as
copies, is in [README.md](README.md).

An identifier is a file name without its extension. There are no anchors inside
documents, so the check "the reference leads somewhere" is the existence of a
file, not a parser.

## Two entities

### Step

One file, one branch, one pull request. Inside it:

- **scenarios** — what we promise. Each becomes a test;
- **transforms** — what does the work. Each becomes a commit.

Both live as sections in the body; their names and links are in the header.

### Contract

A promise the code leans on, and the way to check it. Its own file, because it
outlives the step that created it.

Our own promise is a module, its exported functions and their meaning; the check
loads the module and compares. Somebody else's — a library, a service, a binary —
works the same way, only it carries `verify`: a command whose success is the
proof. Who makes the promise does not matter; that it can be checked does.

An export is written either as a name with an arity, `run/3`, or as the whole
signature, `run(binary(), keyword()) :: {:ok, term()}`. The short form promises
that the function exists; the long one promises its shape as well, and where the
language can be asked about types — Elixir can — the check compares it against
what the module declares and prints the module's own version of any difference.
Arguments may be named, the way the compiler itself names them; where a function
carries several `@spec` lines, any one of them may be the promise. Write as much
as you mean to promise: the short form is not weaker for the long one existing.
Promise a shape in a language that keeps none, and the check says so rather than
passing.

A promise nothing can check is not a contract but a boundary, and it lives as a
paragraph inside a transform.

## Links

| Edge | From, to | What for |
|---|---|---|
| `depends_on` | step → step | the order of work |
| `proves` | scenario → contract | the contract has proof |
| `contracts` | transform → contract | what it implements, plus the revision |
| `implements` | transform → scenario | which commit brings which promise closer |
| tag in a test | test → scenario | the promise is proved, and this exact revision of it |

An edge lives in the header, not in the prose. Written as prose — bold code plus
an anchor plus a relative path — it is three things that drift apart separately,
and no check will catch up with them.

## Scope: exact files

A transform lists its files by name, before the work. Globs are not used: under a
glob an agent will produce ten files where one was meant, and nothing will make a
sound.

- a step has no scope of its own — it is the sum of its transforms;
- the check runs **both ways**: touching outside the list shows, and so does
  declaring something and never touching it;
- **the whole branch** is checked, not a single commit;
- extending the list is allowed and stays as a line in the diff. Drift is not
  forbidden — it is named.

**What only lays the ground rides in the files of the transform that needs the
ground, never as a transform of its own.** A build file, a first dependency, a
project skeleton — none of them promises anything or brings any scenario closer,
so none of them has earned a transform: `gaps` will refuse it for implementing
no scenario, and `gaps` will be right. Put them in the file list of the first
transform that cannot compile without them.

Keel's own furniture is not subject to scope, on a plan branch or a work branch
alike. The documents under `keel/`, the skills, the hooks, the CI and the block
in `AGENTS.md` are plan and scaffolding, not work, and `keel update` running mid
work should not demand that our own generated file be declared in somebody's
transform. A `plan/*` branch is the mirror image — it should touch nothing else,
and the check says so.

Scope is compared against the branch: its name is the step's name. Where git does
not know that name — on CI the head is detached — it arrives through the
`--branch` flag. Skipping the check in silence would mean green where nobody
compared anything.

## Revisions

A reference may carry the revision of what it points at — a short hash of the
text. The check compares it with what is written now.

```
@tag proves: :finishes_when_no_tool_called, rev: "a3f1c0"  test → scenario
contracts: [session-run@7c40de]                            transform → contract
```

The first catches "the scenario was rewritten, the test stayed old and green".
The second is its mirror: "the contract was rewritten, the code was left alone".

**A revision is six hexadecimal digits**, compared by prefix: a record of four
digits or more passes if it matches the start of the current hash. A contract is
hashed whole, header included, because changing `exports` changes the promise; a scenario is
hashed by the body of its section.

**A revision changes on any change of a word.** Before hashing, only repeated
spaces and line breaks are collapsed; a comma, a change of case or a rephrasing
all give a new revision, and the test has to be reread.

A step's header holds no scenario revisions: the text lives in the body and the
hash is computed from it on the fly. Only whoever leans on a text records its
revision.

## Closure is derived

No status is written by hand.

| What | Closed when | Where it shows |
|---|---|---|
| Transform | the branch has a commit whose message begins with its slug | git log |
| Scenario | a test with its name exists, is green, and the revision matches | the test run, the step's text |
| Contract | the revision matches the one recorded, and the promise is confirmed — by exports or by a command | the file, the compiled module, `verify` |
| Step | every transform closed, every scenario proved, the whole gate green — the checks, the plan, the project's own CI command, and no two documents disagreeing | all of the above together |

## Six checks

1. References lead somewhere: every slug in a header has its file or its section,
   and every link in the text has its file, outside `keel/` as well.
2. `depends_on` without cycles.
3. Contract revisions match the current text.
4. The files a branch changed match those declared in the transforms — both ways.
5. Every scenario has a test with its name, the tests are green, the revision in
   the tag matches.
6. Contracts hold: a module exports what was promised, and a `verify` command succeeds.

A seventh, minor one: the set of names in the header matches the set of headings
in the body.

**What green means is said here, because the difference is not obvious.** A
green fifth means "a test of that name exists, its revision matches, and it
passes" — not "the promise is proven": whether that test actually checks what
the scenario promises is asked by nobody, and cannot be asked mechanically. A
green sixth means "the promised shape is there" — the module exports the named
functions and the signatures match what was declared — not "the promise is
kept": a contract may promise in prose that a broken tool does not bring the
session down, keep a flawless shape, and have the code do the opposite.

That gap is exactly what `/keel-review` aims at with its single question — **what
did we stay silent about**. The checks catch a difference between the documents
and the facts; the review catches one between what was written and what was
meant.

Before any of them comes reading the documents themselves. A header that does not
parse, and a field of the wrong shape — at any level: `transforms` as a list,
`files` as a map — is an error in the document, not an empty default: empty reads
as "nothing is declared", and that is what would switch the guard off, or accuse
a transform of declaring no files while its files sit right there, mis-shaped.

None of them parses prose: they all read the header, git and compiled modules.
Which are fast, what each one does and how they run is in [README.md](README.md).

## What is not here

No requirements, no questions, no log, no statuses, no tags, no numbers inside a
step, no separate decisions. A promise is written once, as a scenario; a question lives for hours in a
pull request discussion rather than for years in a graph; git holds the history;
status is counted.

A constraint is not a field either: what can be checked is a scenario, what is
structural is scope, and the rest is the "boundaries" paragraph in a transform.

Decisions were here, and they left. There was a directory for them, but no header
field pointed at it and no check looked at it; the rule "its own file once two
steps lean on it" was counted by nobody. What promises something became a
contract; what we deliberately do not do became a boundary; a rule about
architecture belongs to the linter's config.

Every new entity has to hurt by being missing first, and every one that is here
has to prove it still hurts.

## Example

`keel/steps/0007-session-loop.md`

```markdown
---
depends_on: [0005-flat-tools, 0006-graph-tools]

scenarios:
  finishes-when-no-tool-called:   {proves: session-run@7c40de}
  only-handed-tools-are-callable: {proves: session-run@7c40de}

transforms:
  declare-outcome-struct:
    implements: [finishes-when-no-tool-called]
    contracts:  [session-outcome@d91c4a]
    files:      [lib/keel_agent/session/outcome.ex]

  drive-turns-on-reqllm:
    implements: [finishes-when-no-tool-called, only-handed-tools-are-callable]
    contracts:  [session-run@7c40de]
    files:
      - lib/keel_agent/session.ex
      - test/keel_agent/session_test.exs
---

## Why

One conversation with a model against a set of tools handed in from outside.
The session does not know which tools it was given — so both paths are driven
through it, and the difference between runs is the difference in tools.

## scenario: finishes-when-no-tool-called

**Given** an opening context and an empty set of tools,
**When** the model answers with text and no call,
**Then** the conversation ends in `:finished` and the trace holds a turn.

## transform: drive-turns-on-reqllm

Keep turning while the model calls tools. A tool's answer is appended to the
trace before the next turn.

Boundaries: there is no attempt counter — a tool's refusal is an answer, and
the next turn is the retry.
```

`keel/contracts/session-run.md`

```markdown
---
module: KeelAgent.Session
exports:
  - "run(Context.t(), [Tool.t()], Config.t()) :: Outcome.t()"
---

One conversation with one model. `opening` is the first context, `tools` are the
tools the model may call, `config` is the one whose `model` says which model, and
`step_budget_ms` bounds the whole call.
```

`test/keel_agent/session_test.exs`

```elixir
@tag proves: :finishes_when_no_tool_called, rev: "a3f1c0"
test "the conversation ends when the model calls no tools" do
  outcome = Session.run(opening(), [], config())

  assert outcome.stop == :finished
  assert Enum.any?(outcome.trace.events, &(&1.kind == :turn))
end
```

## Names

**A commit names its transform by slug** in its message. That is the only link
between the work and the plan, and because of it no hash is recorded anywhere: a
transform is closed by the fact that a commit carrying its slug is on the branch.
There is no `commit` field in the header — it would be a status written by hand.

**The number in a step's name** (`0007-`) is a unique prefix, not an order. The
order is derived from `depends_on`. Otherwise the temptation to renumber appears,
and references break — the same trap that was left behind by giving up codes.

## Quality cuts

`keel/QUALITY.md` — forty questions under the nine characteristics of ISO/IEC
25010. It is **a checklist, not a structure**: no step has to have a scenario for
every cut. The agent walks the list and gives one of three answers — does not
apply, answered by this scenario, stayed silent. Silent means: the cut is
relevant, nothing closes it, and it needs either a scenario or a decision saying
"no" out loud.

**One pass per step**, where the scenarios are written. Not at every level and
not until it converges: there are no levels any more, there are steps.

The point is not completeness. The point is that the case nobody thought of can
no longer be passed over in silence. Left alone, an agent writes the happy path.

## Who does what

Rules do not hold up because an agent read them: one agent reads, another does
not, and better instructions do not cure it. The division is this:

| Who | Responsible for |
|---|---|
| The tool | state, skeletons, checks, "the one next action" |
| The agent | the text: why, scenarios, the split into transforms, the code |
| The hook | not letting anyone walk past |
| The skill | how to think where there is judgement — that is, while planning |

Short version: **the tool knows the state, the agent has the judgement.** The
tool writes no prose — it has no model. The agent remembers no rules — it asks
the tool.

## The cycle

Two pull requests per step: the plan separately, the work separately. The
operator starts each stage with a single command; the tool drives the rest.

```
PLAN                                branch plan/0007-session-loop

  /keel-plan session-loop           the operator calls it
      keel new step                   the file skeleton
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
which state the step is in, and puts the answer into the context along with the
name of the skill to take.

**Approval of a plan is written nowhere — it is derived.** The step file being on
the main branch means a person read it and let it through; `keel next` hands out
no transforms from a step that is not there yet. No fields and no log lines.

The agent remembers nothing between moves, and that is deliberate: in an
autonomous run the context is cleared anyway. The stages did not disappear — they
became state that is derived, instead of a page somebody has to hold in mind.
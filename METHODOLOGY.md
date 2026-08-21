# Keel: the method

The normative text. Here is **what has to be true**; the grounds are given
wherever they are not obvious, because a constraint without a reason is routed
around by anyone able to read it.

What runs this, which commands, hooks and skills exist — [README.md](README.md).
The quality cuts — `QUALITY.md`. The principles — `PRINCIPLES.md`, and they
reach a project as a block inside `AGENTS.md`.

Citations take the form `§4.2`. Cite from anywhere: a commit message, an error
string, a skill, a review finding.

---

## Chapter 1. Interpretation

**§1.1.** This document is normative. A contradiction between it and any other
text of the method is resolved in its favour, and the other text is corrected.

**§1.2.** A document's header is written in English. Its prose is written in the
project's own language.

*Grounds: header fields become file names, test tags and code; the prose is read
and approved by a person.*

**§1.3.** The header is YAML between three dashes. The body is Markdown.

**§1.4.** A document's identifier is its file name without the extension. There
are no anchors inside documents.

*Grounds: checking a reference reduces to the file being there, and needs no
parser.*

**§1.5.** Paragraph numbers never change. A paragraph that loses force keeps its
number, marked withdrawn, and the number is not reused.

*Grounds: a citation whose number was changed points at a different rule and
lies silently.*

**§1.6.** A fact that lives in code is not restated here.

*Grounds: a restated fact drifts from its source with nothing to bring it back.
The number of checks, for one, is nowhere given as a number — it is the number
of paragraphs in chapter 7.*

---

## Chapter 2. Documents

**§2.1.** There are two kinds of document: the **wave** and the **contract**.
There are no others.

**§2.2.** A wave is one file, one branch, one pull request.

**§2.3.** A scenario is a promise about behaviour. Every scenario becomes a test.

**§2.4.** A transform is the work that fulfils a promise. Every transform becomes
exactly one commit.

**§2.5.** The names of scenarios and transforms, and the edges between them, live
in the header; their texts live as sections in the body.

**§2.6.** A contract is a promise the code leans on, together with the means of
checking it. A contract lives in its own file.

*Grounds: the promise outlives the wave that created it.*

**§2.7.** Our own promise is declared as a module, the list of functions it
exports, **and the meaning written in prose**. The check loads the module and
compares the first two; the third nothing compares — see §7.8.

**§2.8.** Somebody else's promise — a library, a service, a binary — carries
`verify`: a command whose success is the proof. Who makes the promise does not
matter; that it can be checked does.

**§2.9.** An export is written either as a name with an arity (`run/3`) or as a
whole signature (`run(binary(), keyword()) :: {:ok, term()}`). The short form
promises the function exists; the long one promises its shape as well. Where the
language can be asked about types, the check compares the promised shape against
the declared one. Write as much as you promise.

**§2.10.** A promise nothing checks is not a contract. It is a **boundary**, and
it is written as a paragraph inside a transform.

---

## Chapter 3. Edges

**§3.1.** The edges of the graph, and no others:

| Edge | From, to | What for |
|---|---|---|
| `depends_on` | wave → wave | the order of work |
| `proves` | scenario → contract | the contract has proof |
| `contracts` | transform → contract | what it implements, plus the revision |
| `implements` | transform → scenario | which commit brings which promise closer |
| tag in a test | test → scenario | the promise is proved, and this exact revision of it |

**§3.2.** An edge lives in the header, not in the prose.

*Grounds: a reference written as prose is bold code plus an anchor plus a
relative path — three things that drift apart separately, and no check will
catch up with them.*

**§3.3.** `proves` is never empty. So contracts are decided **together with** the
scenarios rather than after them: if the promise a scenario needs does not exist
yet, this wave brings it; if it already does, the scenario points at it, and a
second one is not wanted.

---

## Chapter 4. Scope

**§4.1.** A transform lists its files by name **before** the work starts.

**§4.2.** Globs are not used.

*Grounds: under a glob an agent creates ten files where one was meant, and
nothing objects.*

**§4.3.** A wave has no scope of its own: it is the sum of its transforms'.

**§4.4.** Scope is checked in both directions: touched outside the list, and
declared but never touched.

**§4.5.** The whole branch is checked, not a single commit.

*Grounds: work is handed out one transform at a time, so an early commit of a
wave legitimately has not yet touched the files of later ones.*

**§4.6.** Widening the file list is allowed and remains a line in the diff. Drift
is not forbidden — it is named.

**§4.7.** Whatever only prepares the ground rides in the file list of the
transform that needs it, and gets no transform of its own.

*Grounds: a build file, a first dependency and a project skeleton promise
nothing and bring no scenario closer, so a transform without a scenario is
refused — rightly.*

**§4.8.** The method's own furniture — `keel/`, the skills, the CI file, the
block inside `AGENTS.md` — is out of scope on every branch. The hook configs
live inside somebody else's files, so they are furniture only while they are as
the tool left them.

*Grounds: refreshing the method mid-work must not require declaring our own
generated file inside somebody's transform. But a file we never wrote, or one a
hand has touched since, is not furniture: exempting it would take somebody's
configuration out of the check, and nobody would see it happen.*

**§4.9.** A plan branch touches nothing but the furniture of §4.8. No code
appears on it.

**§4.10.** Scope is compared against the branch, whose name is the wave's name.
Where git does not know that name it is passed explicitly; skipping the
comparison silently is not permitted.

*Grounds: green obtained without comparing is worse than red.*

**§4.11.** Code is not written on the main branch: there is no wave there to
declare it. The project's account of itself — `README` and `BACKLOG` at the
root — is the exception, and the allowance is said out loud.

*Grounds: a paragraph about what the project has become carries no promise a
test could prove, so it has no wave and can have none. Forbidding what has
nowhere else lawful to go is not a guard but a detour somebody will eventually
take — through the same tool, past the door the guard watches.*

---

## Chapter 5. Revisions

**§5.1.** A reference may carry the revision of what it points at — a short hash
of the text. The check compares the recorded revision against the text as it
stands.

**§5.2.** A revision is six hexadecimal characters. Comparison is by prefix: a
record of four or more characters passes if it matches the start of the current
hash.

**§5.3.** A contract is hashed whole, header included. A scenario is hashed by
the body of its section.

*Grounds: a change to `exports` is a change to the promise.*

**§5.4.** A revision changes with any change of wording. Only repeated spaces and
newlines are collapsed before hashing.

**§5.5.** Only the one who leans on a text writes its revision. A wave's header
carries no revisions of its own scenarios.

**§5.6.** The revisions of a closed wave (§6.5) are not rewritten, and whoever
left them alone says so out loud.

*Rationale: a revision records the text the work was proven against. Rewriting
it says the work was proven against text that did not exist at the time — and
the gate goes green on the strength of that rewrite.*

---

## Chapter 6. Closure and approval

**§6.1.** No status is written by hand. Everything below is derived.

**§6.2.** A transform is closed when the branch holds a commit whose message
begins with its slug.

**§6.3.** A scenario is closed when a test with its name exists, the test is
green, and the revision in the tag matches.

**§6.4.** A contract is closed when its revision matches the recorded one and the
promise is confirmed — by exports or by the `verify` command.

**§6.5.** A wave is closed when all its transforms are closed, all its scenarios
are proved, the whole gate is green — the checks, the completeness of the plan,
the project's own CI command — and no two documents contradict each other.

**§6.6.** Approval of a plan is written nowhere: it is the fact that the wave's
file reached the main branch. Until then no work is handed out.

**§6.7.** A defect found in a closed wave (§6.5) is repaired by a wave of its
own. Debt, bugs and fixes are waves on the same terms: a promise, a scenario
and a test.

*Rationale: the files of a closed wave belong to somebody else's plan, and §4.4
keeps them out on purpose. An edit forced into the current wave makes its
declared scope a lie, and going around the guard is always cheaper than opening
a wave — so without this rule the road leads past it. Size changes nothing: four
lines that repair something proved are proved the same way as four hundred.*

---

## Chapter 7. The checks

**§7.1.** References lead somewhere: every slug in a header has its file or its
section, every reference in the text has its file.

**§7.2.** `depends_on` has no cycles.

**§7.3.** Contract revisions match the text as it stands.

**§7.4.** The files a branch changed match those declared in its transforms — in
both directions.

**§7.5.** Every scenario has a green test bearing its name, and the revision in
the tag matches.

**§7.6.** Contracts hold: the module exports what was promised, the `verify`
command passes.

**§7.7.** The set of names in the header matches the set of headings in the body.

**§7.8.** A green §7.5 means "a test by that name exists, its revision matches,
and it passes" — not "the promise is proved". A green §7.6 means "the promised
shape is in place" — not "the promise is kept".

*Grounds: whether a test really checks what its scenario promises cannot be
asked mechanically; a contract may promise in prose that a broken tool does not
bring down the session, have a flawless shape, and have code that does the
opposite. That gap is what the review aims at, with its single question: what
did we stay silent about.*

**§7.9.** Reading the documents comes before the checks. A header that does not
parse, and a field of the wrong shape, are an error in the document rather than
an empty default.

*Grounds: empty reads as "nothing was declared" — which switches the guard off,
or accuses a transform of declaring no files while the files sit right there,
merely in the wrong shape.*

**§7.10.** No check parses prose. They all read the header, git, and compiled
modules.

**§7.11.** A wave whose plan has reached the main branch and whose work has not
is not judged by §7.5 and §7.6 on main. It has begun once main closes at least
one of its transforms.

*Rationale: the same as for a plan branch — a promise without code, judged by
green tests and existing modules, is a gate that is always shut. Approving a plan
moves that state onto main rather than ending it, and until the work lands a red
main means nothing. The cost was not only that: while main is red, what §4.11
allows through cannot be pushed either.*

**§7.12.** A test that has never been seen red is undemonstrated. Before
believing green, break what the test guards and watch it fail.

*Rationale: green says nothing about proving power. A test that checks the wrong
thing, compares against the wrong value, or lost its assertion is green as well,
and from outside the two are identical. The difference shows only with the code
broken.*

**§7.13.** The suite is run more than once before green is accepted.

*Rationale: a single run hides everything that depends on order, timing, or a
process that outlived its test. Three or four runs cost a minute and have twice
caught a real defect that would otherwise have surfaced later as "it fails
sometimes" — the worst kind, because by then nobody trusts the gate.*

**§7.14.** A wave does not close without a mutation run. At the close — every
transform committed, the pull request next — the project's own mutation command
runs and has to succeed. An empty setting is red; the word `none` is a decision,
said out loud, and it is silent.

*Rationale: §7.12 asks of one test what a mutation run asks of the whole suite
at once — whether it can fail at all. It is asked at the close because that is
the only place it is affordable: the run breaks the code once per mutant and
runs the suite each time, which is minutes to hours, and a gate nobody can wait
for is a gate that gets skipped. Keel names no tool and demands no flag. The
project names a command and Keel reads its exit code, exactly as with CI and a
contract's `verify` — so narrowing the run to the files the wave declared is the
project's business, and the check stays meetable in a language whose mutation
tool cannot narrow at all.*

---

## Chapter 8. Branches and acceptance

**§8.1.** A wave takes two pull requests: the plan separately, the work
separately.

**§8.2.** The plan branch is named `plan/<wave>`, the work branch `<wave>`. The
wave's name comes from the generated file's name, not from the slug that was
typed.

*Grounds: the tool looks a wave up by the branch name; named otherwise, the
branch links to nothing.*

**§8.3.** On a plan branch §7.5 and §7.6 do not run; the completeness of the plan
is checked in their place.

*Grounds: there is no code on a plan branch by §4.9, so both would be red
always — and a gate that is always shut teaches people not to look at it.*

**§8.4.** A commit names its transform by the slug at the start of its message.
There is no `commit` field in the header.

*Grounds: such a field would be a hand-written status, against §6.1.*

**§8.5.** The number in a wave's name is a unique prefix, not an order. The order
is derived from `depends_on`.

*Grounds: otherwise the temptation to renumber appears, and references break.*

**§8.6.** The plan commit's message carries, in its own paragraph: the question,
the options, the answer, who decided — **and what the author refused out loud, if
they then did it, together with what changed their mind**.

*Grounds: git knows who and when, the diff knows what, and why and on whose call
is known to nobody, because the chat does not travel with the repository. A
withdrawn position is the purest case of what lives in the chat alone: it never
became a file, so the diff cannot carry it. The line runs at said out loud, not
at changed my mind: a thought nobody heard needs no trace, and listing
everything ever weighed turns the paragraph into a diary, which nobody reads.*

---

## Chapter 9. Roles

**§9.1.** The tool knows the state. It does not write prose.

*Grounds: it has no model.*

**§9.2.** The agent has judgement. It does not remember the rules — it asks the
tool.

**§9.3.** The hook is what lets nobody past.

**§9.4.** The skill says how to think where there is judgement to exercise.

**§9.5.** The operator gives direction and approves. Approval takes the form of
§6.6.

**§9.6.** A rule does not hold because the agent read it.

*Grounds: one agent reads, another does not, and better instructions do not cure
that. So every rule meant to hold rests on the tool, a hook or a check — or is
admitted to hold by text alone.*

**§9.7.** A constraint carries its reason with it, and a hint at what to do
instead.

*Grounds: for whoever reads your constraints, every constraint must also be
intelligible and justified — otherwise it gets routed around, and not out of
malice, but because it looked stupid.*

**§9.8.** A guard that is wrong more often than right is removed, not weakened.

---

## Chapter 10. Quality

**§10.1.** `QUALITY.md` is a checklist, not a structure. No wave is required to
have a scenario for every cut.

**§10.2.** The cuts are walked once per wave, where the scenarios are written —
not to convergence, and not at every level.

**§10.3.** Every cut gets exactly one of three answers: does not apply — with the
reason; answered by this scenario; silent.

**§10.4.** "Silent" means the cut is relevant and nothing closes it. Then either
a scenario is written or a decision to say no is taken out loud. This list is
what ends silence; it does not end refusal.

*Grounds: an agent writes the happy path by default.*

**§10.5.** A boundary that says "we deliberately do not do this" is checked
against the dependency exactly as a quality cut is.

*Grounds: `QUALITY.md`, the rule about libraries. The story that produced it
lives there and only there.*

**§10.6.** A wave carries an answer to each of the nine headings of
`QUALITY.md` — one line each. Without them the plan is incomplete.

*Grounds: the walk itself cannot be verified — a tool sees the trace, never the
thinking. Nine lines are the cheapest trace that cannot be written from memory:
naming the headings means opening the file. It is a weaker promise than the
walk and an honest one — the line stays in the diff, and an empty one is
visible to the eye.*

---

## Appendix A. An example

A wave, a contract and the test that ties them together are in
[README.md](README.md), under "What it looks like". They are deliberately not
repeated here: an example illustrates rather than establishes, and two copies of
it would drift the moment the shape changed.

---

## Appendix B. What deliberately does not exist

No requirements, no questions, no journal, no statuses, no tags, no numbers
inside a wave, no decision files. A promise is written once — as a scenario; a
question lives for hours in the discussion of a pull request rather than for
years in the graph; history is git's; status is computed by chapter 6.

A constraint is not a field: what can be checked is a scenario, what is
structural is scope, and the rest is a boundary under §2.10.

Decision files existed and went: no header field pointed at them, no check
looked at them, and the rule about when to open one was counted by nobody. What
promises became a contract; what we deliberately do not do became a boundary; an
architectural boundary became the linter's configuration.

Every new entity must first hurt by its absence, and every existing one must
prove it still hurts.

---

## Revision history

Number and date. **What** changed is visible in the diff, **why** is said in the
commit message; retelling it here would be a third description of the same
thing.

The list is kept by hand although git knows both the number and the date. There
is one reason, and it is real: this document travels into projects as a copy,
and the method's repository history is not there.

| Revision | Date |
|---|---|
| 1 | 2026-08-20 |

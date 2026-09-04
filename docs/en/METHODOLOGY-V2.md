# Keel: the methodology, version 2

> **This is the methodology in force.** The operator's decision of
> 2026-09-04: the "draft" mark is removed. The text is assembled whole
> -- ten chapters, the constitution and the appendices -- and
> thirty-seven waves, the whole tool and every one of its courts run on
> it; the record of how it was assembled is in
> [V2-PROCESS.md](../uk/V2-PROCESS.md). The first version stays in the
> archive: [METHODOLOGY-V1.md](METHODOLOGY-V1.md).
>
> **This text is a translation.** The methodology of this generation
> is written in Ukrainian, and the Ukrainian text is the source of
> truth: the §8.6 decisions are taken in it, and this translation
> follows them. Where the two disagree, the Ukrainian one is right —
> and the disagreement is a defect to report, not a choice to make.
>
> `translated_from: c7fd38` — the revision of the Ukrainian text this
> translation was made from (constitution, rule 4: whoever leans on a
> text holds its revision). `keel check` compares it with the
> Ukrainian methodology as it now stands: change the original and the
> recorded revision goes stale, so the translation cannot fall behind
> unnoticed. This is not a lock but a doorbell.

---

## Constitution

Eight rules that make Keel Keel. Everything else in this methodology
can be changed by the ordinary order of work. These eight change only
by a separate, deliberate decision recorded in this same list — and we
measure every decision taken while building v2 against them.

1. **State is not written by hand — it is derived.** What is done,
   what is in progress, what is merely planned: the methodology learns
   it from git and from tests, not from anybody's word. "Done" can
   lie; a commit and a green test cannot. So statuses that somebody
   sets by hand do not exist here at all.

2. **Every promise has a check — or an honest record that it has
   none.** A promise about the behaviour of code is backed by a test
   or by a command that confirms it. If there is nothing to check
   with, or we deliberately do not do something, that is written into
   the document in as many words: "we do not check this / do not do
   this, and here is why." What is forbidden is not the absence of a
   check but the unspoken "it is obvious anyway".

3. **Files are named before the work; deviation is visible.** Before
   any code is written, the plan lists exactly which files will be
   changed. Going beyond that list is allowed — but the departure
   shows up in the diff and in the check, and cannot pass unnoticed.
   What is forbidden is not deviation from the plan but *silent*
   deviation.

4. **Whoever leans on a text holds its revision.** A revision is a
   short hash of the text. A test remembers the revision of the
   scenario it proves; work remembers the revision of the contract it
   leans on. Change the text and the check says the recorded revision
   is stale and must be updated deliberately. This is not a lock but
   a doorbell: you may come in, but not unnoticed.

5. **Silence is forbidden; refusal is not.** Skipping a question
   without a trace is forbidden. Answering "no, we are not doing this,
   and here is why" is fine and even good. The methodology fights the
   unsaid, not refusals.

6. **A rule is held by machinery, not by memory.** What must happen is
   held by a tool, a hook or a check in CI — by something that
   physically cannot let a violation through. A rule that lives only
   in prose is honestly marked as such, so that everyone knows it
   rests on goodwill and nobody mistakes it for a guarantee.

7. **The new appears only when its absence hurts.** A new field, a new
   kind of document, a new command are added when their absence has
   already caused a real problem — not "just in case" and not "in case
   we need it". And everything already here must prove from time to
   time that its absence would still hurt — otherwise it is deleted.

8. **Everything needed to carry on the work is in git.** An agent's
   session can end at any moment; that is normal. A new agent opens
   the repository, asks the tool "what next", and continues from the
   same place. Nothing important exists only in a chat or in somebody's
   head.

---

## Chapter 1. Reading this document

**§1.1.** This document is the norm. If any other text of the
methodology contradicts it, this document is right and the other one
is corrected. In this translation, "this document" means the
methodology itself, not this English rendering of it: where this text
and the Ukrainian original disagree, §1.1 speaks through the
Ukrainian one.

**§1.2.** A document's header — what stands between the three dashes `---`
at the top of the file — is written in English, because its
fields become code: file names, test tags, slugs in commits. The prose
is written in the project's language, because a person reads and
approves it.

**§1.3.** The header is YAML. The body is Markdown.

**§1.4.** A document's identifier is its file name without the
extension. There are no anchors inside documents: to check a
reference it is enough to check that the file exists, with no parsing
of text at all.

**§1.5.** Paragraph numbers never change. A paragraph that has lost
force stays at its number marked "revoked", and the number is never
reused. The reason: a citation like "§4.2" whose number was reused
points at a different rule and lies silently. v2 applies the same rule
more widely: scenarios and contracts do not disappear either, they are
taken down with a `withdrawn` mark — chapters 2 and 4 speak of that.

**§1.6.** A fact that lives in code or in the tool is not retold by
this text. A retold number or list drifts from its source, and nothing
brings them back together. How many checks the gate runs is asked of
the tool, not of this document.

**§1.7.** The text of the methodology is written in human language —
so that the operator and any developer understand it, not only whoever
wrote it. Industry terms in English are not translated: commit, PR,
merge, branch, scope, hook, diff, CI, hash are written as they are.
A word of the methodology's own — one the industry does not have — is
explained in plain words where it is first used. Brevity does not
excuse obscurity: two sentences too many beat one that has to be read
three times.

**§1.8.** The methodology of this generation is written in
**Ukrainian**, and the Ukrainian text is the **source of truth**: the
§8.6 decisions are taken in it. The English text is a hand
translation that follows it. Where the two disagree the Ukrainian one
is right, and **a disagreement is reported as a defect** rather than
settled by picking the reading one prefers. The skeleton of both --
the chapters, the paragraph numbers, and that none of them is empty
-- is held by machine (`keel check`); the meaning is not, and that is
a limit by construction.

---

## Chapter 2. Documents

**§2.1.** There are two kinds of document: the **wave** and the
**contract**. There are no others. A wave is a unit of work: what we
are doing now and why. A contract is a promise the code leans on,
which outlives the wave.

**§2.2.** A wave is one file in `keel/waves/`. A light wave travels on
one branch and one PR; a full one on two branches and two PRs — the
plan apart, the work apart. What makes a wave light and what makes it
full is chapter 6; how the branches are named is chapter 8.

**§2.3.** A scenario is a promise about behaviour: what will be true
when the work is finished. Every scenario becomes a test that proves
that promise.

**§2.4.** A transform is a portion of work that brings the wave's
promises closer. Every transform becomes at least one commit, and the
message of each such commit begins with its slug — the name from the
wave's header. How many there are does not matter: a transform is
closed by the fact of such a commit (§6.2), and dispatching a
reviewer's findings (§9.9) gives a second and a third by itself. The
rule said "exactly one" until wave 0034 and was broken every wave,
not least by the methodology itself.

**§2.5.** The names of scenarios and transforms, and the links between
them, live in the header; their texts are sections in the body. What
is checked mechanically lies in the header; what a person reads lies
in the body.

**§2.6.** A contract lies in its own file in `keel/contracts/`,
because it outlives the wave that created it: the wave closes and the
promise goes on being used.

**§2.7.** Our own contract describes the **public interface** — what a
unit of code opens to the outside. What that unit is called is for the
language to decide: module, class, package. The contract names it,
lists the signatures of the functions or methods it promises, and adds
the meaning, written as prose. A check through the language adapter
verifies that what is named exists and has the promised shape;
machinery cannot verify meaning written as prose — a reviewer and a
person read that (chapter 7 states this limit honestly).

**§2.8.** Somebody else's promise — a library's, a service's, a
binary's — is recorded as a contract with `verify`: a command whose
success is the proof. Who gives the promise does not matter; that it
can be checked does. A command from `verify` runs only after trust has
been recorded for it: a new or changed command does not run until a
person confirms it explicitly (chapter 7).

**§2.9.** A promise about a function or a method is recorded as a
**signature — written the way the language itself writes it**. A short
signature is the name alone: "such a function exists". A full one
carries types where the language has them: "and its shape is this".
Examples: `run(binary(), keyword()) :: {:ok, term()}` in Elixir,
`def run(text: str, opts: dict) -> Report` in Python,
`run(text: string, opts: Options): Report` in TypeScript. Write as
much as you promise: where the language can be asked about types, the
check compares the promised signature with the declared one; where it
cannot, the check says honestly that nobody compared the shape.

**§2.10.** A promise nothing checks is not a contract. It is recorded
as a **boundary** — a paragraph in the transform saying outright "this
we deliberately do not do" or "this is true, but no test will prove
it" — and why. A boundary is honest by being direct: an unspoken
assumption is forbidden, a refusal said aloud is not.

**§2.11.** A transform with no promise is a **chore**: a refactor, a
dependency bump, formatting, documentation. In the header it carries
`chore: "<reason>"` instead of `implements`. Everything else is as for
an ordinary transform: the list of files, the slug in the commit, the
scope checks in both directions. A wave whose transforms are all
chores must be light: a large body of work with no promise at all is a
reason to stop and think, not to slip through.

**§2.12.** A promise can die. A later wave marks a scenario or a
contract `withdrawn: "<reason>"`, optionally
`superseded_by: <successor slug>`. A withdrawn scenario or contract
stays where it is — the history is visible without digging through
git — but leaves the checks, and its test is deleted by that same PR.
Deleting wave and contract files stays forbidden, as before (chapter
4): the death of a promise is a mark, not a hole.

---

## Chapter 3. Links

**§3.1.** The links of the graph — and no others:

| Link | From → to | What for |
|---|---|---|
| `depends_on` | wave → wave | the order of work |
| `proves` | scenario → contract | the contract gains a proof |
| `covers` | scenario → quality cut | which cut this scenario closes |
| `implements` | transform → scenario | which commit brings which promise closer |
| `contracts` | transform → contract | what the work leans on, with a revision |
| `decisions` | wave → quality cut | a refusal or a non-applicability, with a reason |
| `superseded_by` | scenario or contract → successor | who took the place of the withdrawn one |
| a tag in a test | test → scenario | the promise is proved, and this very revision of it |

**§3.2.** A link lives in the header, not in the prose. A reference
written as text is an emphasis, an anchor and a relative path: three
things that break separately, and no check catches any of them. A check reads the header whole.

**§3.3.** Every scenario leans on something: `proves` on a contract,
or `covers` on at least one quality cut; both are allowed. A scenario
with no support at all is an error. A contract is not required for
this: a scenario without one is a promise at the level of the
application, and the test itself carries it. The question "must this
be a contract" is a judgement made while planning, not a compulsion.

**§3.4.** The quality cuts that `covers` and `decisions` point at are
the vocabulary of `QUALITY.md`. A reference to a cut that is not there
is an error, exactly like a reference to a file that is not there.

---

## Chapter 4. Scope

**§4.1.** A transform lists its files by name — before the work
starts. For a file whose name cannot be known in advance (a migration
with a date in it, a snapshot), the line `one new in <dir>/` is
written — "exactly one new file in this directory"; if two are needed,
two lines are written. Such a line is checked just as strictly: no new
file in the directory is red, two are red, exactly one is green.

**§4.2.** Globs — patterns like `lib/**/*.ex` — are not used. The
reason is not patterns as such but the unbounded count: under a glob
an agent creates ten files where one was meant, and no check will
object. The `one new in` line of §4.1 is not a glob: its count is
fixed.

**§4.3.** A wave has no scope of its own: it is made of the scopes of
its transforms.

**§4.4.** Scope is checked in both directions: "touched a file outside
the list", and "declared a file and never touched it". The second
usually means the transform was described differently from how it was
done, or a piece of work was forgotten.

**§4.5.** The whole branch is checked, not a single commit: the work
is delivered one transform at a time, so an early commit of a wave has
legitimately not yet touched the files of later transforms.

**§4.6.** The list of files may be widened — and the widening stays a
line in the diff. Deviation from the plan is not forbidden; silent
deviation is. Every such widening is **drift**: a divergence of the
plan from the fact. Drift is not merely visible — the tool assembles
its list itself (which files were added to the scope after the plan
was approved: for a full wave after the plan PR is merged, for a light
one after the wave file's first commit) and puts that list into the
reviewer's package (chapter 9) as a separate obligatory item. For each
file on the list the reviewer answers: the widening is justified, and
why — or it is scope creep, and then it is a finding. The list
cannot be passed over in silence.

**§4.7.** Whatever merely prepares the ground — a build file, a first
dependency, the skeleton of a project — travels in the file list of
the transform that needs it, and gets no transform of its own. And if
the preparation is itself the whole of the work, it is a chore transform
(§2.11): it need not pretend to be a promise.

**§4.8.** The methodology's own files — the `keel/` directory, the
skills, the CI file, the block in `AGENTS.md` — are outside scope on
any branch: updating the methodology in the middle of the work must
not require declaring its files in somebody's transform. This holds
only for the files as the tool left them: a file the tool never wrote,
or one edited by hand since, is not the methodology's furniture and is
not taken out from under the checks.

**§4.9.** The plan branch of a full wave does not touch the project's
code: on it there are only the methodology's documents and its own
files (§4.8). A light wave has no plan branch — its file travels with
the work (chapters 6 and 8).

**§4.10.** Scope is compared against the branch whose name is the
wave's name. Where git does not know that name (CI with a detached
HEAD), the branch is named explicitly. The comparison cannot be
skipped in silence: a green obtained without comparing is worse than
a red.

**§4.11.** No code is written on main: there is no wave there to
declare it. The exception is the project's account of itself: `README`
and `BACKLOG` at the root. A paragraph about what the project has
become carries no promise a test would prove, so it has no wave and
can have none — and forbidding what has no lawful place is how you
grow a workaround.

**§4.12.** A document that changed its name says so itself: the new
file carries `renamed_from: <old slug>` in its header. Then the
disappearance of the old file is not an error but a line in the diff;
the old name still leads where it should, and closed waves pointing at
it are not rewritten. Red is when two documents claim one inheritance,
or when the move crosses directories. Deleting a wave or contract file
is always red: a promise dies by a `withdrawn` mark (§2.12), not by a
file's disappearance.

**§4.13.** Research lives on `spike/*` branches — outside the
methodology, and that is said outright: hooks are silent there, the
checks do not judge the branch. But merging such a branch into main is
always forbidden, and the ban is held by machinery: the check on a PR
from `spike/*` is red with the explanation "research is not merged;
bring the finding back as a wave". The trace of research is the Why
section of the wave that grew out of it.

---

## Chapter 5. Revisions

**§5.1.** A reference may carry the revision of what it points at — a
short hash of the text: `session-run@7c40de`. The check compares the
recorded revision with the text as it now stands: if they differ, red.
That is a signal to re-read and update deliberately, not to rewrite in
silence.

**§5.2.** A revision is six hexadecimal characters. Comparison is by
prefix: a record of four characters or more passes if it matches the
beginning of the current hash.

**§5.3.** A contract is hashed as a whole file, header included: a
change of signatures is a change of the promise. A scenario is hashed
by the body of its section.

**§5.4.** The revision changes with any change of wording. Before
hashing, only repeated spaces and line breaks are collapsed:
reformatting is not a change, rewording is.

**§5.5.** The revision is held by whoever leans on the text: a test
holds a scenario's, a transform holds a contract's. A wave's header
holds no revisions of its own scenarios: nobody leans on themselves.

**§5.6.** The revisions of a closed wave are not rewritten: they are
the record of which text the work was proved against. Rewriting them
would claim the work had been proved against a text that did not exist
at the time; leaving them in place says so out loud. The only lawful
way to change the fate of a proved promise is to withdraw it with a
`withdrawn` mark (§2.12) or to bring a successor through
`superseded_by`: history is not edited, it is continued.

**§5.7.** Changing a text somebody holds by revision is an event with
consequences, and they are not left to the conscience of whoever
changed it. There are three layers. First: a wave that changes a
contract is always full (chapter 6), so a person reads the grounds for
the change in the plan PR before any code. Second: the shape of the
new text is verified by machinery — signatures and `verify` must agree
with the new text (chapter 7), and if the change broke behaviour
proved earlier, the tests of those waves go red on the changer's own
branch, and closing by consequences will not let that into main.
Third: machinery does not see meaning, so the tool assembles an
**impact list** — everyone who held a revision of the changed text:
transforms with a recorded contract revision, scenarios with `proves`
— and puts it in the reviewer's package next to the diff of the old
and new text. For each dependant the reviewer answers: "the change
does not touch it" — or a finding: "its behaviour has to change too".
The list cannot be passed over in silence.

---

## Chapter 6. Closing and approval

**§6.1.** No status is written by hand. Everything in this chapter is
derived from git and from tests: a label that can be set can also lie.

**§6.2.** A transform is closed when the wave's branch carries a
commit whose message begins with its slug. This is branch discipline:
it holds the order and the atomicity of the work, and by it the tool
knows what has been delivered and what has not. After the merge nobody
reads commit messages: on main, closing is judged by consequences
(§6.5).

**§6.3.** A scenario passes through states, and each one is derived:

1. **promised** — the section exists in the wave file;
2. **witnessed red** — the branch's history holds its red commit
   (`red: <scenario>`): the test was born and failed before a hook's
   eyes (the grammar is chapter 8, the check is chapter 7);
3. **proved** — the test is green now, and the revision in its tag
   matches the scenario's text. A scenario is never proved without
   step 2: a green test nobody ever saw red proves nothing;
4. **withdrawn** — the `withdrawn` mark (§2.12); a withdrawn scenario
   is outside judgement.

**§6.4.** A contract holds when its promise is confirmed — the
signatures are in place, or the `verify` command passed — and the
recorded revisions of those who lean on it match the text (with the
allowance of §5.6 for closed waves).

**§6.5.** A wave is closed — and this is judged by consequences, not
by the archaeology of commits. A full wave is closed when every
scenario of it that is not withdrawn is proved — a green test with a
matching revision on main — and every contract it brought or holds
agrees. A light wave is closed by the fact of its merge: its file and
its work arrive in main in one PR. And "the plan is on main, there are
no tests yet" is not red: that is "approved, not yet started" — an
ordinary state, with no special machinery at all.

**§6.6.** Approval of a plan is written nowhere: it is the fact that
the wave file reached main. For a full wave that is the merge of the
plan PR, and no work is delivered before it. For a light one it is the
merge of its single PR: a person approves the plan and the
consequences together, on one screen.

**§6.7.** A defect in a closed wave is fixed by a wave of its own.
Debt, bugs and fixes are waves on the same terms: a promise, a
scenario, a test. Size does not change the rules — but it changes the
weight: a four-line fix is almost always a light wave, one PR, and the
ceremony is within its means.

**§6.8.** A wave's weight is derived from its file, not set by hand. A
wave is **light** when all of these hold at once: it has one transform,
it neither creates nor changes contracts, and it withdraws nothing
(`withdrawn`). Otherwise the wave is **full**. A light one travels on
one branch and one PR; a full one keeps the plan apart from the work
(chapter 8). The ceremony scales with the risk: a new contract or the
death of a promise is risk, and it gets two human looks; one transform
without them does not.

---

## Chapter 7. Checks

**§7.1.** References lead somewhere: every slug in a header has its
file or its section, every reference in the text has its file.

**§7.2.** `depends_on` has no cycles.

**§7.3.** Recorded contract revisions match the text as it now stands
(closed waves by §5.6).

**§7.4.** The files a branch changed match those declared in its
transforms — in both directions (§4.4), including the `one new in`
lines (§4.1).

**§7.5.** Every scenario that is not withdrawn has a green test
carrying its name and a matching revision in the tag. On a branch this
is judged by the branch; on main by consequences (§6.5), and a wave
that is "approved, not yet started" is not red.

**§7.6.** Contracts hold: the promised signatures are in place, the
`verify` command passes. The shape is verified by the language
adapter; where the language does not answer about types, the check
says honestly "nobody compared the shape" instead of showing green.

**§7.7.** The set of names in the header matches the set of headings
in the body.

**§7.8.** A green check is not yet a proved promise, and this is said
outright. A green §7.5 means "a test with this name exists, the
revision matches, it passes" — not "the promise is proved in
substance". A green §7.6 means "the shape is in place" — not "the
meaning is kept". Machinery cannot close this gap; a fresh reviewer
closes it (chapter 9) with the question "what did we stay silent
about" and the three questions of completeness.

**§7.9.** Documents are read first, then checked. A header that does
not parse, or a field of the wrong shape, is an error of the document,
not an empty value: "empty" would read as "nothing was declared" and
would silently switch the protection off.

**§7.10.** No check parses prose. They all read headers, git, and
built code.

**§7.11.** *Revoked.* In v1 an exception lived here for waves whose
plan was already on main while the work was not. Closing by
consequences (§6.5) made the exception unnecessary: "approved, not yet
started" is an ordinary state.

**§7.12.** "Seen red" is a derived state, not a discipline. A hook
lets a `red: <scenario>` commit through only if the named test fails;
a transform's commit only if the tests of its scenarios are green. The
branch check requires that every proved scenario has its red commit.
The honest limit: where the adapter can tell a failing check from a
build error, an actual failure is required; where it cannot, any
failure is accepted, and the check says so.

**§7.13.** Before trusting a green, the test suite is run several
times: a single run hides everything that depends on order, on time,
or on a process that outlived its test.

**§7.14.** *Revoked.* In v1 deliberate hand-made breakages lived here,
at the end of a wave, after a gate on automatic mutations had been
honestly withdrawn. The red commit (§7.12) does the same thing
earlier, more cheaply and mechanically: every test is broken before
the code is written.

**§7.15.** A test does not disappear in silence: if a scenario's tag
was present at the point the branch started from and is gone at HEAD,
while the scenario is not withdrawn (§2.12), the branch is red. The
protection against quietly disarming old promises lives where the
disappearance happens, not in an excavation of main.

**§7.16.** A command from the repository's files — a contract's
`verify` or the project's `ci` — runs only when its digest is recorded
as trusted (§2.8). A new or changed command does not run: the check is
red, with the text of the command and a hint on how to record trust.
Recording trust is a line in the diff, approved by a merge like
everything else.

---

## Chapter 8. Branches and acceptance

**§8.1.** A full wave is two PRs: the plan apart, the work apart. A
light one is a single PR: the wave file and the work together (§6.8).
The ceremony follows the weight, not habit.

**§8.2.** Branches are named after the wave: a full wave's plan is
`plan/<wave>`, its work is `<wave>`; a light wave lives entirely on
`<wave>`. The wave's name is the name of its file, not a slug somebody
typed into a command: the tool looks for the wave by the branch's
name, and a branch named otherwise leads nowhere. Research is
`spike/*` (§4.13).

**§8.3.** On a plan branch the checks of tests and contracts (§7.5,
§7.6) are not run: there is no code there by §4.9, so they would
always be red — and a gate that is always shut stops being read. In
their place the completeness of the plan is checked.

**§8.4.** A commit message begins with a slug, and the grammar has two
forms:

- `red: <scenario>` — the birth of a test; the hook allows it only if
  the test fails (§7.12);
- `<transform>: <text>` — work; the hook allows it only if the tests
  of its scenarios are green.

There is no "commit" field in the header and there will not be: that
would be a status written by hand, against §6.1.

**§8.5.** The number in a wave's name is a unique prefix, not an
order: the order of work is derived from `depends_on`. Otherwise the
temptation to renumber appears — and references break.

**§8.6.** The plan commit carries, in a paragraph of its own: the
questions that stood; the options; the answer; who decided. And — if
the author refused something aloud and then did it after all — what
changed their mind. Git knows "who" and "when", the diff knows "what";
"why" and "on whose word" nobody knows, because the chat does not
travel with the repository. The line runs along "said aloud": a
thought nobody heard needs no trace, and a list of everything ever
reconsidered is a diary nobody reads.

**§8.7.** A PR is merged with the "Create a merge commit" button —
always. Squash and rebase are switched off in the repository's
settings (Settings → Allow squash merging / Allow rebase merging →
off): the rule is held by a disabled button, not by memory, and
`keel init` reminds you to do it. The reason: the branch's commits
hold the records of decisions (§8.6) and the red commits (§7.12), and
a merge commit keeps all of them in the history. If a PR is merged
some other way anyway, the state does not break, because §6.5 judges
by consequences; only the history grows poorer, and the methodology
says so plainly.

**§8.8.** Two waves may run in parallel — each on its own branch and
with its own agent — if there is no `depends_on` edge between them and
their scopes do not overlap. An overlap of files between independent
waves is a question for the planning stage. Two waves with one number
are an error: whoever merges their plan second takes the next free
number before the merge; the tool looks for a free number across all
branches, not only on main.

---

## Chapter 9. Roles

**§9.1.** The tool knows the state. It writes no prose: it has no
model, and its word about the state is true exactly because it is
derived, not invented.

**§9.2.** The agent has judgement. It does not remember the rules — it
asks the tool: `next` says what to do, `check` says what is wrong.
Nothing has to be held in the head, and that is not laziness but
resilience (§9.10).

**§9.3.** The hook is what lets nobody through. It neither advises nor
reminds — it refuses: a write outside the scope, a "red" commit with a
green test, a working commit with a red one.

**§9.4.** The skill says how to think where judgement is needed: how
to cut work into transforms, how to tell a scenario from a boundary,
how to put a question to the operator. It does not retell what the
tool prints anyway.

**§9.5.** The operator steers and approves. Approval has one form —
§6.6: the wave file reached main.

**§9.6.** A rule is not held by an agent having read it: one will
read, another will not, and better instructions do not cure that. So
every rule that must be kept leans on the tool, a hook or a check — or
is honestly marked as textual (constitution, rule 6).

**§9.7.** A restriction carries its reason and a hint about what to do
instead: an incomprehensible restriction gets worked around — not out
of malice, but because it looks stupid.

**§9.8.** A protection that is wrong more often than it catches
something is deleted, not weakened.

**§9.9.** The reviewer is an agent with a clean context, and never the
author: the question "what did we stay silent about" cannot be put to
whoever has just stayed silent. The tool assembles a self-sufficient
package — the wave's Why, the scenarios with revisions, the
boundaries, the chore reasons, the branch's full diff — and three
obligatory lists:

1. **drift** (§4.6) — the files added to the scope after the plan was
   approved;
2. **the quality map** (chapter 10) — all forty cuts with their
   answers;
3. **the impact of a contract change** (§5.7) — everyone who held a
   revision of the changed text.

For every line of every list the reviewer answers — "fine, because…" —
or records a finding; a line cannot be passed over in silence. Above
the lists stand four questions of judgement: what did we stay silent
about; are all the possible scenarios accounted for; is everything
promised implemented, without a quiet narrowing; does the test cover
the whole scenario rather than a corner of it. Every finding gets one
of two things: a fix, or a refusal said aloud in a line of the PR —
there is no third state. The reviewer's report is kept as a file,
`keel/reviews/<wave>.md`, beside the wave: the evidence that a review
happened travels in the repository rather than staying in a chat, and
**no wave** without that file is merged — a barrier machinery holds
(`keel close` does not call the wave closed, and `keel next` leads to
the reviewer). Weight (§6.8) decides the number of pull requests and
nothing else: the operator's decision of 2026-09-04 is a reviewer for
every wave, since "one transform" does not make work safe, only
small. The honest limit: machinery does not prove the
reviewer's context was clean; the visible trace of that cleanliness is
the report itself, and the findings and refusals in it.

**§9.10.** The document is the loop's memory. An agent's session is
mortal, and that is the normal mode: everything needed to carry on
lies in git, and a fresh agent continues from `keel next` at the same
place. Nothing important exists only in a chat or in somebody's head.

---

## Chapter 10. Quality

**§10.1.** `QUALITY.md` is a checklist, not a structure: forty
questions under nine headings (this is ISO/IEC 25010 — the standard's
own list of quality characteristics), each with a stable slug. Nobody
demands a scenario for every cut — an answer to every cut is demanded.

**§10.2.** The cuts are walked twice per wave, and these are two
different passes, not a repetition of one:

- **the planning pass** — the author's, where the scenarios are
  written: every cut gets an answer in `covers` or in `decisions`
  (§10.3) before any code;
- **the checking pass** — the reviewer's, after the implementation and
  before the PR: over the quality map (§10.7), against the code that
  now exists — is every recorded answer honest.

The heads are different and so are the questions: the author decides
what to promise, the reviewer verifies what was delivered. What does
not happen is a second pass by the same head: it gives the same
answers at twice the price, and that is exactly how checklists stop
being read.

**§10.3.** Each of the forty cuts gets exactly one answer, and it is
written down:

- **closed** — some scenario carries this cut in its `covers`;
- **decided** — a line in `decisions:` with a reason: "does not apply,
  because…" or "we deliberately do not do this, because…".

There is no third answer: silence is forbidden at the level of the
field. A cut that is in nobody's `covers` and in no `decisions` means
the plan is incomplete, and the check is red before any review. This
is the plan author's duty; a reviewer's pass does not lift it.

**§10.4.** Refusal is a lawful answer. "There are no backups in this
wave — the volume is zero" is a decision, said aloud and recorded with
a reason. The methodology fights the unsaid, not refusals
(constitution, rule 5).

**§10.5.** A boundary (§2.10) is checked against the dependency the
same way a cut is: "we deliberately do not do this" is also a
statement about somebody else's code, and it can be false from the
moment it is written — the library may do it by default. Before
writing that something is not there, look and make sure it really is
not.

**§10.6.** *Revoked.* In v1 nine obligatory lines of trace lived here
— an agent wrote them from memory without opening the file, so they
proved nothing. The trace of quality is now the `covers` and
`decisions` records (§10.3), and those are checked.

**§10.7.** From the records of §10.3 the tool draws a **quality map**:
every cut → what closes it or how it was decided. The map travels in
the reviewer's package as an obligatory item (§9.9): for every line
the reviewer confirms the answer is honest — that the test really
closes the whole cut and not a corner of it; that the reason for a
refusal really holds against the code that now exists — or records a
finding.

---

## Appendix A. An example

A wave, a contract and the test that ties them live as an example in
the README, in the section "What it looks like". They are deliberately
not repeated here: an example illustrates, it does not establish, and
two copies would diverge with the first change of shape. Until the
README is updated for v2, read its example with an allowance for this
methodology's new fields: `covers`, `decisions`, `chore`, `withdrawn`,
red commits.

---

## Appendix B. What is deliberately absent

There are no requirements, no questions, no journal, no statuses, no
tags, no numbers inside a wave, no decision files. A promise is
written once, as a scenario; a question lives for hours in a PR
discussion, not for years in the graph; history is git's business; the
status is derived by chapter 6.

Absent too is what v1 had and never proved it hurt without: the nine
lines of trace (§10.6 — revoked), the machinery of "unstarted" waves
(§7.11 — revoked), the mutate command (the red commit does its work
earlier and more cheaply), the obligatory `proves` (a contract is by
need), the second PR for light waves.

Every new entity must first hurt by its absence, and every existing
one must prove from time to time that it still hurts.

---

## Appendix C. Migration from the first version

- The document format is compatible: the new fields — `covers`,
  `decisions`, `chore`, `withdrawn`, `superseded_by`, the `one new in`
  lines — are optional; v1 waves and contracts are read as they are.
- `keel/keel.json` gains `"version": 2`; the tool does not mix
  versions in one project.
- Closing is re-read by consequences (§6.5): closed v1 waves with
  green tests stay closed, and waves "orphaned" by a squash in the
  history will show the truth for the first time.
- Red commits are demanded only of new scenarios. Inherited v1
  scenarios live without a witness, and the check says of them "not
  seen red — inherited" rather than staying silent. The first change
  to an inherited scenario's text moves it onto v2's rules.
- "Silence is forbidden" (§10.3) switches on with the first new wave;
  old waves are not judged retroactively.

---

## Revision history

The number and the date. What changed is visible in the diff; why, in
the commit message; retelling it here would be a third description of
the same thing. The list is kept by hand for one reason, and it is a
real one: the document travels into projects as a copy, and the
history of the methodology's repository does not travel with it.

| Revision | Date |
|---|---|
| 1 | 2026-09-01 |

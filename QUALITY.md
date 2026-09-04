# Quality model

Forty cuts under nine headings, which every wave is walked through, and the file
Keel points at when it says "walk the cuts".

These are the product quality characteristics of **ISO/IEC 25010:2023**, in the
standard's own order, with the sub-characteristics as questions. Nothing here is
invented: the value of a standard list is that it was written by people who had
already forgotten something, and the point is to stop depending on whatever
happened to occur to one particular agent.

## When this is read

**Two passes per wave, by two different heads** (§10.2). Not one pass twice:

- the **planning pass** — the author, where the scenarios are written. Every cut
  asks about what is being planned, and whatever it turns up becomes a scenario
  or a decision, before the code;
- the **checking pass** — the reviewer, after the implementation and before the
  PR, against code that now exists: is every recorded answer honest?

Different heads, different questions: the author decides what to promise, the
reviewer checks what was delivered. What does not happen is the same head
walking the list twice — that produces the same answers at twice the price, and
that is exactly how lists stop being read.

## How to answer a cut

One of two answers, and exactly one (§10.3):

- **answered** — naming the scenario that carries this cut in its `covers`. A
  scenario that proves something narrower than the cut asks is not an answer; it
  is the next case;
- **decided** — a line in `decisions:` with the reason: "does not apply,
  because…" or "deliberately not doing this, because…". A cut about the person
  at the interface does not apply to a build file.

**There is no third answer: silence is forbidden at the field level.** A cut in
nobody's `covers` and in no `decisions` is an incomplete plan, and the check is
red before the review even starts. That is the author's duty, and the reviewer's
pass does not lift it.

**A cut that is relevant, and deliberately answered "no", is a decision.**
"Backups are not in this wave" is recoverability, said out loud. Silence is what
this list ends; refusal is not.

**Check what came with the library before claiming something is missing.** Three
agents in a row reported that the sync engine had no liveness check; all three
read the project's file, and none of them looked inside the image, where it was
built in.

**And the same the other way: a boundary is checked against the dependency the
way a cut is.** "We deliberately do not do this" is also a statement about
somebody else's code, and it can be false from the moment it is written. A wave
promised by boundary that there were no retries, while the library kept its own
counter of three: one turn went out as four requests. Nobody lied — the retries
arrived as a default. Before saying something is **absent**, look and see that it
is.

**And a third, the most expensive: a library's description is not its code.** The
question is not "is it there" but **where do you know that from** — the code that
will run, or a text about it. Three cases in a single day, all on one project. A
function existed by name and was a deprecated stub inside, printing a warning and
returning `:ok` — the declaration was grepped, the body was not read. A released
version and the `main` branch of the same library told different stories: the
release was three months old, and ten days earlier the repository had changed the
sandbox's language — one channel was asked. And a list of sandboxed functions was
taken from the documentation of a **neighbouring** library, which sandboxed them
itself — so `print` wrote happily into our stdout until a run caught it.

Nobody lied once. Each description was true about what it described.

Two minutes in the source would have settled all three. So: **read the code of
the dependency you lean on, and a run outweighs the code.** For a library whose
last release is some months old, ask the repository as well — the release says
what shipped, the repository says where it is going.

## Forty

Forty questions under nine headings — the whole of ISO/IEC 25010:2023. Written
as a list because it is walked, not read: a cut passed over without a glance is
the thing this file exists against.

### 1. Functional suitability

- **completeness** — is everything that was asked for here
- **correctness** — is the result right
- **appropriateness** — does this make the actual task easier, rather than a neighbouring one

### 2. Performance efficiency

- **time behaviour** — how long does it take
- **capacity** — how much does it hold before it stops working
- **resource utilisation** — what does it consume while it works

### 3. Compatibility

- **co-existence** — what does this take from whatever else runs on the same machine
- **interoperability** — what does it have to agree with to work at all

### 4. Interaction capability

- **appropriateness recognisability** — can a person tell what it is for
- **learnability** — can they learn it
- **operability** — can they drive it
- **user error protection** — does it stop them making a mistake
- **user engagement** — does it hold their attention
- **inclusivity** — does it work for people who read, see or move differently
- **user assistance** — does it help when they are stuck
- **self-descriptiveness** — does it explain itself

### 5. Reliability

- **faultlessness** — is it wrong in ordinary use
- **fault tolerance** — does it survive its dependencies failing
- **availability** — is it there when it is needed
- **recoverability** — what does it take to get it back after it stops

### 6. Security

- **confidentiality** — who else can see this
- **integrity** — who else can change it
- **non-repudiation** — can an act be denied afterwards
- **accountability** — is it visible who did what
- **authenticity** — is the caller who they say they are
- **resistance** — what does it do to somebody trying

### 7. Maintainability

- **modularity** — is it in one piece or several
- **reusability** — is any of it usable elsewhere
- **analysability** — can a person find out what broke
- **modifiability** — how much has to move to change it
- **testability** — can it be tested at all

### 8. Flexibility

- **adaptability** — does it survive a change of environment
- **scalability** — does it survive more of everything
- **installability** — what does installing it take
- **replaceability** — what does replacing it take

### 9. Safety

- **operational constraints** — what must never happen while it runs
- **risk identification** — which of those are known
- **fail safe** — what does it do when it fails
- **hazard warning** — does it say so before harm
- **safe integration** — what does adding it to a running system risk

**Not every cut applies to every wave, and pretending otherwise is the shortest
road to the list no longer being read.** But "does not apply" is an ANSWER too,
and it is written down: a line in `decisions:` with the reason (§10.3). Leaving
nothing behind -- no scenario, no line -- is not allowed: a cut with no answer
makes the plan incomplete, and the check is red before the review. The list
costs two passes, by two different heads; what it buys is
that the case nobody thought of can no longer be passed over in silence.
---
translates: QUALITY.md
source-rev: ff5518
---

# Quality model

Forty cuts under nine headings, which every step is walked through, and the file
Keel points at when it says "walk the cuts".

These are the product quality characteristics of **ISO/IEC 25010:2023**, in the
standard's own order, with the sub-characteristics as questions. Nothing here is
invented: the value of a standard list is that it was written by people who had
already forgotten something, and the point is to stop depending on whatever
happened to occur to one particular agent.

## When this is read

**One pass per step** — where the scenarios are written. Every cut asks about
what is being planned, and whatever it turns up becomes a scenario or a decision.
Not at every level and not until it converges: Keel has no levels, it has steps.

Before the pull request the list is **not walked a second time**. The question
there is narrower and needs no list: not "what else should be true" but "what did
we stay silent about". Whatever that finds gets closed before the PR, like
anything else.

Two full passes over the same step produce almost the same answers at twice the
price, and that is exactly how lists stop being read.

## How to answer a cut

One of three answers, and only one:

- **does not apply** — with a sentence saying why. A cut about the person at the
  interface does not apply to a build file;
- **answered** — naming the scenario that answers it. A scenario that proves
  something narrower than the cut asks is not an answer; it is the next case;
- **silent** — the cut is relevant, nothing closes it, and no decision turns it
  down. Say what specifically can go wrong on this project, and write the
  scenario that closes it.

**A cut that is relevant, and deliberately answered "no", is a decision.**
"Backups are not in this step" is recoverability, said out loud. Silence is what
this list ends; refusal is not.

**Check what came with the library before claiming something is missing.** Three
agents in a row reported that the sync engine had no liveness check; all three
read the project's file, and none of them looked inside the image, where it was
built in.

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

**Not every cut applies to every step, and pretending otherwise is the shortest
road to the list no longer being read.** A cut with nothing to say leaves nothing
behind: no scenario, no line, no note. The list costs one pass; what it buys is
that the case nobody thought of can no longer be passed over in silence.

---
translates: PRINCIPLES.md
source-rev: 4400c4
---

# Keel — principles

One page. Read always, before any work.
The full account is in `KEEL.md`; that one is a reference, read when needed.

## 1. A promise is checked, or there is no promise

A scenario is a test. No test, no promise — only a wish.

## 2. State is derived, not written

What is closed, what is open, when and by whom — counted from git and the tests.
No status is set by hand. There is no log: git is more accurate.

## 3. One step — one branch — one pull request

A step is a transition: the project's new state = step(old state).
Inside it, transforms are separate commits, each one atomic.

## 4. A transform declares the files it will touch

By name, before the work. Going beyond shows in the diff — not forbidden, but named.
If you cannot name the files in advance, the transform is not atomic yet.

## 5. Whoever leans on a text holds its revision

A test carries the scenario's revision, a transform the contract's.
The text changed after something leaned on it — the check turns red.

## 6. A contract is what code promises code

Module, exported functions, meaning. It outlives the step.
What the module actually exports is compared against what was promised.

## 7. Do not add an entity until its absence hurts

Three kinds of document. No tags, no requirements, no questions settling in.
Every new field has to hurt by being missing first.

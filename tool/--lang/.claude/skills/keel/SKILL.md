---
name: "keel"
description: "The Keel v2 loop: what to do next, and what the machine judges"
---

# keel (generated -- do not edit; keel update rewrites this file)

This project follows the Keel v2 methodology. What lives here:
`keel/waves/` (what was promised and proven), `keel/contracts/`
(promises that outlive a wave), `keel/reviews/` (a fresh reader's
verdict on each wave) and `keel.toml` (the project's config).

The loop, one step at a time:

- `keel next` -- the single next step, and nothing beyond it
- `keel status` -- where the wave stands
- `keel plan <name>` / `keel new contract <name>` -- skeletons a
  person fills; the tool never writes the content of a plan
- `keel check` -- the documents judged
- `keel close` -- whether a wave may merge
- `keel review` -- the package for a fresh reviewer (§9.9)

What a wave promises is a person's decision, never the tool's and
never an agent's alone: bring a card -- the problem, two to four
options with their consequences, a recommendation and why -- and
write the plan after their word (§8.6). `keel plan` lays the
skeleton and never the content.

Two rules a machine holds here, so no memory has to: a scenario is born red -- the commit `red: <scenario>` passes the commit-msg hook only when its test really fails -- and the work commit `<transform>: <words>` passes only when that scenario's tests are green. Ask `keel next` instead of guessing the order.

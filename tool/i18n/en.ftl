### keel tool messages -- English (the fallback language).
### One file per language; adding a language is one translated file
### plus a release (NEW-CONCEPT, "Config -> Tool output languages").

## refusal frame
word-refusal = refusal
word-reason = reason
word-instead = instead
word-green = green
word-red = red

## labels interpolated into messages
what-waves = waves
what-contracts = contracts
what-wave-header = the wave header
what-contract-header = the contract header
what-field = field "{ $name }"
what-scenario = scenario "{ $name }"
what-transform = transform "{ $name }"
what-decision-reason = the reason in decisions "{ $name }"

## docs module
docs-keel-missing = the keel/ directory is not here -- the methodology lives in keel/waves/ and keel/contracts/
docs-keel-missing-instead = create keel/waves/ and keel/contracts/, or run keel from the project root
docs-dir-missing = the directory for the { $what } documents is missing
docs-dir-missing-instead = create it -- an empty directory beats a missing one: absence is indistinguishable from a typo in the path
docs-dir-among = a directory among the { $what } documents -- documents live flat
docs-dir-among-instead = move its documents straight into this directory and remove it
docs-alien-file = a foreign file among the { $what } documents -- only .md lives here
docs-alien-file-instead = remove the file, or rename it to .md if it is a methodology document
docs-file-slug = the file name "{ $slug }" is not a slug
docs-file-slug-instead = a document's name becomes a branch and a tag (§1.2, §8.2): lowercase latin letters, digits and hyphens only
docs-unreadable = the file cannot be read: { $error }
docs-unreadable-instead = check the path and access permissions
docs-not-utf8 = the file is not UTF-8 -- methodology documents are written in UTF-8
docs-not-utf8-instead = re-save the file in UTF-8 encoding
docs-file-empty = no header: the file is empty
docs-no-header = no header: the file does not start with a --- line
docs-header-start-instead = start the file with a header -- a --- line, the fields, --- again (chapter 2)
docs-header-unclosed = the header is not closed: no second --- line found
docs-header-unclosed-instead = close the header with a --- line after the last field
docs-field-twice = field "{ $name }" is declared twice (lines { $first } and { $second })
docs-field-twice-instead = keep one entry: the methodology will not guess which of the two is right
docs-key-not-string = a field name must be a string (line { $line })
docs-key-not-string-instead = write the field name as a plain word
docs-yaml-broken = the header does not read as YAML: { $error }
docs-yaml-broken-instead = fix the markup -- the methodology writes only fields, lists and strings
docs-yaml-anchor = a YAML anchor in the header (line { $line })
docs-yaml-anchor-instead = the methodology writes no anchors -- repeat the value in words
docs-yaml-tag = a YAML tag in the header (line { $line })
docs-yaml-tag-instead = the methodology writes no tags -- remove it
docs-header-empty = the header is empty
docs-header-empty-instead = a header must carry the document's fields (chapter 2)
docs-unknown-field = { $what }: unknown field "{ $name }" (line { $line })
docs-unknown-field-instead = { $what } knows only: { $known }
docs-not-fields = { $what } must be a set of "name: value" fields (line { $line })
docs-not-fields-instead = see the shape in the README example or in keel/waves/ nearby
docs-field-blank = { $what } -- blank (line { $line })
docs-field-blank-instead = fill the field in or drop its line entirely
docs-value-blank = { $what } -- blank (line { $line })
docs-value-blank-instead = fill the value in or drop the line entirely
docs-not-string = { $what } must be a string (line { $line })
docs-not-string-instead = write the value as a single string
docs-not-list = { $what } must be a list (line { $line })
docs-not-list-instead = write it as a list: [a, b], or dash lines
docs-scenario-name-not-slug = the scenario name "{ $name }" (line { $line }) is not a slug
docs-transform-name-not-slug = the transform name "{ $name }" (line { $line }) is not a slug
docs-name-not-slug-instead = names become code (§1.2): lowercase latin letters, digits and hyphens only
docs-contract-ref-bad = { $what }: a contract reference must be "slug@revision", not "{ $value }" (line { $line })
docs-contract-ref-bad-instead = a revision is 4-6 hex characters, e.g. session-run@7c40de (§5.1-§5.2)
docs-wave-no-transforms = the wave header has no transforms -- a wave without work does not exist
docs-wave-no-transforms-instead = declare at least one transform (§2.4) or a chore (§2.11)
docs-scenario-bare = { $what } leans on nothing: neither proves nor covers
docs-scenario-bare-instead = give it footing -- a contract (proves) or a quality cut (covers), §3.3; mark a retired scenario withdrawn
docs-transform-both = { $what } has both implements and chore
docs-transform-both-instead = a transform carries exactly one thing: promises -- or a chore with a reason (§2.11)
docs-transform-neither = { $what } has neither implements nor chore
docs-transform-neither-instead = name the scenarios it advances, or write chore: "<reason>" (§2.11)
docs-transform-no-files = { $what } names no files
docs-transform-no-files-instead = files are listed by name before the work starts (§4.1)
docs-one-new-in-no-slash = { $what }: a "one new in" line must name a directory with a trailing slash (line { $line })
docs-one-new-in-no-slash-instead = write, for example: one new in priv/migrations/
docs-glob = { $what }: a glob "{ $value }" in the file list (line { $line })
docs-glob-instead = files are named one by one (§4.2); for a file with an unknown name there is one new in <dir>/
docs-exports-empty = exports is empty (line { $line })
docs-exports-empty-instead = list the signatures -- or drop the field and give verify (§2.7-§2.8)
docs-exports-no-module = exports without module: nobody is named as the one promising
docs-exports-no-module-instead = name the code unit in the module field (§2.7)
docs-contract-empty = the contract promises nothing: neither exports nor verify
docs-contract-empty-instead = give signatures with module (§2.7) or a verify command (§2.8); words without a check are a caveat in a wave, not a contract (§2.10)

## rev module
rev-missing-section = scenario "{ $name }" is declared in the header but has no "## scenario:" section in the body
rev-missing-section-instead = write the section or remove the declaration; a revision needs a body to hash (§5.3)
rev-dup-section = the "## scenario: { $name }" section appears more than once in the body
rev-dup-section-instead = keep one section: the methodology will not guess which body is the promise
rev-empty-section = the "## scenario: { $name }" section has an empty body
rev-empty-section-instead = a promise needs words: write the scenario body or withdraw the declaration (§2.3)
rev-transform-no-body = transform "{ $name }" is declared in the header but has no body section "## transform:" (§7.7)
rev-transform-no-body-instead = write the section -- the work's words live in the body -- or remove the declaration
rev-orphan-section = the body carries an orphan section "## { $kind }: { $name }" -- "{ $name }" is declared by no header entry, and an orphan does not live in silence (§7.7)
rev-orphan-section-instead = declare it in the header, or remove the section deliberately
rev-nearmiss = the heading "## { $heading }" spells the section word without its space -- not recognised as a section, and not free prose either (§7.7)
rev-nearmiss-instead = write "## scenario: <name>" / "## transform: <name>" with the space, or rename the heading away from the section words
rev-dup-transform = the body carries the "## transform: { $name }" section more than once — "{ $name }" is not guessed between (§7.7)
rev-dup-transform-instead = keep one section: the methodology will not guess which body carries the work's words (§2.10)

rev-write-title = keel rev --write — the drifted records (NEW-CONCEPT)
rev-write-needs-adapter = the rewriting hand needs the rust adapter named in keel.toml (old spelling cargo accepted): closedness is judged by tags
rev-write-needs-adapter-instead = set adapter = "rust" — the language's name; "cargo" is an accepted synonym (NEW-CONCEPT, Config); other languages come with their own waves
rev-write-rewritten = { "  " }{ $wave }: { $contract }@{ $old } → { $contract }@{ $new } — the record now holds the current revision
rev-write-kept = { "  " }{ $wave }: closed — leaving its records to history's court (§5.6)
rev-write-none = nothing has drifted in the open waves — every record they hold is current
rev-write-count = records rewritten: { $count }

## graph module
graph-unknown-cut = "{ $holder }" points at a cut "{ $slug }" that is not in the vocabulary
graph-unknown-cut-instead = the forty cuts ship with the release (§3.4); pick one of them or fix the typo
graph-double-cover = the cut "{ $slug }" has { $count } live covers: scenarios { $holders } (§10.3 -- exactly one answer)
graph-double-cover-instead = keep one cover; the other scenario stands on its proves or another cut (§3.3)
graph-double-decided = the cut "{ $slug }" is closed by scenario "{ $holder }" and decided at once (§10.3)
graph-double-decided-instead = drop the decisions line -- the scenario answers; or withdraw the cover deliberately (§2.12)
graph-silence = cuts without an answer: { $missing }
graph-silence-instead = every cut gets exactly one answer -- a covers or a decisions line (§10.3); silence is forbidden
graph-implements-missing = transform "{ $transform }" implements "{ $scenario }", which is not in the header
graph-implements-missing-instead = name an existing scenario or remove the entry (§7.1)
graph-depends-missing = depends_on points at "{ $target }", which is not a wave here
graph-depends-missing-instead = name an existing wave or drop the edge (§7.1)
graph-superseded-missing = scenario "{ $scenario }" names successor "{ $successor }", unknown to any wave
graph-superseded-missing-instead = a successor must exist somewhere (§2.12); write it first or fix the slug
graph-superseded-self = scenario "{ $scenario }" names itself as its successor
graph-superseded-self-instead = a successor is another scenario that took over (§2.12); name it or drop the mark
graph-cycle = depends_on cycle: { $chain }
graph-cycle-instead = order comes from depends_on, so it cannot loop (§7.2); break the cycle

## scope module
scope-drift = the branch touches "{ $file }", which no transform of the wave names
scope-drift-instead = name the file in a transform's files before the work (§4.1) or take the change out (§4.6)
scope-untouched = the declared file "{ $file }" is untouched by the branch
scope-untouched-instead = the work is still ahead -- or the name was idle; judged across the whole branch (§4.4, §4.5), so finish it or drop the name deliberately
scope-one-new-none = no new file appeared in "{ $dir }" -- exactly one was promised
scope-one-new-none-instead = create the one promised file, or drop the `one new in` line if none is coming (§4.1)
scope-one-new-many = more than one new file in "{ $dir }": { $files }
scope-one-new-many-instead = `one new in` promises exactly one (§4.1); name the extra files in files, each by itself
scope-one-new-count = the `one new in` lines promise { $promised } new files in "{ $dir }", the branch adds { $found }
scope-one-new-count-instead = make the counts meet (§4.1): one line per expected file, one new file per line
scope-git-failed = git refuses here: { $error }
scope-git-failed-instead = scope is judged against the branch (§4.5); make git answer in this directory and re-run keel check

## tags module
tags-stale = the tag on test "{ $test }" holds { $scenario }@{ $recorded }, the scenario's text now gives { $actual }
tags-stale-instead = reread the scenario and update the tag deliberately (§5.1, §7.5) -- or the test no longer holds what changed
tags-orphan = the tag on test "{ $test }" proves "{ $scenario }" -- no wave knows that scenario
tags-orphan-instead = name an existing scenario or drop the tag (§5.5)
tags-dangling = the tag proves: { $scenario }@{ $rev } has no test function right after it
tags-dangling-instead = put the tag on its test (§5.5); a record that holds nothing is worse than none
tags-bad-rev = the tag on "{ $scenario }" holds the record "{ $rev }" -- a revision is written as 4-6 hex characters (§5.2)
tags-bad-rev-instead = recompute with keel rev and record its prefix; a crooked record holds nothing
tags-vanished = the tag of scenario "{ $scenario }" was at the fork point and is gone at HEAD -- the scenario is alive
tags-vanished-instead = bring the test back or withdraw the scenario deliberately (§7.15, §2.12); old promises are not disarmed in silence
tags-vanished-gone = the tag of scenario "{ $scenario }" was at the fork point and is gone at HEAD -- gone with its wave: the promise was erased whole
tags-vanished-gone-instead = documents are not deleted (§4.12): bring the wave back and withdraw the scenario in its file (§2.12) -- destruction without a trace is forbidden

# -- trust: the TOFU court of commands (§7.16, §2.8) ----------------
trust-untrusted = the command "{ $command }" is not trusted (§7.16 -- new or changed does not run)
trust-untrusted-instead = record the trust: keel trust -- the line lands in the diff the merge approves
trust-ci-empty = ci is written empty -- undecided
trust-ci-empty-instead = name the command, or say "none" aloud
trust-crooked = the trust line for "{ $command }" carries a crooked fingerprint
trust-crooked-instead = rerun keel trust to rewrite it, if the command is meant to be trusted
trust-door = the trust line "{ $command }" answers to no live command -- a door opened in advance
trust-door-instead = remove the line: change or withdrawal does not inherit trust (§7.16)
trust-title = keel trust — trust recorded by fingerprint (§7.16)
trust-recorded-line = recorded: "{ $command }" = { $fingerprint }
trust-nothing-new = nothing new to trust: every verify/ci command already carries its fingerprint
trust-approves = the lines land in the diff the merge approves (§7.16)
trust-no-config = keel.toml is not here -- nowhere to prepare the trust line
trust-no-config-instead = create the config first: the trust command invents nothing
trust-surgery-broken = the surgery cannot keep this file's shape ({ $error }) -- nothing was written
trust-surgery-broken-instead = tidy the [trust] block by hand, then run keel trust again

# -- holding: the form court of contracts (§7.6, §2.9) --------------
holding-diverged = the contract "{ $contract }" promises "{ $signature }" -- the code's "{ $name }" does not match it (§2.9)
holding-diverged-instead = align the code or the promise; changing a held contract is a full wave with its impact list (§5.7)
holding-vanished = the contract "{ $contract }" promises "{ $name }" -- no such unit in the module's file (§7.6)
holding-vanished-instead = bring the unit back, or change/withdraw the contract aloud (§2.12, §5.7)
check-holding-count = signatures checked: { $count }
check-holding-uncompared = { $contract } — no one compared the form: { $why } (§7.6)
holding-why-no-adapter = no adapter named in keel.toml
holding-why-unknown-adapter = the named adapter is not of this release (it serves "rust")
holding-why-deep = the module path is deeper than this generation compares
holding-why-no-file = the module's file was not found in the crate
check-holding-plan = a plan branch: the form court does not run (§8.3) — exports may grow ahead of the code (§4.9)
check-holding-window = { $contract } — the form is not judged: the promise is grown by the approved, not started wave { $wave } (§6.5); the wave's first tag brings the court back

# -- review: the reviewer's package (§9.9) --------------------------
review-title = keel review — the reviewer's package (§9.9)
review-wave = the wave: { $wave }
review-why-header = ## Why, verbatim
review-why-missing = (the wave has no Why section)
review-scenarios-header = ## Scenarios with revisions (§5.3)
review-scenario-withdrawn = { " " }(withdrawn)
review-transforms-header = ## Transforms, verbatim — the caveats ride here (§2.10)
review-chores-header = ## Chore reasons (§2.11)
review-chores-none = none
review-drift-header = ## Drift (§4.6) — files added to scope after the anchor (the first commit of the wave file: { $sha })
review-drift-line = { $file } — added after the anchor
review-drift-removed-line = { $file } — removed from scope after the anchor
review-drift-empty = empty — no file added or removed after the anchor
review-drift-unverified = ## Drift (§4.6) not verified: history does not testify — no git, or a truncated clone cannot prove the anchor
review-drift-unreadable = the wave file at the anchor does not read — judge the drift by hand
review-map-header = ## The quality map (§10.7)
review-impact-header = ## Contract-change impact (§5.7)
review-impact-none = empty — no held contract text changed against the fork point
review-impact-unverified = not verified: no fork point to compare against
review-impact-contract = contract { $slug }: { $old } -> { $new }
review-impact-current = matches the new text
review-impact-stale = stale against the new text
review-diff-header = ## The full branch diff (against { $base })
review-diff-empty = empty
review-diff-unverified = ## The full branch diff: not verified — no fork point
review-not-wave = the branch "{ $branch }" is not named as a wave (§8.2) — the package does not guess which wave it is for
review-not-wave-instead = checkout the wave's branch: the package is assembled for the branch's wave (§9.9)
review-scenarios-none = none — a chore wave promises no scenarios (§6.8)
review-transform-no-body = (no body section — §7.7's header-vs-body floor is a rung ahead; judge it by hand)
review-protocol-header = ## What the reviewer does with this (§9.9)
review-protocol-rows = every row of every list above gets an answer — "fine, because…" — or becomes a finding; skipping a row in silence is not allowed
review-protocol-questions = above the lists, four judgement questions: what did we keep silent about; are all possible scenarios accounted for; is everything promised implemented, with no quiet narrowing; does the test cover the whole scenario, not a corner of it
review-protocol-report = the report lands as keel/reviews/{ $wave }.md next to the wave — keel close holds the wave open until it does

## adapter module
adapter-no-crate = no Cargo.toml at the root and none exactly one level down
adapter-no-crate-instead = the cargo adapter needs a crate: put Cargo.toml at the root or in one first-level directory
adapter-many-crates = several first-level crates: { $found }
adapter-many-crates-instead = the adapter does not guess; keep one crate at the first level or run keel from the crate's own project
adapter-cargo-failed = cargo refuses: { $error }
adapter-cargo-failed-instead = the judgement needs cargo running (journal A3); make cargo answer here and retry
adapter-battery-mismatch = cargo announces { $stems } targets and prints { $blocks } verdict blocks -- the stitch does not meet (a harness = false target?)
adapter-battery-mismatch-instead = the court does not judge by a shifted seam; run that target apart or give it a harness, then retry keel close

## gate module
gate-mode = mode: { $mode }
gate-mode-default = mode: strict (the default -- it does not pass itself off as read)
gate-manual = mode: manual -- the judgement is off, discipline by hand (as in v1)
gate-not-wave = branch "{ $branch }" is not named as any wave that reads -- nothing to judge, passing with this word
gate-outside = the message is neither a birth nor transform work -- outside the judgement, passing with this word
gate-chore = the transform is a chore -- no promises to run (§2.11), passing
gate-red-pass = red birth of "{ $scenario }": the test "{ $test }" truly fails -- the commit passes (§7.12)
gate-red-green = red birth of "{ $scenario }" claimed, but the test "{ $test }" is green -- an unearned "seen red" does not enter history (§7.12)
gate-red-unknown = red: names "{ $slug }", not a scenario of wave { $wave }
gate-red-withdrawn = "{ $scenario }" is withdrawn -- a dead promise is not born (§2.12)
gate-red-untagged = red birth of "{ $scenario }" claimed, but no test carries its proves tag (§5.5)
gate-red-many-tags = "{ $scenario }" carries { $count } proves tags -- which one is being born is not guessed
gate-red-broken = red birth of "{ $scenario }" claimed, but the tests do not compile -- a build break is not a failure (A3): { $words }
gate-red-notrun = red birth of "{ $scenario }" claimed, but the run executed no test named "{ $test }" -- zero runs is not a failure (A3)
gate-work-pass = transform "{ $transform }": { $count } scenario tests green with matching tags -- the work passes (§8.4)
gate-work-red = transform "{ $transform }": the test "{ $test }" of scenario "{ $scenario }" fails -- the work is not done
gate-work-stale = transform "{ $transform }": the tag of scenario "{ $scenario }" holds { $recorded }, the text gives { $actual } (§7.5)
gate-work-untagged = transform "{ $transform }": scenario "{ $scenario }" has no proves tag in the tests (§5.5)
gate-work-broken = transform "{ $transform }": the tests do not compile: { $words }
gate-work-notrun = transform "{ $transform }": the run executed no test named "{ $test }" for scenario "{ $scenario }"
gate-unknown-slug = "{ $slug }" is neither red: nor a transform of wave { $wave } -- a typo does not pass as "outside the judgement" (§8.4)
gate-case = "{ $head }" wears capitals -- red: and slugs are written lowercase (§1.2, §8.4); a capitalized twin does not pass as outside the judgement
gate-work-vacuum = transform "{ $transform }": no live scenario left to judge -- the withdrawn are outside the judgement (§2.12), passing with this word
gate-soft = mode: soft -- the same words, a warning only
gate-hook-installed = the commit-msg hook now calls keel gate -- written to { $path }
gate-adapter-unjudged = the adapter "{ $name }" is not of this release (it serves "rust") — the commit is not judged: the word stands aloud, the judgement waits for its adapter's wave
gate-adapter-absent-name = not named
gate-hook-already = the hook is already ours -- quietly the same file
gate-hook-foreign = a commit-msg hook already exists here, and it is not ours
gate-hook-foreign-instead = keel does not overwrite someone's hook (§9.7); read it and merge or remove it yourself, then re-run keel hook

## rev command
rev-title = keel rev -- current revisions
rev-next = next step: hold these revisions in proves/contracts and in test tags (§5.5); reread before updating a stale one (§5.1)

## check command
check-title = keel check -- documents (rung 1)
check-config-present = config: keel.toml (lang = { $lang })
check-config-absent = no keel.toml -- defaults in effect (lang = en); a default does not pass itself off as read
check-config-lang-default = config: keel.toml (lang not set -- default en in effect; a default does not pass itself off as read)
check-refs-count = contract references checked: { $count }
check-refs-historic = old revisions, true in the file's history, held by closed waves: { $count } (§5.6)
check-refs-historic-item = { $wave }: { $contract }@{ $recorded } -- old, true in history (§5.6)
check-refs-shallow = history is truncated (shallow clone) -- old revisions cannot be verified, and are not judged
check-refs-no-history = no git history here -- old revisions cannot be verified, and are not judged (§5.6)
check-tags-count = test tags checked: { $count }
check-trust-count = commands verify/ci judged: { $count }
check-trust-ci-none = ; ci is a refusal aloud: none
check-trust-ci-absent = ; ci is not declared
check-trust-skipped-broken = commands verify/ci not judged: a broken document may hide the very command -- fix the named files first
check-tags-skipped-no-adapter = test tags not compared: no adapter named in keel.toml -- said aloud, not painted green
check-tags-skipped-adapter = test tags not compared: adapter "{ $name }" is not of this release — it serves "rust" (old spelling "cargo"); said aloud, not painted green
check-tags-skipped-refused = test tags not compared: the adapter refused mid-way -- its refusal stands among the findings
check-scope-compared = scope: branch "{ $branch }" is the wave -- compared against { $base }
check-scope-base-main = the merge-base with main @ { $sha }
check-scope-base-first = the first commit of the branch @ { $sha } (no main here)
check-scope-skipped-not-wave = scope not compared: branch "{ $branch }" is not named as any wave that reads (§8.2) -- said aloud, not painted green
check-scope-skipped-no-git = scope not compared: git serves no branch for this root -- said aloud, not painted green
check-scope-skipped-refused = scope not compared: git refused mid-way -- its refusal stands among the findings
check-header-reads = header reads
check-no-documents = no documents yet
check-checked = checked by this floor: headers -- vocabulary and shape (chapters 2-4, §7.9); contract references and their revisions (§7.1, §7.3), an old revision judged against the file's history for closed waves (§5.6); graph links (chapter 3: cuts, silence, implements, depends_on, successors; §7.2, §10.3); scope of the branch named as a wave (§4.1, §4.4-§4.6, §4.8); scenario revisions in test tags (§5.5, §7.5) and tags vanished against the fork point (§7.15); trust of verify/ci commands against recorded fingerprints (§7.16, §2.8); the form of contracts held (§7.6, §2.9); header-vs-body both ways (§7.7); closure is judged by keel close (§6.5)
check-adapter-synonym = adapter = "cargo" is an accepted synonym — the canonical name is the language's: adapter = "rust" (NEW-CONCEPT, Config; wave 0017)
check-borders = the border of green (§7.8): a green test means it exists, matches and passes -- not that the promise is proven in essence; green form is not yet meaning. No mechanics closes that gap: the fresh reviewer holds it with the four questions (§9.9)
check-ref-missing = wave { $wave }: the reference { $contract }@{ $recorded } points to a contract file that does not exist
check-ref-missing-instead = create keel/contracts/{ $contract }.md or fix the slug (§7.1)
check-ref-stale = wave { $wave }: recorded { $contract }@{ $recorded }, the contract text now gives { $actual }
check-ref-stale-instead = reread the contract and update the reference deliberately (§5.1); if this wave is already closed, the old revision is legal (§5.6)
check-summary = summary: { $docs ->
        [one] { $docs } document
       *[other] { $docs } documents
    }, { $refusals ->
        [one] { $refusals } finding
       *[other] { $refusals } findings
    }
check-next-fix = next step: fix the named files and re-run keel check
check-next-first-wave = next step: create the first wave in keel/waves/
check-next-rung = next step: rung 21 -- a contract naming a module that does not exist must be a finding off the plan branch, not advice (review 0022 R-13)

## close command (§6.5)
close-title = keel close -- the closure court (§6.5)
close-battery = battery: { $count } tests × { $runs } runs (§7.13) — green only when green in every run
close-closed = { $wave }: closed -- every live scenario proven, references converge, the review lies next to it
close-closed-unjudged = { $wave }: closed -- every live scenario proven, the review lies next to it; { $count } references not judged: history cannot testify here (§5.6)
close-closed-light = { $wave }: closed (light) -- chores only, closed by the fact of merge
close-plan = { $wave }: approved, not started -- a plan without tests is not red (§6.5)
close-progress = { $wave }: in progress -- the missing, by name:
close-lack-untagged = scenario "{ $scenario }": no proves tag in the tests (§5.5)
close-lack-stale = scenario "{ $scenario }": the tag holds { $recorded }, the text gives { $actual } (§7.5)
close-lack-red = scenario "{ $scenario }": the test "{ $test }" is red -- not proven (§6.3)
close-lack-notrun = scenario "{ $scenario }": the battery ran no test named "{ $test }"
close-lack-flaky = scenario "{ $scenario }": the test "{ $test }" is green in { $green } of { $runs } runs — not green (§7.13)
close-lack-ref = the reference { $contract }@{ $recorded } does not converge (§6.4)
close-lack-review = the review file keel/reviews/<wave>.md is not next to the wave (§9.9)
close-needs-adapter = the closure court needs the rust adapter named in keel.toml (old spelling cargo accepted)
close-needs-adapter-instead = set adapter = "rust" — the language's name; "cargo" is an accepted synonym (NEW-CONCEPT, Config); other languages come with their own waves
close-blockers = blockers of this branch's wave { $wave }: { $count } -- a full wave does not merge unproven (§6.5, §9.9)
close-no-blockers = no blockers: this branch is named as no unclosed wave -- the states above inform
close-verify-count = verify commands judged: { $count }
close-verify-passed = verify "{ $command }" of { $contract } — passed
close-verify-failed = verify "{ $command }" of { $contract } — FAILED ({ $words }) — a broken foreign promise does not merge (§2.8)
close-verify-untrusted = verify "{ $command }" of { $contract } — did not run: not trusted (§7.16); check holds that verdict
close-verify-blockers = broken foreign promises: { $count } — the exit is red
close-verify-no-words = the command left no words
close-ci-passed = ci "{ $command }" — passed: the project's own gate is green
close-ci-failed = ci "{ $command }" — FAILED ({ $words }) — the project's own gate is red, the wave does not merge (§7.16); run the command yourself to see its whole word
close-ci-untrusted = ci "{ $command }" — did not run: not trusted (§7.16); check holds that verdict — record trust with keel trust
close-ci-none = ci = "none" — a refusal aloud, lawful; nothing runs
close-ci-undecided = ci = "" — undecided; nothing runs (check's finding)
close-ci-absent = ci not declared — nothing runs
close-ci-blocker = the project's own gate is red: ci FAILED — the exit is red
close-plan-own = the wave of this branch is approved, not started -- a plan PR merges as a plan (§6.6); the work is issued after

## map command (§10.7)
map-title = keel map -- the quality map (§10.7)
map-view-wave = the map of wave { $wave }: this branch is named as it (§8.2) -- the reviewer package item (§9.9); honesty per row stays the reviewer's work
map-view-project = the project map: branch "{ $branch }" is named as no wave -- per cut, the youngest answering wave's word
map-covered = closed: "{ $scenario }" -- { $proof }
map-proof-proven = proven (the tag matches, §6.3; the test's green is keel close's court)
map-proof-unproven = not yet proven (no matching tag)
map-proof-unread = proof not read (no adapter named in keel.toml)
map-proof-unknown = proof not read (the named adapter is not of this release — it serves "rust")
map-decided = decided: "{ $reason }"
map-unanswered = no answer -- the silence court is keel check (§10.3)
map-older = older answers: { $count }

## status command (§6.5, §6.8, §9.2)
status-title = keel status -- where we stand (§6.5, §6.8)
status-branch-wave = the branch "{ $branch }" is the wave itself -- we stand inside it (§8.2)
status-branch-plan = the branch "{ $branch }" is a plan branch -- the plan is being written (§8.2)
status-branch-other = the branch "{ $branch }" is named as no wave -- an overview without one
status-branch-none = git named no branch for this root -- the overview rides without one, never a guess
status-branch-broken = the branch "{ $branch }" is named as a wave whose document refused — mend it; the refusal rows stand below
status-wave-closed = { "  " }{ $wave } — full, closed structurally: tags match, references converge, the review lies next to it
status-wave-closed-unjudged = { "  " }{ $wave } — full, closed structurally; { $count } references not judged: history cannot testify here (§5.6)
status-wave-closed-light = { "  " }{ $wave } — light, closed by merge (§6.8)
status-wave-light-own = { "  " }{ $wave } — light, riding this branch as one PR — it closes by the fact of merge (§6.8)
status-wave-plan = { "  " }{ $wave } — full, approved, not started (§6.5)
status-wave-progress = { "  " }{ $wave } — full, in progress; the lacks, by name:
status-awaiting = { "  " }awaits its start: the wave { $wave } — the branch "{ $wave }" (§8.2)
status-counts = counted: closed { $closed }, in progress { $working }, plans { $plans }
status-no-battery = the stage here is structural (tags, references, the review) — the battery was not run: green tests are judged by close and the hook (§9.2)
status-next = onwards — keel next
status-needs-adapter = the stage eye needs the rust adapter named in keel.toml (old spelling cargo accepted): tags are the memory of stages
status-needs-adapter-instead = set adapter = "rust" — the language's name; "cargo" is an accepted synonym (NEW-CONCEPT, Config); other languages come with their own waves

## next command (§9.2, §9.10, §8.4)
next-title = keel next -- one step (§9.2)
next-needs-adapter = the step hand needs the rust adapter named in keel.toml (old spelling cargo accepted): without tags the stage would be a guess
next-needs-adapter-instead = set adapter = "rust" — the language's name; "cargo" is an accepted synonym (NEW-CONCEPT, Config); other languages come with their own waves
next-step-fix = the step: mend the document { $file } — { $reason }; instead: { $instead }
next-step-fix-more = { "  " }and { $count } more { $count ->
        [one] refusal
       *[other] refusals
    } — keel check names them all
next-step-red = the step: write the test of scenario "{ $scenario }" and commit `red: { $scenario }` — it must fail; the hook lets only a red one through (§7.12, §8.4)
next-body-label = { "  " }the body of the scenario (@{ $rev }), verbatim:
next-tag-line = { "  " }the tag in the test: /// proves: { $scenario }@{ $rev }
next-tests-dir = { "  " }the cargo adapter reads tests in { $dir }
next-step-stale = the step: the revision of scenario "{ $scenario }" drifted — the tag records { $recorded }, the body now gives { $actual }; update the test to the new body and rewrite the tag (§5.5)
next-step-transform = the step: transform "{ $name }" — work exactly in the named files, then commit `{ $name }: <words>`; the hook lets it through only green (§8.4)
next-step-chore = the step: chore "{ $name }" ({ $reason }) — work exactly in the named files, then commit `{ $name }: <words>` (§2.11, §8.4)
next-files-label = { "  " }the files:
next-section-label = { "  " }the section of "{ $name }", verbatim:
next-section-missing = { "  " }the header declares it, yet the wave body has no body section "## transform: { $name }" — keel check reddens this (§7.7); mend the body before the work
next-contract-label = { "  " }the contract { $contract }@{ $rev }, the current text verbatim:
next-contract-missing = the file of contract "{ $contract }" is missing — keel check names the broken reference (§7.1)
next-run-label = { "  " }the run of its scenarios' tests:
next-run-none = { "  " }tests of its scenarios do not exist yet — the run appears with the tags (a withdrawn scenario never gets one)
next-step-review = the step: the wave is assembled — time for the review (§9.9): gather the package with `keel review` for a fresh agent; the report lands at keel/reviews/{ $wave }.md
next-step-pr-light = the step: time for the PR — the light wave rides to its one PR (§6.8), merged by the merge-commit button (§8.7); the merge is its approval and its closure in one (§6.6, §6.5)
next-step-pr = the step: the review lies next to the wave — time for the PR, merged by the merge-commit button (§8.7); the last word on lacks belongs to keel close
next-plan-branch = the step: this is the plan branch of wave { $wave } (§8.3) — prove the plan's fullness (keel check, the map), merge the plan PR; the work will ride the branch "{ $wave }"
next-ready = { "  " }start the branch "{ $wave }" — the wave is approved and not started, its dependencies closed (§6.5, §8.2)
next-working = { "  " }the branch "{ $wave }" continues — the wave is in progress
next-all-closed = every wave is closed and none awaits — plan a new wave: this generation writes plans by hand, approval is the merge of the wave file (§6.6)

## init command (NEW-CONCEPT cross-cutting, §8.7)
init-title = keel init — the frame of the methodology, one move
init-born = { "  " }born: { $piece }
init-stands = { "  " }already stands: { $piece } — not a byte is touched
init-fed = { "  " }fed: { $piece } — .gitkeep, so the standing empty directory outlives git
init-failed = { "  " }did not stand: { $piece } — { $error }; instead: clear the obstruction and re-run keel init
init-config-header = keel.toml — the §2.9 vocabulary; uncomment to enable, the defaults stay with keel's own words
init-ignore-missing = ignore rules: git ignores nothing of the adapter's build directory ({ $path }) — add exactly this line to .gitignore: { $rule } (the frame advises; it writes no file of the project's own)
init-ignore-stands = ignore rules: the build directory ({ $path }) stands ignored — the rule comes from { $source }, and it travels with the repository
init-ignore-exclude-only = ignore rules: { $path } is ignored only by { $source }, which does not travel with the repository — add exactly this line to .gitignore: { $rule }
init-ignore-no-crate = ignore rules: the adapter found no crate to name a build directory by ({ $error })
init-ignore-no-adapter = ignore rules: no adapter of this release is named in keel.toml, so there is no build directory to name
init-ignore-unknown-adapter = ignore rules: the adapter is named "{ $name }", and this release does not serve it — its own wave will bring its build directory
init-ignore-unjudged = ignore rules: git said nothing here ({ $error }) — the rule is not judged
init-eight-seven = §8.7: turn squash and rebase merging off in the repository settings — the rule is held by the disabled button, not by memory
init-next = onwards — keel plan <the first wave>

## plan command (§10.2, §8.2, §8.5)
plan-created = born { $file } — the scaffolding is deliberately red: keel check leads (§3.3) until the plan is full, so the unfinished never merges by accident
plan-branches = the branches of §8.2: a full wave plans on "plan/{ $slug }" and works on "{ $slug }"; a light one (§6.8) rides "{ $slug }" whole
plan-branches-unread = git named no branches here — the number was judged against the disk waves only (§8.8), said aloud
plan-cuts = the author's pass (§10.2): each of the forty cuts gets its answer in covers or decisions before the code (§10.3); silence is judged by keel check
plan-next = onwards: fill the skeleton by hand — the plan's content is never written by the tool — and run keel check (§8.3)
plan-no-number = the wave name "{ $slug }" starts with no number (§8.5)
plan-no-number-instead = begin the name with the wave's number, e.g. 0042-session-loop — the number is a unique prefix, never an order
plan-number-taken = the number { $number } is already held by a wave or a branch (§8.8)
plan-number-taken-instead = take the next free number: { $next } — the tool searched the disk waves and every branch, not only main
plan-number-taken-instead-disk = take the next free number: { $next } — git named no branches here, the disk waves alone were judged (§8.8)
plan-number-huge = the number { $head } does not fit this generation's counting (§8.5)
plan-number-huge-instead = take a shorter number — the width of this generation is four digits, growing only as far as it counts
plan-write-failed = the file was not born: { $error }
plan-write-failed-instead = check the rights on the keel/ directories and repeat the birth — no stub is left behind
plan-bad-slug = the name "{ $slug }" is not a slug
plan-bad-slug-instead = a document's name becomes a branch and a tag (§1.2, §8.2): lowercase latin letters, digits and hyphens only
plan-exists = keel/waves/{ $slug }.md already exists — nothing is ever overwritten
plan-exists-instead = fill the existing file, or pick another name for a new wave
plan-skel-header = the scaffolding of wave { $slug } — fill the promises by hand, then remove these words (§10.2)
plan-skel-why = why must this wave exist — in your own words (§2.2)
plan-skel-scenario = **Given** ..., **when** ..., **then** ... (§2.3); every promise leans on proves or covers (§3.3)
plan-skel-transform = the words of the work; caveats live here (§2.10)
newc-created = born { $file } — it deliberately promises nothing yet: keel check leads (§2.9) until exports with module or verify stand
newc-next = onwards: give signatures with module (§2.7) or a verify command (§2.8) by hand — keel check says what is missing
newc-exists = keel/contracts/{ $slug }.md already exists — nothing is ever overwritten
newc-exists-instead = fill the existing file, or pick another name for a new contract
newc-skel-header = the scaffolding of contract { $slug }: no promise given yet — fill §2.7 or §2.8 and remove the scaffolding
newc-skel-body = whose words this contract lets outlive the wave — and why (§2.6)

## version command (NEW-CONCEPT, the commands table; wave 0018)
version-running = keel { $version } -- the binary answering
version-pin-held = pin keel.toml: "{ $pin }" -- held; the courts judge with this very binary
version-pin-mismatch = pin keel.toml: "{ $pin }" -- NOT this binary: the courts refuse until the pin and the binary meet
version-pin-none = the version field is not set -- no pin; the concept advises one: version = "{ $version }"
version-no-file = keel.toml is absent -- no pin, the binary above runs
version-unread = keel.toml not read ({ $reason }) -- the pin unknown; the config court says the refusal in full

update-title = keel update — the generated integrations refreshed (NEW-CONCEPT, Distribution)
generated-born = { $file } — born, generated by this release
generated-appended = { $file } — the keel block appended; the text above it is untouched
generated-refreshed = { $file } — refreshed by this release
generated-stands = { $file } — already stands as this release writes it
generated-removed = { $file } — removed by hand: a decision, not a gap; nothing is written back. To have it again, delete its line in [generated] of keel.toml and run keel update
{ $snippet }
next-unknown-agent = agent "{ $agent }" is not one this release knows: { $known }
next-unknown-agent-instead = name one of { $known } — the answer shape of a session hook is the agent's own, and an unnamed agent has no documented shape to speak in
generated-guest-empty = { $file } — this file exists and is empty, and keel writes over nothing it did not write. Delete it and run keel update, or paste this whole document into it yourself:
{ $snippet }
generated-guest-taken = { $file } — this file is yours, and keel wrote nothing into it: not one byte moved. If you want the loop's hook, add these entries under the "{ $key }" key of your own file — and if there is no such key yet, add the key with exactly this content. Do not paste them beside your file's outer braces: that would not be JSON, and merging them over your own entries would lose them:
{ $snippet }
generated-guest-edited = { $file } — this file was written by keel and has since been edited by hand, so keel wrote nothing over it. Keep it as it is, or restore these entries under the "{ $key }" key; to hand the file back to keel entirely, delete it AND its line in [generated] of keel.toml, then run keel update:
{ $snippet }
generated-foreign-file = { $file } — a file of this name is generated by keel, and what stands here is not what this release writes; nothing is recorded for it, so it is not ours: NOT overwritten (§9.7). Keep it as it is, or delete the file, then run keel update (this path is a shared namespace -- another tool may own the file)
generated-changed-file = { $file } — this file is generated by keel, and what stands here is neither what this release writes nor what was recorded (recorded { $recorded }, found { $actual }): NOT overwritten (§9.7). Keep it as it is, or delete the file AND its line in [generated] of keel.toml, then run keel update
generated-changed = { $file } — the keel block is neither what this release writes nor what was recorded (recorded { $recorded }, found { $actual }): NOT overwritten (§9.7). Keep it as it is, or delete the block AND its line in [generated] of keel.toml, then run keel update
generated-no-config = no keel.toml here: this is not a keel project, and nothing is invented. Run keel init to make it one
generated-many-blocks = { $file } — more than one keel block stands here: which one is ours is not guessed. Leave a single pair of markers and run keel update
generated-none = nothing
generated-half-marked = { $file } — one keel marker without the other: where the block ends is not guessed; repair the markers or delete both
generated-unjudged-config = the generated integrations are not judged: keel.toml is not readable (the config court says why)
generated-unread = { $file } could not be read ({ $error }) — it is not judged
generated-write-failed = { $file } could not be written ({ $error })
generated-config-failed = the digest of { $file } could not be recorded in keel.toml ({ $error })
generated-config-failed-instead = repair keel.toml so it parses, then run keel update

## CLI frame
main-unknown-command = refusal: unknown command "{ $command }"
main-unknown-command-reason = reason: this is not one of the commands keel knows
main-no-command = refusal: no command given
main-no-command-reason = reason: keel does not guess what to do
main-gate-no-message = refusal: gate needs the commit message file
main-gate-no-message-reason = reason: the judgement reads the message the commit-msg hook hands over
main-plan-no-slug = refusal: plan needs the new wave's name
main-plan-no-slug-reason = reason: the skeleton is born under the name that becomes its file and its branches (§8.2)
main-new-unknown = refusal: keel new knows only: contract
main-new-unknown-reason = reason: other document kinds are born by their own commands (waves by keel plan)
main-new-no-slug = refusal: new contract needs the contract's name
main-new-no-slug-reason = reason: the skeleton is born under the name that becomes its file (§1.4)
main-usage = instead: keel check [dir] | keel rev [--write] [dir] | keel gate <message-file> [dir] | keel close [dir] | keel map [dir] | keel review [dir] | keel status [dir] | keel next [--for <agent>] [dir] | keel plan <slug> [dir] | keel new contract <slug> [dir] | keel init [--lang <l>] [--adapter <a>] [--mode <m>] [--agents <a,b>] [--hooks|--no-hooks] [--no-ask] [dir] | keel trust [dir] | keel hook [dir] | keel version [dir] | keel update [dir]

# The settings wizard (wave 0026)
ask-lang = Which human language does this project speak? (its prose, its refusals)
ask-adapter = Which language is the code in? ("-" leaves it unnamed for now)
ask-mode = How strict is the commit court? (strict blocks, soft warns, manual is off)
ask-agents = Which agents should keel generate integrations for? (space to tick, at least one)
ask-hooks = Install the session hooks, so an agent knows the next step as it opens?
ask-unknown-field = "{ $field }" is not one of the settings this release asks about
ask-unknown-field-instead = the settings are: { $known }
ask-unknown-value = "{ $value }" is not a value the "{ $field }" setting takes
ask-unknown-value-instead = { $field } takes one of: { $known }
ask-nobody = "{ $field }" names nobody, and at least one is required
ask-nobody-instead = name at least one of { $known } — several may be ticked, at least one must be
ask-interrupted = the question about "{ $field }" was not answered: { $error }
ask-interrupted-instead = answer it, or give the answers as flags (--lang, --adapter, --mode, --agents, --hooks), or run keel init --no-ask for the plain defaults
init-config-answered = keel.toml — born from your answers

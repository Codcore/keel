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

## graph module
graph-unknown-cut = "{ $holder }" points at a cut "{ $slug }" that is not in the vocabulary
graph-unknown-cut-instead = the forty cuts ship with the release (§3.4); pick one of them or fix the typo
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

## adapter module
adapter-no-crate = no Cargo.toml at the root and none exactly one level down
adapter-no-crate-instead = the cargo adapter needs a crate: put Cargo.toml at the root or in one first-level directory
adapter-many-crates = several first-level crates: { $found }
adapter-many-crates-instead = the adapter does not guess; keep one crate at the first level or run keel from the crate's own project
adapter-cargo-failed = cargo refuses: { $error }
adapter-cargo-failed-instead = the judgement needs cargo running (journal A3); make cargo answer here and retry

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
check-tags-skipped-no-adapter = test tags not compared: no adapter named in keel.toml -- said aloud, not painted green
check-tags-skipped-adapter = test tags not compared: adapter "{ $name }" is not served on this rung (only cargo is) -- said aloud, not painted green
check-tags-skipped-refused = test tags not compared: the adapter refused mid-way -- its refusal stands among the findings
check-scope-compared = scope: branch "{ $branch }" is the wave -- compared against { $base }
check-scope-base-main = the merge-base with main @ { $sha }
check-scope-base-first = the first commit of the branch @ { $sha } (no main here)
check-scope-skipped-not-wave = scope not compared: branch "{ $branch }" is not named as any wave that reads (§8.2) -- said aloud, not painted green
check-scope-skipped-no-git = scope not compared: git serves no branch for this root -- said aloud, not painted green
check-scope-skipped-refused = scope not compared: git refused mid-way -- its refusal stands among the findings
check-header-reads = header reads
check-no-documents = no documents yet
check-checked = checked by this floor: headers -- vocabulary and shape (chapters 2-4, §7.9); contract references and their revisions (§7.1, §7.3), an old revision judged against the file's history for closed waves (§5.6); graph links (chapter 3: cuts, silence, implements, depends_on, successors; §7.2, §10.3); scope of the branch named as a wave (§4.1, §4.4-§4.6, §4.8); scenario revisions in test tags (§5.5, §7.5) and tags vanished against the fork point (§7.15); closure is judged by keel close (§6.5)
check-unchecked = not yet checked: a doubled answer per cut (§10.3), contracts holding (§7.6), header-vs-body (§7.7) -- rungs ahead
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
check-next-rung = next step: rung 6 -- the quality map (§10.7)

## close command (§6.5)
close-title = keel close -- the closure court (§6.5)
close-battery = battery: { $count } tests judged in one run
close-closed = { $wave }: closed -- every live scenario proven, references converge, the review lies next to it
close-closed-light = { $wave }: closed (light) -- chores only, closed by the fact of merge
close-plan = { $wave }: approved, not started -- a plan without tests is not red (§6.5)
close-progress = { $wave }: in progress -- the missing, by name:
close-lack-untagged = scenario "{ $scenario }": no proves tag in the tests (§5.5)
close-lack-stale = scenario "{ $scenario }": the tag holds { $recorded }, the text gives { $actual } (§7.5)
close-lack-red = scenario "{ $scenario }": the test "{ $test }" is red -- not proven (§6.3)
close-lack-notrun = scenario "{ $scenario }": the battery ran no test named "{ $test }"
close-lack-ref = the reference { $contract }@{ $recorded } does not converge (§6.4)
close-lack-review = the review file keel/reviews/<wave>.md is not next to the wave (§9.9)
close-needs-adapter = the closure court needs the cargo adapter named in keel.toml
close-needs-adapter-instead = set adapter = "cargo" (NEW-CONCEPT, Config); other adapters come with their own waves
close-blockers = blockers of this branch's wave { $wave }: { $count } -- a full wave does not merge unproven (§6.5, §9.9)
close-no-blockers = no blockers: this branch is named as no unclosed wave -- the states above inform

## CLI frame
main-unknown-command = refusal: unknown command "{ $command }"
main-unknown-command-reason = reason: this is not one of the commands keel knows
main-no-command = refusal: no command given
main-no-command-reason = reason: keel does not guess what to do
main-gate-no-message = refusal: gate needs the commit message file
main-gate-no-message-reason = reason: the judgement reads the message the commit-msg hook hands over
main-usage = instead: keel check [dir] | keel rev [dir] | keel gate <message-file> [dir] | keel close [dir] | keel hook [dir]

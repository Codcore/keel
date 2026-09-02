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

## check command
check-title = keel check -- documents (rung 1)
check-config-present = config: keel.toml (lang = { $lang })
check-config-absent = no keel.toml -- defaults in effect (lang = en); a default does not pass itself off as read
check-config-lang-default = config: keel.toml (lang not set -- default en in effect; a default does not pass itself off as read)
check-header-reads = header reads
check-no-documents = no documents yet
check-checked = checked by this floor: headers -- vocabulary and shape (chapters 2-4, §7.9)
check-unchecked = not yet checked: links (chapter 3, §7.1-§7.2), revisions (§5), scope (§4), tests (§7.5), contracts (§7.6), header-vs-body (§7.7) -- rungs ahead
check-summary = summary: { $docs ->
        [one] { $docs } document
       *[other] { $docs } documents
    }, { $refusals ->
        [one] { $refusals } refusal
       *[other] { $refusals } refusals
    }
check-next-fix = next step: fix the named files and re-run keel check
check-next-first-wave = next step: create the first wave in keel/waves/
check-next-rung = next step: rung 2 -- revisions (keel rev)

## CLI frame
main-unknown-command = refusal: unknown command "{ $command }"
main-unknown-command-reason = reason: the first bootstrap rung -- there is one command so far
main-no-command = refusal: no command given
main-no-command-reason = reason: keel does not guess what to do
main-usage = instead: keel check [dir]

//! Methodology chapter 2: documents.
//!
//! The only door through which the tool gets waves and contracts off
//! the disk (contract tool-docs). Strictness per §7.9: a header that
//! does not read is a document error, never an empty value; at the
//! same time an absence the methodology allows is not an error. A
//! refusal is interface: the file, a human reason, what to do
//! instead.

use std::path::{Path, PathBuf};

use saphyr_parser::{Event, Parser};

use crate::i18n::{t, ta};
use crate::targs;

pub use crate::refusal::Refusal;

fn refuse(file: &Path, reason: String, instead: String) -> Refusal {
    Refusal {
        file: file.to_path_buf(),
        reason,
        instead,
    }
}

/// A contract reference with a revision: `session-run@7c40de` (§5.1-§5.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractRef {
    pub slug: String,
    pub rev: String,
}

/// A scenario is a promise about behaviour (§2.3); fields per §3.1.
#[derive(Debug, Clone, Default)]
pub struct Scenario {
    pub proves: Option<ContractRef>,
    pub covers: Vec<String>,
    pub withdrawn: Option<String>,
    pub superseded_by: Option<String>,
}

/// A scope line: a path by name or `one new in <dir>/` (§4.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeLine {
    Path(String),
    OneNewIn(String),
}

/// A transform's work: promises -- or a chore with a reason (§2.11).
#[derive(Debug, Clone)]
pub enum TransformKind {
    Implements(Vec<String>),
    Chore(String),
}

/// A transform is a portion of work, exactly one commit (§2.4).
#[derive(Debug, Clone)]
pub struct Transform {
    pub kind: TransformKind,
    pub contracts: Vec<ContractRef>,
    pub files: Vec<ScopeLine>,
}

/// A wave is the unit of work (§2.1). Entry order as in the document.
#[derive(Debug, Clone, Default)]
pub struct Wave {
    pub slug: String,
    pub scenarios: Vec<(String, Scenario)>,
    pub transforms: Vec<(String, Transform)>,
    pub decisions: Vec<(String, String)>,
    pub depends_on: Vec<String>,
    pub renamed_from: Option<String>,
}

/// A contract is a promise that outlives its wave (§2.6-§2.8).
#[derive(Debug, Clone, Default)]
pub struct Contract {
    pub slug: String,
    pub module: Option<String>,
    pub exports: Vec<String>,
    pub verify: Option<String>,
    pub withdrawn: Option<String>,
    pub superseded_by: Option<String>,
    pub renamed_from: Option<String>,
}

/// Everything read under the root together with every refusal: one
/// broken file hides neither its neighbours nor itself.
#[derive(Debug, Default)]
pub struct Scan {
    pub waves: Vec<Wave>,
    pub contracts: Vec<Contract>,
    pub refusals: Vec<Refusal>,
}

/// Reads a wave file: strict header, the methodology's full vocabulary.
pub fn read_wave(path: &Path) -> Result<Wave, Refusal> {
    let slug = named_slug(path)?;
    let (yaml, off) = load_header(path)?;
    let root = parse_header(&yaml, off, path)?;
    wave_from(root, slug, path)
}

/// Reads a contract file: our promise (module + exports, §2.7) or a
/// foreign one (verify, §2.8).
pub fn read_contract(path: &Path) -> Result<Contract, Refusal> {
    let slug = named_slug(path)?;
    let (yaml, off) = load_header(path)?;
    let root = parse_header(&yaml, off, path)?;
    contract_from(root, slug, path)
}

/// Walks `keel/waves/` and `keel/contracts/` under the root. An
/// error returns only when keel/ is missing entirely; everything
/// else goes into `Scan.refusals`, so one broken file cannot hide
/// its neighbours.
pub fn scan(root: &Path) -> Result<Scan, Refusal> {
    let keel = root.join("keel");
    if !keel.is_dir() {
        return Err(refuse(
            &keel,
            t("docs-keel-missing"),
            t("docs-keel-missing-instead"),
        ));
    }
    let mut out = Scan::default();
    for file in doc_files(&keel.join("waves"), &t("what-waves"), &mut out.refusals) {
        match read_wave(&file) {
            Ok(w) => out.waves.push(w),
            Err(r) => out.refusals.push(r),
        }
    }
    for file in doc_files(
        &keel.join("contracts"),
        &t("what-contracts"),
        &mut out.refusals,
    ) {
        match read_contract(&file) {
            Ok(c) => out.contracts.push(c),
            Err(r) => out.refusals.push(r),
        }
    }
    Ok(out)
}

/// A document's name is its file name without the extension (§1.4).
/// It becomes code -- a branch name (§8.2) -- so it must be a slug
/// (§1.2).
fn named_slug(path: &Path) -> Result<String, Refusal> {
    let slug = path
        .file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    if !slug_ok(&slug) {
        return Err(refuse(
            path,
            ta("docs-file-slug", targs!("slug" => slug.clone())),
            t("docs-file-slug-instead"),
        ));
    }
    Ok(slug)
}

/// The `.md` files of a directory, sorted by name. A missing
/// directory and a foreign file in it are refusals, not silence;
/// dot-files are operating system noise and are skipped.
fn doc_files(dir: &Path, what: &str, refusals: &mut Vec<Refusal>) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => {
            refusals.push(refuse(
                dir,
                ta("docs-dir-missing", targs!("what" => what.to_string())),
                t("docs-dir-missing-instead"),
            ));
            return Vec::new();
        }
    };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            // Operating system noise (.DS_Store, .gitkeep) --
            // deliberately outside judgement.
            continue;
        }
        if path.is_dir() {
            refusals.push(refuse(
                &path,
                ta("docs-dir-among", targs!("what" => what.to_string())),
                t("docs-dir-among-instead"),
            ));
            continue;
        }
        if path.extension().is_some_and(|e| e == "md") {
            files.push(path);
        } else {
            refusals.push(refuse(
                &path,
                ta("docs-alien-file", targs!("what" => what.to_string())),
                t("docs-alien-file-instead"),
            ));
        }
    }
    files.sort();
    files
}

/// Reads the file and cuts out the YAML header: the text between the
/// first `---` line and the next such line. Returns the header text
/// and the line offset (the header starts on the file's second line).
fn load_header(path: &Path) -> Result<(String, usize), Refusal> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::InvalidData {
            refuse(path, t("docs-not-utf8"), t("docs-not-utf8-instead"))
        } else {
            refuse(
                path,
                ta("docs-unreadable", targs!("error" => e.to_string())),
                t("docs-unreadable-instead"),
            )
        }
    })?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    if text.trim().is_empty() {
        return Err(refuse(
            path,
            t("docs-file-empty"),
            t("docs-header-start-instead"),
        ));
    }
    let mut pos = 0usize;
    let mut line_no = 0usize;
    let mut body_start = 0usize;
    for raw in text.split_inclusive('\n') {
        line_no += 1;
        let line = raw.trim_end_matches(['\n', '\r']);
        if line_no == 1 {
            if line != "---" {
                return Err(refuse(
                    path,
                    t("docs-no-header"),
                    t("docs-header-start-instead"),
                ));
            }
            body_start = raw.len();
        } else if line == "---" {
            return Ok((text[body_start..pos].to_string(), 1));
        }
        pos += raw.len();
    }
    Err(refuse(
        path,
        t("docs-header-unclosed"),
        t("docs-header-unclosed-instead"),
    ))
}

// ---------------------------------------------------------------------------
// YAML -> value. Our own event receiver instead of a ready-made
// tree: ready-made ones swallow duplicate keys silently -- and
// "silently" is forbidden here.
// ---------------------------------------------------------------------------

/// A header value. The methodology writes only strings, lists and
/// field sets; numbers, booleans, anchors and tags do not live in
/// headers.
#[derive(Debug)]
enum Val {
    Str(String, usize),
    Seq(Vec<Val>, usize),
    Map(Vec<(String, usize, Val)>, usize),
}

impl Val {
    fn line(&self) -> usize {
        match self {
            Val::Str(_, l) | Val::Seq(_, l) | Val::Map(_, l) => *l,
        }
    }
}

/// Blank per YAML: a value the author never wrote.
fn is_blank(s: &str) -> bool {
    s.is_empty() || s == "~" || s == "null"
}

enum Frame {
    Seq(Vec<Val>, usize),
    Map(Vec<(String, usize, Val)>, usize, Option<(String, usize)>),
}

fn parse_header(src: &str, off: usize, file: &Path) -> Result<Val, Refusal> {
    let mut stack: Vec<Frame> = Vec::new();
    let mut root: Option<Val> = None;

    fn feed(
        stack: &mut [Frame],
        root: &mut Option<Val>,
        v: Val,
        file: &Path,
    ) -> Result<(), Refusal> {
        match stack.last_mut() {
            None => {
                *root = Some(v);
                Ok(())
            }
            Some(Frame::Seq(items, _)) => {
                items.push(v);
                Ok(())
            }
            Some(Frame::Map(fields, _, pending)) => match pending.take() {
                Some((key, key_line)) => {
                    fields.push((key, key_line, v));
                    Ok(())
                }
                None => match v {
                    Val::Str(name, line) => {
                        if let Some((_, prev, _)) = fields.iter().find(|(k, _, _)| *k == name) {
                            return Err(refuse(
                                file,
                                ta(
                                    "docs-field-twice",
                                    targs!("name" => name.clone(), "first" => *prev as u64, "second" => line as u64),
                                ),
                                t("docs-field-twice-instead"),
                            ));
                        }
                        *pending = Some((name, line));
                        Ok(())
                    }
                    other => Err(refuse(
                        file,
                        ta("docs-key-not-string", targs!("line" => other.line() as u64)),
                        t("docs-key-not-string-instead"),
                    )),
                },
            },
        }
    }

    for item in Parser::new_from_str(src) {
        let (event, span) = item.map_err(|e| {
            refuse(
                file,
                ta("docs-yaml-broken", targs!("error" => e.to_string())),
                t("docs-yaml-broken-instead"),
            )
        })?;
        let line = span.start.line() + off;
        match event {
            Event::Nothing
            | Event::StreamStart
            | Event::StreamEnd
            | Event::DocumentStart(_)
            | Event::DocumentEnd => {}
            Event::Alias(_) => {
                return Err(refuse(
                    file,
                    ta("docs-yaml-anchor", targs!("line" => line as u64)),
                    t("docs-yaml-anchor-instead"),
                ));
            }
            Event::Scalar(value, _, anchor, tag) => {
                if anchor != 0 {
                    return Err(refuse(
                        file,
                        ta("docs-yaml-anchor", targs!("line" => line as u64)),
                        t("docs-yaml-anchor-instead"),
                    ));
                }
                if tag.is_some() {
                    return Err(refuse(
                        file,
                        ta("docs-yaml-tag", targs!("line" => line as u64)),
                        t("docs-yaml-tag-instead"),
                    ));
                }
                feed(
                    &mut stack,
                    &mut root,
                    Val::Str(value.into_owned(), line),
                    file,
                )?;
            }
            Event::SequenceStart(anchor, tag) => {
                if anchor != 0 {
                    return Err(refuse(
                        file,
                        ta("docs-yaml-anchor", targs!("line" => line as u64)),
                        t("docs-yaml-anchor-instead"),
                    ));
                }
                if tag.is_some() {
                    return Err(refuse(
                        file,
                        ta("docs-yaml-tag", targs!("line" => line as u64)),
                        t("docs-yaml-tag-instead"),
                    ));
                }
                stack.push(Frame::Seq(Vec::new(), line));
            }
            Event::SequenceEnd => match stack.pop() {
                Some(Frame::Seq(items, line)) => {
                    feed(&mut stack, &mut root, Val::Seq(items, line), file)?;
                }
                _ => unreachable!("the parser closes only what it opened"),
            },
            Event::MappingStart(anchor, tag) => {
                if anchor != 0 {
                    return Err(refuse(
                        file,
                        ta("docs-yaml-anchor", targs!("line" => line as u64)),
                        t("docs-yaml-anchor-instead"),
                    ));
                }
                if tag.is_some() {
                    return Err(refuse(
                        file,
                        ta("docs-yaml-tag", targs!("line" => line as u64)),
                        t("docs-yaml-tag-instead"),
                    ));
                }
                stack.push(Frame::Map(Vec::new(), line, None));
            }
            Event::MappingEnd => match stack.pop() {
                Some(Frame::Map(fields, line, _)) => {
                    feed(&mut stack, &mut root, Val::Map(fields, line), file)?;
                }
                _ => unreachable!("the parser closes only what it opened"),
            },
        }
    }

    match root {
        Some(v) => Ok(v),
        None => Err(refuse(
            file,
            t("docs-header-empty"),
            t("docs-header-empty-instead"),
        )),
    }
}

// ---------------------------------------------------------------------------
// Vocabularies: the wave header and the contract header.
// ---------------------------------------------------------------------------

type Slot = Option<(String, usize, Val)>;

fn take(slots: &mut [Slot], name: &str) -> Option<(usize, Val)> {
    slots
        .iter_mut()
        .find(|s| s.as_ref().is_some_and(|(k, _, _)| k == name))
        .and_then(Option::take)
        .map(|(_, line, v)| (line, v))
}

/// The first field left untaken is unknown to the methodology.
fn unknown_left(slots: Vec<Slot>, what: &str, known: &str, file: &Path) -> Result<(), Refusal> {
    match slots.into_iter().flatten().next() {
        Some((name, line, _)) => Err(refuse(
            file,
            ta(
                "docs-unknown-field",
                targs!("what" => what.to_string(), "name" => name, "line" => line as u64),
            ),
            ta(
                "docs-unknown-field-instead",
                targs!("what" => what.to_string(), "known" => known.to_string()),
            ),
        )),
        None => Ok(()),
    }
}

fn as_fields(v: Val, what: &str, file: &Path) -> Result<Vec<(String, usize, Val)>, Refusal> {
    match v {
        Val::Map(fields, _) => Ok(fields),
        Val::Str(s, line) if is_blank(&s) => Err(refuse(
            file,
            ta(
                "docs-field-blank",
                targs!("what" => what.to_string(), "line" => line as u64),
            ),
            t("docs-field-blank-instead"),
        )),
        other => Err(refuse(
            file,
            ta(
                "docs-not-fields",
                targs!("what" => what.to_string(), "line" => other.line() as u64),
            ),
            t("docs-not-fields-instead"),
        )),
    }
}

fn as_text(v: Val, what: &str, file: &Path) -> Result<(String, usize), Refusal> {
    match v {
        Val::Str(s, line) if !is_blank(&s) => Ok((s, line)),
        Val::Str(_, line) => Err(refuse(
            file,
            ta(
                "docs-value-blank",
                targs!("what" => what.to_string(), "line" => line as u64),
            ),
            t("docs-value-blank-instead"),
        )),
        other => Err(refuse(
            file,
            ta(
                "docs-not-string",
                targs!("what" => what.to_string(), "line" => other.line() as u64),
            ),
            t("docs-not-string-instead"),
        )),
    }
}

fn as_texts(v: Val, what: &str, file: &Path) -> Result<Vec<(String, usize)>, Refusal> {
    match v {
        Val::Seq(items, _) => items
            .into_iter()
            .map(|item| as_text(item, what, file))
            .collect(),
        other => Err(refuse(
            file,
            ta(
                "docs-not-list",
                targs!("what" => what.to_string(), "line" => other.line() as u64),
            ),
            t("docs-not-list-instead"),
        )),
    }
}

/// A slug: what will become code -- a branch name, a test tag (§1.2).
fn slug_ok(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// `slug@revision` -- §5.1-§5.2: a revision is 4-6 hex characters.
fn as_contract_ref(s: &str, line: usize, what: &str, file: &Path) -> Result<ContractRef, Refusal> {
    let split = s.split_once('@');
    if let Some((slug, rev)) = split {
        let rev_ok = (4..=6).contains(&rev.len())
            && rev
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());
        if slug_ok(slug) && rev_ok {
            return Ok(ContractRef {
                slug: slug.to_string(),
                rev: rev.to_string(),
            });
        }
    }
    Err(refuse(
        file,
        ta(
            "docs-contract-ref-bad",
            targs!("what" => what.to_string(), "value" => s.to_string(), "line" => line as u64),
        ),
        t("docs-contract-ref-bad-instead"),
    ))
}

fn wave_from(root: Val, slug: String, file: &Path) -> Result<Wave, Refusal> {
    const KNOWN: &str = "scenarios, transforms, decisions, depends_on, renamed_from";
    let mut slots: Vec<Slot> = as_fields(root, &t("what-wave-header"), file)?
        .into_iter()
        .map(Some)
        .collect();
    let mut wave = Wave {
        slug,
        ..Wave::default()
    };

    if let Some((_, v)) = take(&mut slots, "scenarios") {
        for (name, line, val) in
            as_fields(v, &ta("what-field", targs!("name" => "scenarios")), file)?
        {
            if !slug_ok(&name) {
                return Err(refuse(
                    file,
                    ta(
                        "docs-scenario-name-not-slug",
                        targs!("name" => name.clone(), "line" => line as u64),
                    ),
                    t("docs-name-not-slug-instead"),
                ));
            }
            wave.scenarios
                .push((name.clone(), scenario_from(val, &name, file)?));
        }
    }

    if let Some((_, v)) = take(&mut slots, "transforms") {
        for (name, line, val) in
            as_fields(v, &ta("what-field", targs!("name" => "transforms")), file)?
        {
            if !slug_ok(&name) {
                return Err(refuse(
                    file,
                    ta(
                        "docs-transform-name-not-slug",
                        targs!("name" => name.clone(), "line" => line as u64),
                    ),
                    t("docs-name-not-slug-instead"),
                ));
            }
            wave.transforms
                .push((name.clone(), transform_from(val, &name, file)?));
        }
    }
    if wave.transforms.is_empty() {
        return Err(refuse(
            file,
            t("docs-wave-no-transforms"),
            t("docs-wave-no-transforms-instead"),
        ));
    }

    if let Some((_, v)) = take(&mut slots, "decisions") {
        for (name, _, val) in as_fields(v, &ta("what-field", targs!("name" => "decisions")), file)?
        {
            let (why, _) = as_text(
                val,
                &ta("what-decision-reason", targs!("name" => name.clone())),
                file,
            )?;
            wave.decisions.push((name, why));
        }
    }

    if let Some((_, v)) = take(&mut slots, "depends_on") {
        wave.depends_on = as_texts(v, &ta("what-field", targs!("name" => "depends_on")), file)?
            .into_iter()
            .map(|(s, _)| s)
            .collect();
    }

    if let Some((_, v)) = take(&mut slots, "renamed_from") {
        wave.renamed_from =
            Some(as_text(v, &ta("what-field", targs!("name" => "renamed_from")), file)?.0);
    }

    unknown_left(slots, &t("what-wave-header"), KNOWN, file)?;
    Ok(wave)
}

fn scenario_from(v: Val, name: &str, file: &Path) -> Result<Scenario, Refusal> {
    let what = ta("what-scenario", targs!("name" => name.to_string()));
    let mut slots: Vec<Slot> = as_fields(v, &what, file)?.into_iter().map(Some).collect();
    let mut sc = Scenario::default();

    if let Some((line, v)) = take(&mut slots, "proves") {
        let (s, _) = as_text(v, &format!("{what}: proves"), file)?;
        sc.proves = Some(as_contract_ref(&s, line, &what, file)?);
    }
    if let Some((_, v)) = take(&mut slots, "covers") {
        sc.covers = as_texts(v, &format!("{what}: covers"), file)?
            .into_iter()
            .map(|(s, _)| s)
            .collect();
    }
    if let Some((_, v)) = take(&mut slots, "withdrawn") {
        sc.withdrawn = Some(as_text(v, &format!("{what}: withdrawn"), file)?.0);
    }
    if let Some((_, v)) = take(&mut slots, "superseded_by") {
        sc.superseded_by = Some(as_text(v, &format!("{what}: superseded_by"), file)?.0);
    }

    unknown_left(
        slots,
        &what,
        "proves, covers, withdrawn, superseded_by",
        file,
    )?;

    if sc.proves.is_none() && sc.covers.is_empty() && sc.withdrawn.is_none() {
        return Err(refuse(
            file,
            ta("docs-scenario-bare", targs!("what" => what.clone())),
            t("docs-scenario-bare-instead"),
        ));
    }
    Ok(sc)
}

fn transform_from(v: Val, name: &str, file: &Path) -> Result<Transform, Refusal> {
    let what = ta("what-transform", targs!("name" => name.to_string()));
    let mut slots: Vec<Slot> = as_fields(v, &what, file)?.into_iter().map(Some).collect();

    let implements = take(&mut slots, "implements");
    let chore = take(&mut slots, "chore");
    let kind = match (implements, chore) {
        (Some((_, v)), None) => TransformKind::Implements(
            as_texts(v, &format!("{what}: implements"), file)?
                .into_iter()
                .map(|(s, _)| s)
                .collect(),
        ),
        (None, Some((_, v))) => {
            TransformKind::Chore(as_text(v, &format!("{what}: chore"), file)?.0)
        }
        (Some(_), Some(_)) => {
            return Err(refuse(
                file,
                ta("docs-transform-both", targs!("what" => what.clone())),
                t("docs-transform-both-instead"),
            ));
        }
        (None, None) => {
            return Err(refuse(
                file,
                ta("docs-transform-neither", targs!("what" => what.clone())),
                t("docs-transform-neither-instead"),
            ));
        }
    };

    let mut contracts = Vec::new();
    if let Some((_, v)) = take(&mut slots, "contracts") {
        for (s, line) in as_texts(v, &format!("{what}: contracts"), file)? {
            contracts.push(as_contract_ref(&s, line, &what, file)?);
        }
    }

    let files = match take(&mut slots, "files") {
        Some((_, v)) => as_texts(v, &format!("{what}: files"), file)?
            .into_iter()
            .map(|(s, line)| scope_line(&s, line, &what, file))
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    if files.is_empty() {
        return Err(refuse(
            file,
            ta("docs-transform-no-files", targs!("what" => what.clone())),
            t("docs-transform-no-files-instead"),
        ));
    }

    unknown_left(slots, &what, "implements or chore, contracts, files", file)?;
    Ok(Transform {
        kind,
        contracts,
        files,
    })
}

/// `one new in <dir>/` -- exactly one new file in the directory
/// (§4.1); the rest are paths by name. The methodology writes no
/// globs (§4.2) -- and reads none.
fn scope_line(s: &str, line: usize, what: &str, file: &Path) -> Result<ScopeLine, Refusal> {
    if let Some(dir) = s.strip_prefix("one new in ") {
        if !dir.ends_with('/') {
            return Err(refuse(
                file,
                ta(
                    "docs-one-new-in-no-slash",
                    targs!("what" => what.to_string(), "line" => line as u64),
                ),
                t("docs-one-new-in-no-slash-instead"),
            ));
        }
        return Ok(ScopeLine::OneNewIn(dir.to_string()));
    }
    if s.contains(['*', '?', '[']) {
        return Err(refuse(
            file,
            ta(
                "docs-glob",
                targs!("what" => what.to_string(), "value" => s.to_string(), "line" => line as u64),
            ),
            t("docs-glob-instead"),
        ));
    }
    Ok(ScopeLine::Path(s.to_string()))
}

fn contract_from(root: Val, slug: String, file: &Path) -> Result<Contract, Refusal> {
    const KNOWN: &str = "module, exports, verify, withdrawn, superseded_by, renamed_from";
    let mut slots: Vec<Slot> = as_fields(root, &t("what-contract-header"), file)?
        .into_iter()
        .map(Some)
        .collect();
    let mut c = Contract {
        slug,
        ..Contract::default()
    };

    if let Some((_, v)) = take(&mut slots, "module") {
        c.module = Some(as_text(v, &ta("what-field", targs!("name" => "module")), file)?.0);
    }
    if let Some((line, v)) = take(&mut slots, "exports") {
        c.exports = as_texts(v, &ta("what-field", targs!("name" => "exports")), file)?
            .into_iter()
            .map(|(s, _)| s)
            .collect();
        if c.exports.is_empty() {
            return Err(refuse(
                file,
                ta("docs-exports-empty", targs!("line" => line as u64)),
                t("docs-exports-empty-instead"),
            ));
        }
        if c.module.is_none() {
            return Err(refuse(
                file,
                t("docs-exports-no-module"),
                t("docs-exports-no-module-instead"),
            ));
        }
    }
    if let Some((_, v)) = take(&mut slots, "verify") {
        c.verify = Some(as_text(v, &ta("what-field", targs!("name" => "verify")), file)?.0);
    }
    if let Some((_, v)) = take(&mut slots, "withdrawn") {
        c.withdrawn = Some(as_text(v, &ta("what-field", targs!("name" => "withdrawn")), file)?.0);
    }
    if let Some((_, v)) = take(&mut slots, "superseded_by") {
        c.superseded_by = Some(
            as_text(
                v,
                &ta("what-field", targs!("name" => "superseded_by")),
                file,
            )?
            .0,
        );
    }
    if let Some((_, v)) = take(&mut slots, "renamed_from") {
        c.renamed_from =
            Some(as_text(v, &ta("what-field", targs!("name" => "renamed_from")), file)?.0);
    }

    unknown_left(slots, &t("what-contract-header"), KNOWN, file)?;

    if c.exports.is_empty() && c.verify.is_none() {
        return Err(refuse(
            file,
            t("docs-contract-empty"),
            t("docs-contract-empty-instead"),
        ));
    }
    Ok(c)
}

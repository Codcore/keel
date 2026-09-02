//! Глава 2 методики: документи.
//!
//! Єдині двері, якими інструмент дістає хвилі і контракти з диска
//! (контракт tool-docs). Суворість — за §7.9: шапка, що не
//! читається, — помилка документа, а не порожнє значення; водночас
//! відсутність, яку методика дозволяє, — не помилка. Відмова — це
//! інтерфейс: файл, причина людською мовою, що робити натомість.

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

/// Посилання на контракт із редакцією: `session-run@7c40de` (§5.1–§5.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractRef {
    pub slug: String,
    pub rev: String,
}

/// Сценарій — обіцянка про поведінку (§2.3); поля — словник §3.1.
#[derive(Debug, Clone, Default)]
pub struct Scenario {
    pub proves: Option<ContractRef>,
    pub covers: Vec<String>,
    pub withdrawn: Option<String>,
    pub superseded_by: Option<String>,
}

/// Рядок scope: шлях поіменно або `one new in <тека>/` (§4.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeLine {
    Path(String),
    OneNewIn(String),
}

/// Робота трансформи: обіцянки — або chore із причиною (§2.11).
#[derive(Debug, Clone)]
pub enum TransformKind {
    Implements(Vec<String>),
    Chore(String),
}

/// Трансформа — порція роботи, рівно один commit (§2.4).
#[derive(Debug, Clone)]
pub struct Transform {
    pub kind: TransformKind,
    pub contracts: Vec<ContractRef>,
    pub files: Vec<ScopeLine>,
}

/// Хвиля — одиниця роботи (§2.1). Порядок записів — як у документі.
#[derive(Debug, Clone, Default)]
pub struct Wave {
    pub slug: String,
    pub scenarios: Vec<(String, Scenario)>,
    pub transforms: Vec<(String, Transform)>,
    pub decisions: Vec<(String, String)>,
    pub depends_on: Vec<String>,
    pub renamed_from: Option<String>,
}

/// Контракт — обіцянка, що живе довше за хвилю (§2.6–§2.8).
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

/// Все прочитане під коренем разом з усіма відмовами: один зіпсований
/// файл не ховає ні сусідів, ні себе.
#[derive(Debug, Default)]
pub struct Scan {
    pub waves: Vec<Wave>,
    pub contracts: Vec<Contract>,
    pub refusals: Vec<Refusal>,
}

/// Читає файл хвилі: сувора шапка, повний словник методики.
pub fn read_wave(path: &Path) -> Result<Wave, Refusal> {
    let slug = named_slug(path)?;
    let (yaml, off) = load_header(path)?;
    let root = parse_header(&yaml, off, path)?;
    wave_from(root, slug, path)
}

/// Читає файл контракту: наша обіцянка (module + exports, §2.7) або
/// чужа (verify, §2.8).
pub fn read_contract(path: &Path) -> Result<Contract, Refusal> {
    let slug = named_slug(path)?;
    let (yaml, off) = load_header(path)?;
    let root = parse_header(&yaml, off, path)?;
    contract_from(root, slug, path)
}

/// Обходить `keel/waves/` і `keel/contracts/` під коренем. Помилка
/// повертається лише коли теки `keel/` нема взагалі; все інше — у
/// `Scan.refusals`, щоб один зіпсований файл не ховав сусідів.
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

/// Імʼя документа — імʼя файлу без розширення (§1.4). Воно стає
/// кодом — імʼям гілки (§8.2), тож мусить бути слагом (§1.2).
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

/// Список `.md` у теці, відсортований за іменем. Тека, якої нема, і
/// чужий файл у ній — відмови, не тиша; файли, що починаються з
/// крапки, — шум операційної системи, їх обходимо.
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
            // Шум операційної системи (.DS_Store, .gitkeep) — свідомо
            // поза судом.
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

/// Читає файл і вирізає YAML-шапку: текст між першим рядком `---` і
/// наступним таким самим. Повертає текст шапки і зсув рядків (шапка
/// починається з другого рядка файлу).
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
// YAML → значення. Свій приймач подій замість готового дерева, бо
// готові мовчки ковтають дублікати ключів — а «мовчки» тут заборонене.
// ---------------------------------------------------------------------------

/// Значення шапки. Методика пише лише рядки, списки і набори полів;
/// числа, булеві, якорі і теги в шапках не живуть.
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

/// Порожньо за YAML: значення, якого автор не написав.
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
                _ => unreachable!("парсер закриває лише те, що відкрив"),
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
                _ => unreachable!("парсер закриває лише те, що відкрив"),
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
// Словники: шапка хвилі і шапка контракту.
// ---------------------------------------------------------------------------

type Slot = Option<(String, usize, Val)>;

fn take(slots: &mut [Slot], name: &str) -> Option<(usize, Val)> {
    slots
        .iter_mut()
        .find(|s| s.as_ref().is_some_and(|(k, _, _)| k == name))
        .and_then(Option::take)
        .map(|(_, line, v)| (line, v))
}

/// Перше поле, яке лишилося не взятим, — невідоме методиці.
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

/// Слаг: те, що стане кодом — імʼям гілки, тегом теста (§1.2).
fn slug_ok(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// `slug@редакція` — §5.1–§5.2: редакція — 4–6 шістнадцяткових знаків.
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

    unknown_left(slots, &what, "implements або chore, contracts, files", file)?;
    Ok(Transform {
        kind,
        contracts,
        files,
    })
}

/// `one new in <тека>/` — рівно один новий файл у теці (§4.1); решта —
/// шляхи поіменно. Glob-и методика не пише (§4.2) — і не читає.
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

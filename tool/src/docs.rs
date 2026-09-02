//! Глава 2 методики: документи.
//!
//! Єдині двері, якими інструмент дістає хвилі і контракти з диска
//! (контракт tool-docs). Суворість — за §7.9: шапка, що не
//! читається, — помилка документа, а не порожнє значення; водночас
//! відсутність, яку методика дозволяє, — не помилка. Відмова — це
//! інтерфейс: файл, причина людською мовою, що робити натомість.

use std::fmt;
use std::path::{Path, PathBuf};

use saphyr_parser::{Event, Parser};

/// Відмова: файл, причина людською мовою і що зробити натомість.
#[derive(Debug, Clone)]
pub struct Refusal {
    pub file: PathBuf,
    pub reason: String,
    pub instead: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "відмова: {}\n  причина: {}\n  натомість: {}",
            self.file.display(),
            self.reason,
            self.instead
        )
    }
}

fn refuse(file: &Path, reason: String, instead: &str) -> Refusal {
    Refusal {
        file: file.to_path_buf(),
        reason,
        instead: instead.to_string(),
    }
}

/// Посилання на контракт із редакцією: `tool-docs@2ab9a9` (§5.1–§5.2).
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
    let (yaml, off) = load_header(path)?;
    let root = parse_header(&yaml, off, path)?;
    wave_from(root, slug_of(path), path)
}

/// Читає файл контракту: наша обіцянка (module + exports, §2.7) або
/// чужа (verify, §2.8).
pub fn read_contract(path: &Path) -> Result<Contract, Refusal> {
    let (yaml, off) = load_header(path)?;
    let root = parse_header(&yaml, off, path)?;
    contract_from(root, slug_of(path), path)
}

/// Обходить `keel/waves/` і `keel/contracts/` під коренем. Помилка
/// повертається лише коли теки `keel/` нема взагалі; все інше — у
/// `Scan.refusals`, щоб один зіпсований файл не ховав сусідів.
pub fn scan(root: &Path) -> Result<Scan, Refusal> {
    let keel = root.join("keel");
    if !keel.is_dir() {
        return Err(refuse(
            &keel,
            "теки keel/ тут нема — методика живе в keel/waves/ і keel/contracts/".into(),
            "створи keel/waves/ і keel/contracts/ або запусти keel з кореня проєкту",
        ));
    }
    let mut out = Scan::default();
    for file in doc_files(&keel.join("waves"), "хвилі", &mut out.refusals) {
        match read_wave(&file) {
            Ok(w) => out.waves.push(w),
            Err(r) => out.refusals.push(r),
        }
    }
    for file in doc_files(&keel.join("contracts"), "контракти", &mut out.refusals) {
        match read_contract(&file) {
            Ok(c) => out.contracts.push(c),
            Err(r) => out.refusals.push(r),
        }
    }
    Ok(out)
}

/// Імʼя документа — імʼя файлу без розширення (§1.4).
fn slug_of(path: &Path) -> String {
    path.file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned())
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
                format!("теки для документів «{what}» нема"),
                "створи її — порожня тека краща за відсутню: відсутність не відрізнити від одруку в шляху",
            ));
            return Vec::new();
        }
    };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || path.is_dir() {
            continue;
        }
        if path.extension().is_some_and(|e| e == "md") {
            files.push(path);
        } else {
            refusals.push(refuse(
                &path,
                format!("чужий файл серед документів «{what}» — тут живуть лише .md"),
                "прибери файл або перейменуй на .md, якщо це документ методики",
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
        refuse(
            path,
            format!("файл не читається: {e}"),
            "перевір шлях і права доступу",
        )
    })?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
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
                    "шапки нема: файл не починається з рядка ---".into(),
                    "почни файл шапкою — рядок ---, поля, знову --- (глава 2)",
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
        "шапка не закрита: другий рядок --- не знайдено".into(),
        "закрий шапку рядком --- після останнього поля",
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
                                format!("поле \"{name}\" оголошене двічі (рядки {prev} і {line})"),
                                "лиши один запис: методика не вгадує, котрий із двох правий",
                            ));
                        }
                        *pending = Some((name, line));
                        Ok(())
                    }
                    other => Err(refuse(
                        file,
                        format!("імʼя поля мусить бути рядком (рядок {})", other.line()),
                        "запиши імʼя поля простим словом",
                    )),
                },
            },
        }
    }

    for item in Parser::new_from_str(src) {
        let (event, span) = item.map_err(|e| {
            refuse(
                file,
                format!("шапка не читається як YAML: {e}"),
                "полагодь розмітку — методика пише лише поля, списки і рядки",
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
                    format!("якір YAML у шапці (рядок {line})"),
                    "методика не пише якорів — повтори значення словами",
                ));
            }
            Event::Scalar(value, _, _, tag) => {
                if tag.is_some() {
                    return Err(refuse(
                        file,
                        format!("тег YAML у шапці (рядок {line})"),
                        "методика не пише тегів — прибери його",
                    ));
                }
                feed(
                    &mut stack,
                    &mut root,
                    Val::Str(value.into_owned(), line),
                    file,
                )?;
            }
            Event::SequenceStart(_, tag) => {
                if tag.is_some() {
                    return Err(refuse(
                        file,
                        format!("тег YAML у шапці (рядок {line})"),
                        "методика не пише тегів — прибери його",
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
            Event::MappingStart(_, tag) => {
                if tag.is_some() {
                    return Err(refuse(
                        file,
                        format!("тег YAML у шапці (рядок {line})"),
                        "методика не пише тегів — прибери його",
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
            "шапка порожня".into(),
            "шапка мусить нести поля документа (глава 2)",
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
            format!("{what}: невідоме поле \"{name}\" (рядок {line})"),
            &format!("{what} знає лише: {known}"),
        )),
        None => Ok(()),
    }
}

fn as_fields(v: Val, what: &str, file: &Path) -> Result<Vec<(String, usize, Val)>, Refusal> {
    match v {
        Val::Map(fields, _) => Ok(fields),
        Val::Str(s, line) if is_blank(&s) => Err(refuse(
            file,
            format!("{what} — порожньо (рядок {line})"),
            "заповни поле або прибери його рядок зовсім",
        )),
        other => Err(refuse(
            file,
            format!(
                "{what} мусить бути набором полів «імʼя: значення» (рядок {})",
                other.line()
            ),
            "подивись форму в прикладі README або в keel/waves/ поруч",
        )),
    }
}

fn as_text(v: Val, what: &str, file: &Path) -> Result<(String, usize), Refusal> {
    match v {
        Val::Str(s, line) if !is_blank(&s) => Ok((s, line)),
        Val::Str(_, line) => Err(refuse(
            file,
            format!("{what} — порожньо (рядок {line})"),
            "заповни значення або прибери рядок зовсім",
        )),
        other => Err(refuse(
            file,
            format!("{what} мусить бути рядком (рядок {})", other.line()),
            "запиши значення одним рядком",
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
            format!("{what} мусить бути списком (рядок {})", other.line()),
            "запиши як список: [a, b] або рядками з дефісом",
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
        format!(
            "{what}: посилання на контракт мусить бути «slug@редакція», а не \"{s}\" (рядок {line})"
        ),
        "редакція — 4–6 шістнадцяткових знаків, як-от session-run@7c40de (§5.1–§5.2)",
    ))
}

fn wave_from(root: Val, slug: String, file: &Path) -> Result<Wave, Refusal> {
    const KNOWN: &str = "scenarios, transforms, decisions, depends_on, renamed_from";
    let mut slots: Vec<Slot> = as_fields(root, "шапка хвилі", file)?
        .into_iter()
        .map(Some)
        .collect();
    let mut wave = Wave {
        slug,
        ..Wave::default()
    };

    if let Some((_, v)) = take(&mut slots, "scenarios") {
        for (name, line, val) in as_fields(v, "поле \"scenarios\"", file)? {
            if !slug_ok(&name) {
                return Err(refuse(
                    file,
                    format!("імʼя сценарію \"{name}\" (рядок {line}) — не слаг"),
                    "імена стають кодом (§1.2): лише малі латинські літери, цифри і дефіс",
                ));
            }
            wave.scenarios
                .push((name.clone(), scenario_from(val, &name, file)?));
        }
    }

    if let Some((_, v)) = take(&mut slots, "transforms") {
        for (name, line, val) in as_fields(v, "поле \"transforms\"", file)? {
            if !slug_ok(&name) {
                return Err(refuse(
                    file,
                    format!("імʼя трансформи \"{name}\" (рядок {line}) — не слаг"),
                    "імена стають кодом (§1.2): лише малі латинські літери, цифри і дефіс",
                ));
            }
            wave.transforms
                .push((name.clone(), transform_from(val, &name, file)?));
        }
    }
    if wave.transforms.is_empty() {
        return Err(refuse(
            file,
            "шапка хвилі не має transforms — хвилі без роботи не буває".into(),
            "оголоси хоч одну трансформу (§2.4) або chore (§2.11)",
        ));
    }

    if let Some((_, v)) = take(&mut slots, "decisions") {
        for (name, _, val) in as_fields(v, "поле \"decisions\"", file)? {
            let (why, _) = as_text(val, &format!("причина в decisions \"{name}\""), file)?;
            wave.decisions.push((name, why));
        }
    }

    if let Some((_, v)) = take(&mut slots, "depends_on") {
        wave.depends_on = as_texts(v, "поле \"depends_on\"", file)?
            .into_iter()
            .map(|(s, _)| s)
            .collect();
    }

    if let Some((_, v)) = take(&mut slots, "renamed_from") {
        wave.renamed_from = Some(as_text(v, "поле \"renamed_from\"", file)?.0);
    }

    unknown_left(slots, "шапка хвилі", KNOWN, file)?;
    Ok(wave)
}

fn scenario_from(v: Val, name: &str, file: &Path) -> Result<Scenario, Refusal> {
    let what = format!("сценарій \"{name}\"");
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
    Ok(sc)
}

fn transform_from(v: Val, name: &str, file: &Path) -> Result<Transform, Refusal> {
    let what = format!("трансформа \"{name}\"");
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
                format!("{what} має і implements, і chore"),
                "трансформа несе рівно одне: обіцянки — або chore з причиною (§2.11)",
            ));
        }
        (None, None) => {
            return Err(refuse(
                file,
                format!("{what} не має ні implements, ні chore"),
                "назви, які сценарії вона наближає, — або chore: \"<причина>\" (§2.11)",
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
            format!("{what} не називає файлів"),
            "файли перелічуються поіменно до роботи (§4.1)",
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
                format!(
                    "{what}: рядок \"one new in\" мусить називати теку зі скісною рискою в кінці (рядок {line})"
                ),
                "напиши, наприклад: one new in priv/migrations/",
            ));
        }
        return Ok(ScopeLine::OneNewIn(dir.to_string()));
    }
    if s.contains(['*', '?', '[']) {
        return Err(refuse(
            file,
            format!("{what}: glob \"{s}\" у списку файлів (рядок {line})"),
            "файли називаються поіменно (§4.2); для файлу без відомого імені є one new in <тека>/",
        ));
    }
    Ok(ScopeLine::Path(s.to_string()))
}

fn contract_from(root: Val, slug: String, file: &Path) -> Result<Contract, Refusal> {
    const KNOWN: &str = "module, exports, verify, withdrawn, superseded_by, renamed_from";
    let mut slots: Vec<Slot> = as_fields(root, "шапка контракту", file)?
        .into_iter()
        .map(Some)
        .collect();
    let mut c = Contract {
        slug,
        ..Contract::default()
    };

    if let Some((_, v)) = take(&mut slots, "module") {
        c.module = Some(as_text(v, "поле \"module\"", file)?.0);
    }
    if let Some((line, v)) = take(&mut slots, "exports") {
        c.exports = as_texts(v, "поле \"exports\"", file)?
            .into_iter()
            .map(|(s, _)| s)
            .collect();
        if c.exports.is_empty() {
            return Err(refuse(
                file,
                format!("exports порожній (рядок {line})"),
                "перелічи сигнатури — або прибери поле і дай verify (§2.7–§2.8)",
            ));
        }
        if c.module.is_none() {
            return Err(refuse(
                file,
                "exports без module: не названо, хто обіцяє".into(),
                "назви одиницю коду в полі module (§2.7)",
            ));
        }
    }
    if let Some((_, v)) = take(&mut slots, "verify") {
        c.verify = Some(as_text(v, "поле \"verify\"", file)?.0);
    }
    if let Some((_, v)) = take(&mut slots, "withdrawn") {
        c.withdrawn = Some(as_text(v, "поле \"withdrawn\"", file)?.0);
    }
    if let Some((_, v)) = take(&mut slots, "superseded_by") {
        c.superseded_by = Some(as_text(v, "поле \"superseded_by\"", file)?.0);
    }
    if let Some((_, v)) = take(&mut slots, "renamed_from") {
        c.renamed_from = Some(as_text(v, "поле \"renamed_from\"", file)?.0);
    }

    unknown_left(slots, "шапка контракту", KNOWN, file)?;

    if c.exports.is_empty() && c.verify.is_none() {
        return Err(refuse(
            file,
            "контракт нічого не обіцяє: ні exports, ні verify".into(),
            "дай сигнатури з module (§2.7) або команду verify (§2.8); слова без перевірки — застереження в хвилі, не контракт (§2.10)",
        ));
    }
    Ok(c)
}

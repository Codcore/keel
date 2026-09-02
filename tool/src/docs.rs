//! Глава 2 методики: документи.
//!
//! Єдині двері, якими інструмент дістає хвилі і контракти з диска
//! (контракт tool-docs). Суворість — за §7.9: шапка, що не
//! читається, — помилка документа, а не порожнє значення; водночас
//! відсутність, яку методика дозволяє, — не помилка.

use std::fmt;
use std::path::{Path, PathBuf};

/// Відмова — інтерфейс, а не службовий шум: файл, причина людською
/// мовою і що зробити натомість.
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
    let _ = path;
    todo!("щабель 1, трансформа read-headers")
}

/// Читає файл контракту: наша обіцянка (module + exports) або чужа
/// (verify).
pub fn read_contract(path: &Path) -> Result<Contract, Refusal> {
    let _ = path;
    todo!("щабель 1, трансформа read-headers")
}

/// Обходить `keel/waves/` і `keel/contracts/` під коренем.
pub fn scan(root: &Path) -> Result<Scan, Refusal> {
    let _ = root;
    todo!("щабель 1, трансформа read-headers")
}

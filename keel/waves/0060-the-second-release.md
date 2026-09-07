---
depends_on: [0059-rails-out-of-the-box]

transforms:
  the-second-release:
    chore: "версія крейта і пін проєкту стають 1.1.0 одним комітом; теґ v1.1.0 їде на злитий коміт, щоб агент ставив інструмент із GitHub (рішення оператора 2026-09-07)"
    files:
      - tool/Cargo.toml
      - tool/Cargo.lock
      - keel.toml
      - README.md
      - tool/src/config.rs
      - tool/tests/common/versions.rs
      - keel/reviews/0060-the-second-release.md
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
decisions:
  functional.completeness: "зміряно рукою перед хвилею: реліз v1.0.0 стоїть, `sh install.sh 1.0.0` ставить його з GitHub без git і cargo; ця хвиля робить те саме для 1.1.0 — версія крейта і пін одним комітом, теґ на злитий коміт (контракт tool-release)"
  functional.appropriateness: "не застосовується"
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  functional.correctness: "не застосовується"
  interaction.user-error-protection: "не застосовується"
  compatibility.co-existence: "названо: після теґа на машині розробника стоятимуть і 1.0.0, і 1.1.0 — кожна у своєму домі ~/.keel/versions/; межа «дві теки на один пін» лишається тією самою, що названа контрактом tool-launcher"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "не застосовується"
  interaction.learnability: "не застосовується"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "не застосовується"
  interaction.self-descriptiveness: "не застосовується"
  reliability.faultlessness: "не застосовується"
  reliability.fault-tolerance: "не застосовується"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "тримає реліз: workflow збирає чотири цілі тим самим release.sh, кладе поруч .sha256 і attestation; launcher звіряє checksum до розпакування"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "названо межу: provenance доводить attestation GitHub і звіряє `gh attestation verify` — launcher цього не робить і каже про це"
  security.resistance: "не застосовується"
  maintainability.modularity: "не застосовується"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "названо чесно (рецензія 0060 R-3): суд тримає рівно ДВА місця — маніфест крейта і пін проєкту, — і вони рухаються одним комітом, бо між двома окремими кожен суд червоний (exit 2 з обох боків). Як дані число живе ще в Cargo.lock, а як ТВЕРДЖЕННЯ про теперішній світ — у прозі, якої не тримає жоден суд: цю хвилю такі речення знайдено в README, config.rs і common/versions.rs (виправлено тут) і у двох уступах tool-launcher.md (борг, названий нижче). Знаходить їх свіжий читач, не машина"
  maintainability.modifiability: "не застосовується"
  maintainability.testability: "не застосовується"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "тримає ця хвиля: після теґа агент ставить інструмент дорогою `sh install.sh 1.1.0` — з GitHub, без git і без cargo (рішення оператора 2026-09-07)"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "названо вікно: між merge і пушем теґа пін уже називає 1.1.0, а релізу ще нема — launcher у ньому відмовляє поіменно; закриває вікно пуш теґа одразу за злиттям (контракт tool-release)"
  safety.risk-identification: "не застосовується"
  safety.fail-safe: "не застосовується"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "не застосовується"
---

## Why

**Рішення оператора 2026-09-07: реліз 1.1.0 одразу після 0059** —
«треба, щоб агент встановлював його з гітхабу як полагається». Дорога
`sh install.sh <версія>` качає ОПУБЛІКОВАНИЙ теґ, тож поки теґа нема,
з GitHub ставиться 1.0.0 — інструмент без третього читання ruby, тобто
без Rails.

Порядок — той, який контракт `tool-release` записав після хвилі 0056 і
який хвиля 0058 виправила в його прозі: (1) версія крейта
(`tool/Cargo.toml`, `tool/Cargo.lock`) і пін `version` у `keel.toml`
**одним комітом** — між двома окремими комітами кожен суд піна червоний
(зміряно з обох боків: exit 2); (2) теґ `v1.1.0` на злитий коміт —
workflow збере чотири архіви з `.sha256`, і `release.sh --tag`
відмовить, якщо дерево відповідає іншим числом.

Хвиля легка (§6.8): одна chore-трансформа, контрактів не чіпає.

## transform: the-second-release

Два числа і запис.

**Борг контракту, названий тут (рецензія 0060 R-1).** Підняття числа
робить хибними два уступи `keel/contracts/tool-launcher.md`: «`keel.toml`
самого keel відтоді тримає `version = "1.0.0"`» і «збірка з гілки main
відповідає `1.0.0` так само, як теґ v1.0.0». Файл не змінено — неправдою
його зробило дерево навколо, тож §5.7 цього не побачить ніколи. Правити
його ТУТ не можна: один контрактний рядок у `files` робить хвилю повною
(§2.11, §6.8), а ця хвиля — два числа і теґ. Робимо, як зробила хвиля
0056 з тим самим боргом: називаємо вголос тут і рядком у `BACKLOG.md`,
і платить його наступна повна хвиля. Борг без імені — не борг, а
неправда, яка лишається.

**Що ще каже число, і що з цим зроблено.** Хвиля 0056 несла в chore
цілий список текстів «про світ без релізу»; цей список звужено — і
рецензент має рацію, що звужено було НЕвиміряно. Зміряно тепер:
`install.sh` каже це минулим часом, обидва `.ftl` числа не тримають,
`help_test.rs` читає `CARGO_PKG_VERSION` — правити нема чого; README,
`config.rs` і `common/versions.rs` казали `1.0.0` про теперішній світ —
виправлено в цій хвилі, і `README` дістав ще й слово про те, що ЧИСЛО
йде дорогою релізу, а не git-ref (R-2, R-4, R-5). Що опубліковано — перевіряється руками за теґом:
`sha256sum -c` над архівом і `sh install.sh 1.1.0` у чистій теці.

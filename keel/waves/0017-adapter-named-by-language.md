---
depends_on: [0016-drifted-records]

scenarios:
  adapter-named-by-language:
    proves: tool-config@840bd9
    covers: [functional.appropriateness, interaction.appropriateness-recognisability]

transforms:
  language-name:
    implements:
      - adapter-named-by-language
    contracts: [tool-config@840bd9, tool-plan@89aa74, tool-adapter-cargo@348769, tool-close@1b6b8e, tool-rev@2ef198, tool-next@ec56ff, tool-docs@2ab9a9]
    files:
      - tool/src/config.rs
      - tool/src/check.rs
      - tool/src/close.rs
      - tool/src/status.rs
      - tool/src/next.rs
      - tool/src/rev.rs
      - tool/src/holding.rs
      - tool/src/map.rs
      - tool/src/init.rs
      - tool/src/plan.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/adapter_name_test.rs
      - tool/tests/rev_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS), and the first field report lands (FIELD-0001)"
    files:
      - docs/uk/V2-PROCESS.md
      - docs/uk/FIELD-0001.md

decisions:
  functional.correctness: "свідомо без окремого тесту: rust_adapter — один matches! над двома рядками; його вживання судять наявні тести всіх поверхів"
  functional.completeness: "свідомо без нового тесту: всі девʼять місць порівняння зведені в один дім — grep за Some(\"cargo\") у src лишає нуль"
  performance.time-behaviour: "не застосовується: одне порівняння рядка"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "свідомо без нового тесту: «cargo» лишається прийнятим синонімом — жоден наявний keel.toml не ламається; тримає adapter-named-by-language"
  compatibility.interoperability: "не застосовується: git і cargo не чіпаються"
  interaction.learnability: "свідомо без нового тесту: риштування init і всі «натомість» кажуть канонічне імʼя rust"
  interaction.operability: "не застосовується: нових ручок нема"
  interaction.user-error-protection: "свідомо без нового тесту: невідомий адаптер відмовляє тими самими школами, тепер із канонічним іменем у «натомість»"
  interaction.user-assistance: "свідомо без нового тесту: синонім — слово check-а з канонічним іменем поруч"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "свідомо без нового тесту: нові тексти — ключами через i18n, доведеними в 0002"
  interaction.self-descriptiveness: "свідомо без нового тесту: слово синоніма каже і що прочитано, і як зветься канонічно"
  reliability.faultlessness: "свідомо без окремого тесту: один дім замість девʼяти порівнянь — менше місць для розбіжности"
  reliability.fault-tolerance: "не застосовується: шляхи відмов ті самі"
  reliability.availability: "не застосовується: локальний бінарник"
  reliability.recoverability: "не застосовується: стану нема"
  security.confidentiality: "не застосовується"
  security.integrity: "свідомо без нового тесту: конфіг не пишеться — лише читається"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "свідомо без фаззингу: вхід — рядок після суворого toml"
  maintainability.modularity: "свідомо без тесту: питання адаптера живе в config — суди питають, не переказують (школа 0015)"
  maintainability.reusability: "не застосовується: внутрішній метод"
  maintainability.analysability: "свідомо без нового тесту: канонічне імʼя і синонім названі в одному місці контракту"
  maintainability.modifiability: "свідомо без тесту: нова мова релізу — новий рукав matches! і свій адаптер своєю хвилею"
  maintainability.testability: "свідомо без окремого тесту: пісочниці — справжні проєкти школи 0005–0016"
  flexibility.adaptability: "свідомо: словник адаптерів цього релізу — rust (синонім cargo); elixir та інші мови концепту — своїми хвилями зі своїми адаптерами"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "свідомо без тесту: значення поля — звичайний рядок toml"
  safety.operational-constraints: "не застосовується: жодних бігів"
  safety.risk-identification: "не застосовується як окрема робота: загроза — зламаний наявний keel.toml зі старим написанням — і є сценарієм хвилі"
  safety.fail-safe: "свідомо без нового тесту: невідоме імʼя адаптера — відмова вголос, ніколи не здогад"
  safety.hazard-warning: "свідомо без нового тесту: слово синоніма — мʼяке попередження про старе написання"
  safety.safe-integration: "свідомо без тесту: новий файл — лише тест; девʼять судів міняють одне порівняння на виклик одного дому; словесний борг R-5 рецензії 0016 їде тут-таки (plan.rs і tool-plan)"
---

## Why

Рішення оператора, зафіксоване 2026-09-03: «adapter — має бути
назва мови». Буква концепту каже те саме від початку (NEW-CONCEPT,
Config: «adapter = "elixir" — мова проєкту»), а bootstrap з хвилі
0009 вимагав імени тулчейна — «cargo» — і жодного разу не назвав
цього відступом. Хвиля вирівнює інструмент із концептом: канонічне
імʼя адаптера цього релізу — `rust` (виконує його cargo-адаптер,
tool-adapter-cargo — імʼя реалізації чесно лишається за
тулчейном); старе написання «cargo» лишається прийнятим синонімом,
про який `keel check` каже словом уголос — жоден наявний keel.toml
не ламається мовчки.

Цією ж хвилею — словесний борг R-5 рецензії 0016: дім запису
plan::write_new відтоді має мешканців, що сідають поверх наявного
(rev --write), і його doc-слова та перелік писарів tool-plan
відстали — слова вирівнюються з ділом.

Відступи bootstrap, названі вголос: хвиля їде робочою гілкою сесії;
план затверджено словом оператора (§8.6, рішення 2026-09-03 у
журналі); журнал їде chore-трансформою.

## scenario: adapter-named-by-language

**Дано** проєкт із `adapter = "rust"` у keel.toml, той самий проєкт
зі старим написанням `adapter = "cargo"`, і проєкт із невідомим
`adapter = "elixir"`,
**коли** біжать суди, яким потрібен адаптер (status, next, close,
rev --write, check),
**тоді** з `rust` усі вони працюють як досі з cargo; зі старим
написанням — теж працюють, а `keel check` каже словом уголос, що
«cargo» — синонім, і називає канонічне імʼя `rust`; з невідомим —
відмова вголос, чиє «натомість» називає `rust`; риштування
`keel init` радить `# adapter = "rust"`.

## transform: language-name

`Config` дістає один дім питання — `rust_adapter()` (канонічне
`rust` або синонім `cargo`); всі девʼять порівнянь у судах (check
×3, close, status, next, rev, holding, map) питають його. `check`
друкує слово синоніма за старого написання; всі «натомість»
needs-adapter-ключів і риштування init кажуть `rust`. Разом їде
словесний борг R-5 рецензії 0016: doc-слова `plan::write_new` і
перелік писарів tool-plan (→ 89aa74) кажуть правду про мешканців
дому запису.

Застереження: семантика адаптера не міняється — виконавцем
лишається cargo-адаптер (tool-adapter-cargo стоїть як стояв);
міняються імʼя поверхні і слова; «cargo» приймається без строку
давности цього покоління — зняття синоніма, якщо прийде, буде
окремою хвилею.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10).

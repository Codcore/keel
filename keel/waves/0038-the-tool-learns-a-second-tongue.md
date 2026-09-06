---
depends_on: [0037-the-last-of-it]

scenarios:
  the-adapter-is-chosen-by-name:
    covers: [flexibility.adaptability, functional.correctness]
  ruby-tests-are-read-and-run:
    covers: [functional.completeness, reliability.faultlessness]
  a-ruby-contract-holds-its-form:
    covers: [functional.appropriateness, maintainability.analysability]

transforms:
  the-adapter-is-a-choice:
    implements:
      - the-adapter-is-chosen-by-name
    contracts: [tool-adapter-cargo@b0e68e]
    files:
      - tool/src/adapter.rs
      - tool/src/config.rs
      - tool/src/check.rs
      - tool/src/close.rs
      - tool/src/gate.rs
      - tool/src/init.rs
      - tool/src/map.rs
      - tool/src/next.rs
      - tool/src/rev.rs
      - tool/src/status.rs
      - tool/src/ask.rs
      - tool/src/generated.rs
      - keel/contracts/tool-adapter-cargo.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/adapter_choice_test.rs
  ruby-tests-are-found-and-run:
    implements:
      - ruby-tests-are-read-and-run
    files:
      - tool/src/adapter.rs
      - tool/src/lib.rs
      - tool/src/tags.rs
      - tool/src/close.rs
      - one new in tool/src/
      - keel/contracts/tool-adapter-ruby.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/ruby_tests_test.rs
  a-ruby-module-is-compared:
    implements:
      - a-ruby-contract-holds-its-form
    files:
      - tool/src/holding.rs
      - tool/src/check.rs
      - keel/contracts/tool-holding.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/ruby_holding_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0038-the-tool-learns-a-second-tongue.md

decisions:
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "тримає the-adapter-is-chosen-by-name: два адаптери живуть поруч, і жоден проєкт не міняє поведінки через появу другого"
  compatibility.interoperability: "свідомо без тесту: адаптер кличе те, що і людина в терміналі — ruby і cargo, без власних протоколів"
  interaction.appropriateness-recognisability: "свідомо без тесту: невідомий адаптер відмовляє переліком тих, що знає реліз"
  interaction.learnability: "свідомо без тесту: команд не додається — той самий keel check"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає the-adapter-is-chosen-by-name: одрук у назві мови — відмова з переліком, а не мовчазний пропуск судів"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "свідомо без тесту: суд слів проти коду (хвиля 0035) тримає підставлення в кожному новому повідомленні"
  interaction.user-assistance: "свідомо без тесту: кожна нова відмова несе «натомість»"
  reliability.fault-tolerance: "тримає ruby-tests-are-read-and-run: у ruby «не зібрався» не відрізнити кодом виходу, і суд каже це вголос замість вгадувати"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "свідомо без тесту: адаптер не пише в проєкт нічого — тільки читає і жене те, що названо"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "тримає a-ruby-contract-holds-its-form: мовне обличчя живе в адаптері, а суди питають адаптер, а не мову"
  maintainability.reusability: "не застосовується"
  maintainability.modifiability: "свідомо без тесту, і число назване чесно: третя мова — це модуль, рядок у Language::NAMES і дотик до шести місць диспетчеризації (adapter, holding, ask, next, generated, i18n); «один модуль і рядок» було б неправдою (рецензія 0038 R-11)"
  maintainability.testability: "свідомо без тесту: усі три проби будують пісочниці спільною рукою 0030 і дають git власну особу"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий числом — суди з мовним обличчям не бігли в жодному не-Rust проєкті"
  safety.fail-safe: "тримає ruby-tests-are-read-and-run: де адаптер не певен, він каже «не перевірено», а не малює зелене"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "тримає the-adapter-is-chosen-by-name: проба вибору міряє, що назване імʼя справді вмикає мовні суди, а не лише що вихід нульовий, і окремо — що Rust-файл із ruby-фікстурою в сирому рядку читається як Rust. Побайтову тотожність бази і гілки міряв рецензент; механікою хвилі вона не тримається, і це сказано, а не приписано пробі (рецензія 0038 R-12)"
---

## Why

Оператор прочитав README і спитав просто: **а де інші мови?** Rust —
мова, якою написаний сам keel, а не мова проєктів, які він має вести.

Концепт («Відкриті питання», п. 6, **рішення оператора**) назвав
стартовий набір: Elixir, Ruby, Python, TypeScript/JS. У коді — **жоден
із чотирьох**. Є лише `cargo`, і він службовий: щоб keel міг судити
сам себе. Отже інструмент сьогодні не годиться для жодного чужого
проєкту, крім Rust-ового — два суди з мовним обличчям (теги тестів
§5.5 і форма контрактів §7.6) там просто не біжать, і `keel check`
чесно про це каже, але від того вони не з'являються.

Зміряно перед планом: `adapter.rs` — 279 рядків за одним контрактом
`tool-adapter-cargo`; `config.rust_adapter()` питають **в
одинадцяти місцях у десяти модулях**. Тобто робота двоскладова, і
цією хвилею вона робиться повністю: спершу **вибір** (адаптер стає
іменем, яке реліз знає, а не єдиною зашитою мовою), тоді **друга
мова**. Друга мова — **Ruby**: рішення оператора 2026-09-05, і в
концепті вона стоїть із позначкою «обовʼязково».

Зміряно і про саму мову, до того як писати:

| питання адаптера | Ruby (minitest) |
|---|---|
| де тести | `test/**/*_test.rb` |
| один тест | `ruby -Itest <файл> -n <метод>` — 0 зелений, 1 червоний |
| батарея | той самий шлях; вивід називає `Клас#метод` на кожне падіння |
| джерело модуля | `Toy::Bar` → `lib/toy/bar.rb` |
| «впав» проти «не зібрався» | **кодом виходу не відрізнити** — обидва 1 |

Останній рядок — не дрібниця, а межа, яку норма прямо передбачила
(§7.12, «де адаптер уміє відрізнити»). Ruby не вміє кодом; вміє лише
текстом (`SyntaxError`, `LoadError`), і там, де тексту не досить,
адаптер має сказати це вголос, а не вгадати.

RSpec у цьому середовищі не встановлений, тож адаптер цієї хвилі —
**minitest**; RSpec іде окремим рядком у чергу, а не мовчазною
обіцянкою.

## scenario: the-adapter-is-chosen-by-name

**Дано** `keel.toml` із полем `adapter`,
**коли** інструмент судить проєкт,
**тоді** мовні суди веде адаптер, названий цим полем: `rust`
(синонім `cargo`) або `ruby`. Імʼя, якого реліз не знає, — **знахідка
з переліком тих, що знає**, а не мовчазний пропуск: проєкт із чужим
іменем далі дістає суди документів, звʼязків, scope і редакцій, але
зеленого над мовними судами не малюється. Проєкт без поля
лишається як був: документи, звʼязки, scope і редакції судяться,
мовні — ні, і суд каже, яких саме не було. Наявний rust-проєкт після
цієї зміни судиться **побайтово так само**.

## scenario: ruby-tests-are-read-and-run

**Дано** ruby-проєкт із minitest,
**коли** його судить `keel check`, `keel gate` чи `keel close`,
**тоді** теги `# proves: <сценарій>@<редакція>` читаються з
`test/**/*_test.rb`, один названий тест біжить окремо (народження
червоного, §7.12), а батарея біжить уся і називає кожен упалий тест його
іменем і файлом — тим самим способом, що й для Rust: суди вище не
знають, якою мовою написаний проєкт, і не мусять. Де вивід не дає відрізнити «впав» від «не
зібрався», адаптер каже це вголос і приймає падіння як падіння — межа
§7.12 названа, а не обійдена.

## scenario: a-ruby-contract-holds-its-form

**Дано** контракт із `module: Toy::Bar` у ruby-проєкті,
**коли** суд форми §7.6 звіряє його `exports`,
**тоді** він читає `lib/toy/bar.rb` і порівнює обіцяні сигнатури зі
згорнутим джерелом — так само, як робить це для Rust. Модуля, якого
нема, — знахідка зі шляхом, за яким шукали. Межа сказана вголос:
Ruby не пише типів, тож порівнюється **імʼя методу і його
параметри**, і зелена форма тут ще менше означає сенс, ніж у мові з
типами (§7.8).

## transform: the-adapter-is-a-choice

`adapter.rs` перестає бути одним зашитим cargo: адаптер — імʼя, яке
реліз знає, і за ним вибирається виконавець. `config.rs` замість
`rust_adapter()` віддає, який саме адаптер названо. На що адаптер
мусить відповідати, описує контракт `tool-adapter-cargo` — той самий
файл, бо диспетчер живе в `keel::adapter` поруч із рукою cargo, і
контракт це каже вголос замість обіцяти окремий, якого нема
(рецензія 0038 R-13).

Вибір мусить дійти й туди, куди спершу не дійшов: `ask.rs` пропонує
кожну мову, яку реліз веде — інакше `keel init --adapter ruby`
відмовляв тому, що суди вже розуміли; `next.rs` питає адаптер, де
лежать тести і якою командою жене один; `generated.rs` пише крок
батареї мовою проєкту, а де мови не названо — рядок про те, що кроку
нема і чому; `init.rs` не каже ruby-проєктові, що не знайшов його
крейта (R-5, R-9, R-18).

## transform: ruby-tests-are-found-and-run

Виконавець для Ruby: пошук тестових файлів, біг одного тесту, біг
батареї, розбір падінь і чесна межа про «не зібрався». Вироки батареї
читаються з власного голосу minitest (`-v`), а не з тексту файлу:
читання тексту робило зеленим файл, який не завантажився, надувало
число тестів методами, яких minitest не жене, і тягло зелений тест
червоним за іменем, яке він лише починає (R-1, R-6, R-20). Читач
тегів `tags.rs` при цьому питає **імʼя файлу**, а не мову проєкту:
`#` — це коментар у ruby і паркан сирого рядка в Rust (R-3).

## transform: a-ruby-module-is-compared

`holding.rs` питає адаптер, де лежить джерело модуля, замість знати
це про Rust, — і якими знаками в цій мові відкривається коментар:
рустівські `//` і `/* */` у ruby-файлі пускали закоментований метод
за живий і різали URL навпіл (R-4). Межу §7.6 для мови без типів
каже сам `keel check` рядком меж, а не лише проза хвилі.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10). BACKLOG
втрачає два рядки з пʼяти, README перестає казати «одна мова». Сюди ж
лягає звіт рецензії.

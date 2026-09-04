---
depends_on: [0034-what-the-audits-found]

scenarios:
  a-contract-names-a-unit-that-exists:
    covers: [functional.correctness, security.integrity]
  the-branch-can-be-named-where-git-hides-it:
    covers: [compatibility.co-existence, reliability.faultlessness]
  a-scenario-name-belongs-to-one-wave:
    covers: [functional.completeness, security.non-repudiation]
  the-tool-answers-when-asked-for-help:
    covers: [interaction.learnability, interaction.user-error-protection]

transforms:
  a-missing-module-is-a-finding:
    implements:
      - a-contract-names-a-unit-that-exists
    files:
      - tool/src/holding.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/module_exists_test.rs
      - tool/tests/check_test.rs
      - tool/tests/close_test.rs
      - tool/tests/holding_test.rs
  the-branch-said-aloud:
    implements:
      - the-branch-can-be-named-where-git-hides-it
    files:
      - tool/src/scope.rs
      - tool/src/generated.rs
      - .github/workflows/keel.yml
      - tool/tests/named_branch_test.rs
  one-name-one-home:
    implements:
      - a-scenario-name-belongs-to-one-wave
    contracts: [tool-graph@5c19a5]
    files:
      - tool/src/graph.rs
      - tool/src/check.rs
      - keel/contracts/tool-graph.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/one_home_test.rs
      - tool/tests/graph_test.rs
      - tool/tests/holding_test.rs
  the-tool-knows-its-own-words:
    implements:
      - the-tool-answers-when-asked-for-help
    files:
      - tool/src/main.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/help_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - keel/reviews/0035-what-hurts-on-a-live-project.md
  the-generator-owns-what-it-writes:
    chore: "рецензія R-9: рядок, дописаний рукою в згенерований файл, лишає лагодження в цьому репозиторії — його місце в генераторі, і сторож класу це тепер міряє"
    files:
      - tool/src/generated.rs
      - .github/workflows/keel.yml
      - keel.toml
      - tool/tests/generated_stands_test.rs

decisions:
  functional.appropriateness: "свідомо без тесту: доречність зміряна тим, що це перші чотири речі, які вкусять чужий проєкт — оператор так і сказав, і аудит багів назвав кожну числом"
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.interoperability: "свідомо без тесту: імʼя гілки береться зі змінної середовища, яку CI кладе сам — нічого нового поза git"
  interaction.appropriateness-recognisability: "тримає the-tool-answers-when-asked-for-help: людина набирає --help першим, і це має бути відповідь, а не відмова про неіснуючу теку"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "тримає the-tool-answers-when-asked-for-help: у виводі не лишається незаміненого підставлення — сьогодні там стирчить { $snippet }"
  interaction.user-assistance: "свідомо без тесту: кожна нова відмова несе «натомість»"
  reliability.fault-tolerance: "тримає the-branch-can-be-named-where-git-hides-it: коли гілки не знає ні git, ні середовище, суд каже «не перевірено» — як і досі, але тепер це справді останній випадок, а не перший"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "не застосовується"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "свідомо без тесту: кожна нова знахідка називає контракт, модуль або хвилю поіменно"
  maintainability.modifiability: "не застосовується"
  maintainability.testability: "свідомо без тесту: усі чотири проби будують пісочниці спільною рукою 0030"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: кожну з чотирьох загроз названо числом в аудитах"
  safety.fail-safe: "не застосовується"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "тримає a-contract-names-a-unit-that-exists: суди стають суворішими, і це може почервонити чужий проєкт, який жив на хибному зеленому — саме тому кожна нова відмова каже, що робити"
---

## Why

Оператор сказав: спершу те, що болітиме на живому проєкті. Це не
рішення про норму (вони їдуть наступною хвилею) — це чотири речі, які
вкусять чужу людину в перший тиждень.

**1. Контракт, що називає неіснуючий модуль, дає зелене.** Аудит
багів B3: суд форми ріже `module` по `::`, і якщо сегмент один —
читає `<крейт>/src/lib.rs`, **хоч би що стояло в полі**. Контракт із
`module: /etc/passwd` дає «сигнатур звірено: 1» і **нуль знахідок**.
Аудит відповідності зміряв те саме мʼякше: `module: keel::nowhere` дає
73 замість 74 і чесний рядок «форму ніхто не порівнював» — але не
червоне. Отже перейменування модуля **тихо роззброює** §7.6 з усіма
його сигнатурами. Це дзеркало §7.15, який такого зникнення тестам не
пробачає.

**2. У CI на `pull_request` scope не звіряється взагалі.** Аудит
відповідності, ВАЖКА-5: `actions/checkout` для цієї події лишає
detached HEAD, git гілки не дає, і суд scope **пропускається цілком**
— «названо вголос», але порівняння не було. §4.10 прямо передбачив
цей випадок і велить «назвати гілку явно»; способу назвати її нема:
ані прапорця, ані змінної середовища.

**3. Два однойменні сценарії в різних хвилях — один тест закриває
обидві.** Аудит багів B4: копія хвилі під новим номером, той самий
слаг сценарію — `keel close` каже «закрита» **обом**, хоч у другої
нема жодного власного тесту. Норма ніде не каже, що слаги сценаріїв
унікальні (аудит норми С-9), а машина індексує теги **голим імʼям**.

**4. `--help` ковтається як шлях до теки.** Аудит багів B9: перше, що
людина набирає, дає відмову про неіснуючу теку `--help`. Там же B10:
зайві аргументи мовчки ігноруються в сімнадцяти командах із двадцяти,
і B8: у виводі стирчить незамінений `{ $snippet }` — обома мовами.

## scenario: a-contract-names-a-unit-that-exists

**Дано** контракт із полем `module`,
**коли** `keel check` судить форму,
**тоді** модуль, якого в коді нема, — **знахідка**, і вона називає
контракт і шлях, за яким його шукали. Однослівне імʼя більше не
підмінюється мовчки на `src/lib.rs`: суд шукає саме те, що написано,
і каже, де шукав. Сигнатури такого контракту не рахуються звіреними.

## scenario: the-branch-can-be-named-where-git-hides-it

**Дано** робочу теку, де git гілки не дає (detached HEAD, як у CI на
`pull_request`),
**коли** середовище називає гілку саме (`KEEL_BRANCH`),
**тоді** суд scope **відбувається** проти неї, а не пропускається. Де
не названо ніде — лишається чесне «не перевірено», але це тепер
останній випадок, а не перший.

## scenario: a-scenario-name-belongs-to-one-wave

**Дано** дві хвилі з однаковим імʼям сценарію,
**коли** їх читає `keel check`,
**тоді** це **знахідка**, що називає обидві хвилі й імʼя: тег тесту
голий, і машина не має способу знати, чию обіцянку він доводить. Одне
імʼя — один дім.

## scenario: the-tool-answers-when-asked-for-help

**Дано** людину, що вперше бачить інструмент,
**коли** вона набирає `keel --help` (чи `-h`),
**тоді** вона дістає перелік команд, а не відмову про теку з такою
назвою. Невідомий прапорець — **відмова з переліком**, а не мовчазна
тека. Зайвий аргумент — відмова, у кожній команді, а не в трьох із
двадцяти. І в жодному рядку виводу не лишається незаміненого
підставлення.

## transform: a-missing-module-is-a-finding

`holding.rs` шукає названий модуль чесно — саме там, де написано, з
обома законними розкладками (`src/a.rs` і `src/a/mod.rs`), на всю
глибину імені — і сам віддає знахідку з шляхом, за яким шукав. Імʼя,
що виводить за межі crate (скісна риска, `..`), не шукається зовсім:
про нього окрема відповідь. Рецензія 0035 R-13: тут стояло «`check.rs`
робить із неї знахідку», а `check.rs` цією трансформою не чіпаний —
знахідку віддає сам суд форми.

## transform: the-branch-said-aloud

`scope.rs` питає середовище, коли git мовчить.

## transform: one-name-one-home

`graph.rs` бачить однойменні сценарії у двох живих хвилях — і імʼя
обіцянки, зайняте слаґом контракту (рецензія R-17): простір імен один,
бо тег тесту голий. `check.rs` перестає прощати живій обіцянці її суди
через однойменну зняту в іншій хвилі (R-4) і подає судові слаґи живих
контрактів.

## transform: the-tool-knows-its-own-words

`main.rs` вчиться `--help`, невідомим прапорцям і зайвим аргументам —
у кожній команді, включно з тими, що беруть власне слово перед текою
(`gate`, `plan`, `new`), і `--help` та `--version` відповідають
усюди, де їх набрано. Прапорець читається там, де стоїть. Довідка і
рядок «натомість» звучать мовою проєкту. Підставлення в i18n стає
повним, і повноту тримає не обхід кількох доріг, а суд самого файлу
слів проти коду: повідомлення, що називає аргумент, мусить його
діставати.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10). BACKLOG
втрачає чотири рядки, які ця хвиля закриває. Сюди ж лягає звіт
рецензії.

## transform: the-generator-owns-what-it-writes

Рядки, яких потребує `.github/workflows/keel.yml`, пише генератор, а
не рука: інакше лагодження лишається в цьому репозиторії, а кожен
чужий проєкт дістає workflow без них, і §9.7 назавжди відмовляється
чіпати «правлений рукою» файл. Сторож класу міряє це на самому
репозиторії: жоден названий у `[generated]` файл не сміє розходитися
з тим, що пише реліз.

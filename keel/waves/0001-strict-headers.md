---
scenarios:
  broken-header-refuses:
    proves: tool-docs@2ab9a9
    covers: [safety.fail-safe, maintainability.analysability]
  unknown-field-refuses:
    proves: tool-docs@2ab9a9
    covers: [interaction.user-error-protection]
  valid-wave-parses:
    proves: tool-docs@2ab9a9
    covers: [functional.correctness, reliability.faultlessness]
  valid-contract-parses:
    proves: tool-docs@2ab9a9
    covers: [functional.completeness]
  duplicate-name-refuses:
    proves: tool-docs@2ab9a9
  check-reports-every-file:
    proves: tool-docs@2ab9a9
    covers: [reliability.fault-tolerance, safety.hazard-warning, interaction.self-descriptiveness]
  missing-keel-dir-refuses:
    covers: [interaction.user-assistance]
  dir-among-docs-refuses:
    proves: tool-docs@2ab9a9
  bare-scenario-refuses:
    proves: tool-docs@2ab9a9

transforms:
  read-headers:
    implements:
      - broken-header-refuses
      - unknown-field-refuses
      - valid-wave-parses
      - valid-contract-parses
      - duplicate-name-refuses
    contracts: [tool-docs@2ab9a9]
    files:
      - tool/.gitignore
      - tool/Cargo.toml
      - tool/Cargo.lock
      - tool/src/main.rs
      - tool/src/lib.rs
      - tool/src/docs.rs
      - tool/tests/docs_test.rs
      - .github/workflows/tool-ci.yml
  check-walks-project:
    implements:
      - check-reports-every-file
      - missing-keel-dir-refuses
    contracts: [tool-docs@2ab9a9]
    files:
      - tool/src/main.rs
      - tool/src/check.rs
      - tool/tests/check_test.rs
  review-findings:
    implements:
      - dir-among-docs-refuses
      - bare-scenario-refuses
    contracts: [tool-docs@2ab9a9]
    files:
      - tool/src/docs.rs
      - tool/src/check.rs
      - tool/tests/docs_test.rs
      - tool/tests/check_test.rs

decisions:
  functional.appropriateness: "свідомо без тесту: доречність check судять наступні щаблі самонаведення — вони його перші споживачі"
  performance.time-behaviour: "свідомо не міряємо: вимога мілісекунд стоїть у концепті і почне боліти у хвилі hook-ів — там і вимір"
  performance.capacity: "не застосовується: документів у проєкті десятки, не тисячі — межі місткості нема чого міряти"
  performance.resource-utilisation: "свідомо не міряємо: один прохід читання малих файлів; стане питанням разом із hook-ами"
  compatibility.co-existence: "не застосовується: check читає файли і виходить — не тримає портів, локів чи демонів"
  compatibility.interoperability: "свідомо без окремого тесту: єдина домовленість щабля — YAML-шапка, і її тримають сценарії розбору"
  interaction.appropriateness-recognisability: "свідомо не робимо: «кожна команда друкує наступний крок» — правило концепту для всіх команд, перевіриться CLI-хвилею"
  interaction.learnability: "не застосовується: одна команда без опцій — вчитися ще нема чого"
  interaction.operability: "свідомо не робимо: керування (--json та інші опції) прийде CLI-хвилею"
  interaction.user-engagement: "не застосовується: інструмент перевірки не має тримати увагу"
  interaction.inclusivity: "свідомо не робимо: вивід — простий текст без кольору; кольори і їх вимикання — питання CLI-хвилі"
  reliability.availability: "не застосовується: локальний бінарник — доступність вирішує щабель launcher-а, не цей"
  reliability.recoverability: "не застосовується: у check нема стану — перезапуск і є відновлення"
  security.confidentiality: "не застосовується: читає файли репозиторію, які бачить кожен, хто має репозиторій; нікуди нічого не шле"
  security.integrity: "свідомо без тесту: модуль нічого не пише на диск — обіцяно контрактом, видно в коді"
  security.non-repudiation: "не застосовується: дій, що змінюють стан, щабель не має"
  security.accountability: "не застосовується: жодної зміни стану — нема чого обліковувати"
  security.authenticity: "не застосовується: нікого не автентифікує; довіра до команд — щабель TOFU"
  security.resistance: "свідомо не робимо фаззингу понад суворий розбір: зіпсований вхід — відмова вголос (сценарії); фаззинг додамо, коли заболить"
  maintainability.modularity: "свідомо без тесту: будову «модуль на главу» тримає концепт і око рецензента"
  maintainability.reusability: "не застосовується: внутрішній модуль інструмента, не бібліотека"
  maintainability.modifiability: "свідомо без тесту: міра «скільки зрушити» — судження рецензента, не автоматика"
  maintainability.testability: "свідомо без окремого тесту: кожна перевірка народжується червоним commit-ом — неперевірна не пройде власного народження"
  flexibility.adaptability: "свідомо не робимо: крос-платформність заявлена концептом і перевіриться CI-матрицею у хвилі релізів"
  flexibility.scalability: "не застосовується: обсяги малі за побудовою (див. performance.capacity)"
  flexibility.installability: "не застосовується: встановлення — щабель launcher-а"
  flexibility.replaceability: "не застосовується: замінності модуля docs не обіцяємо — він і є та частина, якою інструмент відрізняється"
  safety.operational-constraints: "свідомо без окремого тесту: «не писати у файли проєкту» тримається тим, що в модулі нема жодного запису; зʼявиться запис — зʼявиться тест"
  safety.risk-identification: "не застосовується як окрема робота: головна відома загроза — тихий пропуск зіпсованого документа (урок №4 розбору), і її закривають сценарії цієї хвилі"
  safety.safe-integration: "свідомо без тесту: tool/ — нова тека, жоден наявний файл репозиторію не чіпається; єдиний дотик — новий CI-файл, названий у трансформі"
---

## Why

Перший щабель самонаведення (NEW-CONCEPT, розділ «Самонаведення»):
доки формат документів не тримає машина, кожна наступна хвиля спиралася
б на руки. Тому перша обіцянка інструмента — суворе читання власних
документів і чесна відмова там, де читання неможливе. Урок №4 розбору
нотаток: зіпсований документ, який тихо вимикає перевірки, — брехня
зеленим; у v1 `rev --write` через незакриту шапку пропустив сім
посилань і сказав «усі редакції збігаються».

Відступ bootstrap-у, названий вголос (тихий відступ заборонений —
конституція, п. 3 і 5): тексти v2 ще не в main, тому ця повна хвиля не
має план-PR — план затверджується словом оператора в чаті, записаним у
commit (§8.6), і їде робочою гілкою v2. Повна механіка гілок (§8.1,
§8.2) вмикається для хвиль після виходу v2 у main.

## scenario: broken-header-refuses

**Дано** файл хвилі, чия шапка не читається (незакриті `---`, шапки
нема зовсім або битий YAML),
**коли** його читає `read_wave`,
**тоді** повертається відмова, що називає файл, причину людською
мовою і що зробити, щоб полагодити, — а не порожній документ і не
тихий пропуск.

## scenario: unknown-field-refuses

**Дано** шапку з полем, якого методика не знає (одрук на кшталт
`scenarois`),
**коли** її читають,
**тоді** відмова називає невідоме поле і файл: одрук, прочитаний як
«нічого не оголошено», вимкнув би захист мовчки (§7.9).

## scenario: valid-wave-parses

**Дано** файл хвилі з цілою шапкою — сценарії зі звʼязками, трансформи
з файлами, decisions,
**коли** його читає `read_wave`,
**тоді** всі поля доступні як дані: імена сценаріїв, proves і covers,
файли трансформ, причини decisions — без втрат і без вигаданих значень.

## scenario: valid-contract-parses

**Дано** файл контракту з module і exports,
**коли** його читає `read_contract`,
**тоді** імʼя одиниці коду і сигнатури доступні як дані.

## scenario: duplicate-name-refuses

**Дано** шапку, де два сценарії чи дві трансформи назвались одним
імʼям,
**коли** її читають,
**тоді** відмова називає імʼя-дубль: YAML мовчки лишив би останнього,
і половина плану зникла б без сліду.

## scenario: check-reports-every-file

**Дано** теку `keel/` з кількома документами, один з яких зіпсований,
**коли** біжить `keel check`,
**тоді** у звіті є рядок по кожному файлу: цілі — перевірені,
зіпсований — названий з причиною; зіпсований не зупинив перевірку
сусідів і не зник зі звіту. І звіт називає, що́ цим поверхом
перевірено (шапки), а що ще ні (редакції, scope, тести, §7.7 —
щаблі попереду): зелене про неперевірене заборонене (урок №4).

## scenario: missing-keel-dir-refuses

**Дано** запуск `keel check` там, де теки `keel/` нема,
**коли** команда завершується,
**тоді** відмова каже, чого бракує і що зробити натомість (створити
`keel/waves/` і `keel/contracts/` або перейти в корінь проєкту), — і
код виходу ненульовий.

## transform: read-headers

Модуль `docs`: типи Wave, Contract, Refusal і функції читання. Разом
їде ґрунт (§4.7): cargo-проєкт у `tool/`, його `.gitignore` (щоб
`target/` не засмічував diff) і CI-крок, що ганяє тести.

Кожен тест називає параграф методики, який він тримає, — вимога
концепту «Відповідність тексту і коду»: перевірка без свого параграфа
не має права існувати, параграф без перевірки чесно позначений
текстовим.

Застереження: рецепт редакції в цій хвилі виконано руками — повторні
пробіли і переноси згорнуті в один пробіл (§5.4), sha256, перші шість
шістнадцяткових знаків. Машинним рецепт стане на щаблі 2 (`keel rev`),
і тест щабля 2 зобовʼязаний відтворити його на цьому ж файлі
контракту.

Застереження: `scan` бере корінь проєкту аргументом, а не з
`keel.toml` — читати конфіг інструмент ще не вміє; це щабель конфіга.

Застереження: все, що ця хвиля обіцяє про саму себе (scope, звʼязки,
редакції, повнота decisions), перевірено руками автора за текстом
METHODOLOGY-V2 — машинної перевірки ще нема, і саме її ця хвиля
будує.

## scenario: dir-among-docs-refuses

**Дано** теку, підтеку або symlink на теку серед `keel/waves/` чи
`keel/contracts/`,
**коли** біжить `scan` чи `keel check`,
**тоді** це відмова, що називає теку: документи живуть пласко, і
ніщо — включно з хвилею, захованою в підтеці, — не зникає зі звіту
мовчки.

## scenario: bare-scenario-refuses

**Дано** сценарій у шапці без жодної опори — ні `proves`, ні `covers`,
ні позначки `withdrawn`,
**коли** шапку читають,
**тоді** відмова називає сценарій: без опори він не обіцяє нічого
перевірюваного (§3.3).

## transform: check-walks-project

Підкоманда `check`: обхід `keel/` через `scan`, звіт по кожному файлу,
ненульовий вихід за будь-якої відмови. Це ще не весь check методики —
лише перший поверх: «документи читаються». Стадії перевірок (глава 7)
виростуть наступними щаблями, кожна своїм червоним commit-ом.

## transform: review-findings

Виправлення за знахідками свіжого рецензента (§9.9): дві нові відмови
(сценарії вище); повніший рядок «ще не перевірено» у звіті check —
дописані звʼязки (глава 3, §7.1–§7.2) і контракти (§7.6); перевірка
слага імені файлу документа (§1.2, §8.2); відмова на визначення якоря
YAML; дошиті assert-и на transform.contracts і на позначки контракту
withdrawn/superseded_by/renamed_from; людські причини відмов для
не-UTF-8 і порожнього файлу. Кожна знахідка дістала виправлення;
відмов вголос нема — відмовлятися не було від чого.

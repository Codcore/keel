---
depends_on: [0001-strict-headers]

scenarios:
  config-reads-strictly:
    proves: tool-config@63406a
    covers: [functional.correctness, interaction.user-error-protection]
  missing-config-defaults:
    proves: tool-config@63406a
    covers: [reliability.fault-tolerance]
  output-follows-lang:
    proves: tool-config@63406a
    covers: [interaction.inclusivity]
  missing-key-falls-back:
    covers: [safety.fail-safe]
  plural-forms-correct:
    covers: [functional.completeness]

transforms:
  read-config:
    implements:
      - config-reads-strictly
      - missing-config-defaults
    contracts: [tool-config@63406a]
    files:
      - tool/Cargo.toml
      - tool/Cargo.lock
      - tool/src/config.rs
      - tool/src/lib.rs
      - tool/tests/config_test.rs
  speak-by-keys:
    implements:
      - output-follows-lang
      - missing-key-falls-back
      - plural-forms-correct
    contracts: [tool-config@63406a, tool-docs@2ab9a9]
    files:
      - tool/Cargo.toml
      - tool/Cargo.lock
      - tool/src/i18n.rs
      - tool/src/refusal.rs
      - tool/src/docs.rs
      - tool/src/check.rs
      - tool/src/main.rs
      - tool/src/lib.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/docs_test.rs
      - tool/tests/check_test.rs
  english-comments:
    chore: "переклад коментарів у tool/ на англійську — мовна політика, борг хвилі 0001"
    files:
      - tool/src/docs.rs
      - tool/src/check.rs
      - tool/src/main.rs
      - tool/src/lib.rs
      - tool/src/config.rs
      - tool/src/i18n.rs
      - tool/src/refusal.rs
      - tool/tests/docs_test.rs
      - tool/tests/check_test.rs
      - tool/tests/config_test.rs

decisions:
  functional.appropriateness: "свідомо без тесту: доречність однієї ручки lang щойно вирішена оператором карткою — судити її будуть проєкти"
  performance.time-behaviour: "свідомо не міряємо: ініціалізація Fluent — разова на запуск; вимір прийде з hook-хвилею, де мілісекунди почнуть боліти"
  performance.capacity: "не застосовується: один конфіг на проєкт, десятки ключів перекладу — межі місткості нема чого міряти"
  performance.resource-utilisation: "свідомо не міряємо: два вшиті .ftl-файли в памʼяті — споживання стане питанням разом із hook-ами"
  compatibility.co-existence: "не застосовується: читає файли і виходить — портів, локів і демонів як не було, так і нема"
  compatibility.interoperability: "свідомо без окремого тесту: дві домовленості щабля — TOML-конфіг і Fluent-формат — тримаються сценаріями розбору і виводу"
  interaction.appropriateness-recognisability: "свідомо не робимо: впізнаваність команд — CLI-хвиля; рядок «конфіга нема — типові значення» вже каже, звідки взялась мова"
  interaction.learnability: "не застосовується: одне нове поле lang з двома значеннями — вчитися нема чого"
  interaction.operability: "свідомо не робимо: керування мовою — рядок у keel.toml; опції CLI прийдуть своєю хвилею"
  interaction.user-engagement: "не застосовується: інструмент перевірки не має тримати увагу"
  interaction.self-descriptiveness: "свідомо без нового тесту: звіт уже пояснює себе (хвиля 0001), нове тут — лише мова цього пояснення, і її тримає output-follows-lang"
  interaction.user-assistance: "свідомо без нового тесту: «що робити натомість» у кожній відмові — обіцянка tool-docs, успадкована; нові відмови конфіга їдуть тим самим типом Refusal"
  reliability.faultlessness: "свідомо без окремого тесту: звичайний вжиток покривають config-reads-strictly і output-follows-lang проти живих файлів"
  reliability.availability: "не застосовується: локальний бінарник — доступність вирішує щабель launcher-а"
  reliability.recoverability: "не застосовується: стану нема — перезапуск і є відновлення"
  security.confidentiality: "не застосовується: keel.toml — файл репозиторію, який бачить кожен, хто має репозиторій; нікуди нічого не шлеться"
  security.integrity: "свідомо без тесту: обидва модулі нічого не пишуть на диск — обіцяно контрактами, видно в коді"
  security.non-repudiation: "не застосовується: дій, що змінюють стан, хвиля не додає"
  security.accountability: "не застосовується: жодної зміни стану — нема чого обліковувати"
  security.authenticity: "не застосовується: поле [trust] читається як дані, але не виконується — TOFU лишається своїм щаблем, і контракт tool-config каже це вголос"
  security.resistance: "свідомо не робимо фаззингу понад суворий розбір: зіпсований TOML — відмова з рядком (сценарій); фаззинг додамо, коли заболить"
  maintainability.modularity: "свідомо без тесту: config і i18n — окремі модулі за правилом «модуль на главу»; спільний Refusal переїздить у свій файл, щоб docs не був чужим держателем"
  maintainability.reusability: "не застосовується: внутрішні модулі інструмента, не бібліотека"
  maintainability.analysability: "свідомо без нового тесту: відмови конфіга несуть імʼя поля і рядок — та сама школа, що вже доведена в tool-docs"
  maintainability.modifiability: "свідомо без тесту: додати мову = один .ftl-файл; додати поле конфіга = один рядок словника — міра тримається будовою"
  maintainability.testability: "свідомо без окремого тесту: кожна перевірка народжується червоним commit-ом — механіка та сама"
  flexibility.adaptability: "свідомо не робимо: крос-платформність перевіриться CI-матрицею у хвилі релізів; нового платформозалежного тут нема"
  flexibility.scalability: "не застосовується: обсяги малі за побудовою (див. performance.capacity)"
  flexibility.installability: "не застосовується: встановлення — щабель launcher-а; нова мова їде релізом, і це записано в концепті"
  flexibility.replaceability: "свідомо без тесту: Fluent замінюваний лише разом із форматом файлів — це названо ціною вибору в NEW-CONCEPT, не приховано"
  safety.operational-constraints: "свідомо без окремого тесту: «не писати у файли проєкту» тримається відсутністю запису в обох модулях"
  safety.risk-identification: "не застосовується як окрема робота: головна загроза щабля — тихе типове значення, що видає себе за прочитане, — і її закриває missing-config-defaults"
  safety.hazard-warning: "свідомо без нового тесту: попередження до шкоди («конфіга нема — типові значення») — частина сценарію missing-config-defaults"
  safety.safe-integration: "свідомо без тесту: нові файли — config.rs, i18n.rs, refusal.rs, i18n/*.ftl; наявні правляться лише в межах named files трансформ"
---

## Why

Мовну політику затверджено: інструмент говорить мовою проєкту через
одну ручку `lang`, а в коді й commit-ах цього репозиторію — тільки
англійська. Хвиля 0001 лишила записаний борг: українські рядки в
сирцях і українські коментарі. Ця хвиля закриває борг правильним
механізмом замість латки: модуль `config` читає keel.toml (перший
крок до піна версій — поле version уже в словнику), модуль `i18n`
виносить усі тексти в Fluent-файли (en/uk), а chore-трансформа
перекладає коментарі. Щабель редакцій (`keel rev`) свідомо
посунувся на хвилю 0003: кожен наступний модуль писатиме ключі
одразу, а не конвертуватиметься заднім числом.

Відступ bootstrap-у, названий вголос: хвиля їде робочою гілкою сесії
(не гілкою `0002-...` за §8.2) — гілка в сесії одна; план
затверджується словом оператора в чаті, записаним у commit (§8.6).

## scenario: config-reads-strictly

**Дано** `keel.toml` у корені проєкту,
**коли** його читає `config::read`,
**тоді** прочитано весь словник (`version`, `adapter`, `ci`, `lang`,
`[trust]`, `[generated]`), а невідоме поле чи кривий тип — відмова з
іменем поля і рядком, не мовчазний пропуск.

## scenario: missing-config-defaults

**Дано** проєкт без `keel.toml`,
**коли** біжить `keel check`,
**тоді** інструмент працює з типовими значеннями (`lang = "en"`) і
звіт каже про це вголос — типове значення не видає себе за прочитане.

## scenario: output-follows-lang

**Дано** `keel.toml` з `lang = "uk"` — і той самий проєкт з
`lang = "en"`,
**коли** біжить `keel check`,
**тоді** увесь звіт і відмови йдуть названою мовою: українською в
першому випадку, англійською в другому.

## scenario: missing-key-falls-back

**Дано** мову, в якій бракує перекладу якогось ключа,
**коли** інструмент друкує цей текст,
**тоді** виходить англійський текст ключа, а не діра і не падіння —
зелений звіт не ламається через неповний переклад.

## scenario: plural-forms-correct

**Дано** звіти з 1, 2 і 5 документами,
**коли** друкується підсумок,
**тоді** множина українською правильна: «1 документ», «2 документи»,
«5 документів» — правилами CLDR, а не if-ами в коді.

## transform: read-config

Модуль `config`: тип Config і `read` — суворий розбір keel.toml через
toml+serde зі словником усіх полів концепту. Залежності toml/serde
їдуть цим же commit-ом (§4.7).

Застереження: семантику в цій хвилі має лише `lang`; `version`,
`adapter`, `ci`, `[trust]`, `[generated]` читаються і віддаються як
дані — їхні щаблі попереду, і контракт каже це прямо.

## transform: speak-by-keys

Модуль `i18n` (Fluent: `i18n/en.ftl` + `i18n/uk.ftl`, вшиті в
бінарник; вибір за `lang`; fallback — англійська; множина — CLDR) і
переведення всіх текстів docs/check/main на ключі. Тип Refusal
переїздить у свій модуль `refusal` — він більше не належить самим
docs. Наявні тести перекладаються на англійські очікування (мова
пісочниць без keel.toml — англійська), окремі перевірки — на
українську з `lang = "uk"`.

Застереження: тексти хвилі 0001 у тестах звірялись українськими
підрядками; після цієї трансформи типова мова тестових пісочниць —
англійська, і українські підрядки перевіряються лише там, де тест
явно кладе `lang = "uk"`. Сценарії хвилі 0001 це переживають без
зміни текстів — вони обіцяють зміст відмов, не мову.

## transform: english-comments

Chore: коментарі й doc-коментарі в `tool/` перекладаються на
англійську за мовною політикою. Поведінка не змінюється, обіцянок не
додається; тести мусять лишитись зеленими без жодної правки в цьому
commit-і.

---
depends_on: [0002-config-and-language]

scenarios:
  revision-recipe-reproduced:
    proves: tool-rev@c72e2a
    covers: [functional.correctness]
  contract-refs-verified:
    proves: tool-rev@c72e2a
    covers: [reliability.faultlessness, safety.hazard-warning]
  missing-contract-named:
    proves: tool-rev@c72e2a
    covers: [safety.fail-safe]
  rev-command-prints:
    covers: [interaction.user-assistance, interaction.self-descriptiveness]

transforms:
  compute-revisions:
    implements:
      - revision-recipe-reproduced
    contracts: [tool-rev@c72e2a]
    files:
      - tool/src/rev.rs
      - tool/src/lib.rs
      - tool/tests/rev_test.rs
  check-verifies-refs:
    implements:
      - contract-refs-verified
      - missing-contract-named
    contracts: [tool-rev@c72e2a, tool-docs@2ab9a9, tool-config@63406a]
    files:
      - tool/src/check.rs
      - tool/src/rev.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/check_test.rs
  rev-command:
    implements:
      - rev-command-prints
    contracts: [tool-rev@c72e2a]
    files:
      - tool/src/main.rs
      - tool/src/rev.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/check_test.rs

decisions:
  functional.completeness: "свідомо звужено вголос: цей щабель звіряє редакції КОНТРАКТІВ у шапках хвиль; редакції сценаріїв живуть у тегах тестів, і їх звірить щабель адаптерів — check каже це в рядку «ще не перевірено»"
  functional.appropriateness: "свідомо без тесту: доречність rev судитимуть автори хвиль — команда народжена з болю ручного рахунку в 0001–0002"
  performance.time-behaviour: "свідомо не міряємо: хешування десятків малих файлів; вимір прийде з hook-хвилею"
  performance.capacity: "не застосовується: документів десятки, sha256 потоковий"
  performance.resource-utilisation: "свідомо не міряємо: разовий прохід читання"
  compatibility.co-existence: "не застосовується: читає файли і виходить"
  compatibility.interoperability: "свідомо без окремого тесту: єдина домовленість — рецепт §5.4, і його тримає revision-recipe-reproduced"
  interaction.appropriateness-recognisability: "свідомо не робимо: «наступний крок» після rev друкується, повна CLI-довідка — своєю хвилею"
  interaction.learnability: "не застосовується: команда без опцій, вивід — пари імʼя@редакція"
  interaction.operability: "свідомо не робимо: опції (--json) — CLI-хвилею"
  interaction.user-error-protection: "свідомо без нового тесту: криві посилання ловить суворий розбір tool-docs, доведений у 0001"
  interaction.user-engagement: "не застосовується: інструмент перевірки"
  interaction.inclusivity: "свідомо без нового тесту: вивід іде ключами через i18n, доведеними в 0002; нових неперекладних текстів хвиля не додає"
  reliability.fault-tolerance: "свідомо без нового тесту: зіпсований документ не валить сусідів — тримає scan (0001); rev успадковує його відмови"
  reliability.availability: "не застосовується: локальний бінарник"
  reliability.recoverability: "не застосовується: стану нема"
  security.confidentiality: "не застосовується: читає файли репозиторію, нікуди не шле"
  security.integrity: "свідомо без тесту: модуль rev нічого не пише на диск — обіцяно контрактом, видно в коді"
  security.non-repudiation: "не застосовується: дій зі станом нема"
  security.accountability: "не застосовується: те саме"
  security.authenticity: "не застосовується: нікого не автентифікує"
  security.resistance: "свідомо не робимо фаззингу: вхід уже пройшов суворий розбір docs; sha256 не боїться зловмисного тексту"
  maintainability.modularity: "свідомо без тесту: rev — окремий модуль за главою 5, check лише викликає його"
  maintainability.reusability: "не застосовується: внутрішній модуль"
  maintainability.analysability: "свідомо без нового тесту: розбіжність редакцій називає обидві — записану і чинну (сценарій contract-refs-verified це тримає)"
  maintainability.modifiability: "свідомо без тесту: рецепт — одна функція text_rev, решта — тонкі обгортки"
  maintainability.testability: "свідомо без окремого тесту: механіка червоних народжень та сама"
  flexibility.adaptability: "свідомо не робимо: платформозалежного нового нема; sha256 однаковий всюди"
  flexibility.scalability: "не застосовується: обсяги малі"
  flexibility.installability: "не застосовується: launcher-щабель"
  flexibility.replaceability: "свідомо без тесту: зміна рецепта зламала б усі записані редакції — тому рецепт і прибитий контрактом та golden-тестами; це названа ціна, не пастка"
  safety.operational-constraints: "свідомо без окремого тесту: запису на диск у модулі нема; зʼявиться rev --write — зʼявиться і тест"
  safety.risk-identification: "не застосовується як окрема робота: головна загроза щабля — рецепт, що тихо розійшовся з ручним рахунком, і її закриває revision-recipe-reproduced golden-векторами з 0001–0002"
  safety.safe-integration: "свідомо без тесту: нові файли — rev.rs і rev_test.rs; наявні правляться лише в межах named files; рядок «ще не перевірено» у звіті чесно коротшає, а не зникає"
---

## Why

Щабель 2 драбини самонаведення: редакції відтепер тримає машина. Дві
хвилі поспіль редакції рахували руки автора — скриптом, за рецептом,
записаним застереженням у 0001, і тест цього щабля зобовʼязаний
відтворити той рецепт на живих контрактах репозиторію (tool-docs@2ab9a9
і tool-config@63406a — редакції, що вже тримаються шістнадцятьма
посиланнями). З цієї хвилі `keel check` сам звіряє записані редакції
контрактів із текстом (§7.3) і сам ловить посилання в нікуди (§7.1
для контрактів), а `keel rev` друкує чинні редакції — авторові більше
нема чого рахувати руками. Редакції сценаріїв живуть у тегах тестів:
їх звірить щабель адаптерів, і звіт каже це чесно.

Відступ bootstrap-у, названий вголос: хвиля їде робочою гілкою сесії;
план затверджується словом оператора в чаті, записаним у commit
(§8.6). Звіт рецензента ляже в keel/reviews/0003-revisions.md — за
затвердженим правилом §9.9.

## scenario: revision-recipe-reproduced

**Дано** рецепт §5.3–§5.4 і редакції, пораховані руками у хвилях
0001–0002,
**коли** їх рахує `keel::rev`,
**тоді** результати збігаються byte-у-byte: golden-вектори тримають
згортання пробілів і переносів (переформатування — не зміна,
переформулювання — зміна), а живі контракти репозиторію дають рівно
tool-docs@2ab9a9 і tool-config@63406a; порівняння за префіксом
приймає 4–6 знаків (§5.2).

## scenario: contract-refs-verified

**Дано** хвилю, чиє посилання тримає редакцію контракту,
**коли** біжить `keel check`,
**тоді** записана редакція звірена з чинним текстом контракту:
збіглися — пораховано у звіті; розійшлися — знахідка, що називає
обидві редакції — записану і чинну — і каже перечитати й оновити
свідомо, а не переписати мовчки (§5.1); рядок «ще не перевірено»
більше не згадує редакцій контрактів.

## scenario: missing-contract-named

**Дано** посилання на контракт, файлу якого нема в keel/contracts/,
**коли** біжить `keel check`,
**тоді** це знахідка з імʼям загубленого контракту і хвилею, що на
нього показує (§7.1), — а не тихий пропуск і не падіння.

## scenario: rev-command-prints

**Дано** проєкт із хвилями і контрактами,
**коли** біжить `keel rev`,
**тоді** надруковано чинні редакції мовою проєкту: кожен контракт —
`слаг@редакція`, кожен сценарій кожної хвилі — `хвиля/сценарій@редакція`,
і наступний крок — щоб автор копіював редакції звідси, а не рахував
руками.

## transform: compute-revisions

Модуль `rev`: text_rev (рецепт), contract_rev (цілий файл),
scenario_revs (тіла секцій з читанням через tool-docs), matches
(префікс §5.2). Golden-вектори в тестах — з ручних рахунків 0001–0002.

Застереження: `rev --write` (автопідстановка редакцій у шапки) свідомо
не в цій хвилі — запис у документи зʼявиться, коли заболить ручне
копіювання, і принесе свій тест незіпсованости (урок №4 розбору
нотаток: саме на --write v1 тихо пропускав сім посилань).

## transform: check-verifies-refs

`keel check` дістає другий поверх: по кожному посиланню
slug@редакція з шапок хвиль — існування файлу контракту (§7.1) і збіг
редакції (§7.3). Рядки «перевірено/ще не перевірено» чесно
переписуються: редакції контрактів переїжджають у «перевірено»,
редакції сценаріїв (теги тестів) лишаються названими в «ще не
перевірено» до щабля адаптерів.

## transform: rev-command

Підкоманда `keel rev`: друк чинних редакцій усіх документів мовою
проєкту, з наступним кроком. Нові ключі в обох .ftl.

---
depends_on: [0015-one-home]

scenarios:
  stale-refs-rewritten:
    proves: tool-rev@2ef198
    covers: [functional.correctness, functional.appropriateness]
  light-pr-words-honest:
    proves: tool-next@ec56ff
    covers: [interaction.self-descriptiveness]

transforms:
  rewrite-hand:
    implements:
      - stale-refs-rewritten
    contracts: [tool-rev@2ef198, tool-docs@2ab9a9, tool-config@7dd1d7, tool-close@1b6b8e, tool-tags@4a0d5e, tool-adapter-cargo@348769, tool-plan@5d861c]
    files:
      - tool/src/rev.rs
      - tool/src/main.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/rev_write_test.rs
  light-words:
    implements:
      - light-pr-words-honest
    contracts: [tool-next@ec56ff, tool-close@1b6b8e]
    files:
      - tool/src/next.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/next_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md

decisions:
  functional.completeness: "свідомо без окремого тесту: перепис бере всі записи шапки — proves і contracts трансформ — одним прохідом; тримає stale-refs-rewritten наскрізним асертом обох місць"
  performance.time-behaviour: "свідомо не міряємо: один скан, один прохід шапок, записи лише розійшлі"
  performance.capacity: "не застосовується: шапок — десятки"
  performance.resource-utilisation: "свідомо не міряємо: те саме"
  compatibility.co-existence: "свідомо без окремого тесту: хірургія лише в шапці, тіла секцій не торкаються — редакції сценаріїв стоять; тримає stale-refs-rewritten"
  compatibility.interoperability: "свідомо без окремого тесту: git не кличеться — закритість судиться структурно тегами (школа close)"
  interaction.appropriateness-recognisability: "свідомо без нового тесту: імʼя прапорця — з таблиці NEW-CONCEPT (rev --write), usage називає"
  interaction.learnability: "свідомо без нового тесту: кожен перепис — рядок поіменно зі старою і новою редакціями"
  interaction.operability: "свідомо без тесту: один прапорець --write, корінь і все"
  interaction.user-error-protection: "свідомо без окремого тесту: перед записом шапка перечитується суворим розбором — битим файл не лишається; тримає stale-refs-rewritten другим бігом"
  interaction.user-assistance: "свідомо без нового тесту: закрита хвиля — «лишаю» словом із §5.6; відсутній контракт — до check (§7.1)"
  interaction.user-engagement: "не застосовується: інструмент записів"
  interaction.inclusivity: "свідомо без нового тесту: всі нові тексти — ключами через i18n, доведеними в 0002"
  reliability.faultlessness: "свідомо без окремого тесту: перепис — точна заміна slug@стара → slug@нова в зрізі шапки; жодних здогадів"
  reliability.fault-tolerance: "свідомо без окремого тесту: биті документи — відмова scan-у до першого запису"
  reliability.availability: "не застосовується: локальний бінарник"
  reliability.recoverability: "свідомо без нового тесту: запис dot-тимчасовим файлом і rename-ом — школа 0013 одним домом plan::write_new"
  security.confidentiality: "не застосовується: пише локальні файли, нікуди не шле"
  security.integrity: "свідомо без нового тесту: переписуються лише розійшлі записи відкритих хвиль — закрите і чуже не чіпається; тримає stale-refs-rewritten"
  security.non-repudiation: "не застосовується: перепис видно в git"
  security.accountability: "не застосовується: те саме"
  security.authenticity: "свідомо без нового тесту: нова редакція — живим contract_rev з диска, не переказом"
  security.resistance: "свідомо без фаззингу: вхід — документи після суворого docs"
  maintainability.modularity: "свідомо без тесту: write живе в rev (власник редакцій); закритість питається в close, запис — у plan::write_new"
  maintainability.reusability: "не застосовується: внутрішні модулі"
  maintainability.analysability: "свідомо без нового тесту: кожен перепис і кожен пропуск — поіменно"
  maintainability.modifiability: "свідомо без тесту: нове поле з записами — ще один збір refs у тому самому проході"
  maintainability.testability: "свідомо без окремого тесту: пісочниці — справжні проєкти школи 0005–0015"
  flexibility.adaptability: "свідомо: перепис знає записи цього покоління (proves, contracts); нові види записів — новою хвилею"
  flexibility.scalability: "не застосовується: обсяги малі"
  flexibility.installability: "не застосовується: нових установок нема"
  flexibility.replaceability: "свідомо без тесту: вивід — текст у stdout; файли — той самий markdown"
  safety.operational-constraints: "свідомо без нового тесту: жодного бігу тестів — закритість структурна; батарею write не жене"
  safety.risk-identification: "не застосовується як окрема робота: загрози щабля — переписана закрита хвиля і битий після хірургії файл — і є сценарієм stale-refs-rewritten"
  safety.fail-safe: "свідомо без нового тесту: не-парсибельний наслідок хірургії — відмова без запису, ніколи не битий файл на диску"
  safety.hazard-warning: "свідомо без нового тесту: «лишаю: закрита» — попередження поіменно в самому звіті"
  safety.safe-integration: "свідомо без тесту: нові файли — rev_write_test.rs; rev росте однією рукою, next — одним словом; контракти кажуть це наперед"
---

## Why

Щабель 14 самонаведення: остання команда з таблиці NEW-CONCEPT —
`keel rev --write`. Редакції розходяться законно: контракт виріс —
і кожна відкрита хвиля, що тримає його стару редакцію, мусить
переписати запис рукою; чотирнадцять хвиль поспіль це робив sed
сесії. Тепер перепис — робота машини: розійшлі записи відкритих
хвиль переписуються на чинні поіменно, закриті лишаються — їх судить
історія (§5.6), і про це кажеться словом. Тіла секцій хірургія не
торкається ніколи, а переписана шапка перечитується перед записом —
битим файл не лишається.

Цією ж хвилею — борг, названий догфудом 0015 (§6.7 поіменно): слова
pr-кроку `keel next` «звіт рецензії поруч» для легкої хвилі були
замальовкою — легка звіту не потребувала. Крок легкої дістає власні
чесні слова, і контракт tool-next каже це наперед.

Відступи bootstrap, названі вголос: хвиля їде робочою гілкою сесії;
план затверджено словом оператора наперед (§8.6, стояче слово в
журналі 2026-09-02); журнал їде chore-трансформою.

## scenario: stale-refs-rewritten

**Дано** контракт, чий текст виріс, відкриту хвилю, чиї proves і
contracts тримають стару редакцію, і закриту хвилю зі старою
редакцією та збіжним тегом поруч зі звітом,
**коли** біжить `keel rev --write`,
**тоді** записи відкритої хвилі переписані на чинну редакцію в обох
місцях — поіменно, зі старою і новою редакціями в рядку; тіла
секцій не торкнуті (редакції сценаріїв стоять як стояли); закрита
хвиля не змінена ані байтом — «лишаю» словом із §5.6; повторний біг
каже, що розійшлого нема; переписаний файл читається тим самим
суворим розбором (§7.9).

## scenario: light-pr-words-honest

**Дано** легку хвилю (§6.8) на її власній гілці з довершеним chore,
**коли** біжить `keel next`,
**тоді** крок — «час PR» словами легкої: один PR, закриття фактом
merge — і жодного слова про звіт рецензії, якого легка не
потребувала (§9.9 просить звіт лише від повної); повна хвиля зі
звітом чує свої слова, як і досі.

## transform: rewrite-hand

`rev` дістає руку `write`: збір записів шапки (proves сценаріїв,
contracts трансформ), суд закритости через close::structural,
хірургія `slug@стара → slug@нова` лише в зрізі шапки, перечит
суворим розбором, запис через plan::write_new (один дім школи
0013). main.rs вчить `rev --write`; usage росте.

Застереження: закритість судиться структурно (тегами) — батарея не
біжить; запис на контракт, якого нема на диску, не переписується —
його називає check; хірургія точна — заміна повного токена
`slug@редакція`, і не-парсибельний наслідок — відмова без запису.

## transform: light-words

Крок «час PR» у `next` обирає слова за вагою §6.8 (close::light,
відкрита в 0012 і вжита в гейті рецензії ще тоді): легка — власний
ключ без слова про звіт; повна — як досі. Контракт tool-next каже
світлу правду наперед (→ ec56ff).

Застереження: лише слова кроку — порядок кроків не міняється; вага
питається тим самим предикатом, що гейтить крок рецензії.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10).

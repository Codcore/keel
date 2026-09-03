---
depends_on: [0013-planning-skeletons]

scenarios:
  init-births-the-frame:
    proves: tool-init@c77997
    covers: [functional.completeness, interaction.operability]
  init-never-tramples:
    proves: tool-init@c77997
    covers: [security.integrity, reliability.fault-tolerance]

transforms:
  frame-hand:
    implements:
      - init-births-the-frame
      - init-never-tramples
    contracts: [tool-init@c77997, tool-gate@ef42fc, tool-config@7dd1d7]
    files:
      - tool/src/init.rs
      - tool/src/lib.rs
      - tool/src/main.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/init_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md

decisions:
  functional.correctness: "свідомо без окремого сценарію: кожен шмат рами — «народжено» / «вже стоїть» / відмова вголос; обидва сценарії проходять всі три слова наскрізь"
  functional.appropriateness: "свідомо без окремого тесту: init — рівно рядок «Наскрізні» таблиці NEW-CONCEPT; нічого понад раму"
  performance.time-behaviour: "свідомо не міряємо: чотири mkdir, два записи, один git-виклик"
  performance.capacity: "не застосовується: одна рама на проєкт"
  performance.resource-utilisation: "свідомо не міряємо: те саме"
  compatibility.co-existence: "свідомо без окремого тесту: наявне ніколи не топчеться — тримає init-never-tramples"
  compatibility.interoperability: "свідомо без окремого тесту: hook ставиться школою gate 0005 (git-path hooks — worktree і hooksPath знані там)"
  interaction.appropriateness-recognisability: "свідомо без нового тесту: імʼя команди — з таблиці NEW-CONCEPT, usage називає"
  interaction.learnability: "свідомо без нового тесту: хвіст звіту сам веде далі — §8.7 і «далі — keel plan»"
  interaction.user-error-protection: "свідомо без нового тесту: повторний init нешкідливий байт-у-байт — тримає init-never-tramples"
  interaction.user-assistance: "свідомо без нового тесту: кожен рядок рами каже «народжено»/«вже стоїть»/відмову з «натомість»"
  interaction.user-engagement: "не застосовується: інструмент рами"
  interaction.inclusivity: "свідомо без нового тесту: всі тексти — ключами через i18n, доведеними в 0002; словник keel.toml коментарями мовою проєкту"
  interaction.self-descriptiveness: "свідомо без нового тесту: звіт називає кожен шмат і його стан поіменно"
  reliability.faultlessness: "свідомо без окремого тесту: рядки — прямі наслідки перевірок файлової системи"
  reliability.availability: "не застосовується: локальний бінарник"
  reliability.recoverability: "свідомо без тесту: keel.toml їде dot-тимчасовим файлом і rename-ом школою 0013 — цілий або відмова; повторний біг доносить теки, яких бракує"
  security.confidentiality: "не застосовується: пише локальні файли, нікуди не шле"
  security.non-repudiation: "не застосовується: народження рами видно в git"
  security.accountability: "не застосовується: те саме"
  security.authenticity: "свідомо без нового тесту: рама не підробляє змісту — keel.toml свідомо нічого не вмикає"
  security.resistance: "свідомо без фаззингу: вхід — лише корінь; жодних слагів"
  maintainability.modularity: "свідомо без тесту: init — окремий модуль-контракт; hook ставить gate, не дубль"
  maintainability.reusability: "не застосовується: внутрішній модуль"
  maintainability.analysability: "свідомо без нового тесту: кожен шмат рами — окремий рядок поіменно"
  maintainability.modifiability: "свідомо без тесту: новий шмат рами — новий рядок поруч"
  maintainability.testability: "свідомо без окремого тесту: пісочниці — справжні теки з git і без"
  flexibility.adaptability: "свідомо: склад рами — цього покоління (три теки, конфіг, hook); новий шмат — новою хвилею"
  flexibility.scalability: "не застосовується: обсяги малі"
  flexibility.installability: "свідомо без окремого тесту: init і Є установка методики в проєкт — його сценарії і є тестом установки"
  flexibility.replaceability: "свідомо без тесту: вивід — текст у stdout; файли — звичайні markdown/toml"
  safety.operational-constraints: "свідомо без нового тесту: жодного бігу тестів чи команд, окрім одного git rev-parse у школі gate"
  safety.risk-identification: "не застосовується як окрема робота: загроза щабля — потоптана чужа рама — і є сценарієм init-never-tramples"
  safety.fail-safe: "свідомо без нового тесту: невідомий стан шмата — відмова вголос, ніколи не перезапис; тримає init-never-tramples"
  safety.hazard-warning: "свідомо без нового тесту: нагадування §8.7 — попередження в самому хвості звіту"
  safety.safe-integration: "свідомо без тесту: нові файли — init.rs і його тест; gate і config не міняються"
---

## Why

Щабель 13 самонаведення, перша половина: рама для поля. Всі
тринадцять хвиль рама цього проєкту ставилась рукою — теки, конфіг,
hook, і кожен свіжий контейнер згадував це заново (журнал
2026-09-02: hook у свіжому середовищі був відсутній, ставився
живцем). Чужий проєкт цього не пробачить: перший крок методики в
полі мусить бути одним рухом. `keel init` ставить раму — теки
методики, keel.toml зі словником-коментарем, commit-msg hook рукою
gate — і нічого не топче: повторний біг каже «вже стоїть», чуже
лишається чужим. Хвіст звіту несе нагадування §8.7 (правило тримає
вимкнена кнопка) і веде до keel plan. Друга половина щабля — сам
чужий проєкт — прийде окремою хвилею, коли оператор назве ціль.

Відступи bootstrap, названі вголос: хвиля їде робочою гілкою сесії;
план затверджено словом оператора наперед (§8.6, стояче слово в
журналі 2026-09-02); журнал їде chore-трансформою.

## scenario: init-births-the-frame

**Дано** чистий проєкт із git і без жодного сліду методики,
**коли** біжить `keel init`,
**тоді** народжуються keel/waves/, keel/contracts/, keel/reviews/
(кожна з .gitkeep) і keel.toml зі закоментованим словником конфіга
(NEW-CONCEPT, Config), ставиться commit-msg hook; кожен шмат — свій
рядок «народжено» поіменно; хвіст звіту нагадує §8.7 (вимкнути
squash і rebase — правило тримає кнопка) і каже «далі — keel plan»;
вихід зелений, і `keel check` по народженій рамі читає порожній
проєкт без відмов.

## scenario: init-never-tramples

**Дано** проєкт, де рама вже стоїть або стоїть чуже: наявний
keel.toml з чиїмось змістом, наявні теки, чужий commit-msg hook —
і проєкт зовсім без git,
**коли** біжить `keel init`,
**тоді** жоден наявний байт не міняється: кожен шмат, що стоїть, —
рядок «вже стоїть» поіменно (для рішень рами зміст keel.toml не
судиться — це робота config §7.9); наявній теці без .gitkeep файл
догодовується окремим словом — «доносить те, чого бракує» правда і
тут; битий keel.toml не править виклик мовчки: відмова config
кажеться вголос, рама доноситься типовою мовою; чужий hook —
відмова вголос рядком, не перезапис; без git hook не ставиться —
рядок відмови з причиною; рахунок шматів, що не стали, червонить
вихід, а решта рами доноситься попри нього.

## transform: frame-hand

Модуль `init` — четверта рука, що пише: теки з .gitkeep, keel.toml
dot-тимчасовим файлом і rename-ом (школа 0013), hook — наявною
рукою gate::install_hook (§9.3, без дубля). Звіт — рядок на шмат:
«народжено» / «вже стоїть» / відмова вголос; другим числом —
рахунок шматів, що не стали. main.rs дістає команду `init`; usage
росте.

Застереження: для рішень рами зміст наявного keel.toml не
судиться — «вже стоїть» кажеться про факт файлу (зміст судить
config §7.9), а мова виводу — наскрізна школа config, і битий
конфіг називається вголос, не правлячи виклик мовчки; словник у
народженому конфізі — коментарями, типові значення лишаються
словам config; hook без git — чесна відмова, не тиша; рама
доноситься по шматах — повторний біг добудовує те, чого бракує,
включно з .gitkeep у наявній порожній теці.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10).

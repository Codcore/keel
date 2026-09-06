---
depends_on: [0039-the-tool-in-someone-elses-project]

scenarios:
  every-reading-command-answers-in-json:
    covers: [functional.completeness, compatibility.interoperability]
  one-envelope-for-every-command:
    covers: [interaction.learnability, maintainability.modularity]
  the-frame-takes-the-place-and-the-branch:
    covers: [functional.appropriateness, interaction.user-error-protection]

transforms:
  the-envelope:
    implements:
      - one-envelope-for-every-command
    files:
      - one new in tool/src/
      - tool/Cargo.toml
      - tool/src/lib.rs
      - keel/contracts/tool-json.md
      - tool/tests/one_envelope_test.rs
  every-court-fills-it:
    implements:
      - every-reading-command-answers-in-json
    files:
      - tool/src/main.rs
      - tool/src/check.rs
      - tool/tests/json_out_test.rs
  the-frame-takes-flags:
    implements:
      - the-frame-takes-the-place-and-the-branch
    files:
      - tool/src/main.rs
      - tool/src/check.rs
      - keel/contracts/tool-cli.md
      - keel.toml
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/frame_flags_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0040-the-cli-contract.md

decisions:
  functional.correctness: "тримає every-reading-command-answers-in-json: код виходу і сам вердикт на дорозі --json побайтово ті самі, що й на прозовій — інакше машина і людина бачили б різні проєкти"
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "свідомо без тесту: пакет складається з того, що суд уже порахував; жодного суду вдруге не жене"
  compatibility.co-existence: "не застосовується"
  interaction.appropriateness-recognisability: "свідомо без тесту: прапорець зветься --json, як у всьому світі, і стоїть у рядку допомоги кожної команди, що його бере"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "тримає one-envelope-for-every-command: пакет несе свою версію (keel: 1) і імʼя команди, тож harness читає, що саме дістав, а не вгадує"
  interaction.user-assistance: "тримає the-frame-takes-the-place-and-the-branch: невідомий прапорець — відмова з переліком тих, що бере ця команда, а не тихий пропуск"
  reliability.faultlessness: "тримає one-envelope-for-every-command: пакет складає справжній серіалізатор (serde_json), а не конкатенація рядків — лапка, зворотний слеш і кирилиця в шляху не ламають виводу"
  reliability.fault-tolerance: "тримає every-reading-command-answers-in-json: відмова — теж пакет, із причиною і «натомість», а не голий stderr посеред JSON"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "свідомо без тесту: пакет несе рівно те, що команда і так друкує людині — жодного нового поля з машини"
  security.integrity: "не застосовується"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "свідомо без окремої роботи: JSON пише бібліотека, тож ані шлях, ані повідомлення відмови не можуть вилізти з рядка і зробити чужу структуру"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "тримає every-reading-command-answers-in-json: у пакеті findings несуть file, тож harness каже, ЩО зламано, не розбираючи прозу двома мовами"
  maintainability.modifiability: "свідомо без тесту: нове поле додається в одному місці — конверті; команда додає лише свої"
  maintainability.testability: "свідомо без тесту: проби розбирають вивід справжнім serde_json, а не пошуком підрядків — школа 0025"
  flexibility.adaptability: "тримає the-frame-takes-the-place-and-the-branch: -C і --branch існують саме заради середовищ, де тека і гілка не там, де їх шукають (CI з detached HEAD)"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий числом — жодна команда сьогодні не дає машинного виводу, тож кожен harness розбирає прозу, яка ще й двомовна"
  safety.fail-safe: "тримає every-reading-command-answers-in-json: де команда не має структури понад прозу, пакет несе report і каже це полем, а не вдає структуру, якої нема"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "тримає every-reading-command-answers-in-json: без --json вивід кожної команди лишається побайтово тим самим — наявні скрипти й проби не ламаються"
---

## Why

Концепт обіцяє CLI-контракт **двічі** і дуже конкретно:

> «**CLI-контракт:** кожна команда має `--json`; вивід — самодостатній
> пакет; кожна відмова — причина плюс «що робити натомість»»
> (`NEW-CONCEPT.md:251`)

> «`--json` на кожній читальній команді — для скриптів і harness-ів;
> `-C <тека>` — де працювати; `--branch <імʼя>` — де git не знає
> гілки (CI з detached HEAD), і мовчки пропустити порівняння не можна
> (§4.10)» (`NEW-CONCEPT.md:332`)

Зміряно перед планом, а не прочитано:

| обіцяне | у коді |
|---|---|
| `--json` | **нема ніде**: сім згадок слова «json» у сирцях — усі про формат hook-конфігів, жодна про вивід |
| `-C <тека>` | **нема**: `-C` є лише всередині, як аргумент git-ові (`scope.rs:27`) |
| `--branch <імʼя>` | **нема прапорця**; є змінна середовища `KEEL_BRANCH`, і саме її мусить ставити згенерований workflow |

Наслідок не теоретичний. Кожен harness — і той, що ми самі пишемо в
CI, — мусить **розбирати прозу**, яка ще й буває двома мовами: щоб
дізнатись, скільки знахідок, треба читати рядок «підсумок: 62
документи, 0 знахідок» українською або «summary: …» англійською. Це
не інтерфейс, це ворожіння. А `--branch` як прапорця нема взагалі,
хоч §4.10 прямо забороняє мовчки пропускати порівняння там, де git
гілки не знає, — і згенерований CI сьогодні лікує це змінною
середовища, бо іншого шляху нема.

**Межа цієї хвилі, названа наперед.** Пакет несе **ту структуру, яку
суд уже має**: знахідки з файлом і причиною, межі, числа підсумку,
код виходу — і повний прозовий звіт полем `report`, щоб нічого не
загубилось. Він **не** перетворює кожне речення суду на типове поле:
там, де в команди понад прозу структури нема, пакет каже це полем, а
не вдає структуру, якої нема. Типізація кожного вироку — окремий
рядок черги, а не мовчазна обіцянка.

## scenario: every-reading-command-answers-in-json

**Дано** будь-яку читальну команду (`check`, `close`, `status`,
`next`, `map`, `review`, `version`, `cuts`, `rev`),
**коли** її кличуть із `--json`,
**тоді** на stdout лягає **рівно один** JSON-обʼєкт і більше нічого:
він несе імʼя команди, код виходу, чи зелено, корінь, мову — і
структуру цієї команди: знахідки з файлом і причиною, межі, числа
підсумку, і повний прозовий звіт полем `report`. **Відмова — теж
пакет**, із `refusal` (file, reason, instead), а не голий stderr.
Код виходу той самий, що й без прапорця. Без `--json` вивід
лишається **побайтово** тим самим.

## scenario: one-envelope-for-every-command

**Дано** harness, який навчився читати вивід однієї команди,
**коли** він читає будь-яку іншу,
**тоді** зовнішня форма та сама: `keel` (версія конверта), `command`,
`ok`, `exit`, `root`, `lang` — і далі поля цієї команди. Пакет складає
справжній серіалізатор, тож лапка, зворотний слеш, перенос рядка і
кирилиця у шляху чи в повідомленні не ламають структури.

## scenario: the-frame-takes-the-place-and-the-branch

**Дано** середовище, де тека не поточна, а git гілки не знає (CI з
detached HEAD),
**коли** команду кличуть із `-C <тека>` і `--branch <імʼя>`,
**тоді** вона працює в названій теці і бере названу гілку — не мовчки
пропускаючи порівняння (§4.10). Прапорець важить більше за змінну
`KEEL_BRANCH`, а змінна лишається чинною там, де прапорця нема.
Невідомий прапорець — **відмова з переліком** тих, що ця команда
бере.

## transform: the-envelope

Один модуль складає конверт: спільні поля, поля команди, відмова.
`serde_json` стає справжньою залежністю, а не лише тестовою.
Контракт `tool-json` описує форму, на яку harness має право
спиратися.

## transform: every-court-fills-it

`check` і `close` віддають не лише прозу, а й те, що вже порахували —
знахідки з файлом, межі, числа. Решта читальних команд дістає конверт
із прозою і своїми числами.

## transform: the-frame-takes-flags

Рама вчиться двом прапорцям, спільним для всіх команд: `-C` і
`--branch`. Невідомий прапорець і далі відмова, тепер із переліком.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — `docs/uk/V2-PROCESS.md` (§9.10). BACKLOG
втрачає важкий рядок про `--json`, README дістає розділ про машинний
вивід. Сюди ж лягає звіт рецензії.

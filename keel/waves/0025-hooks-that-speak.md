---
depends_on: [0024-agents-and-their-formats]

scenarios:
  hook-speaks-the-next-step:
    proves: tool-generated@2d09a4
    covers: [functional.correctness, interaction.learnability]

transforms:
  hooks-in-both-tongues:
    implements:
      - hook-speaks-the-next-step
    contracts: [tool-generated@2d09a4, tool-next@f3cc04, tool-config@0e4d22]
    files:
      - tool/src/generated.rs
      - tool/src/next.rs
      - tool/src/main.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/generated_test.rs
      - tool/Cargo.toml
      - tool/Cargo.lock
      - keel.toml
      - .claude/settings.json
      - .cursor/hooks.json
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md

decisions:
  functional.appropriateness: "свідомо без окремого тесту: доречність судить рішення оператора §8.6 («те ж стосується хуків») і буква концепту — hook-конфіги перелічені серед згенерованих інтеграцій"
  functional.completeness: "свідомо: подія одна — старт сесії, бо саме там агент не знає нічого. Решта подій обох інструментів (правки файлів, дозволи на команди, компакт) — своїми хвилями, і вони перелічені в Застереженні, а не змовчані"
  performance.time-behaviour: "тримає hook-speaks-the-next-step частково: hook кличе keel next, і його стеля — timeout 30 с у конфізі Claude; у Cursor sessionStart — fire-and-forget за їхніми доками, тож стелі там нема і ми її не вигадуємо"
  performance.capacity: "свідомо без тесту: два маленькі JSON-файли"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "тримає hook-speaks-the-next-step: **власний файл людини не міняється жодним байтом** — це головна клаузула хвилі; замість запису слово каже, що саме вставити"
  compatibility.interoperability: "тримає hook-speaks-the-next-step: форму кожного файлу судить справжній JSON-парсер, а гілка — та, яку документує сам агент; вивід keel next --for cursor судиться як JSON із additional_context, бо саме так Cursor приймає контекст на старті сесії"
  interaction.appropriateness-recognisability: "свідомо без нового тесту: рядки hook-конфігів стоять серед рядків рами тією ж родиною слів"
  interaction.operability: "свідомо без окремого тесту: жодної нової команди — один прапорець --for на next"
  interaction.user-error-protection: "тримає hook-speaks-the-next-step: невідоме імʼя в --for — відмова, що називає відомі; чужий файл не переписується ніколи"
  interaction.user-assistance: "тримає hook-speaks-the-next-step: для чужого файлу слово несе **готовий уривок** — саме те, що треба вставити; порада «прибери файл» для чужих налаштувань була б шкодою, і хвиля її не дає"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "свідомо без нового тесту: слова — ключами через i18n (0002)"
  interaction.self-descriptiveness: "виправлено під час роботи, бо перша редакція обіцяла неправду: hook-конфіги — ЄДИНІ артефакти, що НЕ можуть сказати про себе «згенеровано». JSON коментаря не має, а вигадати ключ у чужій схемі — рівно те, що ця хвиля забороняє (Cursor у своїй схемі має additionalProperties: false, а Claude на невідомий ключ при пакуванні дає тверду помилку). Тож каже інструмент: рядок звіту рами і запис у [generated] keel.toml"
  reliability.faultlessness: "свідомо без окремого тесту: новий вид межі — одна гілка в тій самій функції one, а не другий механізм"
  reliability.fault-tolerance: "свідомо без нового тесту: невдача одного артефакта не спиняє інших (обіцянка 0023)"
  reliability.availability: "не застосовується: локальний бінарник"
  reliability.recoverability: "свідомо без нового тесту: вихід той самий, що в 0022; для чужого файлу виходу й не треба — ми його не рухали"
  security.confidentiality: "свідомо без тесту: hook не читає нічого, крім документів проєкту, і нічого не надсилає"
  security.integrity: "тримає hook-speaks-the-next-step: правка людини не топчеться — ні в чужому файлі, ні в нашому"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "свідомо без нового тесту: digest — той самий рецепт"
  security.resistance: "свідомо без фаззингу: вхід — два JSON-файли проєкту, читані як текст"
  maintainability.modularity: "свідомо без тесту: форма кроку для агента живе в next (один дім кроку), а не в generated"
  maintainability.reusability: "свідомо без тесту: наступна подія — новий рядок у тому самому шаблоні"
  maintainability.analysability: "свідомо без нового тесту: контракт називає подію, поля, форму відповіді кожного інструмента і джерело, з якого це прочитано"
  maintainability.modifiability: "свідомо без тесту: додати подію — дописати гілку в шаблон"
  maintainability.testability: "тримає hook-speaks-the-next-step: до проби додано **справжній валідатор** інструмента (claude plugin validate) — там, де він є під рукою; це soft-суддя, і так він і названий"
  flexibility.adaptability: "не застосовується: hook не залежить від адаптера мови"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "свідомо без тесту: hook-конфіг ставить keel init, як і решту артефактів"
  flexibility.replaceability: "свідомо без тесту: хто прибрав агента — не дістає його hook-конфіга; хто прибрав файл при живому записі — сказав «не треба» (школа 0022 R-2)"
  safety.operational-constraints: "тримає hook-speaks-the-next-step: hook кличе рівно одну команду читання (next) і нічого не пише; у Cursor він fire-and-forget, тож і блокувати нічого не може"
  safety.risk-identification: "не застосовується як окрема робота: загроза — переписаний файл чужих налаштувань — і є головною клаузулою сценарію"
  safety.fail-safe: "тримає hook-speaks-the-next-step: за будь-якої непевности не пишеться нічого, а слово каже, що вставити рукою"
  safety.hazard-warning: "свідомо без нового тесту: рядок звіту для чужого файлу попереджає до біди, не після"
  safety.safe-integration: "тримає hook-speaks-the-next-step: проєкт, що вже має свої hook-и, лишається зі своїми — keel не додає нічого і не забирає нічого"
---

## Why

Щабель 22 драбини bootstrap і друге слово рішення оператора §8.6:
«те ж стосується хуків». Досі машина тримала два правила через
commit-msg gate — тобто **після** того, як агент уже щось зробив.
Hook старту сесії — це перше слово інструмента **до** роботи: агент
відкриває сесію і одразу знає єдиний наступний крок, не вгадуючи.

Формати взято з доків самих інструментів, і цього разу — з
перепитуванням віку джерела, бо саме на віці я і впав у 0024:

- **Claude Code** (`code.claude.com`): `.claude/settings.json`, ключ
  `hooks` → імʼя події → масив груп `{matcher, hooks: [{type:
  "command", command, timeout}]}`. Події старту: `SessionStart` із
  matcher-ами `startup|resume|clear|compact|fork`. Вивід hook-а на
  виході 0: якщо це не JSON — **звичайний текст іде в контекст
  агента**. Тобто для Claude достатньо, щоб hook просто напечатав
  крок.
- **Cursor** (`cursor.com/docs/hooks`): `.cursor/hooks.json`,
  `{"version": 1, "hooks": {"sessionStart": [{"command": "…"}]}}` —
  поле `command` чинне, це їхній живий приклад. Події: `sessionStart`
  / `sessionEnd`, `preToolUse` / `postToolUse` /
  `postToolUseFailure`, `subagentStart` / `subagentStop`,
  `beforeShellExecution` / `afterShellExecution`, `beforeMCPExecution`
  / `afterMCPExecution`, `beforeReadFile` / `afterFileEdit`,
  `beforeSubmitPrompt`. `sessionStart` — fire-and-forget, а його
  **вивід несе `additional_context` і `env`**: контекст на старті
  Cursor приймає, але **лише JSON-ом**, не звичайним текстом.

Звідси головна асиметрія хвилі: **два інструменти беруть той самий
крок у різній формі відповіді**. Тому `keel next` дістає прапорець
`--for <агент>`: для `cursor` крок їде в `{"additional_context":
"…"}`, для `claude` — звичайним текстом, як сьогодні. Форма кроку
живе в `next` (один дім кроку), не в `generated`.

Сором, названий вголос, бо він і є причиною цієї обережности: у
хвилі 0024 я написав, що v1 генерував Cursor-у неіснуючі події, і
взяв перелік із машинної схеми `cursor-hooks@1.1.6` з npm —
десятимісячної. Оператор спитав «чого це жодної сесійної?» — і мав
рацію: `sessionStart` існує, `command` чинне, **v1 був правий**.
Машинне джерело я перевірив, а вік його — ні. Виправлено окремим
коммітом; урок цієї хвилі: **джерело мусить бути живим, а не лише
машинним**.

Третій вид межі, який дає ця хвиля: `.claude/settings.json` — файл
**чужих налаштувань**. Для нього «прибери файл» — не порада, а
шкода. Тож новий вид: **гість** — keel народжує файл, якщо його
нема (і далі судить його цілим, як своє), а якщо файл стоїть і не
наш — не пише **жодного байта** і каже, який саме уривок вставити.

Ще одне, за словом оператора («може у них є ліба для тестування як у
клода — використовуй»): у проби зʼявляється **справжній валідатор**
інструмента — `claude plugin validate` на скілах хвилі 0024, коли
бінарник Claude Code є під рукою. Це soft-суддя (без нього проба не
падає), і так він названий; догфуд же жене його завжди.

Відступи bootstrap, названі вголос: план їде план-гілкою §8.3
(exports `tool-next` ростуть наперед коду); журнал — chore-трансформою.

## scenario: hook-speaks-the-next-step

**Дано** проєкт, що назвав `claude` і не має `.claude/settings.json`;
проєкт, що назвав `cursor` і не має `.cursor/hooks.json`; проєкт,
чий `.claude/settings.json` — **власний файл людини** з її
налаштуваннями; проєкт, чий згенерований hook-конфіг людина
виправила рукою; і проєкт, що назвав лише одного з двох,
**коли** біжать `keel init`, `keel update` і сам hook —
`keel next --for <агент>`,
**тоді** без файлу hook-конфіг народжується і **парситься справжнім
JSON-парсером**, маючи рівно ту гілку, яку документує його
інструмент: у Claude — `hooks.SessionStart` із групою, де
`hooks[].type == "command"`, а matcher називає джерела старту; у
Cursor — `version == 1` і `hooks.sessionStart[].command`. Команда,
яку hook кличе, **існує в бінарнику**. `keel next --for cursor`
віддає **валідний JSON** із непорожнім `additional_context`, а
`--for claude` — той самий крок звичайним текстом; невідоме імʼя —
відмова, що називає відомі. **Власний файл людини не міняється
жодним байтом**: замість запису рядок каже, що вставити, і уривок у
слові — той самий текст, що ліг би у файл. Наш файл, виправлений
рукою, не топчеться. Агент, не названий у конфізі, hook-конфіга не
дістає. І скіли, згенеровані хвилею 0024, проходять `claude plugin
validate`, коли той є під рукою.

## transform: hooks-in-both-tongues

Таблиця артефактів дістає два рядки і третій вид межі. `Kind::Guest`
— «гість у чужому файлі»: народжується, якщо файлу нема; судиться
цілим, якщо наш; і **ніколи не пишеться** поверх чужого — замість
того слово несе готовий уривок. `next` дістає `step_for(root,
agent)` — той самий крок у формі відповіді названого агента; імена
агентів беруться з єдиного дому `config::AGENTS`.

Застереження: подія одна — **старт сесії**. Решта подій обох
інструментів (`preToolUse`/`PreToolUse` над правками, `afterFileEdit`,
`preCompact`, `beforeSubmitPrompt`, `stop`) — своїми хвилями; хвиля
не робить вигляду, що їх нема, і не обіцяє їх наперед. Про Cursor
сказано вголос і слабке місце: його `sessionStart` — fire-and-forget,
і в їхньому ж форумі люди скаржаться, що `additional_context` не
завжди доходить до агента; ми пишемо документовану форму, а
доставку гарантувати не можемо — це їхня межа, не наша.
Про `.claude/settings.json` сказано ще одне: Cursor читає **й
його** («third-party configs»), тож проєкт із двома агентами може
дістати той самий крок двічі — тією ж ціною, що названа в 0024 R-2,
і з тієї ж причини: не тримати підтримку на чужому шимі.
Названо вголос за §4.6: `tool/Cargo.lock` дописано в список під час
роботи — його змінив `serde_json` у dev-залежностях, і замовчати
рядок у diff-і було б тим самим дрейфом, за який суд scope і
існує. Валідатор `claude plugin validate` — soft-суддя: там, де бінарника
нема, проба про це каже і не падає; догфуд жене його завжди.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10).

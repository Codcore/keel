---
depends_on: [0045-the-fourth-tongue]

scenarios:
  javascript-tests-are-read-and-run:
    covers: [functional.completeness, reliability.faultlessness]
  a-javascript-contract-holds-its-form:
    covers: [functional.appropriateness, maintainability.analysability]
  a-tongue-that-cannot-tell-says-so:
    covers: [functional.correctness, safety.fail-safe]

transforms:
  the-javascript-hand:
    implements:
      - javascript-tests-are-read-and-run
      - a-tongue-that-cannot-tell-says-so
    files:
      - one new in tool/src/
      - tool/src/lib.rs
      - tool/src/adapter.rs
      - tool/src/config.rs
      - tool/src/tags.rs
      - tool/src/check.rs
      - keel/contracts/tool-adapter-javascript.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/javascript_tests_test.rs
      - tool/tests/javascript_border_test.rs
  a-javascript-module-is-compared:
    implements:
      - a-javascript-contract-holds-its-form
    files:
      - tool/src/holding.rs
      - keel/contracts/tool-holding.md
      - tool/tests/javascript_holding_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0046-the-fifth-tongue.md

decisions:
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "свідомо без тесту, і ціна названа: node --test нічого не збирає і нічого не пише — зміряно порожнім find після бігу; теки збірки нема, відмови за вільним місцем нема"
  compatibility.co-existence: "тримає javascript-tests-are-read-and-run: пʼята мова стає в перелік і не міняє поведінки жодного наявного проєкту — rust, ruby, elixir, python судяться побайтово так само"
  compatibility.interoperability: "свідомо без тесту: адаптер кличе `node --test` — те саме, що людина в терміналі; імʼя тесту йде окремим аргументом `--test-name-pattern`, не крізь шел, і екранується як регулярний вираз"
  interaction.appropriateness-recognisability: "свідомо без тесту: імʼя адаптера — `javascript`, синоніми `typescript`, `node`, `js`, `ts`; TypeScript не окрема мова для цього бігуна — node 22 знімає типи сам"
  interaction.learnability: "свідомо без тесту: команд не додається"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає javascript-tests-are-read-and-run: одрук у назві мови лишається відмовою з переліком — суд вибору хвилі 0038 тримає це для кожної нової мови"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "тримає a-tongue-that-cannot-tell-says-so: там, де мова НЕ розрізняє станів кодом, суд каже саме це — власну межу node, а не чужу; і каже, що читає TAP, а не код"
  interaction.user-assistance: "свідомо без тесту: кожна нова відмова несе «натомість» — суд слів проти коду (хвиля 0035) тримає підставлення"
  reliability.fault-tolerance: "тримає a-tongue-that-cannot-tell-says-so: SyntaxError у модулі — відмова вголос зі словами node, а не червоний тест, хоч код той самий, що в падіння; імʼя без збігу — «не бігло», хоч node виходить із 0 і рахує файл за пройдений тест"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "свідомо без тесту, і сказано точно: адаптер у проєкт не пише нічого — зміряно; node --test кешу не тримає"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "свідомо без окремої роботи, і названо, бо хвиля 0044 навчила: імʼя тесту з тега йде в `--test-name-pattern` як регулярний вираз — тож воно ЕКРАНУЄТЬСЯ до `^…$` з усіма метасимволами, інакше імʼя `a.b` збігалось би з `axb`, а `(` ламало б бігун; це тримає проба"
  maintainability.modularity: "тримає a-javascript-contract-holds-its-form: мовне обличчя живе в адаптері мови, суди питають адаптер — `battery_dir`, `battery_key`, `strip_comments`"
  maintainability.reusability: "свідомо без тесту, і сказано чесно: javascript НЕ ділить читача коментарів ні з rust (шаблонні рядки в зворотних лапках — свій текст), ні з родиною `#`; це четвертий читач, малий і один на js/ts, бо це та сама робота для обох"
  maintainability.modifiability: "свідомо без тесту, і число з README перераховане: сімнадцять місць, що гілкуються за мовою; ця хвиля додає ноги до них і жодного нового місця"
  maintainability.testability: "свідомо без тесту: проби будують справжні node-проєкти спільною рукою 0030 і женуть справжній node; де його нема — зупиняються вголос рукою хвилі 0044"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий числом — концепт назвав чотири стартові мови; з цією хвилею збудовано всі чотири; лишається RSpec"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "тримає javascript-tests-are-read-and-run: наявні rust-, ruby-, elixir- і python-проєкти судяться так само — це міряє проба вибору хвилі 0038 і батарея цілком"
  flexibility.adaptability: "тримає javascript-tests-are-read-and-run: пʼята мова — перша, чий бігун не розрізняє станів кодом виходу, тож диспетчер мусить пропускати ВЕСЬ вирок через текст, і це перевірка, що адаптер справді володіє вироком, а не тільки командою"
---

## Why

Концепт назвав стартовий набір: **Elixir, Ruby, Python, TypeScript/JS**.
Ця хвиля закриває набір. Бігун — `node --test`, вбудований у node 22:
без `npm install`, без залежностей, і TypeScript біжить тим самим
бігуном, бо node 22 знімає типи сам (`test/typed.test.ts` — зміряно,
rc=0, `ok 1 - ts works`). Jest/vitest/mocha — інші бігуни, інші хвилі;
сказано вголос.

**Зміряно справжнім node v22.22.2 перед тим, як писати план.**

**Головне число — і воно погане: node НЕ розрізняє станів кодом
виходу.**

| стан | код | TAP |
|---|---|---|
| зелене | 0 | `ok N - <імʼя>` |
| тест упав | **1** | `not ok N - <імʼя>` + діагностика |
| SyntaxError у модулі | **1** | `not ok 1 - test/toy.test.js` (файл як тест) + `SyntaxError: …` |
| імʼя без збігу | **0** | `ok 1 - test/toy.test.js`, `# tests 1`, `# pass 1` — **файл порахований пройденим тестом** |
| нема тестів узагалі | **0** | `# tests 0` |
| `skip` / `todo` | 0 | `ok 1 - x # SKIP`, `not ok 2 - y # TODO`, `# fail 0` |

Тобто це мова класу ruby, і гірша: у ruby хоч «не бігло» не зелене, а
тут node виходить із 0 і **рахує сам файл за один пройдений тест**,
коли жоден тест не збігся з іменем. Ворота, що читали б код, благословили
б `work:` над тестом, якого нема. Тож `classify` тут читає **лише
TAP**, і код — ніколи: `ok … - <імʼя тесту>` — зелене; `not ok … -
<імʼя тесту>` — червоне; `not ok … - <шлях файлу>` із `SyntaxError` у
діагностиці — злам збірки, відмова зі словами node; жодного рядка з
іменем тесту — «не бігло», байдуже що там код 0. `# SKIP`/`# TODO` —
не бігло.

**Голос node — TAP, вкладений відступом:**
```
# Subtest: grouped
    # Subtest: inside
    ok 1 - inside
ok 3 - grouped
```
Імʼя в TAP — **голе** імʼя тесту (`inside`), і `--test-name-pattern='^inside$'`
вибирає саме його; `describe` не додає префікса, на відміну від ExUnit.
Тож два тести з одним іменем у різних `describe` — межа, названа
вголос: тег називає імʼя, а імʼя не унікальне. Рядок `ok N - <describe>`
для самого блока — не тест і в мапу не йде.

**Імʼя тесту — рядок**, як в elixir: `test('it works', …)`, `it("…")`,
або в зворотних лапках. Читач тегів вивчає `test(` та `it(` із трьома
видами лапок. І **імʼя йде в регулярний вираз** — тож екранується:
`--test-name-pattern='^it works$'`, а `a.b (x)` → `^a\.b \(x\)$`. Це
тримає проба, бо хвиля 0044 навчила, що рядок, який іде в чужу
команду, — поверхня.

**Де живуть тести і модулі.** node сам шукає `**/*.test.{js,mjs,cjs}`,
`**/*-test.*`, `**/*_test.*`, `**/test-*.*`, `**/test/**/*.*` — де
завгодно. keel читає `test/**` і `tests/**` із розширеннями
`.test.js|.test.mjs|.test.cjs|.test.ts|.test.mts` — і називає межу:
що node збирає ширше. Модуль: `module: toy` → `src/toy.ts`,
`src/toy.js`, `src/toy/index.ts|js`, `toy.ts|js`; `module: toy.bar` →
`src/toy/bar.*` — крапки роблять теки, як у python (слеш у сегменті
суд форми відкидає як «поза проєктом», і це лишається).

**Читач коментарів — четвертий, і це сказано чесно.** JS має `//` і
`/* */` як rust, але ще й **шаблонні рядки** в зворотних лапках на
кілька рядків, і `'…'` — рядок, а не char-літерал. Ні rust-ів читач
(лапка `'` там — час життя), ні родина `#` не читають цього правильно.
Малий власний читач: `//`, `/* */`, `"…"`, `'…'`, `` `…` `` з екранами —
один на js і ts, бо це та сама робота.

**Нічого не пише** — зміряно порожнім `find`.

## scenario: javascript-tests-are-read-and-run

**Дано** node-проєкт із `keel.toml`, де `adapter = "javascript"`
(або `typescript`), `package.json`, тест у `test/toy.test.js` (чи `.ts`)
із тегом `// proves: <сценарій>@<редакція>` над `test('…', …)`, і
модуль у `src/`.
**Коли** біжить `keel check`, `keel gate` над `work:`-комітом, `keel close`.
**Тоді** тег прочитано і звірено з редакцією; один тест біжить
`node --test --test-name-pattern='^<екрановане імʼя>$' <файл>` і його
вирок читається з TAP-рядка з цим іменем; батарея біжить уся, і
**перелік, і вироки** беруться з TAP (`ok`/`not ok` з іменем тесту),
тож тест, якого читач не вмів назвати, для суду існує разом зі своїм
падінням; тест усередині `describe` зветься голим іменем, як його зве
node. `.ts` читається і біжить так само.

Адаптер не пише в проєкт нічого — зміряно після `close`.

## scenario: a-javascript-contract-holds-its-form

**Дано** контракт із `module: toy` (чи `toy.bar`) і `exports`, що
називають `export function works(a: number): boolean`.
**Коли** біжить `keel check`.
**Тоді** сигнатуру звірено з сирцем у тій розкладці, де node тримає
модуль (ts перед js, файл перед `index`); коментарі `//`, `/* */` і
**текст у будь-яких із трьох лапок, шаблонні рядки на кілька рядків
включно** — не сирець; розійшлась — названо; модуля нема — названо всі
шляхи, де шукали. 110 сигнатур keel лишаються зеленими.

## scenario: a-tongue-that-cannot-tell-says-so

**Дано** node-проєкт.
**Коли** біжить `keel close` чи `keel gate`.
**Тоді** SyntaxError у модулі — відмова вголос **зі словами node**, а не
червоний тест, хоч код виходу той самий, що в падіння; імʼя, з яким не
збігся жоден тест, — «не бігло», хоч node виходить із 0 і рахує файл за
пройдений; `skip`/`todo` — не бігло, не зелене й не червоне. І `keel
check` друкує **власну** межу node: станів кодом не розрізняє, вирок
читається з TAP; читається `test/` і `tests/`, а node збирає ширше;
однакові імена в різних `describe` не розрізнити; jest/vitest/mocha не
читаються.

## transform: the-javascript-hand

`tool/src/javascript.rs`: `test_files`, `unread_files`, `module_paths`,
`run_test` (імʼя екрановане в `^…$`), `run_all` (перелік і вироки з
TAP, ключ — `adapter::battery_key`), `classify` **лише за текстом**,
`first_error` зі словами node. Рядок у `NAMES` (`javascript`,
`typescript`, `node`, `js`, `ts`), `battery_command` = `node --test`.
Читач тегів учиться `.js/.mjs/.cjs/.ts/.mts`: `//`, `test(`/`it(` із
трьома лапками. `check` дістає межу node. Словник — обома мовами.
Контракт `tool-adapter-javascript.md` — з таблицею кодів, яка каже, чому
код не читається.

## transform: a-javascript-module-is-compared

`holding.rs`: `module_paths` для javascript, `strip_js` — четвертий
читач, малий. Контракт `tool-holding.md` називає четвертого читача і
чому він не третій член родини.

## transform: journal

Записи журналу цього покоління їдуть із хвилею (V2-PROCESS), разом із
чергою, README і звітом рецензії.

---
depends_on: [0054-the-norm-names-what-holds-it]

scenarios:
  a-skipped-test-proves-nothing:
    covers: [reliability.faultlessness, functional.correctness]
  the-form-court-reads-every-letter:
    covers: [reliability.fault-tolerance, interaction.inclusivity]
  the-config-is-written-as-toml:
    covers: [functional.completeness, interaction.user-error-protection]
  the-installer-takes-what-it-promises:
    covers: [flexibility.installability, flexibility.adaptability]
  the-report-is-committed-and-the-courts-say-how:
    covers: [interaction.user-assistance, maintainability.analysability]
  furniture-is-not-drift:
    covers: [maintainability.modifiability, safety.risk-identification]
  the-generated-frame-carries-the-tongue:
    covers: [compatibility.interoperability, safety.safe-integration]
  the-courts-agree-on-one-tree:
    covers: [safety.fail-safe, safety.hazard-warning]
  the-words-lead-somewhere:
    covers: [interaction.self-descriptiveness, interaction.learnability]

transforms:
  a-skip-is-not-a-run:
    implements:
      - a-skipped-test-proves-nothing
    files:
      - tool/src/ruby.rs
      - tool/src/elixir.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-adapter-ruby.md
      - keel/contracts/tool-adapter-elixir.md
      - tool/tests/skipped_test.rs
  the-boundary-is-a-char:
    implements:
      - the-form-court-reads-every-letter
    files:
      - tool/src/holding.rs
      - keel/contracts/tool-holding.md
      - tool/tests/unicode_unit_test.rs
  values-are-toml-strings:
    implements:
      - the-config-is-written-as-toml
    files:
      - tool/src/ask.rs
      - keel/contracts/tool-ask.md
      - tool/tests/config_quoting_test.rs
  the-installer-keeps-its-word:
    implements:
      - the-installer-takes-what-it-promises
    files:
      - install.sh
      - keel/contracts/tool-launcher.md
      - tool/tests/installer_word_test.rs
      - tool/tests/common/versions.rs
  the-report-and-the-refusals-say-what-next:
    implements:
      - the-report-is-committed-and-the-courts-say-how
    files:
      - tool/src/gate.rs
      - tool/src/next.rs
      - tool/src/close.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-gate.md
      - keel/contracts/tool-next.md
      - keel/contracts/tool-close.md
      - tool/tests/report_commit_test.rs
  furniture-of-the-tongue:
    implements:
      - furniture-is-not-drift
    files:
      - tool/src/scope.rs
      - tool/src/adapter.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-scope.md
      - tool/tests/tongue_furniture_test.rs
  the-frame-names-the-tongue:
    implements:
      - the-generated-frame-carries-the-tongue
    files:
      - tool/src/generated.rs
      - tool/src/gate.rs
      - keel/contracts/tool-generated.md
      - keel/contracts/tool-gate.md
      - tool/tests/frame_tongue_test.rs
  one-tree-one-verdict:
    implements:
      - the-courts-agree-on-one-tree
    files:
      - tool/src/adapter.rs
      - tool/src/close.rs
      - tool/src/check.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-adapter-cargo.md
      - keel/contracts/tool-close.md
      - keel/contracts/tool-cli.md
      - tool/tests/one_verdict_test.rs
  the-words-are-exact:
    implements:
      - the-words-lead-somewhere
    files:
      - tool/src/speak.rs
      - tool/src/tags.rs
      - tool/src/adapter.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-speak.md
      - keel/contracts/tool-tags.md
      - tool/tests/exact_words_test.rs
  the-asserts-can-fall:
    chore: "six negative asserts hunt English phrases the tool never says, and two probes of 0053-0054 hold their clauses by a weaker fact than written: the asserts are made able to fall and dead_assert_test reads English phrases too (final review, tests R-5, R-6)"
    files:
      - tool/tests/dead_assert_test.rs
      - tool/tests/court_words_test.rs
      - tool/tests/one_home_test.rs
      - tool/tests/body_test.rs
      - tool/tests/review_test.rs
      - tool/tests/rule_truth_test.rs
      - tool/tests/speak_test.rs
      - tool/tests/own_ci_test.rs
      - tool/tests/norm_marks_test.rs
  the-documents-tell-the-day:
    chore: "README, the concept and the generated texts say what the tool does on day one: the manual mode and the hooks, the release line, keel update and spike, the frame's own words about red births and the installer (final review, method R-5, R-6, R-7, R-14, R-15, R-16)"
    files:
      - README.md
      - docs/uk/NEW-CONCEPT.md
      - tool/src/generated.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS); the final review's three reports queued in BACKLOG, the rows this wave strikes struck"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - keel/reviews/0055-the-first-day-in-a-strangers-project.md
decisions:
  functional.appropriateness: "свідомо без тесту: жодної нової команди — хвиля лагодить те, що фінальні рецензії 2026-09-06 зміряли в чужому проєкті на пʼяти мовах; кожен рядок цитує параграф, який тримає (§4.8, §5.5, §6.2, §7.12, §7.16, §8.4, §9.7, §9.9)"
  performance.time-behaviour: "свідомо без тесту, і ціна названа: жоден суд не стає повільнішим — читачі виходу бігунів дістають по одній перевірці (позначка S, рядок (skipped), код виходу); проба з harness = false і хук pytest коштують два біги cargo/pytest у батареї"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "тримає furniture-is-not-drift: lock-файли, які лишає бігун мови, і .gitkeep під keel/ — не дрейф і не змінений контракт; у проєкт користувача інструмент нового не пише"
  interaction.appropriateness-recognisability: "не застосовується: команд не додається"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "тримає the-config-is-written-as-toml: значення з лапками і зворотними скісними пишуться в keel.toml рядком TOML, який парсер прочитає, — жоден суд не втрачає конфігу через власну ж підказку"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "свідомо без окремої роботи, і названо: чужий текст у назвах тестів і в keel.toml проходить через екранування (TOML, регулярні вирази, shell) — грали фінальні рецензенти, тримають старі проби; нових поверхонь хвиля не відкриває"
  maintainability.modularity: "свідомо без тесту: кожна правка живе в модулі, чиє слово вона виправляє — адаптер, ask, holding, gate, next, close, scope, generated, speak; нового модуля нема"
  maintainability.reusability: "не застосовується"
  maintainability.testability: "тримає the-courts-agree-on-one-tree: мутант cargo exit 101 при зелених вироках, що пережив батарею, дістає пробу з harness = false; дев'ять проб народжуються червоними на першій клаузі (§7.12)"
  flexibility.scalability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "свідомо без тесту, і названо: реліз 1.0.0 іде наступною легкою хвилею після цієї (рішення оператора 2026-09-06); черга 0056 — решта знахідок фінальних рецензій, названа в BACKLOG поіменно"
---

## Why

Фінальна перевірка перед релізом 1.0.0 (рішення оператора
2026-09-06): три свіжі агенти на трьох зрізах — баги і день перший у
чужому проєкті, повнота тестів, відповідність методиці — над main із
хвилями до 0054 (f744252). Разом 5 + 1 важких, 14 + 1 + 4 середніх,
7 + 4 + 13 легких; ця хвиля бере все, що збреше або впаде завтра в
чужому проєкті, і слова, які ведуть у нікуди; решта — черга 0056
(BACKLOG). Кожен рядок нижче зміряно рецензентом у пісочниці, а
важкі й перші середні відтворено автором на бінарнику f744252 перед
планом.

**Зміряно перед планом.**

**Пропущений тест доводить обіцянку.** ruby minitest `skip "later"`:
`ruby -Itest … -v` друкує `= S`, `1 runs, 0 assertions, 0 failures,
0 errors, 1 skips`, exit 0 — `keel gate` над `work:`: «1 тестів
сценаріїв зелені — робота проходить», а `keel close` каже «падав у
кожному бігу» (баги R-2, R-25; відтворено). elixir `@tag :skip`: `mix
test file:line` виходить 0 (`1 excluded, 1 skipped`), gate пропускає
роботу, `run_all` читає рядок `* test it works [L#6]` як біг і лічить
фантом «it works (skipped)» третім тестом; `close` зве обіцянку
доведеною (R-1, R-23; відтворено). Python і node цю діру закрили 0045
і 0046; ruby `classify` не дивиться на позначку `S`, elixir — на
`(skipped)`.

**Суд форми падає на не-ASCII імені.** Контракт `exports: ["def
self.ünïcode(a)"]`, у джерелі `def self.ünïcodex(a)`: `keel check` —
`panicked at src/holding.rs:382: byte index 23 is not a char
boundary`, exit 101; `close` так само (R-3; відтворено).
`found_bounded` після незбіглої межі робить `from = at + 1` і ріже
рядок посеред символа; межі токена читає байтами.

**`keel init` пише конфіг, який не читається.** `--adapter ruby`:
підказка `# ci = "ruby -Itest -e 'Dir.glob("test/**/*_test.rb")…'"`
несе лапки всередині лапок; розкоментована — «TOML parse error at
line 5»; через `--ci` — той самий рядок записується активним, і жоден
суд далі не працює, а `init` каже «born from your answers» (R-4;
відтворено). `ask::answered_rows`/`config_body` беруть значення у
`"…"` без екранування.

**Інсталятор не тримає слова.** `KEEL_REF=<гілка> sh install.sh` —
«no such version» і список тегів v0.8.x: перевірка `git rev-parse
"$wanted^{commit}"` у свіжому клоні бачить лише `main`, віддалені
гілки живуть як `origin/<name>`; launcher і `keel version` радять саме
цю команду (R-5). `KEEL_REPO=…` при наявному `source` мовчки тягне з
GitHub і каже «installed» (R-10). Launcher пише `KEEL_HOME="${KEEL_HOME:-…}"`
без `export`, тож бінарник читає `~/.keel` і `keel version` каже «no
version stands here» з-під власного дому (R-11). Тести R-4: обіцянки
launcher-а про `.keel-current` і `keel.before-launcher` тримає лише
рука.

**Файл рецензії: як закомітити і хто його читає.** Commit `review:
0001-a-wave` — «"review" — не red: і не трансформа хвилі; одрук не
проходить як «поза судом»» без «натомість»; `keel next` каже, куди
ляже звіт, і не каже, яким комітом (R-6; відтворено). Незакомічений
файл `keel/reviews/<хвиля>.md` — `keel close` читає з робочого дерева
і каже «closed» (R-7). Відмови hook-а — 34 ключі `gate-*`, з них
«натомість» несуть чотири (методика R-4).

**Меблі як дрейф.** `keel init` на гілці кладе `keel/contracts/.gitkeep`
— `check`/`next`/`close` кажуть «гілка легкої хвилі змінює контракт
.gitkeep» і ведуть до повної хвилі (R-8; відтворено); перший `red:`
крізь hook збирає крейт і лишає `Cargo.lock` — «гілка чіпає Cargo.lock,
якого жодна трансформа не називає» (R-20; відтворено); те саме чекає
`mix.lock`, `package-lock.json`, `Gemfile.lock`.

**Рама не несе мови.** Згенерований `keel.yml` python/elixir-проєкту:
checkout → інструмент → `keel check` → `keel close` → батарея, без
`setup-python`/`pip install pytest` і без `setup-beam` — блок мов
пишеться лише для репозиторію самого keel (R-14). Hook `exec keel gate
"$1"` без PATH GUI-клієнта — «exec: keel: not found» без слова
інструмента (R-15). Claude-hook кличе `keel next "${CLAUDE_PROJECT_DIR}"`
без `--for claude` — відмова конфігу (пін 9.9.9, зайве поле) іде
stderr-ом з exit 2 і до агента не доходить; cursor — `--for cursor`
(методика R-1; відтворено).

**Два суди над одним деревом.** `cargo test` з `[[test]] harness =
false`, чий раннер друкує зелений блок і виходить 1: чистий бінарник
відмовляє («cargo вийшов із 101, а читач не побачив жодного
червоного»), а мутант, що знімає умову, зве хвилю закритою — і
переживає всю батарею (тести R-1). `pytest` із хуком сесії `exitstatus
= 1` при «1 passed»: `gate` — «тест падає», `close` — «закрита»
(тести R-2). `#[should_panic]`: `run_all` бере ключ «it_panics - should
panic», `close` — «батарея не бігла тесту it_panics», gate — зелений
(R-9). Недовірений `verify`: `check` червоний, `close` — «closed, no
blockers», exit 0 (R-13). Скасована почата хвиля: `check` червонить її
теги як сироти «такого сценарію не знає жодна хвиля» (методика R-2).

**Слова ведуть у нікуди.** `keel method §6.3-б` — «у методиці цього
покоління нема параграфа» (є, :412; методика R-3; відтворено); рядок
запуску з `keel next` для elixir-тесту з українською назвою — `mix
test --only 'test:test додає…'` → «no test was executed», а суд давно
біжить `file:line` (R-12); один latin-1 файл тесту — відмова «stream
did not contain valid UTF-8; натомість: перевір права доступу» (R-16);
`test "it's \"quoted\""` — «тег не має тест-функції одразу за собою»,
відмова всього суду (R-17); `close-price` каже «зміряно на цьому
дереві» у чужому проєкті (методика R-8); заголовок keel.toml — «словник
§2.9» (методика R-5).

**Документи.** README: `manual` «only you» проти хука сесії, який
пише і в manual (методика R-6); «власний CI ставить пін 0.1.0 через
install.sh» — з 0053 збирає з дерева (R-7); концепт: `keel update` —
«керування версіями» (R-15), `check` на spike «заборона merge» (R-15);
AGENTS.md/скіл: «`red:` проходить лише тоді, коли тест справді падає»
— і з `mutant:` над зеленим (R-16); коментар кроку CI чужого проєкту —
«clones … builds from source» під шапкою «release first» (R-16).
Тести R-5: шість негативних `contains` на фрази, яких інструмент не
каже; R-6: `own_ci_test` і `norm_marks_test` тримають клаузи «`keel
check` зелений» слабшим фактом.

**Черга 0056 (BACKLOG, не ця хвиля):** воркспейс з двома крейтами
(R-18), стовбур `develop` (R-19), `__pycache__` після `ci` (R-21),
нормалізація рядків scope (R-22), теги в `src/` (R-24), порада піна
після 1.0.0 (R-26); методика R-9, R-11, R-12, R-13, R-14, R-17, R-18;
тести R-3 (три сценарії 0032/0033 без власного `red:` — борг історії,
названий).

## scenario: a-skipped-test-proves-nothing

**Дано** ruby-проєкт із тестом `skip` і elixir-проєкт із `@tag :skip`
над тегованими тестами, у хвилі з трансформою над ними.
**Коли** біжать `keel gate` над `work:` і `keel close`.
**Тоді** обидва суди кажуть, що тест не біг (пропущений — не зелений і
не червоний), одним словом на обидва суди: gate відмовляє роботі,
close зве обіцянку не доведеною; батарея elixir не лічить фантома
«(skipped)».

## scenario: the-form-court-reads-every-letter

**Дано** контракт з експортом, чиє імʼя починається не-ASCII літерою,
і джерело, де стоїть довше імʼя з тим самим початком; і те саме імʼя,
що стоїть точно.
**Коли** біжать `keel check` і `keel close`.
**Тоді** жодної паніки: довший двійник — «такої одиниці нема» поіменно,
точне імʼя — сигнатура звірена; межі токена читаються символами.

## scenario: the-config-is-written-as-toml

**Дано** `keel init --no-ask --adapter ruby` і `keel init --ci` зі
значенням, що несе лапки й зворотні скісні.
**Коли** конфіг народжується.
**Тоді** і активний рядок, і закоментована підказка — рядки TOML, які
парсер читає; `keel check` після розкоментування підказки судить, а не
відмовляє; `init` каже правду про народжене.

## scenario: the-installer-takes-what-it-promises

**Дано** `install.sh` цього дерева і світ проб із локальним `KEEL_REPO`.
**Коли** біжить `KEEL_REF=<гілка>`, повторний біг із іншим `KEEL_REPO`
при наявному `source`, launcher без `KEEL_HOME` у середовищі.
**Тоді** гілка віддаленого репозиторію ставиться за іменем; названий
`KEEL_REPO` стає джерелом і при наявному `source` (або скрипт каже, що
джерело інше, і не каже «installed»); launcher експортує `KEEL_HOME`, і
`keel version` бачить версії свого дому; `.keel-current`, якого нема, і
чужий `keel` на PATH — тримаються пробою, як обіцяє контракт.

## scenario: the-report-is-committed-and-the-courts-say-how

**Дано** гілку хвилі зі звітом рецензії в робочому дереві.
**Коли** біжать `keel next`, commit під темою `review: …` крізь hook,
`keel close` до і після коміту звіту, і відмови hook-а над одруком,
червоним тестом і тегом, якого нема.
**Тоді** `next` каже, яким комітом лягає звіт; кожна відмова hook-а
несе «натомість» (§9.7); `close` на гілці читає файл звіту з HEAD —
незакомічений звіт не «лежить поруч».

## scenario: furniture-is-not-drift

**Дано** гілку легкої хвилі, на якій `keel init` лишив
`keel/contracts/.gitkeep`, і гілку, де бігун мови лишив lock-файл
(`Cargo.lock`, `mix.lock`, `package-lock.json`, `Gemfile.lock`).
**Коли** біжать `keel check`, `keel next`, `keel close`.
**Тоді** `.gitkeep` — не змінений контракт; lock-файл бігуна — меблі
мови, не дрейф і не «оголошений і не торкнутий»; жоден із трьох судів
не веде до повної хвилі через них.

## scenario: the-generated-frame-carries-the-tongue

**Дано** `keel init` у python-, elixir-, ruby- і node-проєкті.
**Коли** народжуються `keel.yml`, hook і `.claude/settings.json`.
**Тоді** `keel.yml` чужого проєкту ставить його мову перед судами
(pytest, mix з hex, ruby з rspec, node), hook кличе інструмент так, що
без PATH GUI-клієнта відмова каже, де стоїть launcher, а Claude-hook
кличе `keel next --for claude`, і відмова конфігу доходить до агента
кроком з exit 0, як у cursor.

## scenario: the-courts-agree-on-one-tree

**Дано** rust-крейт із `[[test]] harness = false`, чий раннер друкує
зелений блок і виходить 1; python-проєкт із хуком сесії, що виходить
1 при «passed»; `#[should_panic]`; недовірений `verify`; скасовану
почату хвилю з тегованими тестами.
**Коли** біжать `keel gate`, `keel check`, `keel close`.
**Тоді** cargo, що вийшов не нулем без прочитаного червоного, — відмова
в обох судах, і пробу тримає мутант; pytest, що вийшов не нулем без
рядка падіння, — не зелений у батареї; `should_panic` — один ключ в
обох судах; недовірений verify — нестача хвилі в `close`, не «closed»;
теги скасованої хвилі — не сироти, а «не судиться».

## scenario: the-words-lead-somewhere

**Дано** `keel method §6.3-б`, elixir-тест з українською назвою і
`keel next`, latin-1 файл тесту, elixir-тест з екранованою лапкою в
назві, `keel close` у чужому проєкті, `keel init` заголовок.
**Коли** біжать команди.
**Тоді** `method` подає лічений-з-літерою параграф; рядок запуску —
`mix test файл:рядок`; latin-1 — слово про UTF-8 з правильним
«натомість»; екранована лапка читається, тег судиться; ціна `close`
не зве чуже дерево «цим»; заголовок конфігу не цитує §2.9.

## transform: a-skip-is-not-a-run

`ruby.rs`: позначка `S` у `-v`-рядку і `N skips` у підсумку — `NotRun`
в `classify`; `elixir.rs`: рядок `* test … (skipped)` не лічиться
бігом, `run_test` над пропущеним — `NotRun`. Слова обома мовами.
Контракти адаптерів. Проба над обома мовами і обома судами.

## transform: the-boundary-is-a-char

`holding::found_bounded`: крок після незбіглої межі — на символ, межі
токена — символами (`is_alphanumeric`/`_`). Контракт holding. Проба.

## transform: values-are-toml-strings

`ask.rs`: усі рядкові значення і підказки — через одну руку, що пише
рядок TOML (лапки, скісні); `keel init --ci` з лапками читається.
Контракт ask. Проба.

## transform: the-installer-keeps-its-word

`install.sh`: `KEEL_REF` бачить `origin/<ref>`; `KEEL_REPO` при наявному
`source` — `remote set-url` або чесне слово; launcher експортує
`KEEL_HOME`. Контракт launcher. Проба (світ проб з локальним репо і
`file://`).

## transform: the-report-and-the-refusals-say-what-next

`next.rs`: крок рецензії каже комітом «`<chore хвилі>: …` або тема без
слага»; `gate.rs`: відмови несуть «натомість» (одрук, червоний тест,
тег, застарілий тег, знятий сценарій, велика літера); `close.rs`: файл
звіту читається з HEAD на гілці (`git show HEAD:…`), з диска — лише без
git. Слова обома мовами. Контракти gate, next, close. Проба.

## transform: furniture-of-the-tongue

`scope.rs`: `contracts_changed` пропускає `.gitkeep` і не-`.md`;
lock-файли мови (`adapter::lockfiles`) — меблі мови, названі словом
scope; `check`/`next`/`close` їх не судять. Контракт scope. Проба.

## transform: the-frame-names-the-tongue

`generated.rs`: крок мови для чужого проєкту (pytest, mix+hex, ruby+
rspec, node) перед судами; Claude-hook — `keel next --for claude`;
`gate.rs`: hook несе шлях до launcher-а (або слово, де його шукати,
коли `keel` не на PATH). Контракти generated, gate. Проба.

## transform: one-tree-one-verdict

`adapter.rs`: ключ cargo без « - should panic»; `python.rs`/`adapter`:
не нульовий вихід без рядка падіння — відмова батареї, як у cargo;
`close.rs`: недовірений verify — нестача; `check.rs`: теги скасованої
хвилі — «не судиться», не сироти. Слова. Контракти cargo, close, cli.
Проба з `harness = false` (мутант тестів R-1 — червонить).

## transform: the-words-are-exact

`speak.rs`: лічені-з-літерою параграфи подаються за номером;
`adapter::run_line` elixir — `mix test файл:рядок`; `tags::read` — слово
UTF-8 з «натомість»; `quoted_after` читає екрановану лапку; слова
`close-price`, `init-config-header`. Контракти speak, tags. Проба.

## transform: the-asserts-can-fall

Шість негативних `contains` — на слова, які інструмент каже, або
позитивні; `dead_assert_test` читає й англійські фрази; `own_ci_test`
тримає імʼя інструмента в паніці і `keel check` над деревом;
`norm_marks_test` жене `keel check`.

## transform: the-documents-tell-the-day

README: `manual` і хук сесії, лінія релізу без install.sh; концепт:
`keel update`, spike; згенеровані тексти: «лише тоді» → «або з
мутантом», коментар кроку CI чужого проєкту.

## transform: journal

Записи журналу, три звіти фінальних рецензій у BACKLOG (черга 0056
поіменно), файл рецензії.

---
depends_on: [0046-the-fifth-tongue]

scenarios:
  rspec-examples-are-read-and-run:
    covers: [functional.completeness, reliability.faultlessness]
  a-spec-name-is-what-rspec-calls-it:
    covers: [functional.correctness, maintainability.analysability]
  minitest-and-rspec-live-in-one-project:
    covers: [compatibility.co-existence, safety.fail-safe]

transforms:
  the-second-reading-of-ruby:
    implements:
      - rspec-examples-are-read-and-run
      - minitest-and-rspec-live-in-one-project
    files:
      - tool/src/ruby.rs
      - tool/src/adapter.rs
      - tool/src/check.rs
      - tool/src/next.rs
      - keel/contracts/tool-adapter-ruby.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/rspec_tests_test.rs
      - tool/tests/rspec_border_test.rs
  a-spec-is-named-by-its-groups:
    implements:
      - a-spec-name-is-what-rspec-calls-it
    files:
      - tool/src/tags.rs
      - tool/tests/rspec_names_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0047-rspec-in-the-ruby-hand.md

decisions:
  functional.appropriateness: "свідомо без тесту: суд форми ця хвиля не чіпає — модулі ruby читаються, як читались"
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "свідомо без тесту, і ціна названа: один тест коштує двох бігів rspec — `--dry-run` за ідентифікатором і сам біг — і одного тимчасового файлу ПОЗА проєктом для JSON; це ціна точності, бо `-e` збігається за підрядком, а stdout може належати формату з `.rspec` проєкту"
  compatibility.interoperability: "свідомо без тесту: адаптер кличе `rspec` — те саме, що людина в терміналі; rspec сам додає lib/ і spec/ до шляху завантаження, зміряно без -I; `.rspec` проєкту (з його `--require spec_helper`) читається, як і в людини, бо без нього стандартний проєкт не завантажується"
  interaction.appropriateness-recognisability: "свідомо без тесту: адаптер лишається `ruby` — RSpec не мова, а друге читання тієї самої мови; імені не додається"
  interaction.learnability: "свідомо без тесту: команд не додається; `keel next` дає рядок людини `rspec <файл> -e '<повний опис>'` і каже, що `-e` — підрядок"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає a-spec-name-is-what-rspec-calls-it: тег над однорядковим `it { … }` — прикладом без імені — і тег усередині `shared_examples` — прикладом зі стількома іменами, скільки включень, — відмови вголос, і кожна називає котру"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "тримає minitest-and-rspec-live-in-one-project: `keel check` каже, що прочитано обидва читання, і називає межу кожного"
  interaction.user-assistance: "свідомо без тесту: кожна нова відмова несе «натомість» — суд слів проти коду (хвиля 0035) тримає підставлення"
  reliability.fault-tolerance: "тримає rspec-examples-are-read-and-run: SyntaxError при завантаженні — відмова зі словами ruby з JSON (`errors_outside_of_examples_count`, `messages`), а не червоний приклад, хоч код виходу той самий, що в падіння; ідентифікатор без збігу — «не бігло», хоч rspec виходить із 0 і каже «0 examples»"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "тримає rspec-examples-are-read-and-run: rspec без налаштованого `example_status_persistence_file_path` не пише нічого — зміряно порожнім find після close; JSON адаптера йде у файл поза проєктом (`--out` у тимчасовій теці системи); де проєкт налаштував персистенцію сам, пише проєкт, не адаптер, і межа названа; `SPEC_OPTS` зі середовища знімається — зміряно, що вона може відфільтрувати біг або скинути наш формат (урок 0046 R-6)"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "тримає rspec-examples-are-read-and-run: приклад вибирається ІДЕНТИФІКАТОРОМ (`spec/x_spec.rb[1:2:1]`), знайденим у `--dry-run` за повним описом, — імʼя з тега ніколи не йде в команду рядком; `-e` не використовується, бо збігається за підрядком; це тримає проба з шимом `rspec`, що записує argv (урок 0046 R-8)"
  maintainability.modularity: "тримає rspec-examples-are-read-and-run: друге читання живе в тому самому адаптері ruby, і диспетчер вище не знає, яке з двох читань відповіло"
  maintainability.reusability: "тримає a-spec-name-is-what-rspec-calls-it: читач тегів для spec-файлів бере стек блоків `do…end` за глибиною — ту саму механіку, що для elixir-ового `describe`, лише зі стеком, як у python-их класів"
  maintainability.modifiability: "свідомо без тесту: жодного нового місця диспетчеризації за мовою — читання обирається файлом (`_spec.rb` проти `_test.rb`), а не мовою; ноги в `adapter::run_line` і `next` лишаються в ruby-вій гілці"
  maintainability.testability: "свідомо без тесту: проби будують справжні rspec-проєкти спільною рукою 0030 і женуть справжній rspec; де його нема — зупиняються вголос рукою хвилі 0044; і ворота женуться над ЧЕРВОНИМ тегованим прикладом, над pending і над двома прикладами одного повного опису — урок рецензії 0046"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  flexibility.adaptability: "свідомо без тесту: це не нова мова, а доказ, що адаптер мови може мати два читання, не розсипавшись на два адаптери"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий числом — це остання названа дірка з черги концепту і хвилі 0038; після неї в черзі лишається дистрибуція"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "тримає minitest-and-rspec-live-in-one-project: проєкт лише з test/ судиться побайтово так само, як до хвилі — це міряє проба і батарея цілком"
---

## Why

Хвиля 0038 навчила ruby-адаптер minitest і назвала межу: **RSpec не
читається**. Ця межа стояла в `keel check` кожного ruby-проєкту з того
дня («RSpec приїде своєю хвилею»). Це та хвиля.

**Зміряно справжнім RSpec 3.13 (rspec-core 3.13.6, ruby 3.3.6) перед
тим, як писати план — і форми зіграно наперед, бо рецензії 0042,
0045 і 0046 тричі сказали одне: читач, що знає один вигляд мови, — не
читач.**

**Голос rspec — JSON, і він каже все.** `rspec --format json` дає на
кожен приклад **ідентифікатор** (`./spec/toy_spec.rb[1:9:1:1]`),
**повний опис** (`Toy#works when called returns true`) і **стан**
(`passed`, `failed`, `pending`). Це найкращий голос із усіх пʼяти мов:
ні TAP-відступів, ні підрядків. Перелік і вироки беруться з нього з
першого дня. **Але stdout не наш:** `.rspec` проєкту може нести свій
`--format documentation` — і стандартний проєкт мусить нести
`--require spec_helper`, без якого приклади не завантажаться, тож
обійти `.rspec` не можна. Зміряно: `--format json --out <файл>` кладе
JSON у файл, а stdout лишає формату проєкту; файл живе в тимчасовій
теці системи, не в проєкті. `SPEC_OPTS` зі середовища rspec теж читає
— зміряно: `--tag fast` у ній фільтрує біг, і наш формат зникає, — тож
адаптер її знімає, як 0046 зняв `NODE_COMPILE_CACHE`. І `--no-color`,
бо слова помилки інакше несуть коди кольору.

**Коди виходу — класу ruby**, як і чекалось: 0 зелене, 1 падіння, **1 і
SyntaxError при завантаженні** — і тут JSON каже сам:
`errors_outside_of_examples_count: 1`, а `messages[0]` несе слова ruby
(«While loading ./spec/… a `raise SyntaxError` occurred, RSpec will now
quit», далі `SyntaxError: … lib/toy.rb:2: syntax error`). **0 на
ідентифікатор без збігу** («0 examples, 0 failures»), 0 без прикладів.
Тож текст — перший; код — ніколи не єдиний.

**Імʼя прикладу — те, що rspec зве повним описом**, і воно складається
з груп: `RSpec.describe Toy` + `describe "#works"` + `context "when
called"` + `it "returns true"` = `Toy#works when called returns true`.
Правило зʼєднання зміряне: пробіл, **крім** опису, що починається з `#`
чи `.` — тоді без пробілу (`Toy#works`, `Toy.works`). `RSpec.describe
Toy do` дає імʼя константи; `describe Toy, "with a second arg"` —
`Toy with a second arg`, через пробіл; голе `describe` без `RSpec.` —
те саме. `it`, `specify`, `example` — три імені одного; метадані після
імені (`it "x", :slow, focus: false do`) до імені не входять; лапки —
обидва види. `xit`, `skip:`, `pending` у тілі — стан `pending`: не
зелене, не червоне, в мапі батареї нема; `--dry-run` каже про них
`passed`, тож стан береться лише зі справжнього бігу.

**Два приклади без імені, яке б назвав тег** — і обидва відмови вголос,
що називають котру. Однорядковий `it { … }`: rspec зве його `example
at ./spec/…:33`, повний опис — `Toy ` з пробілом у кінці. І приклад
усередині `shared_examples`: `it_behaves_like "shared"` дає `Toy
behaves like shared is shared`, `include_examples "shared"` — `Toy is
shared`; одне тіло, стільки імен, скільки включень.

**Вибір одного прикладу — ідентифікатором, не імʼям.** `-e` збігається
за **підрядком** (`-e works` бере і `works too`), тож у команду імʼя
не йде ніколи: `rspec --dry-run --format json --out … <файл>` дає
ідентифікатор за повним описом, і біг іде `rspec '<файл>[1:2:1]'` —
зміряно: один приклад, і його стан. Два біги на тест — ціна точності,
названа. Два приклади з одним повним описом (те саме `it` двічі в
одній групі) — обидва ідентифікатори йдуть у біг, і червоне серед них
— червоне: правило, яке рецензія 0046 (R-2) велить мати одне на обидва
суди.

**Нічого не пише** — зміряно порожнім `find` після бігу; `.rspec_status`
зʼявляється лише коли проєкт сам налаштував
`example_status_persistence_file_path`.

**Обидва читання в одному адаптері.** `test/**/*_test.rb` — minitest,
`spec/**/*_spec.rb` — rspec; проєкт може мати обидва, і батарея — це
обидві батареї разом, з ключами `test/…` і `spec/…`, що не
перетинаються (`adapter::battery_key` тримає `spec/` попереду, як
`tests/` у node). Адаптер лишається `ruby`: RSpec — не мова, а друге
читання тієї самої. `spec/support/`, `spec_helper.rb` — не читаються, і
названі.

## scenario: rspec-examples-are-read-and-run

**Дано** ruby-проєкт із `adapter = "ruby"`, `spec/toy_spec.rb` із тегом
`# proves: <сценарій>@<редакція>` над `it "…" do`, і модуль у `lib/`.
**Коли** біжить `keel check`, `keel gate` над `work:`-комітом, `keel close`.
**Тоді** тег прочитано і звірено; один приклад біжить за
**ідентифікатором**, знайденим у `--dry-run` за повним описом, і його
стан читається з JSON; батарея біжить уся, **перелік і вироки** — з
JSON, тож приклад, якого читач не вмів назвати, для суду існує разом
зі своїм падінням; `pending` — не бігло. SyntaxError при завантаженні —
відмова зі словами ruby; ідентифікатор без збігу — «не бігло», хоч
rspec виходить із 0.

Адаптер не пише в проєкт нічого — зміряно після `close`, і з
`SPEC_OPTS` у середовищі теж.

## scenario: a-spec-name-is-what-rspec-calls-it

**Дано** spec-файл із вкладеними `describe`/`context` — константа,
рядок, `#method`, `.method` — і `it`/`specify`/`example` усередині.
**Коли** читач тегів читає тег над прикладом.
**Тоді** імʼя тега — **повний опис**, як його складає rspec: групи через
пробіл, без пробілу перед `#…`/`.…`, константа за іменем, метадані
після імені — не імʼя, — і саме за цим описом приклад знаходиться в
`--dry-run`. Тег над однорядковим `it { … }` і тег усередині
`shared_examples` — відмови вголос, кожна називає котру.

## scenario: minitest-and-rspec-live-in-one-project

**Дано** ruby-проєкт із `test/` (minitest) **і** `spec/` (rspec).
**Коли** біжить `keel check` і `keel close`.
**Тоді** обидва читання прочитано: теги з обох, батарея — обидві разом,
ключі не перетинаються, червоне з будь-якого блокує. Проєкт лише з
`test/` судиться **побайтово так само**, як до хвилі; `keel check`
називає, що прочитано, і межу кожного читання (rspec: однорядкові
приклади без імені; `spec/support/` не читається).

## transform: the-second-reading-of-ruby

`ruby.rs`: `spec_files`, `run_spec` (dry-run → ідентифікатори → біг →
JSON з `--out`), `run_all` для обох читань, `classify_spec` за JSON,
слова ruby при зламі завантаження з `messages`. `adapter.rs`:
`run_test`/`run_all`/`run_line` обирають читання за файлом.
`check`: межа rspec поруч із minitest-овою, і `spec/support/`
поіменно. `next`: рядок людини для spec-файлу. Словник — обома мовами.
Контракт `tool-adapter-ruby.md` — з таблицею JSON-станів і кодів, і
без слів «RSpec не читається».

## transform: a-spec-is-named-by-its-groups

`tags.rs`: `_spec.rb` — стек груп за глибиною `do…end`, правило
зʼєднання rspec, `it`/`specify`/`example` із двома видами лапок і
метаданими після імені, відмови на однорядковий приклад і на приклад
у `shared_examples`. Контракт `tool-tags.md` лишається, як є: він
каже, що модуль не знає мови, і це правда й далі — форму імені
називає контракт адаптера ruby.

## transform: journal

Записи журналу цього покоління їдуть із хвилею (V2-PROCESS), разом із
чергою, README і звітом рецензії.

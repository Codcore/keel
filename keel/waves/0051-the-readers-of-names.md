---
depends_on: [0050-every-red-reaches-the-court]

scenarios:
  a-name-is-read-as-its-tongue-writes-it:
    covers: [functional.correctness, reliability.faultlessness]
  a-test-is-selected-as-its-runner-selects-it:
    covers: [compatibility.interoperability, reliability.fault-tolerance]
  what-is-not-code-is-not-read:
    covers: [maintainability.analysability, interaction.user-error-protection]
  the-readers-hold-their-mutants:
    covers: [maintainability.testability, safety.risk-identification]

transforms:
  the-depth-of-a-file-is-read-from-its-code:
    implements:
      - a-name-is-read-as-its-tongue-writes-it
      - what-is-not-code-is-not-read
    files:
      - tool/src/tags.rs
      - tool/src/docs.rs
      - tool/src/rev.rs
      - keel/contracts/tool-tags.md
      - keel/contracts/tool-docs.md
      - keel/contracts/tool-rev.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/name_readers_test.rs
      - tool/tests/not_code_test.rs
  the-runner-selects-by-what-it-knows:
    implements:
      - a-test-is-selected-as-its-runner-selects-it
    files:
      - tool/src/elixir.rs
      - tool/src/ruby.rs
      - tool/src/adapter.rs
      - tool/src/python.rs
      - keel/contracts/tool-adapter-elixir.md
      - keel/contracts/tool-adapter-ruby.md
      - keel/contracts/tool-adapter-cargo.md
      - keel/contracts/tool-adapter-python.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/runner_selection_test.rs
      - tool/tests/rspec_tests_test.rs
  the-borders-hold-their-mutants:
    implements:
      - the-readers-hold-their-mutants
    files:
      - tool/tests/readers_mutants_test.rs
      - keel/contracts/tool-tags.md
      - keel/contracts/tool-holding.md
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - keel/reviews/0051-the-readers-of-names.md

decisions:
  functional.completeness: "свідомо без тесту: хвиля бере кожен рядок черги «читачі імен і форми» з глобального ревʼю 2026-09-06 — усі названі поіменно у Why; що лишається, лишається в черзі з причиною"
  functional.appropriateness: "не застосовується: команд і форм не додається — читачі читають те, що вже мали читати"
  performance.time-behaviour: "свідомо без тесту: читачі лишаються однопрохідними по рядках; зняття літералів — той самий прохід із станом лапок"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "тримає a-test-is-selected-as-its-runner-selects-it: JSON rspec лягає в приватну теку, створену ексклюзивно, і зникає з нею після бігу — імʼя більше не передбачуване в спільному /tmp"
  compatibility.co-existence: "свідомо без тесту: `ruby -E UTF-8` — кодування аргументів для дитини ruby, не для оболонки людини; локаль машини не чіпається"
  interaction.appropriateness-recognisability: "не застосовується"
  interaction.learnability: "не застосовується"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "тримає a-name-is-read-as-its-tongue-writes-it: імʼя тесту не-ASCII літерами — `def test_ünïcode`, `test \"ünïcode holds\"` — читається цілком і біжить, а не обрізається до `test_`"
  interaction.user-assistance: "свідомо без тесту: кожна нова відмова несе «натомість» — суд слів проти коду (хвиля 0035) тримає підставлення"
  interaction.self-descriptiveness: "свідомо без тесту: межі, що лишаються (`--only` mix над не-ASCII тегом обходиться рядком файлу; heredoc ruby у стрічці рядка), названо в контрактах, не в `keel check` — нових рядків меж не додається"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "свідомо без тесту, і сказано: жоден читач не пише; тимчасова тека rspec — поза проєктом, як і досі"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "тримає what-is-not-code-is-not-read: тег у рядковому літералі, секція у fenced-блоці, BOM перед тегом — жоден не стає ні тегом, ні секцією, ні тишею"
  maintainability.modularity: "свідомо без тесту: жодного нового модуля — глибина ruby/elixir, літерали rust, кодування ruby, вибір mix живуть там, де жили їхні читачі й адаптери"
  maintainability.reusability: "свідомо без тесту: зняття рядкових літералів перед читанням тегів rust бере ту саму механіку, що читач форми (`holding::strip_comments` знає лапки), — одна рука для двох читачів"
  maintainability.modifiability: "свідомо без тесту: жодного нового місця диспетчеризації мов"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "свідомо без тесту, і названо: `[[test]] name` читається з Cargo.toml крейта TOML-читачем; таблиця, якої читач не розбирає, — відмова зі словами читача, не вгадування стема"
  safety.fail-safe: "тримає a-test-is-selected-as-its-runner-selects-it: обидва суди — gate і close — кажуть про один тест одне: ім'я, вибране рядком файлу чи кодуванням, біжить в обох, а не проходить в одному й «не виконане» в другому"
  safety.hazard-warning: "тримає a-name-is-read-as-its-tongue-writes-it: слово `LoadError` у повідомленні падіння minitest — падіння, не злам збірки: злам пізнається за формою рядка ruby, не за словом у тексті людини"
  safety.safe-integration: "свідомо без тесту: проби будують справжні проєкти і женуть справжні бігуни рукою 0030; де бігуна нема — зупиняються вголос рукою 0044"
---

## Why

Глобальне ревʼю 2026-09-06 лишило після хвилі 0050 другу чергу —
**читачі імен і форми**: місця, де інструмент читає імʼя тесту, глибину
файлу чи текст документа не так, як його читає бігун або людина, і
суд каже «не виконав» над тестом, що біг, або мовчить над тим, чого не
мало читати. Кожен рядок черги зміряно наново перед цим планом, на
тому самому бінарнику, у пісочницях.

**Зміряно перед планом.**

**rspec: слово `end` у рядку закриває групу.** `it "syncs end-to-end"
do … expect("the end of it").to include("end") … end` перед тегованим
`it "holds"` — `ruby_depth` рахує `end` словами в будь-якому рядку,
група закривається зарано, keel складає `holds`, а rspec — `Toy holds`:
gate «біг не виконав жодного тесту "holds"» (баги R-9). Читач лапок у
`ruby_depth` є (heredoc, коментар), а рядкових літералів на тому
самому рядку — нема.

**elixir: `endpoint = …` і коментар, що кінчається на ` do`.** Рядок
`endpoint = "http://x"` у тесті перед тегованим — лічильник `describe`
бачить `end` на початку рядка і закриває групу: gate «не виконав
"holds"» замість `group holds` (R-10). Коментар `# TODO: decide what to
do` всередині `describe` — рядок кінчається на ` do`, лічильник
відкриває рівень, і тегований тест **поза** `describe` дістає префікс
`group holds`. Ці два — дзеркало ruby-урока 0047 R-2: глибина за
словами, не за підрядками, і коментар — не код.

**elixir: не-ASCII імʼя не вибирається тегом.** `test "ünïcode holds"`
— gate «не виконав» через `--only test:test ünïcode holds`; сам `mix
test --only "test:test ünïcode holds"` каже «All tests have been
excluded», а з ASCII-іменем — `1 test, 0 failures`: межа mix, не
читача (R-11). Зміряно вихід: `mix test test/toy_test.exs:5` — вибір
**рядком файлу** — `1 test, 0 failures`. Читач тегів знає рядок
оголошення, тож вибір за рядком не потребує імені в команді взагалі;
close закривав і досі, бо батарея читає імена з виводу mix.

**ruby: `def test_ünïcode` читається як `test_`.** `fn_name` обрізає імʼя
на першому не-ASCII символі (R-19); те саме в python. І сам `ruby -n
test_ünïcode` під локаллю машини (`LANG=`) — `0 runs`; `ruby -E UTF-8`
або `LC_ALL=C.UTF-8` — `1 runs`. Кодування аргументів — слово адаптера
до дитини, не слово середовища: `-E UTF-8` у команді.

**python: тека з пробілом, дужки в parametrize.** `tests/my dir/test_toy.py`
— gate проходить (вузол у лапках), close «батарея не виконала»: перелік
`-rA` ріже рядок `split_whitespace`, і шлях із пробілом стає двома
словами (R-17). `@pytest.mark.parametrize("expr", ["f(x", "g(y"])` над
тегованим `def` — лічильник дужок читає літерали, «тег без функції
одразу за собою» (R-20).

**rust: `[[test]] name = "renamed"`, тег у raw string.** Ціль
`tests/w_test.rs` під іменем `renamed` — gate «cargo відмовляє»
(`--test w_test`), close закриває (батарея бачить `Running
tests/w_test.rs`): два суди про один тест (R-14). Рядок `// proves:
it-works@abcdef` усередині `r#"…"#` читається як тег над `fn not_a_test`
з чужого рядка літералу (R-15) — читач форми (`holding`) лапки знає,
читач тегів — ні.

**minitest: слово `LoadError` у повідомленні падіння.** `flunk "this is
not a LoadError, but says the word"` — gate «тести не збираються»,
`red:` теж, а close — «червоний тест» (R-22): злам збірки пізнається
підрядком, а не формою рядка ruby (`…:in 'require': cannot load such
file -- … (LoadError)`).

**Документи: `## scenario:` у fenced-блоці, hex у верхньому регістрі,
BOM.** Приклад файлу хвилі у ```-блоці тіла — «секція-сирота» (R-23).
`proves: it-works@FA2916` — тег читається, а порівняння каже «тримає
FA2916, текст дає fa2916» (R-26). BOM перед тегом у першому рядку —
«тегів звірено: 0» і жодної знахідки: тег невидимий, і мовчки.

**Тимчасовий файл rspec.** `keel-rspec-<pid>-<n>.json` у спільному `/tmp`
— імʼя передбачуване; класичний symlink-race (R-26). Тека, створена
ексклюзивно, з файлом усередині, — імʼя, яке ніхто не поставить наперед.

**Мутації, що вижили батарею** (розріз повноти тестів, R-4–R-7):
`tags::stands_between → true` — код між тегом і оголошенням більше не
«висячий тег», а єдина проба грає кінець файлу без `fn`;
`holding::found_bounded` без межі ПЕРЕД збігом — `pub fn works`
знаходиться в `xpub fn works`; `plan_window all→any` — контракт, який
тримають план-хвиля **і** почата хвиля, прощається; `strip_ruby_comments`
без лапок — `#` у ruby-рядку ріже оголошення. І три проби js без
фікстури: синоніми `typescript`/`node`/`js`/`ts`, «файл перед `index`»,
межі `keel check` (одна з чотирьох тримається) — R-13–R-15.

## scenario: a-name-is-read-as-its-tongue-writes-it

**Дано** файли тестів, де імʼя ховається за формою: spec-файл зі
словом `end` у рядкових літералах перед тегованим прикладом;
elixir-файл із рядком `endpoint = …` і коментарем, що кінчається на
` do`, всередині `describe`; python- і ruby-файли з `def test_ünïcode`;
python-файл із `parametrize("…", ["f(x", "g(y"])` над тегованим `def`;
minitest-падіння зі словом `LoadError` у повідомленні.
**Коли** читач тегів читає тег, і `keel gate` судить `work:`-коміт.
**Тоді** імʼя тега — те, що назвав би бігун: `Toy holds`, `group holds`
(і без префікса поза `describe`), `test_ünïcode` цілком, тест під
parametrize; глибина ruby й elixir рахує `end` словом коду, не
підрядком рядка чи коментаря; падіння зі словом `LoadError` — падіння,
а злам збірки пізнається за формою рядка ruby.

## scenario: a-test-is-selected-as-its-runner-selects-it

**Дано** тест, якого бігун не вибере за самим іменем: elixir-тест із
не-ASCII іменем; ruby-метод `test_ünïcode` під локаллю без UTF-8; ціль
cargo, перейменовану `[[test]] name`; python-тест у теці з пробілом.
**Коли** біжить `keel gate` над `work:`-комітом і `keel close`.
**Тоді** обидва суди виконують саме цей тест: mix — рядком файлу
(`test/x_test.exs:LINE`), ruby — з `-E UTF-8`, cargo — іменем цілі з
Cargo.toml, pytest — вузлом із шляхом у пробілах, і батарея читає його
вирок; і JSON rspec лягає в приватну теку, створену ексклюзивно поза
проєктом.

## scenario: what-is-not-code-is-not-read

**Дано** файл тестів rust із рядком `// proves: …` усередині рядкового
літералу; тіло хвилі з `## scenario: …` у fenced-блоці; тег із hex у
верхньому регістрі; тег у першому рядку файлу за BOM.
**Коли** біжить `keel check`.
**Тоді** літерал — не тег; fenced-блок — не секція; hex порівнюється
без огляду на регістр; BOM не ховає тега — і кожен із чотирьох випадків
судиться так само, як його чиста форма.

## scenario: the-readers-hold-their-mutants

**Дано** чотири мутанти читачів, що пережили батарею (висячий тег із
кодом між тегом і оголошенням; межа перед збігом сигнатури; вікно §6.5
з двома тримачами; `#` у ruby-рядку на рядку оголошення), і три
неграні обіцянки js (синоніми адаптера, файл перед `index`, чотири
межі `keel check`).
**Коли** біжать проби.
**Тоді** кожен випадок тримає окрема проба справжнім проєктом, і
мутант, підставлений у код, червонить її — зіграно і записано в
коміті народження рядком `mutant:` (виняток §6.3).

## transform: the-depth-of-a-file-is-read-from-its-code

`tags.rs`: `ruby_depth` знімає рядкові літерали перед лічбою слів;
elixir-глибина рахує `end` словом і не читає коментарів; `fn_name`
для python і ruby бере ідентифікатор цілком (літери за Unicode);
лічильник дужок parametrize не читає літералів; читач тегів rust
знімає рядкові й raw-літерали тією самою рукою, що читач форми; BOM
знімається перед читанням. `docs.rs`: секції тіла не читаються з
fenced-блоків. `rev.rs`: `matches` без огляду на регістр hex.
`ruby.rs`-класифікатор — у другій трансформі. Контракти tags, docs, rev.

## transform: the-runner-selects-by-what-it-knows

`elixir.rs`: `run_test` вибирає рядком файлу з тега, не тегом імені;
`ruby.rs`: `-E UTF-8` у командах ruby, злам збірки — за формою рядка,
JSON rspec у приватній теці; `adapter.rs`: ціль cargo — за `[[test]]`
Cargo.toml, де шлях перейменовано; `python.rs`: вузол із пробілами в
переліку `-rA`. Контракти чотирьох адаптерів. Проба чотирьох мов і
shim-проба rspec — про теку.

## transform: the-borders-hold-their-mutants

Проба `readers_mutants_test.rs`: сім випадків справжніми проєктами —
чотири мутанти читачів і три обіцянки js; коміт народження несе рядок
`mutant:` на кожен зіграний мутант. Контракти tags і holding кажуть,
що тримається пробою.

## transform: journal

Записи журналу цього покоління їдуть із хвилею (V2-PROCESS), разом із
чергою і звітом рецензії.

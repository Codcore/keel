---
depends_on: [0041-the-tool-lives-in-versions]

scenarios:
  elixir-tests-are-read-and-run:
    covers: [functional.completeness, reliability.faultlessness]
  an-elixir-contract-holds-its-form:
    covers: [functional.appropriateness, maintainability.analysability]
  a-tongue-that-tells-the-two-apart-says-so:
    covers: [functional.correctness, safety.fail-safe]

transforms:
  the-elixir-hand:
    implements:
      - elixir-tests-are-read-and-run
      - a-tongue-that-tells-the-two-apart-says-so
    files:
      - one new in tool/src/
      - tool/src/adapter.rs
      - tool/src/config.rs
      - tool/src/tags.rs
      - tool/src/check.rs
      - keel/contracts/tool-adapter-elixir.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/elixir_tests_test.rs
  an-elixir-module-is-compared:
    implements:
      - an-elixir-contract-holds-its-form
    files:
      - tool/src/holding.rs
      - keel/contracts/tool-holding.md
      - tool/tests/elixir_holding_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0042-the-third-tongue.md

decisions:
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "свідомо без тесту, і ціна названа: mix компілює в _build, тож перший біг батареї коштує компіляції — це та сама ціна, що й у cargo, і суд закриття її вже показує"
  compatibility.co-existence: "тримає elixir-tests-are-read-and-run: третя мова стає в перелік і не міняє поведінки жодного наявного проєкту — ані rust-ового, ані ruby"
  compatibility.interoperability: "свідомо без тесту: адаптер кличе mix — те саме, що людина в терміналі, без власних протоколів"
  interaction.appropriateness-recognisability: "свідомо без тесту: імʼя адаптера — сама мова (elixir), синонім mix, як cargo для rust"
  interaction.learnability: "свідомо без тесту: команд не додається"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає elixir-tests-are-read-and-run: одрук у назві мови лишається відмовою з переліком — суд вибору хвилі 0038 тримає це для кожної нової мови"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "тримає a-tongue-that-tells-the-two-apart-says-so: там, де мова розрізняє «впав» і «не зібрався», суд каже саме це, а не тягне за собою ruby-межу"
  interaction.user-assistance: "свідомо без тесту: кожна нова відмова несе «натомість» — суд слів проти коду (хвиля 0035) тримає підставлення"
  reliability.fault-tolerance: "тримає a-tongue-that-tells-the-two-apart-says-so: зламана компіляція — відмова вголос, а не червоний тест; тест, якого не існує, — «не бігло», а не зелене"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "свідомо без тесту: адаптер у проєкт не пише нічого — читає і жене назване"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "свідомо без окремої роботи: імʼя тесту йде окремим аргументом mix, не крізь шел"
  maintainability.modularity: "тримає an-elixir-contract-holds-its-form: мовне обличчя живе в адаптері мови, суди питають адаптер"
  maintainability.reusability: "тримає an-elixir-contract-holds-its-form: elixir і ruby ділять один зрізач коментарів (#), бо це та сама робота, а не дві схожі"
  maintainability.modifiability: "свідомо без тесту, і число з рецензії 0038 R-11 перевірене цією хвилею: мова — це модуль, рядок у NAMES і дотик до шести місць диспетчеризації плюс словник"
  maintainability.testability: "свідомо без тесту: проби будують справжні mix-проєкти спільною рукою 0030 і жеуть справжній mix"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий числом — концепт назвав чотири стартові мови, збудовано дві; Elixir третя, і саме він у концепті названий основним споживачем (keel-agent)"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "тримає elixir-tests-are-read-and-run: наявні rust- і ruby-проєкти судяться так само — це міряє проба вибору хвилі 0038 і батарея цілком"
  flexibility.adaptability: "тримає elixir-tests-are-read-and-run: третя мова — це перевірка того, що диспетчер справді диспетчер, а не rust із гілкою"
---

## Why

Концепт («Відкриті питання», п. 6, **рішення оператора**) назвав
стартовий набір: **Elixir, Ruby, Python, TypeScript/JS** — і про
Elixir сказано окремо, що він **основний споживач** (keel-agent).
Збудовано дві мови з чотирьох: rust (службова, щоб keel судив себе) і
ruby (хвиля 0038).

**Зміряно перед планом, справжнім `mix`, а не з памʼяті.** Elixir на
цій машині не було; поставив, зробив `mix new`, і питав адаптерові
питання бігом:

| питання адаптера | Elixir (mix + ExUnit) |
|---|---|
| де тести | `test/**/*_test.exs` |
| один тест | `mix test --only 'test:<повне імʼя>'` |
| батарея | `mix test --trace` — кожен тест окремим рядком `* test <імʼя> … [L#N]`, падіння як `N) test <імʼя> (<Модуль>)` |
| джерело модуля | `Toy.Bar` → `lib/toy/bar.ex` |
| корінь | `mix.exs` |
| тека збірки | `_build` |
| коментар | `#`, як у ruby |
| **оголошення тесту** | `test "<імʼя>" do` — імʼя **рядок**, а не ідентифікатор |

І головне число:

| стан | код виходу `mix test` |
|---|---|
| усе зелене | **0** |
| тест упав | **2** |
| **не скомпілювалось** | **1** |
| `--only` не збігся ні з чим | 1, і текст «no test was executed» |

**Це перша мова, де «впав» і «не зібрався» РОЗРІЗНЯЮТЬСЯ.** §7.12
писався саме на такий випадок — «де адаптер уміє відрізнити» — і в
ruby він не вміє (обидва 1, хвиля 0038). Тут уміє, і хвиля мусить це
сказати вголос, а не тягти ruby-межу за собою з чужої мови: межа, яка
не про цей проєкт, — така сама неправда, як і замовчана.

Друге, що зміряно і що ламає наявну форму: **імʼя тесту в ExUnit — це
рядок**. `tags.rs` знає `fn ` (Rust) і `def ` (Ruby) — обидва беруть
ідентифікатор після слова. `test "it works" do` не такий, і читач
тегів мусить навчитись третьої форми.

## scenario: elixir-tests-are-read-and-run

**Дано** elixir-проєкт із `mix.exs` і тестами ExUnit,
**коли** його судить `keel check`, `keel gate` чи `keel close`,
**тоді** теги `# proves: <сценарій>@<редакція>` читаються з
`test/**/*_test.exs`, і імʼям тесту стає **рядок оголошення** —
`test "it works" do` дає `it works`. Один названий тест біжить
окремо (народження червоного, §7.12), а батарея біжить уся і називає
кожен упалий тест його іменем. Наявні rust- і ruby-проєкти судяться
**так само, як судились**.

## scenario: an-elixir-contract-holds-its-form

**Дано** контракт із `module: Toy.Bar` в elixir-проєкті,
**коли** суд форми §7.6 звіряє його `exports`,
**тоді** він читає `lib/toy/bar.ex` і порівнює обіцяні сигнатури зі
згорнутим джерелом. Коментарі зрізає **той самий** зрізач, що й для
ruby: `#` відкриває коментар в обох, і це одна робота, а не дві
схожі. Модуля, якого нема, — знахідка зі шляхами, за якими шукали.
Межа та сама, що в ruby: Elixir типів не вимагає, тож порівнюється
імʼя функції і її параметри.

## scenario: a-tongue-that-tells-the-two-apart-says-so

**Дано** elixir-проєкт, чий тестовий файл не компілюється,
**коли** його судять,
**тоді** це **відмова вголос зі словами компілятора**, а не червоний
тест і не зелена батарея: `mix test` виходить із **1** на зламаній
компіляції і з **2** на падінні, і адаптер читає саме це. Тег над
іменем, якого нема, — «не бігло», а не зелене. І `keel check` у
такому проєкті **не друкує** межі §7.12 про нерозрізненність — вона
про ruby, а не про цей проєкт.

## transform: the-elixir-hand

Третій виконавець: пошук тестових файлів, біг одного тесту, біг
батареї з `--trace`, класифікація за кодом виходу. `Language` дістає
третій рядок, `tags.rs` — третю форму оголошення, `check.rs` — межу,
що зʼявляється лише там, де вона правдива.

## transform: an-elixir-module-is-compared

`holding.rs` питає адаптер, де лежить джерело модуля, і зрізає
коментарі знаками цієї мови — тими самими, що й ruby.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — `docs/uk/V2-PROCESS.md` (§9.10). BACKLOG
втрачає рядок про Elixir, README дістає третю мову в таблицю. Сюди ж
лягає звіт рецензії.

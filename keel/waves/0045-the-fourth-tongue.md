---
depends_on: [0044-green-on-my-machine]

scenarios:
  python-tests-are-read-and-run:
    covers: [functional.completeness, reliability.faultlessness]
  a-python-contract-holds-its-form:
    covers: [functional.appropriateness, maintainability.analysability]
  a-tongue-with-five-answers-says-them:
    covers: [functional.correctness, safety.fail-safe]

transforms:
  the-python-hand:
    implements:
      - python-tests-are-read-and-run
      - a-tongue-with-five-answers-says-them
    files:
      - one new in tool/src/
      - tool/src/lib.rs
      - tool/src/adapter.rs
      - tool/src/config.rs
      - tool/src/tags.rs
      - tool/src/check.rs
      - keel/contracts/tool-adapter-python.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/python_tests_test.rs
      - tool/tests/python_border_test.rs
  a-python-module-is-compared:
    implements:
      - a-python-contract-holds-its-form
    files:
      - tool/src/holding.rs
      - keel/contracts/tool-holding.md
      - tool/tests/python_holding_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0045-the-fourth-tongue.md

decisions:
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "свідомо без тесту, і ціна названа: pytest нічого не збирає; з `-p no:cacheprovider` і `PYTHONDONTWRITEBYTECODE=1` він і не пише нічого — зміряно порожнім `find` після бігу, тож теки збірки нема і відмови за вільним місцем нема"
  compatibility.co-existence: "тримає python-tests-are-read-and-run: четверта мова стає в перелік і не міняє поведінки жодного наявного проєкту — rust, ruby, elixir судяться побайтово так само"
  compatibility.interoperability: "свідомо без тесту: адаптер кличе `pytest` — те саме, що людина в терміналі, без власних протоколів; вузол тесту передається окремим аргументом, не крізь шел"
  interaction.appropriateness-recognisability: "свідомо без тесту: імʼя адаптера — сама мова (python), синонім pytest, як mix для elixir"
  interaction.learnability: "свідомо без тесту: команд не додається"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає python-tests-are-read-and-run: одрук у назві мови лишається відмовою з переліком — суд вибору хвилі 0038 тримає це для кожної нової мови"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "тримає a-tongue-with-five-answers-says-them: там, де мова розрізняє пʼять станів, суд каже саме це — і власну межу python, а не ruby-ву чи elixir-ову"
  interaction.user-assistance: "свідомо без тесту: кожна нова відмова несе «натомість» — суд слів проти коду (хвиля 0035) тримає підставлення"
  reliability.fault-tolerance: "тримає a-tongue-with-five-answers-says-them: злам збору (код 2) — відмова вголос зі словами python, а не червоний тест; вузол, якого pytest не знає (код 4), — «не бігло», а не зелене і не червоне"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "свідомо без тесту, і сказано точно: адаптер у проєкт не пише нічого — ані кешу, ані байткоду, — читає і жене назване; це не «не застосовується», бо хвиля 0044 навчила, що питання розрізу — про те, що робить код"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "свідомо без окремої роботи: вузол тесту йде окремим аргументом pytest, не крізь шел; імʼя береться з оголошення `def`, тобто з ідентифікатора, який не може нести метасимволів"
  maintainability.modularity: "тримає a-python-contract-holds-its-form: мовне обличчя живе в адаптері мови, суди питають адаптер — зокрема `battery_dir` і `strip_comments`"
  maintainability.reusability: "тримає a-python-contract-holds-its-form: python ділить із ruby й elixir один читач коментарів — `#` і трьохлапкові docstring-и це та сама робота, що elixir-ові `\"\"\"`, а не схожа"
  maintainability.modifiability: "свідомо без тесту, і число з README перевірене цією хвилею: мова — це модуль, рядок у NAMES, словник і чотирнадцять місць диспетчеризації; жодного нового місця ця хвиля не додає, і це міряється тим самим переліком"
  maintainability.testability: "свідомо без тесту: проби будують справжні pytest-проєкти спільною рукою 0030 і женуть справжній pytest; де його нема — зупиняються вголос рукою хвилі 0044"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий числом — концепт назвав чотири стартові мови, збудовано три; python четверта, лишається TypeScript/JS і RSpec"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "тримає python-tests-are-read-and-run: наявні rust-, ruby- й elixir-проєкти судяться так само — це міряє проба вибору хвилі 0038 і батарея цілком"
  flexibility.adaptability: "тримає python-tests-are-read-and-run: четверта мова — ще одна перевірка, що диспетчер справді диспетчер; і перша, чиї тести звуться ідентифікатором усередині класу, тож читач вчиться відступів"
---

## Why

Концепт назвав стартовий набір: **Elixir, Ruby, Python, TypeScript/JS**.
Збудовано три мови з чотирьох (rust — службова, щоб keel судив себе).
Python — четверта, і найпоширеніша з тих, що лишились.

**Зміряно справжнім pytest 9.0.2 перед тим, як писати план** — усе, що
нижче, з бігів, а не з памʼяті.

**Головне число: pytest розрізняє ПʼЯТЬ станів кодом виходу.**

| стан | код | текст |
|---|---|---|
| зелене | 0 | `N passed` |
| тест упав | **1** | `FAILED tests/test_toy.py::test_it_falls - assert 1 == 2` |
| злам збору (SyntaxError у тесті чи в модулі) | **2** | `ERROR collecting tests/…` / `Interrupted: 1 error during collection` |
| вузол, якого pytest не знає | **4** | `ERROR: not found: …::test_nope` |
| нічого не зібрано | **5** | `no tests ran` |

Elixir був першою мовою, де «впав» і «не зібрався» розрізняються;
python розрізняє ще й «такого тесту нема» (4) і «нема жодного» (5) —
рівно ті два стани, які §7.12 велить не плутати із зеленим. Тож
`classify` питає код, і лише код: тексту тут не треба зовсім.

**Голос pytest — один рядок на тест, з вироком:**
```
tests/test_toy.py::test_it_works PASSED
tests/test_toy.py::test_it_falls FAILED
tests/test_toy.py::TestGrouped::test_inside PASSED
```
Тобто перелік і вироки беруться **з голосу бігуна** з першого дня — це
урок 0038 R-1 і 0042 R-2, і цього разу він у плані, а не в рецензії.
Вузол `file::[Class::]name` — те саме, чим `pytest` вибирає один тест.

**Дві речі, які мова робить інакше.** Перша: тест **усередині класу**
(`class TestGrouped: def test_inside(self)`) зветься з префіксом класу
— як elixir-ів `describe`, тільки межа блока тут **відступ**, а не
`end`. Читач тегів вчиться відступів: клас відкривається рядком
`class …:` і триває, доки відступ не повернувся. Друга: pytest **пише**
— `.pytest_cache` і `__pycache__`. Адаптер жене з `-p no:cacheprovider`
і `PYTHONDONTWRITEBYTECODE=1`, і **зміряно**, що після цього `find` не
знаходить нічого. Адаптер, який пише в проєкт, — не адаптер.

**Де живе сирець модуля.** `module: toy` → `src/toy/__init__.py`,
`src/toy.py`, `toy/__init__.py`, `toy.py` — усі чотири розкладки
законні, і суд називає ті, що пробував. `module: toy.sub` → `…/toy/sub.py`.
Імʼя файлу — це імʼя модуля, без перекладу регістру: python не
робить snake_case із CamelCase.

**Коментарі й docstring-и.** `#` — як ruby; `"""…"""` і `'''…'''` — рівно
elixir-ова огорожа. Тож python **ділить читач** із ruby й elixir: та
сама рука, `strip_ruby(source, fenced = true)`. Хвиля 0043 зробила її
одним читачем на мовну родину — python третій член родини.

**Чого адаптер не читає, і каже.** pytest збирає `test_*.py` і
`*_test.py` де завгодно під `rootdir`; keel читає їх лише в `tests/`.
`conftest.py` і помічні файли в `tests/` не читаються — і називаються,
як ruby називає свої. unittest-стиль без pytest — інша хвиля.

## scenario: python-tests-are-read-and-run

**Дано** python-проєкт із `keel.toml`, де `adapter = "python"`, тест у
`tests/test_toy.py` із тегом `# proves: <сценарій>@<редакція>` над
`def test_…`, і модуль у `src/`.
**Коли** біжить `keel check`, `keel gate` над `work:`-комітом, `keel
close`.
**Тоді** тег прочитано і звірено з редакцією; один тест біжить
`pytest <файл>::<вузол>` і його вирок читається з коду виходу; батарея
біжить уся, і **перелік, і вироки** беруться з голосу pytest (`-v`),
тож тест, якого читач не вмів назвати, для суду існує разом зі своїм
падінням. Тест у класі зветься `Class::name` і вибирається так само.

Адаптер не пише в проєкт нічого — зміряно.

## scenario: a-python-contract-holds-its-form

**Дано** контракт із `module: toy` (чи `toy.sub`) і `exports`, що
називають `def works(a: int) -> bool`.
**Коли** біжить `keel check`.
**Тоді** сигнатуру звірено з сирцем у тій розкладці, де python тримає
модуль; коментарі й docstring-и — не сирець, тож `def`, живий лише в
`"""…"""`, не тримає; розійшлась — названо поіменно; модуля нема —
названо всі шляхи, де шукали. Ті самі 103 сигнатури keel лишаються
зеленими.

## scenario: a-tongue-with-five-answers-says-them

**Дано** python-проєкт.
**Коли** біжить `keel close` чи `keel gate`.
**Тоді** злам збору (код 2) — відмова вголос **зі словами python**
(`SyntaxError: …`, файл і рядок), а не червоний тест; вузол, якого
pytest не знає (код 4), — «не бігло», а не зелене й не червоне; нічого
не зібрано (код 5) — так само «не бігло». І `keel check` друкує
**власну** межу python: пʼять станів розрізняються, тож межа §7.12 тут
не стоїть; читається `tests/`, а не весь `rootdir`; unittest без pytest
не читається.

## transform: the-python-hand

`tool/src/python.rs` — четвертий адаптер: `test_files`, `run_test`,
`run_all` (перелік і вироки з `-v`), `classify` за кодом, `first_error`
зі словами python, `unread_files`, `module_paths`. Рядок у `NAMES`
(`python`, `pytest`), `battery_command` = `pytest -p no:cacheprovider`.
Читач тегів учиться `.py`: `#`, `def `, класи за відступом, docstring-и
як heredoc. `check` дістає межу python. Словник — обома мовами.
Контракт `tool-adapter-python.md` — з таблицею пʼяти кодів.

## transform: a-python-module-is-compared

`holding.rs`: `module_paths` для python (чотири розкладки), `strip_comments`
→ та сама рука родини `#`. Контракт `tool-holding.md` називає четвертого
члена родини.

## transform: journal

Записи журналу цього покоління їдуть із хвилею (V2-PROCESS), разом із
чергою, README і звітом рецензії.

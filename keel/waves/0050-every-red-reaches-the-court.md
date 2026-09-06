---
depends_on: [0049-the-first-implementation-leaves-the-root]

scenarios:
  every-verdict-keeps-its-own-key:
    covers: [functional.correctness, reliability.faultlessness]
  a-red-nobody-claims-holds-the-wave:
    covers: [safety.fail-safe, maintainability.analysability]
  the-battery-hears-no-word-from-outside:
    covers: [security.integrity, security.resistance]
  close-asks-the-form-of-every-contract:
    covers: [functional.completeness, interaction.self-descriptiveness]
  a-vanished-tag-is-red-in-every-tongue:
    covers: [maintainability.modularity, safety.risk-identification]
  the-generated-close-knows-its-branch:
    covers: [safety.safe-integration, interaction.operability]

transforms:
  the-key-is-the-target:
    implements:
      - every-verdict-keeps-its-own-key
    files:
      - tool/src/adapter.rs
      - tool/src/python.rs
      - tool/src/javascript.rs
      - tool/src/ruby.rs
      - keel/contracts/tool-adapter-cargo.md
      - keel/contracts/tool-adapter-python.md
      - keel/contracts/tool-adapter-javascript.md
      - keel/contracts/tool-adapter-ruby.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/verdict_keys_test.rs
      - tool/tests/javascript_tests_test.rs
  the-court-counts-every-red:
    implements:
      - a-red-nobody-claims-holds-the-wave
      - close-asks-the-form-of-every-contract
      - a-vanished-tag-is-red-in-every-tongue
    files:
      - tool/src/close.rs
      - tool/src/check.rs
      - tool/src/adapter.rs
      - tool/src/elixir.rs
      - keel/contracts/tool-close.md
      - keel/contracts/tool-adapter-cargo.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/every_red_test.rs
      - tool/tests/vanished_tongues_test.rs
      - keel/waves/0006-wave-closure.md
      - keel/waves/0016-drifted-records.md
      - tool/tests/next_test.rs
      - tool/tests/close_test.rs
  no-word-from-outside:
    implements:
      - the-battery-hears-no-word-from-outside
    files:
      - tool/src/python.rs
      - tool/src/javascript.rs
      - tool/src/ruby.rs
      - tool/src/scope.rs
      - tool/src/config.rs
      - tool/src/close.rs
      - install.sh
      - keel/contracts/tool-scope.md
      - keel/contracts/tool-config.md
      - keel/contracts/tool-launcher.md
      - keel/contracts/tool-close.md
      - keel/contracts/tool-adapter-python.md
      - keel/contracts/tool-adapter-javascript.md
      - keel/contracts/tool-adapter-ruby.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/outside_word_test.rs
      - tool/tests/pin_hand_test.rs
      - tool/tests/rev_test.rs
      - keel/waves/0039-the-tool-in-someone-elses-project.md
  the-generated-close-knows-its-branch:
    implements:
      - the-generated-close-knows-its-branch
    files:
      - tool/src/generated.rs
      - .github/workflows/keel.yml
      - keel.toml
      - keel/contracts/tool-generated.md
      - tool/tests/workflow_runs_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0050-every-red-reaches-the-court.md

decisions:
  functional.appropriateness: "свідомо без тесту: жодного нового суду — ті самі суди читають те, чого не читали: ціль cargo, обидва вироки вузла pytest, живість заявника червоного, форму контракту в close, всі пʼять мов у §7.15"
  performance.time-behaviour: "свідомо без тесту, і ціна названа: суд форми в close — читання файлів модулів без жодного бігу, той самий, що вже біжить у check; §7.15 для всіх мов — один `git ls-tree` бази і один `git show` на файл тестів, як для rust і досі"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "свідомо без тесту: PYTEST_ADDOPTS і NODE_OPTIONS знімаються лише з дітей батареї — оболонка людини їх не втрачає; `~/.rspec` і `.rspec-local` — файли машини і людини, не проєкту, тож суд їх не читає, а проєктний `.rspec` читає, як і досі"
  compatibility.interoperability: "свідомо без тесту: `rspec --options .rspec` — документований ключ rspec 3 (`-O`), зміряно перед планом: з ним читається лише файл проєкту, і відсутній файл — не помилка; `--test-reporter=tap` і `-rA` як були"
  interaction.appropriateness-recognisability: "не застосовується: команд і прапорців не додається"
  interaction.learnability: "не застосовується"
  interaction.user-error-protection: "тримає every-verdict-keeps-its-own-key: дві цілі cargo, оголошені одним іменем, — відмова, що називає їх, а не мовчазне злиття вироків одна в одну"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "тримає every-verdict-keeps-its-own-key: minitest, що не сказав ні слова, — вирок «не бігло» з порадою про `minitest/autorun`, а не зелене над тестом, який ніхто не виконав"
  reliability.fault-tolerance: "тримає the-battery-hears-no-word-from-outside: `.keel-ref` читається лише там, де він лежить поруч із бінарником; бінарник, зібраний cargo без інсталятора, судиться версією крейта, як і досі"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.non-repudiation: "свідомо без тесту: зняття двох сценаріїв 0006 і 0016 — позначка withdrawn із причиною та superseded_by у їхніх файлах (§2.12), видна в diff цієї хвилі; їхні тести не зникають, а переходять під every-wave-has-its-reviewer, бо перевіряють саме його правило, і рядок decisions у кожній старій хвилі закриває розріз, який ніс знятий сценарій"
  security.accountability: "не застосовується"
  security.authenticity: "тримає the-battery-hears-no-word-from-outside: суд піна вірить файлу `.keel-ref`, який поруч із бінарником записав інсталятор, а не змінній середовища, яку виставить будь-хто"
  maintainability.reusability: "тримає a-vanished-tag-is-red-in-every-tongue: правило «який шлях є файлом тестів цієї мови» живе в адаптері поруч із `test_files` і читається судом §7.15 через диспетчер — так само, як `tests_dir` читає рука §9.2"
  maintainability.modifiability: "свідомо без тесту: жодного нового місця диспетчеризації мов — предикат шляху стає ще одним рядком у тому самому переліку, і README рахує їх"
  maintainability.testability: "свідомо без тесту: проби будують справжні проєкти пʼяти мов рукою 0030 і женуть справжні інструменти; де інструмента нема — зупиняються вголос рукою 0044; світ проби pin_hand перестає ходити в мережу (рецензія тестів R-1) — тримає шим curl, як у світі 0048"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "тримає the-battery-hears-no-word-from-outside: launcher більше не мусить казати бінарнику його ref — бінарник читає `.keel-ref` сам; текст the-pin-has-a-hand (0039) оновлено свідомо, бо щабель `~/.keel/versions/` прийшов хвилею 0041, і редакція тега йде за текстом (§7.5)"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "свідомо без тесту, і названо: суд форми в close судить ті самі контракти, що й check, — вікно §6.5 «затверджена, ще не почата» і plan-гілка лишаються поза судом форми, як там; і рядок про несудиму форму close повторює словами check"
  safety.hazard-warning: "тримає the-generated-close-knows-its-branch: коментар згенерованого CI більше не каже «релізу ще нема» — він каже, що крок ставить пін проєкту: опублікований реліз, де він є, інакше збірку з сирців"
---

## Why

Хвилі 0046–0049 закрились, і оператор попросив глобального ревʼю трьома
агентами по трьох розрізах — баги, повнота тестів, відповідність
інструмента методиці. Три звіти прийшли з одним і тим самим важким
словом: **суд, який бачив червоне, закривав**. Хвиля 0043 закрила одну
таку діру — червоний тест, якого не заявляв ніхто; рецензенти знайшли
чотири інші дороги, якими червоне не доходить до суду, і в кожній мові
свою. Ця хвиля — про них. Дві інші хвилі цього покоління візьмуть
читачів імен і суди норми.

**Зміряно перед планом.**

**cargo зве цілі шляхом, а батарея — стемом.** `Running unittests
src/lib.rs (…/deps/toy-8a0b…)` і `Running unittests src/main.rs
(…/deps/toy-1ceb…)` — обидва стеми `unittests`, тож `verdicts.insert`
другого перезаписує першого: червоний `tests::smoke` бібліотеки зникає
під зеленим `tests::smoke` бінарника, і `keel close` виходить 0 там, де
`cargo test` виходить 101 (рецензія багів R-1). У воркспейсі
`a/tests/basic.rs` і `b/tests/basic.rs` cargo друкує **той самий**
рядок `Running tests/basic.rs` двічі — розрізнити їх можна лише хешем у
дужках, якого тег не знає: це відмова вголос, не вгадування. І тест,
який падає, друкуючи `test b_really_red ... ok`, показує цей рядок у
розділі `failures:` — а читач вироків читає його як вирок (R-16).

**pytest каже два слова про один вузол.** `test_s1 PASSED` у прогресі і
`test_s1 ERROR` після teardown (`-rA`: `PASSED tests/test_w.py::test_s1`
і `ERROR tests/test_w.py::test_s1 - AssertionError: teardown broke`,
exit 1). `ran()` бере перший — зелений; `gate` над тим самим деревом
судить кодом — червоний (R-3). Два суди на одному дереві розходяться,
і close — хибно зелений.

**node 22 не загортає файл.** `node --test test/w.test.js` друкує тести
на верхньому рівні; рядок `not ok 1 - test/w.test.js` зʼявляється лише
коли файл не завантажився — і `broken()` уже ловить його. Фільтр
`entry.name == shown` лишився від іншого node і сьогодні викидає
червоний тест, названий як свій файл (R-21).

**minitest без `minitest/autorun` мовчить і виходить 0.** `ruby -Itest
test/w_test.rb -n test_s1 -v` — жодного рядка, exit 0; `classify`
питає лише про `0 runs,` і зламану збірку, тож мовчання — зелене:
gate «проходить», close «не виконала» (R-7).

**rspec читає три файли, і два з них — не проєкту.** `~/.rspec` із
`--dry-run` робить кожен приклад `passed` (R-6); `.rspec-local` — так
само. `rspec --options .rspec` (документований `-O`) читає **лише**
файл проєкту, і відсутній `.rspec` — не помилка; зміряно RSpec 3.13:
з `~/.rspec = --dry-run` без ключа — `passed`, з ключем — `failed`.
`PYTEST_ADDOPTS="--deselect …"` і `NODE_OPTIONS=--test-skip-pattern=…`
так само знімають червоне з батареї, а `SPEC_OPTS` знято ще 0047.

**`KEEL_RUNNING_REF` вимикає суд піна словом.** `pin_mismatch` вірить
змінній без перевірки, а `forget_the_hook` її не знімає, тож слово
launcher-а доїжджає в тести проєкту (R-8). Файл `.keel-ref` поруч із
бінарником — те саме знання, яке інсталятор записав на диск: його
бінарник може прочитати сам.

**Суд закриття заявляє за іменем.** `claimed` у close бере всі теги
всіх сценаріїв хвилі — без фільтра withdrawn і без редакцій; червоний
тест із тегом чужої редакції або знятого сценарію не стає ні нестачею,
ні `red_tests`: суд друкує «червоний тест» і «закрита», exit 0 (R-2).
І `check` мовчить над живим тегом знятого сценарію — `continue` там, де
§2.12 каже «його тест видаляється тим самим PR» (методика R-8).

**Форму close не питає.** `close` жене verify і звіряє редакції, а суд
форми §7.6 живе лише в check: контракт, що обіцяє `def missing()`, —
check червоне, close «закрита», exit 0 (методика R-2).

**§7.15 судиться лише для rust.** `vanished_rows` питає
`adapter::crate_root` і `git ls-tree … <крейт>/tests` — для python,
ruby, elixir і javascript файл тесту зникає проти точки розгалуження
мовчки, а рядок «що перевірено» заявляє суд §7.15 (методика R-1, баги
R-12).

**Згенерований CI дає `KEEL_BRANCH` лише check-у.** На pull_request
HEAD відʼєднаний, тож `keel close` рахує блокери «своєї» хвилі — якої
нема — і каже «no blockers», exit 0 над хвилею в роботі (R-4).

**Два «доведені» сценарії обіцяють протилежне пробі.**
`closure-needs-review-file` (0006) і `light-pr-words-honest` (0016)
кажуть, що легка хвиля звіту не потребує; з рішення оператора
2026-09-04 (0037) рецензент у кожної хвилі, і їхні проби тримають рівно
це — а тег зелений, редакція збігається (рецензія тестів R-2). §2.12 дає
цьому слово: withdrawn із причиною, superseded_by, і тест переходить під
обіцянку, яку справді перевіряє.

**Батарея ходить у мережу.** `pin_hand_test` має власний світ без
`KEEL_RELEASES` і без шима curl, тож `install.sh` після 0048 іде
дорогою релізу на github.com двічі за прогін (рецензія тестів R-1) —
проти слова контракту `tool-release`.

## scenario: every-verdict-keeps-its-own-key

**Дано** чотири проєкти, де батарея губила вирок: rust — `src/lib.rs`
і `src/main.rs` з однойменними unit-тестами, червоним і зеленим, і
тегований тест, що падає, друкуючи `test <інший> ... ok`; python —
тест, зелений у тілі, з ERROR у teardown; javascript — червоний тест,
названий як шлях свого файлу; ruby — minitest-файл без
`minitest/autorun`.
**Коли** біжить `keel close` — і `keel gate` над `work:`-комітом, де
названо.
**Тоді** rust: ключ вироку — ціль, як її оголошує cargo (`unittests
src/lib.rs` ≠ `unittests src/main.rs`; `tests/w_test.rs` лишається
стемом, як у тегах), червоне не перезаписується зеленим, розділ
`failures:` не читається як вироки, а дві цілі з одним іменем — відмова,
що їх називає; python: тест із будь-яким червоним вироком — червоний;
javascript: тест, названий як файл, — тест зі своїм вироком; ruby: біг,
що не дав підсумку `N runs,`, — «не бігло», ніколи не зелене, і обидва
суди кажуть про нього те саме.

## scenario: a-red-nobody-claims-holds-the-wave

**Дано** гілку хвилі, чия батарея має червоний тест із тегом чужої чи
застарілої редакції — і червоний тест із тегом знятого сценарію.
**Коли** біжить `keel close` і `keel check`.
**Тоді** close рахує обидва серед блокерів і виходить не нулем:
заявленим є лише тег зі збіжною редакцією живого сценарію; check
називає живий тег над знятим сценарієм знахідкою §2.12 — тест
знімається тим самим PR, що й обіцянка.

## scenario: the-battery-hears-no-word-from-outside

**Дано** середовище з `PYTEST_ADDOPTS`, що знімає тест, `NODE_OPTIONS`,
що його пропускає, `KEEL_RUNNING_REF`, що зве розбіжний пін збіжним, і
(ruby) `~/.rspec` та `.rspec-local` із `--dry-run`.
**Коли** біжать суди.
**Тоді** вирок той самий, що в тиші: адаптери знімають обидві змінні,
rspec читає лише `.rspec` проєкту (`--options .rspec`), а суд піна
вірить файлу `.keel-ref` поруч із бінарником, не змінній; і жодне з цих
слів не досягає тестів проєкту — `KEEL_RUNNING_REF` і `KEEL_BRANCH`
зняті з кожної дитини батареї, а verify і ci біжать без hook-ових
змінних git і без успадкованого CARGO_TARGET_DIR.

## scenario: close-asks-the-form-of-every-contract

**Дано** проєкт, чий контракт обіцяє сигнатуру, якої модуль не тримає.
**Коли** біжить `keel close`.
**Тоді** суд форми (§7.6) біжить у close над тими самими контрактами,
що й у check, кожна знахідка — блокер поіменно, вихід не нуль; на
plan-гілці і у вікні §6.5 форма не судиться — як у check, і close каже
це тими самими словами.

## scenario: a-vanished-tag-is-red-in-every-tongue

**Дано** проєкт python, ruby, elixir чи javascript, де main тримає
доведену хвилю з тегом, а гілка видаляє файл тесту.
**Коли** біжить `keel check` на гілці.
**Тоді** знахідка §7.15 — та сама, що в rust: суд питає адаптер, який
шлях у базі є файлом тестів цієї мови, і не питає про крейт.

## scenario: the-generated-close-knows-its-branch

**Дано** згенерований CI на події pull_request — відʼєднаний HEAD.
**Коли** біжить крок `keel close`.
**Тоді** крок несе `KEEL_BRANCH`, як крок check, і close над
відʼєднаним HEAD із названою гілкою рахує блокери її хвилі; файл CI
цього репозиторію переписано `keel update`, і його відбиток у
keel.toml іде за ним; коментар файлу каже правду про те, що ставить
перший крок.

## transform: the-key-is-the-target

`adapter.rs`: ключ вироку cargo — ціль, як cargo її оголосив; для
`tests/*.rs` — стем, як у тегах; червоне не перезаписується; розділ
`failures:` поза читанням; дві однойменні цілі — відмова.
`python.rs`: `ran()` віддає кожен вирок вузла, батарея складає їх у
червоне. `javascript.rs`: фільтр за іменем файлу знято — зламаний файл
ловить `broken()`. `ruby.rs`: `classify` без підсумку minitest — «не
бігло»; батарея над файлом без підсумку — відмова зі словами про
`minitest/autorun`. Проба чотирьох мов; проба javascript міряє «нічого
не пише» повним обходом дерева (рецензія тестів R-11). Контракти
чотирьох адаптерів — словом про ключ і мовчання.

## transform: the-court-counts-every-red

`close.rs`: `claimed` — лише теги зі збіжною редакцією живих сценаріїв
хвилі; суд форми `holding::court` над тими самими контрактами, що в
check, блокером поіменно. `check.rs`: живий тег знятого сценарію —
знахідка §2.12; `vanished_rows` питає адаптер про файли тестів мови в
базі (`adapter.rs`: предикат шляху), а не про крейт. Два сценарії
0006/0016 зняті з причиною і наступником; їхні проби переходять під
`every-wave-has-its-reviewer`, а decisions старих хвиль закривають
розрізи, що лишились без cover. Контракт `tool-close.md` — про форму і
заявників.

## transform: no-word-from-outside

`python.rs`: `PYTEST_ADDOPTS` знято; `javascript.rs`: `NODE_OPTIONS`
знято; `ruby.rs`: `rspec --options .rspec`, minitest забуває hook;
`scope.rs`: `forget_the_hook` знімає і `KEEL_RUNNING_REF`; `config.rs`:
суд піна читає `.keel-ref` поруч із бінарником; `close.rs`:
`run_command` забуває hook і успадкований CARGO_TARGET_DIR;
`install.sh`: launcher не експортує змінної, якої ніхто не читає.
Світ `pin_hand_test` дістає `KEEL_RELEASES` і шим curl; текст
`the-pin-has-a-hand` (0039) оновлено, редакція тега — за §7.5.
Контракти scope, config, launcher, close і трьох адаптерів — словом.

## transform: the-generated-close-knows-its-branch

`generated.rs`: крок close несе `KEEL_BRANCH`, коментар першого кроку
каже правду після 0048; `keel update` переписує `.github/workflows/keel.yml`
цього репозиторію і відбиток у `keel.toml`. Проба тексту workflow.
Контракт `tool-generated.md`.

## transform: journal

Записи журналу цього покоління їдуть із хвилею (V2-PROCESS): три звіти
глобального ревʼю і те, що з них іде сюди, у 0051 і 0052, — разом із
чергою, README і звітом рецензії.

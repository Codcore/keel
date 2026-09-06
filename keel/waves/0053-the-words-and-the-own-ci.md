---
depends_on: [0052-the-courts-of-the-norm]

scenarios:
  the-word-is-the-courts-word:
    covers: [interaction.self-descriptiveness, functional.correctness]
  a-court-stricter-than-the-norm-says-so:
    covers: [interaction.user-assistance, safety.hazard-warning]
  the-old-revisions-count-once:
    covers: [maintainability.analysability, functional.completeness]
  the-own-ci-runs-the-same-battery:
    covers: [flexibility.installability, compatibility.interoperability]
  the-courts-hold-their-mutants:
    covers: [maintainability.testability, safety.risk-identification]

transforms:
  the-words-say-what-the-court-does:
    implements:
      - the-word-is-the-courts-word
      - the-old-revisions-count-once
    files:
      - tool/src/review.rs
      - tool/src/map.rs
      - tool/src/next.rs
      - tool/src/close.rs
      - tool/src/trust.rs
      - tool/src/check.rs
      - tool/src/docs.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-review.md
      - keel/contracts/tool-map.md
      - keel/contracts/tool-next.md
      - keel/contracts/tool-close.md
      - keel/contracts/tool-status.md
      - keel/contracts/tool-trust.md
      - tool/tests/court_words_test.rs
      - tool/tests/old_revisions_test.rs
  a-stricter-court-says-so:
    implements:
      - a-court-stricter-than-the-norm-says-so
    files:
      - tool/src/gate.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-gate.md
      - keel/contracts/tool-graph.md
      - keel/contracts/tool-rev.md
      - keel/contracts/tool-close.md
      - tool/tests/stricter_courts_test.rs
  the-own-ci-runs-the-same-battery:
    implements:
      - the-own-ci-runs-the-same-battery
    files:
      - .github/workflows/tool-ci.yml
      - .github/workflows/keel.yml
      - keel.toml
      - tool/src/generated.rs
      - tool/tests/common/mod.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-generated.md
      - tool/tests/own_ci_test.rs
  the-courts-hold-their-mutants:
    implements:
      - the-courts-hold-their-mutants
    files:
      - tool/tests/courts_mutants_test.rs
      - keel/contracts/tool-close.md
      - keel/contracts/tool-launcher.md
      - keel/contracts/tool-cli.md
  the-concept-says-what-exists:
    chore: "the concept and the READMEs name what exists, what is queued and what is not (methodology R-19; §2.10)"
    files:
      - docs/uk/NEW-CONCEPT.md
      - README.md
      - docs/uk/README.md
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - keel/reviews/0053-the-words-and-the-own-ci.md
decisions:
  functional.appropriateness: "свідомо без тесту: жодного нового суду — хвиля править слова судів, лічбу і власний CI; кожен рядок цитує параграф, який тримає (§2.10, §5.6, §6.3-а, §6.8, §7.12, §9.9)"
  performance.time-behaviour: "свідомо без тесту, і ціна названа: власний CI ставить ruby, elixir, python і node перед батареєю — хвилини раннера за те, щоб батарея CI дорівнювала батареї судів; локально ціна нульова"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "тримає the-word-is-the-courts-word: ціна `keel close` кажеться зміряним числом — стала і слова про ГіБ сходяться з тим, що лишає закриття на цьому дереві, а не з числом іншого покоління"
  compatibility.co-existence: "свідомо без тесту: жодних нових файлів у проєкті користувача — змінюється лише власний CI цього репозиторію і слова"
  interaction.appropriateness-recognisability: "не застосовується: команд не додається"
  interaction.learnability: "свідомо без тесту: `main-usage` каже однакову форму команд обома мовами — це тримає проба слів, не окремий тест"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає the-word-is-the-courts-word: порожній файл рецензії — одне слово в усіх судах, і `next` більше не веде до PR над ним"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "тримає the-word-is-the-courts-word: жодної відмови англійською поза словником — людина, що обрала мову, чує її в кожній відмові, і review.rs та trust.rs більше не говорять повз i18n"
  reliability.faultlessness: "тримає the-courts-hold-their-mutants: біг батареї, що загубив вирок тесту, не зелений — §7.13 тримає число бігів, не лише їхній колір"
  reliability.availability: "не застосовується"
  reliability.fault-tolerance: "тримає the-courts-hold-their-mutants: launcher над невідомим target-ом і над недосяжним сервером релізів — відмова зі словами, не тиша і не півустановка"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "свідомо без тесту, і сказано: власний CI keel-репозиторію судить гілку її ж бінарником, зібраним із дерева, а не бінарником upstream main — інакше PR судить не те, що в ньому"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "свідомо без окремої роботи, і названо: шим `uname` у пробі launcher-а — це гра пісочниці, не поверхня атаки; launcher читає uname тим самим шляхом, що й людина"
  maintainability.modularity: "свідомо без тесту: жодного нового модуля — слова живуть у словнику, лічба — у check, CI — у своїх файлах"
  maintainability.reusability: "тримає the-own-ci-runs-the-same-battery: рука `machine_has` одна для локальної батареї і для раннера — там, де раннер оголошений (`CI`), нестача інструмента не пропуск, а падіння поіменно"
  maintainability.modifiability: "свідомо без тесту: жодного нового місця диспетчеризації"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "свідомо без тесту, і названо: §7.1 проти §7.10 і §8.6 без позначки «текстове» — суперечність самої норми, і її текст — рядок оператора (BACKLOG), не цієї хвилі"
  safety.fail-safe: "тримає a-court-stricter-than-the-norm-says-so: суд, суворіший за букву, лишається суворішим — хвиля не мʼякшить жодного, а лише каже вголос, чому"
  safety.safe-integration: "тримає the-own-ci-runs-the-same-battery: тобі-ci.yml і keel.yml міняються так, що локальна батарея й батарея раннера читають одну й ту саму множину проб — і проба хвилі читає обидва файли"
---

## Why

Четверта черга глобального ревʼю 2026-09-06 і хвіст рецензій 0050–0052
— **слова судів і власний CI**: місця, де інструмент каже не те, що
робить, лічить не те, що показує, або судить у CI не тим, чим удома.
Кожен рядок зміряно наново на бінарнику 0051–0052 перед планом.

**Зміряно перед планом.**

**Слова не сходяться з судом.** На гілці скасованої хвилі `keel review`
складає пакет, `keel map` — мапу, і жоден не каже «скасовано», хоч
`keel check` каже (§6.3-а; методика R-12). Порожній файл
`keel/reviews/<хвиля>.md`: `next` — «час PR — легка хвиля їде в свій
один PR», а `status`/`close` — «порожній файл не рецензія» — два слова
про один стан (R-13). `close-blockers` каже «повна хвиля не зливається
недоведеною» і про легку. `main-usage` українською не знає `--for`,
англійською знає (R-16). Три відмови говорять англійською повз
словник: `review.rs:41` («the wave file cannot be read»), `trust.rs:170`
і `:187` (R-16). `gate-red-pass` каже «тест справді падає» однаково для
ruby, де падіння від зламу не відрізнити кодом (R-17). Шість судів
суворіші за букву норми і мовчать про це: `graph-scenario-twice`,
`graph-name-taken`, `graph-double-cover`, `gate-case`, `rev-nearmiss`,
`close-no-room` (R-18). `docs.rs:57` — «exactly one commit (§2.4)», а
§2.4 дозволяє кілька (R-21). Ціна `keel close`: стала 2 ГіБ і слово
«одне закриття лишає 1,26 ГіБ», а закриття 0051 лишило 3,0 ГіБ
(R-15).

**Лічба показує рядки, а не редакції.** `keel check` на цьому дереві:
«старих редакцій, справжніх в історії файлу, у закритих хвиль: 169» —
рядків 169, а одна й та сама `хвиля: контракт@редакція` повторюється
за кожним посиланням шапки (R-14: 159 рядків / 88 унікальних на
8363157).

**Власний CI судить не тим і не те.** `tool-ci.yml` жене `cargo test`
без ruby/rspec, mix, pytest, node — одинадцять проб пропускають себе
на раннері мовчки (stderr без `--nocapture`), і батарея CI вужча за
батарею судів (тести R-10). `keel.yml` ставить інструмент через
`install.sh` з `KEEL_REF 0.1.0` з upstream `main` — гілку судить не її
бінарник; коментар «builds it from source» правдивий лише наполовину.

**Проби, яких нема.** `every-reading-command-answers-in-json` (0040)
обіцяє «без --json побайтово те саме», а assert зник між red і HEAD
(тести R-9). Launcher: «невідомий target» і «сервер недосяжний» не
грані — `uname` не шимиться (R-12). Мутації, що вижили батарею (баги
R-25): `close.rs runs.len() == BATTERY_RUNS && all green → all green`
— біг, що загубив вирок, зелений; `KEEL_BRANCH` прибрано з
`forget_the_hook` — проба не помітила.

**Концепт каже те, чого нема.** `keel hook <event>` у концепті — у коді
`keel hook [dir]` ставить commit-msg hook; `keel check --fast` і
«перетини scope паралельних хвиль» — ні в коді, ні в черзі (R-19).
§7.1 проти §7.10 і §8.6 без позначки «текстове» (R-20) — суперечність
самої норми: її текст — рядок оператора, не цієї хвилі.

**Дрейф (§4.6), названий уголос.** З першої трансформи знято
`tool/src/status.rs`: план назвав його, бо `status` казав про порожній
файл рецензії своїм словом, — а вирок один для всіх судів виносить
`close::wave_state`, яким `status` і так читає стан хвилі, тож рука до
`status.rs` не торкнулась, і імʼя стояло б даремно (§4.4).

## scenario: the-word-is-the-courts-word

**Дано** гілку скасованої хвилі; порожній файл рецензії; легку хвилю
з нестачами на її гілці; два словники; джерела інструмента.
**Коли** біжать `keel review`, `keel map`, `keel next`, `keel close`,
`keel status`, і проба читає словники та джерела.
**Тоді** review і map кажуть «скасовано» так само, як check (§6.3-а);
порожній файл рецензії — одне слово в усіх судах, і крок `next` над
ним — не PR; блокери легкої хвилі звуться її вагою; `main-usage` має
одну форму обома мовами; жодна відмова не говорить повз словник —
кожна `reason` і `instead` у джерелах іде через i18n; ціна закриття —
зміряне число, і слова про ГіБ сходяться зі сталою.

## scenario: a-court-stricter-than-the-norm-says-so

**Дано** шість судів, суворіших за букву норми (`graph-scenario-twice`,
`graph-name-taken`, `graph-double-cover`, `gate-case`, `rev-nearmiss`,
`close-no-room`), і `gate` над червоним тестом ruby та rust.
**Коли** їхні слова друкуються.
**Тоді** кожне слово називає параграф, який тримає, або каже, що
суворіше за букву норми і чому; `gate-red-pass` над ruby каже межу —
падіння чи злам ruby кодом не розрізняє (§7.12) — а над rust каже
«справді падає», як і досі.

## scenario: the-old-revisions-count-once

**Дано** закриту хвилю, чия шапка посилається на один контракт однією
старою редакцією кілька разів (`proves` двох сценаріїв і `contracts`
трансформи).
**Коли** біжить `keel check`.
**Тоді** рядок «старих редакцій, справжніх в історії» лічить
редакції, не посилання: одна `хвиля: контракт@редакція` — один рядок
і одиниця в лічбі (§5.6).

## scenario: the-own-ci-runs-the-same-battery

**Дано** власний CI цього репозиторію: `tool-ci.yml` і згенерований
`keel.yml`; раннер, де інструмента бракує.
**Коли** проба читає обидва файли, і батарея біжить під оголошеним
раннером (`CI`).
**Тоді** `tool-ci.yml` ставить ruby з rspec, elixir, python з pytest і
node перед `cargo test`; під оголошеним раннером нестача інструмента —
падіння проби поіменно, не пропуск; `keel.yml` keel-репозиторію
збирає інструмент із дерева гілки, а не тягне upstream main, — і
`keel check` цього дерева зелений над обома.

## scenario: the-courts-hold-their-mutants

**Дано** чотири мутанти й межі, що пережили батарею: assert «без
--json побайтово те саме» (0040); launcher над невідомим target-ом і
над недосяжним сервером; біг батареї, що загубив вирок тесту;
дитина батареї, що читає `KEEL_BRANCH`.
**Коли** біжать проби.
**Тоді** кожен випадок тримає окрема проба справжнім проєктом, і
мутант, підставлений у код, червонить її — зіграно і записано в
коміті народження рядком `mutant:` (виняток §6.3).

## transform: the-words-say-what-the-court-does

`review.rs`/`map.rs`: слово «скасовано» на гілці скасованої хвилі
(§6.3-а); `next.rs`: порожній звіт — крок «звіт порожній», не PR;
`close.rs`: блокери за вагою, стала ціни і слова за зміряним;
`trust.rs`/`review.rs`: відмови через словник; `check.rs`: лічба
старих редакцій без повторів; `docs.rs:57` — коментар за §2.4;
`main-usage` обома мовами. Контракти review, map, next, close, status,
trust (лічба check-а — під tool-cli, як і його інші рядки). Дві проби.

## transform: a-stricter-court-says-so

Слова шести судів кажуть параграф або «суворіше за букву норми, бо…»;
`gate.rs` обирає слово `gate-red-pass` за адаптером — межа ruby
названа. Контракти gate, graph, rev, close. Проба.

## transform: the-own-ci-runs-the-same-battery

`tool-ci.yml`: кроки ruby/rspec, elixir, python/pytest, node перед
батареєю; `common/mod.rs`: `machine_has` під `CI` — падіння поіменно;
`generated.rs`: keel-репозиторій збирає інструмент із дерева;
`keel.yml` перегенеровано, відбиток записано. Контракт generated.
Проба.

## transform: the-courts-hold-their-mutants

Проба `courts_mutants_test.rs`: assert 0040, дві грані launcher-а з
шимом `uname` і `file://` у порожнечу, біг, що загубив вирок,
`KEEL_BRANCH` у дитини; коміт народження несе рядок `mutant:` на
кожен зіграний. Контракти close, launcher, cli кажуть, що тримається
пробою.

## transform: the-concept-says-what-exists

`NEW-CONCEPT.md` і README обома мовами: `keel hook` як є; `--fast` і
«перетини scope» — названо в черзі, не обіцяно.

## transform: journal

Записи журналу цього покоління їдуть із хвилею (V2-PROCESS), разом із
чергою (і рядком оператора про §7.1/§7.10/§8.6) і звітом рецензії.

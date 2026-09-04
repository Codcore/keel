---
depends_on: [0035-what-hurts-on-a-live-project]

scenarios:
  a-plan-branch-carries-no-code:
    covers: [functional.correctness, security.integrity]
  a-document-does-not-vanish:
    covers: [security.non-repudiation, reliability.recoverability]
  research-does-not-merge:
    covers: [security.resistance, safety.operational-constraints]
  the-weight-comes-from-the-file:
    covers: [functional.completeness, safety.risk-identification]
  work-without-a-proof-is-red:
    covers: [functional.appropriateness, reliability.faultlessness]

transforms:
  the-plan-branch-is-judged:
    implements:
      - a-plan-branch-carries-no-code
    files:
      - tool/src/scope.rs
      - tool/src/check.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/plan_branch_test.rs
  a-vanished-document-is-a-finding:
    implements:
      - a-document-does-not-vanish
    files:
      - tool/src/docs.rs
      - tool/src/check.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/vanished_doc_test.rs
  a-spike-is-said-aloud:
    implements:
      - research-does-not-merge
    files:
      - tool/src/scope.rs
      - tool/src/check.rs
      - tool/src/close.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/spike_test.rs
  the-weight-is-derived:
    implements:
      - the-weight-comes-from-the-file
    files:
      - tool/src/docs.rs
      - tool/src/check.rs
      - tool/src/status.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/weight_test.rs
  a-promise-without-a-test-is-red:
    implements:
      - work-without-a-proof-is-red
    files:
      - tool/src/check.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/untested_promise_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - keel/reviews/0036-the-holes-in-the-norm.md

decisions:
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "свідомо без тесту: усі нові суди читають git тією самою рукою scope::git_at, що й наявні"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "свідомо без тесту: кожна нова знахідка називає файл, гілку або хвилю поіменно"
  interaction.learnability: "свідомо без тесту: нові суди не додають команд — людина набирає той самий keel check"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає a-plan-branch-carries-no-code: код, покладений на план-гілку, — найдорожча помилка цієї глави, бо його ніхто не побачить у роботі"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "свідомо без тесту: кожна нова відмова несе «натомість», і суд слів проти коду (хвиля 0035) тримає підставлення"
  interaction.user-assistance: "свідомо без тесту: «натомість» кожної знахідки називає дію, а не правило"
  reliability.fault-tolerance: "тримає a-document-does-not-vanish: де історії нема або вона обрізана, суд каже «не перевірено», а не малює зелене"
  reliability.availability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.accountability: "тримає research-does-not-merge: слід дослідження — розділ Why хвилі, що з нього виросла, і механіка тепер цього вимагає"
  security.authenticity: "не застосовується"
  maintainability.modularity: "свідомо без тесту: суд ваги живе в docs.rs біля читання хвилі, суди гілок — у scope.rs біля імені гілки"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "свідомо без тесту: кожна знахідка називає параграф норми, з якого вона"
  maintainability.modifiability: "не застосовується"
  maintainability.testability: "свідомо без тесту: усі пʼять проб будують пісочниці спільною рукою 0030"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.fail-safe: "тримає work-without-a-proof-is-red: суд додає червоного там, де досі було мовчання, тож жоден із пʼяти не може зробити зелене з червоного"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "свідомо без окремої роботи: усі пʼять суворішають, і чужий проєкт може почервоніти — кожна відмова тому й каже, що робити"
---

## Why

П'ять важких дір, які три аудити знайшли **в самій нормі**: правило
написане, а механіки під ним нема. Кожна з них — зелене, здобуте без
перевірки, і §4.10 прямо каже, що таке зелене гірше за червоне.

**1. §4.9: на план-гілці можна класти код.** Відповідність, ВАЖКА-4.
Норма каже «план-гілка повної хвилі не торкається коду проєкту», і
жодна перевірка цього не судить: гілка `plan/<хвиля>` не зветься як
хвиля, тож суд scope пропускається цілком. Код, покладений там,
не побачить ніхто: у роботі його вже не буде в diff-і.

**2. §4.12: видалення файлу хвилі чи контракту — не червоне.**
Відповідність, ВАЖКА-3. Норма каже «червоне завжди: обіцянка помирає
позначкою `withdrawn`, а не зникненням файлу», і `renamed_from` —
єдиний законний шлях переїзду — не працює **зовсім**: поле парситься
і ніде не читається. Отже файл хвилі можна тихо стерти разом з усіма
її обіцянками.

**3. §4.13: `spike/*` обіцяно механікою, а слова `spike` в коді
нема.** Норма Л-8 і відповідність ВАЖКА-2. Норма каже: «заборона
тримається механікою: перевірка на PR зі `spike/*` червона». Її нема.
Гілка дослідження судиться так само, як будь-яка інша чужа гілка, —
тобто ніяк, і зливається без жодного слова.

**4. §6.8: вага хвилі не виводиться нізвідки.** Норма В-2 і
відповідність ВАЖКА-6. Правило точне: легка — одна трансформа, без
контрактів, без `withdrawn`. Ніхто його не рахує. Наслідок названий
аудитом прямо: chore із **новим контрактом** зветься легкою і в'їжджає
одним PR — тобто **без другого людського погляду**, того самого, що
§6.8 вимагає саме для нового контракту.

**5. §7.5: хвиля з робочими комітами і без жодного тесту не червона
ніде.** Відповідність, ВАЖКА-7. Зміряно: гілка хвилі, робочий commit,
жодного тега — `keel check` мовчить про це геть. §7.5 каже «кожен не
знятий сценарій має зелений тест зі своїм іменем»; на гілці це не
судиться.

## scenario: a-plan-branch-carries-no-code

**Дано** гілку `plan/<хвиля>`,
**коли** `keel check` судить її проти merge-base,
**тоді** кожен файл поза власними файлами методики (§4.8) — **знахідка**
з іменем файлу: план-гілка несе план, не код. Гілка, яка зветься
`plan/` чогось, що хвилею не є, дістає чесне слово, а не мовчазний
пропуск.

## scenario: a-document-does-not-vanish

**Дано** гілку, де файл хвилі чи контракту зник проти merge-base,
**коли** `keel check` судить її,
**тоді** це **знахідка**, що називає зниклий слаг, — **крім** випадку,
коли якийсь живий документ несе `renamed_from: <той слаг>`: тоді
зникнення законне. Два документи з однією спадщиною — знахідка;
переїзд між теками — знахідка. Де історії нема або вона обрізана,
суд каже «не перевірено».

## scenario: research-does-not-merge

**Дано** гілку `spike/*`,
**коли** її судить `keel check`,
**тоді** документи не судяться, і це сказано вголос: дослідження поза
методикою (§4.13). А `keel close` — суд, що каже, чи можна зливати, —
на такій гілці **червоний** з поясненням: дослідження не зливається,
знахідку повертають хвилею.

## scenario: the-weight-comes-from-the-file

**Дано** файл хвилі,
**коли** його читає `keel check` чи `keel status`,
**тоді** вага виводиться з нього за §6.8 і **називається вголос**:
легка — одна трансформа, жодного контракту в оголошених файлах,
жодного `withdrawn`; інакше повна. І якщо **повна** хвиля їде однією
гілкою — файл хвилі народився в тому ж diff-і, що й робота, — це
**знахідка**: два людські погляди, яких вимагає §6.8, не відбулися.

## scenario: work-without-a-proof-is-red

**Дано** гілку хвилі з робочими комітами,
**коли** `keel check` судить її,
**тоді** живий сценарій, що не має жодного тега тесту, — **знахідка**
поіменно. Хвиля «затверджена, ще не почата» (§7.5) лишається не
червоною: робочих комітів у ній нема.

## transform: the-plan-branch-is-judged

`scope.rs` розпізнає імʼя план-гілки і віддає хвилю, яку вона планує;
`check.rs` судить її diff проти merge-base: усе, що не власний файл
методики, — знахідка.

## transform: a-vanished-document-is-a-finding

`docs.rs` віддає `renamed_from` живих документів; `check.rs` порівнює
перелік документів на merge-base з переліком на HEAD і робить із
зниклого слага знахідку, коли спадщини ніхто не заявив.

## transform: a-spike-is-said-aloud

`scope.rs` розпізнає `spike/*`; `check.rs` каже про неї вголос замість
мовчазного пропуску, `close.rs` робить із неї відмову: злиття
дослідження — те, що норма забороняє завжди.

## transform: the-weight-is-derived

`docs.rs` рахує вагу хвилі за §6.8; `check.rs` робить із повної хвилі
на одній гілці знахідку, `status.rs` називає вагу кожної хвилі.

## transform: a-promise-without-a-test-is-red

`check.rs` судить живі сценарії хвилі, чия гілка має робочі коміти:
сценарій без жодного тега — знахідка поіменно.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10). BACKLOG
втрачає пʼять важких рядків, які ця хвиля закриває. Сюди ж лягає звіт
рецензії.

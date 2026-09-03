---
depends_on: [0030-probes-clean-up]

scenarios:
  a-verdict-says-how-much-of-it-is-real:
    covers: [interaction.self-descriptiveness, functional.correctness]
  the-closing-court-names-its-price:
    covers: [performance.resource-utilisation, reliability.availability]

transforms:
  the-verdict-carries-its-own-limits:
    implements:
      - a-verdict-says-how-much-of-it-is-real
    files:
      - tool/src/check.rs
      - tool/src/scope.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/verdict_limits_test.rs
  the-price-said-before-it-is-paid:
    implements:
      - the-closing-court-names-its-price
    files:
      - tool/src/close.rs
      - tool/src/adapter.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/closing_price_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md

decisions:
  functional.completeness: "тримає a-verdict-says-how-much-of-it-is-real: три речі, яких вирок не казав — обрізана історія, база порівняння і її свіжість, запушеність гілки — усі три в підсумку, не в середині простирадла"
  functional.appropriateness: "свідомо без тесту: доречність зміряна ціною — рецензенти 0028 і 0029 обидва НЕ ганяли keel close через тісний диск, а обрізаний клон судив на 141 звірку менше з тим самим підсумком"
  performance.time-behaviour: "свідомо без тесту: жодного походу в мережу — суд запушеності питає лише те, що знає цей клон (refs/remotes/origin/<гілка>). План казав «одне ls-remote з межею часу»; роблячи, я вибрав локальне питання: межа, яка залежить від досяжности сервера, змінюється з погодою, а вирок має бути тим самим двічі поспіль"
  performance.capacity: "свідомо без тесту: три рядки тексту у вироку"
  compatibility.co-existence: "тримає the-closing-court-names-its-price: спільний диск — саме те, за що платили рецензенти три хвилі поспіль"
  compatibility.interoperability: "свідомо без тесту: git ls-remote і df — те, що є всюди, де є git"
  interaction.appropriateness-recognisability: "свідомо без тесту: межі стоять у підсумку, поруч із числом знахідок, а не в переліку поверхів"
  interaction.learnability: "не застосовується"
  interaction.operability: "тримає the-closing-court-names-its-price: ціну сказано ПЕРЕД тим, як її платити — людина на тісному диску встигає спинитись"
  interaction.user-error-protection: "тримає the-closing-court-names-its-price: коли місця не стане, суд відмовляється НА ВХОДІ з числом, а не помирає на півдорозі з «no space left on device»"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "свідомо без тесту: кожна межа несе «натомість» — як дістати повний вирок (git fetch --unshallow, git push)"
  reliability.faultlessness: "свідомо без тесту: жодна з нових меж не міняє того, як рахуються знахідки — вирок лишається тим самим вироком, лише перестає прибріхувати про власну повноту"
  reliability.fault-tolerance: "тримає a-verdict-says-how-much-of-it-is-real: git, що не може відповісти (нема репозиторію, нема такого ref), дає межу, а не відмову — вирок не червоніє від того, чого не знає. Мережі тут нема взагалі (див. performance.time-behaviour)"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується: інструмент нікуди не ходить — усі три межі читаються з власного клону"
  security.integrity: "не застосовується"
  security.non-repudiation: "тримає a-verdict-says-how-much-of-it-is-real: вирок, який каже, чого він не судив, — це і є незречення; сьогодні обрізаний клон дає той самий підсумок «0 знахідок», що й повний, зробивши на 141 звірку менше"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "свідомо без тесту: межі збирає одна рука, вирок її лише друкує"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "тримає a-verdict-says-how-much-of-it-is-real: саме заради цього хвиля і є — вирок, що не каже своєї межі, не аналізується взагалі"
  maintainability.modifiability: "свідомо без тесту: нова межа — рядок у тій самій руці"
  maintainability.testability: "свідомо без тесту: обидві проби роблять обрізаний клон і клон без origin своїми руками — пісочницею хвилі 0030"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "тримає the-closing-court-names-its-price: суд не починає роботи, для якої на диску нема місця"
  safety.risk-identification: "свідомо без окремої роботи: загроза названа числом — 3,3 ГБ у tool/target зараз і 20 ГБ пісочниць у хвилі 0029"
  safety.fail-safe: "тримає the-closing-court-names-its-price: відмова на вході лишає диск таким, яким був"
  safety.hazard-warning: "тримає the-closing-court-names-its-price: попередження йде ДО витрати, а не після"
  safety.safe-integration: "тримає a-verdict-says-how-much-of-it-is-real: жоден наявний суд не слабшає — додаються лише рядки про те, чого суд не робив; число знахідок рахується так само"
---

## Why

Чотири борги з BACKLOG, і всі чотири про одне: **вирок каже більше,
ніж зробив**.

Зміряно перед планом, живими бігами:

- **Обрізаний клон.** Повний вирок — 208 рядків, серед них **141
  стара редакція, звірена по історії** (§5.6). Обрізаний — 67
  рядків, **нуль звірок**. Один рядок посеред простирадла про це
  каже, але **підсумок в обох випадках той самий**: «52 документи, 0
  знахідок», і той самий перелік «перевірено цим поверхом». Той, хто
  читає останній рядок — а його читають усі, — не бачить різниці між
  вироком, що зробив 141 звірку, і вироком, що не зробив жодної.
- **База порівняння.** `keel check` **жодного разу не питає origin** —
  грепом по `check.rs` нема ані слова. Він судить scope проти
  локального `main`, яким би несвіжим той не був, і не каже про це
  нічого.
- **Запушеність.** Ніде в інструменті нема `ls-remote`. `keel close`
  каже «хвиля закрита», не знаючи, чи гілка взагалі дійшла до origin.
  Хвиля, закрита лише на цьому диску, — не закрита.
- **Ціна закриття.** `tool/target` зараз **3,3 ГБ**. І тут важливе
  уточнення до самого BACKLOG: `keel close` робить власний `target`
  **навмисно** — `adapter.rs:116` знімає `CARGO_TARGET_DIR` із
  коментарем «успадкований спільний кеш зсуває вироки (§6.7, лікуван‑
  ня 0005 за R-8 рецензії 0008; бачено живцем і в 0006)». Тобто це не
  вада, а рішення, і хвиля його **не скасовує**. Вада в іншому: ціна
  ніде не названа, і суд помирає на півдорозі, коли місця нема.
  Рецензенти 0028 і 0029 **обидва свідомо не ганяли `keel close`**
  саме через це — суд закриття лишився неперевіреним двічі поспіль.

Спільна межа, яку хвиля не переступає: вона **не робить жодного суду
суворішим**. Вона лише додає вирокові правду про самого себе.

## scenario: a-verdict-says-how-much-of-it-is-real

**Дано** вирок `keel check`,
**коли** щось завадило судити повно — історія обрізана, бази
порівняння нема або вона несвіжа, гілку не запушено,
**тоді** підсумок вироку **сам** несе цю межу: скільки звірок не
зроблено і чому, проти чого порівнювалось і наскільки та база стара,
чи є гілка на origin. Обрізаний клон і повний більше **не дають
однакового останнього рядка**. Кожна межа несе «натомість» — як
дістати повний вирок. Жодного походу в мережу: запушеність
судиться за тим, що знає цей клон, — межа, яка залежить від
досяжности сервера, змінювалась би з погодою.

## scenario: the-closing-court-names-its-price

**Дано** `keel close`,
**коли** його кличуть,
**тоді** він каже **перед** роботою, що збудує власний `target`, де
саме і чому (успадкований кеш зсуває вироки — §6.7), а **коли місця
на диску менше за названий поріг — відмовляється на вході**, назвавши
скільки є і скільки треба, замість померти на півдорозі з «no space
left on device». Після роботи він каже, скільки той `target` важить.

## transform: the-verdict-carries-its-own-limits

`check.rs` збирає межі вироку однією рукою і друкує їх **у
підсумку**; `scope.rs` віддає базу порівняння і її вік.

## transform: the-price-said-before-it-is-paid

`close.rs` питає диск перед роботою і називає ціну; `adapter.rs`
лишається як є — рішення про власний `target` хвиля підтверджує, а не
скасовує.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10).

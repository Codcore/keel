---
depends_on: [0053-the-words-and-the-own-ci]

scenarios:
  the-norm-names-what-holds-it:
    covers: [interaction.self-descriptiveness, safety.operational-constraints]
  two-open-waves-do-not-share-a-file:
    covers: [functional.completeness, safety.risk-identification]
  check-says-only-what-it-judged:
    covers: [interaction.user-error-protection, reliability.faultlessness]
  the-root-copy-does-not-drift:
    covers: [functional.correctness, maintainability.modifiability]

transforms:
  the-norm-marks-its-text-held-rules:
    implements:
      - the-norm-names-what-holds-it
    files:
      - docs/uk/METHODOLOGY-V2.md
      - docs/en/METHODOLOGY-V2.md
      - METHODOLOGY.md
      - tool/tests/norm_marks_test.rs
  the-crossing-court:
    implements:
      - two-open-waves-do-not-share-a-file
    files:
      - tool/src/graph.rs
      - tool/src/check.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-graph.md
      - keel/contracts/tool-cli.md
      - tool/tests/crossing_test.rs
      - tool/tests/check_test.rs
  check-counts-what-it-did-not-judge:
    implements:
      - check-says-only-what-it-judged
    files:
      - tool/src/check.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-cli.md
      - tool/tests/checked_line_test.rs
      - tool/tests/adapter_choice_test.rs
  the-root-copy-is-held:
    implements:
      - the-root-copy-does-not-drift
    files:
      - tool/tests/root_copy_test.rs
      - METHODOLOGY.md
  the-concept-and-the-readme-tell-the-truth:
    chore: "the concept's rows about checksum, the price of a tongue and the frame flags say what is measured, and the README carries the operator's decisions 5 and 7 about v1 (§2.10; queue rows NEW-CONCEPT.md:166, :198, :333, :383, :393)"
    files:
      - docs/uk/NEW-CONCEPT.md
      - README.md
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS); the queue's stale rows are struck with the measurement that struck them"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - keel/reviews/0054-the-norm-names-what-holds-it.md
decisions:
  functional.appropriateness: "свідомо без тесту: жодної нової команди — хвиля править текст норми, додає один суд у check і робить чесними його слова; кожен рядок цитує параграф, який тримає (§6.3, §7.1, §7.10, §8.6, §8.8, конституція п. 6)"
  performance.time-behaviour: "свідомо без тесту, і ціна названа: суд перетинів читає шапки вже прочитаних хвиль — O(хвиль × файлів), жодного бігу git чи тестів"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "свідомо без тесту: жодних нових файлів у проєкті користувача — змінюються слова check-а, тексти норми цього репозиторію і його README"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "не застосовується: команд не додається"
  interaction.learnability: "тримає the-norm-names-what-holds-it: правило, яке тримає лише текст, позначене як таке в самому тексті — читач бачить, на чому воно стоїть (конституція, п. 6)"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "тримає the-norm-names-what-holds-it: позначки і новий параграф — обома мовами, і англійський текст записує редакцію українського (translated_from), тож переклад не відстає мовчки"
  interaction.user-assistance: "тримає two-open-waves-do-not-share-a-file: знахідка каже, що зробити натомість — назвати depends_on або поділити файл між хвилями (§8.8)"
  reliability.fault-tolerance: "не застосовується"
  reliability.availability: "не застосовується"
  reliability.recoverability: "свідомо без тесту, і сказано: §6.3-б описує відкат злитої хвилі як нову хвилю з superseded_by і поверненням коду під її слагом — суди вже читають superseded_by, нової механіки не додається; текстове правило, і воно так позначене"
  security.confidentiality: "не застосовується"
  security.integrity: "не застосовується"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "свідомо без тесту: суд перетинів живе в graph поруч із іншими судами між хвилями; жодного нового модуля"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "тримає two-open-waves-do-not-share-a-file: знахідка називає файл і обидві хвилі поіменно"
  maintainability.testability: "свідомо без тесту: чотири проби народжуються червоними на своїй першій клаузі (§7.12); проби норми й копії читають файли репозиторію, як generated_stands_test"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.fail-safe: "тримає check-says-only-what-it-judged: суд, який не судив, лічиться в «не перевірено», а не замальовується зеленим рядком «що саме перевірено»"
  safety.hazard-warning: "тримає check-says-only-what-it-judged: підсумок називає число несуджених речей, і воно дорівнює рядкам «не перевірено» над ним"
  safety.safe-integration: "свідомо без тесту, і названо: реліз 1.0.0 (рішення оператора 2026-09-06) іде окремою легкою хвилею після цієї — крейт і пін в одному коміті, бо суд піна порівнює бінарник, зібраний із дерева, з піном keel.toml, і коміт із самою версією зробив би власний CI червоним"
---

## Why

Рішення оператора 2026-09-06, поставлені через інструмент після
злиття хвиль 0038–0053, і хвіст рядків BACKLOG, які ці рішення
знімають. Кожен рядок зміряно на дереві fe2533c перед планом.

**Зміряно перед планом.**

**Норма не каже, хто її тримає.** Конституція (п. 6) вимагає: правило,
яке живе лише в тексті, чесно позначене як таке. У тексті норми обома
мовами жодної позначки «текстове» нема (grep — 0). §7.1 каже «кожне
посилання в тексті — свій файл», §7.10 — «жодна перевірка не парсить
прозу»: машина не може тримати першого, не порушивши другого, і не
тримає — а текст мовчить, на чому тримається §7.1 для прози. §8.6
вимагає від commit-у плану абзацу «питання, варіанти, відповідь, хто
вирішив» — машина його не судить і не може без парсингу прози, а
позначки нема. §6.3-а закінчується словами «відкат злитої хвилі і
швидкий шлях для термінового фікса цим параграфом не покриті»; швидкий
фікс є — легка хвиля §6.8 працює, — а відкат злитої хвилі не описаний
ніде (рішення оператора 2026-09-04: «чекають»; 2026-09-06: описати).
Рішення оператора 2026-09-06: §7.1 для прози — текстове; §8.6 —
текстове; відкат злитої хвилі — нова хвиля з superseded_by.

**Дві відкриті хвилі ділять файл мовчки.** §8.8: дві хвилі йдуть
паралельно, «якщо між ними нема ребра depends_on і їхні scope не
перетинаються; перетин файлів між незалежними хвилями — питання ще на
плануванні». Пісочниця: `0001-first` і `0002-second`, обидві відкриті,
обидві оголошують `src/lib.rs`, depends_on нема, план-гілка другої —
`keel check`: «2 документи, 0 знахідок». Концепт і README після хвилі
0053 кажуть, що суду нема; рішення оператора 2026-09-06 — суд потрібен,
знахідкою в `keel check`.

**Check каже «перевірено» про те, чого не судив.** Пісочниця без git і
без адаптера, одна chore-хвиля: `keel check` друкує «теги тестів не
звірялись: adapter не названий» і «scope не звірявся: git не дає
гілки», а нижче — статичний рядок «що саме перевірено: … scope гілки
… редакції сценаріїв у тегах тестів … довіра команд verify/ci …» і
підсумок «1 документ, 0 знахідок» без жодного «не перевірено» в лічбі:
`check-checked` — стала, а лічба `limits` бере лише `verdict_limits`
(shallow, застаріла база, hook) і не бачить рядків стану судів. Рядок
BACKLOG (`check.rs:745/:732`) казав «рахує 2, назвавши чотири» — на
цьому дереві лічба каже «нічого», назвавши два.

**Копію методики в корені ніхто не тримає.** `METHODOLOGY.md` у корені
— копія `docs/en/METHODOLOGY-V2.md` з власною передмовою: тіла від
першого розділу збігаються (diff від 16-го рядка — 7 рядків передмови,
0 у тілі), і жоден суд цього не каже; перед тим (2026-09-04) там
лежав текст v1, застарілий на пʼять місяців. Рішення оператора
2026-09-06 — суд копії потрібен.

**Концепт і README кажуть не те, що зміряно.** `checksum опційно` у
прикладі keel.toml (NEW-CONCEPT:166) — парсер відмовляє на зайве поле
(`deny_unknown_fields`), і checksum-и релізів перевіряє launcher сам;
«додати мову = покласти один перекладений файл» (:198) — рецензія 0038
R-11 зміряла шість місць коду; «`-C` і `--branch` з концепту не
існують» у BACKLOG — застаріле: обидва прапорці живуть із хвилі 0040
(`main-usage`), а README не називає `KEEL_BRANCH`, яку ставить
згенерований CI; рішення оператора №5 і №7 (:383, :393): README не
каже «v1 заморожено — останній реліз v0.8.11, далі лише критичні
латки» і не показує на гілку-архів `skarha-nazyvaye-prychynu` (вона
є на origin). Рядки BACKLOG «v2 досі чернетка» і «внутрішня нотатка про
щабель у чужому проєкті» — застарілі: позначку знято 2026-09-04
(обидва тексти кажуть це в передмові), `check-next-rung` каже «план
наступної хвилі (§6.6)» без слова про щабель keel-а.

**Що стоїть за цією хвилею (рішення оператора 2026-09-06).** Реліз
1.0.0 — окрема легка хвиля після цієї: крейт і пін в одному коміті
(суд піна порівнює бінарник із дерева з піном; коміт із самою версією
зробив би власний CI червоним), теґ `v1.0.0` на злитий коміт пушить
агент, workflow збирає й публікує. Типізовані вироки `--json` і
структура пакета рецензента — ще одна хвиля після релізу.

**Дрейф (§4.6), названий уголос.** У третю трансформу дописано
`tool/tests/adapter_choice_test.rs`: проба 0038 тримала «1 річ не
перевірено» над проєктом без адаптера, і трималась лише тим, що поруч
стояла інша межа, — тепер стояння суду тегів осторонь саме лічиться,
і проба тримає рівність числа рядкам «не перевірено». У четверту
дописано `METHODOLOGY.md`: передмова копії називає пробу, яка тримає
її тіло, — рядок, без якого читач копії не знав би, що її хтось
судить (тіло копії й далі йде за en у першій трансформі). У другу
дописано `tool/tests/check_test.rs`: три проби хвиль 0001–0005 ділили
один файл між хвилями своїх пісочниць (`a`, `lib/a.ex`) — не за
задумом, а за зручністю — і суд перетинів їх червонить; тепер кожна
хвиля пісочниці має свій файл; а проба «звіт каже, що перевірив»
тримала статичний рядок «test tags» без адаптера — тепер тримає
стояння суду тегів осторонь як лічений рядок.

## scenario: the-norm-names-what-holds-it

**Дано** обидва тексти норми (uk — джерело, en — переклад із
`translated_from`) і копію в корені.
**Коли** проба читає їх, і біжить `keel check` цього репозиторію.
**Тоді** §7.1 несе позначку, що його друга половина — посилання в
прозі — текстове правило (тримає рецензент за §9.9, не машина); §8.6
несе таку саму позначку; §6.3-б каже, що відкат злитої хвилі — нова
хвиля, яка знімає обіцянки старої через superseded_by і повертає код
комітами під своїм слагом, і швидкий фікс — легка хвиля §6.8; усе
обома мовами, і `translated_from` англійського тексту дорівнює
редакції українського, тож `keel check` не каже «застарів».

## scenario: two-open-waves-do-not-share-a-file

**Дано** проєкт із двома відкритими хвилями, що оголошують один файл
без ребра depends_on між ними; ті самі дві з ребром; закриту і
відкриту з одним файлом; скасовану і відкриту.
**Коли** біжить `keel check` — на план-гілці, на робочій гілці і на
main.
**Тоді** перша пара — знахідка з іменем файлу й обох хвиль і словом
натомість «назви depends_on або поділи файл» (§8.8); пара з ребром,
пара з закритою і пара зі скасованою — без знахідки.

## scenario: check-says-only-what-it-judged

**Дано** проєкт без git і без адаптера; проєкт із git і адаптером.
**Коли** біжить `keel check`.
**Тоді** у першому суди scope і тегів кажуть «не судив» і лічаться в
підсумку («N речей не перевірено» дорівнює рядкам «не перевірено» над
ним), а рядок «що саме перевірено» називає лише суди, які бігли, — без
scope, тегів і довіри; у другому рядок називає їх.

## scenario: the-root-copy-does-not-drift

**Дано** `METHODOLOGY.md` у корені і `docs/en/METHODOLOGY-V2.md`.
**Коли** біжить проба.
**Тоді** тіла обох від першого розділу («## …») збігаються байт у байт;
передмови можуть різнитись; розбіжність у тілі — червона проба.

## transform: the-norm-marks-its-text-held-rules

`docs/uk/METHODOLOGY-V2.md`: §7.1 і §8.6 дістають позначку «текстове —
тримає рецензент (§9.9), не машина (конституція, п. 6)», §6.3-а
втрачає останнє речення, §6.3-б описує відкат злитої хвилі.
`docs/en/METHODOLOGY-V2.md` — те саме англійською, `translated_from`
переписано на нову редакцію. `METHODOLOGY.md` — тіло йде за en. Проба.

## transform: the-crossing-court

`graph.rs`: `crossing_findings` над відкритими хвилями — файл,
оголошений двома без ребра depends_on, — трійка «хвиля, причина,
натомість»; `check.rs` дає йому відкриті (структурно не закриті і не
скасовані) хвилі. Слова обома мовами. Контракти graph і cli. Проба.

## transform: check-counts-what-it-did-not-judge

`check.rs`: рядки стану судів (scope, теги, довіра, форма) несуть, чи
судили; несуджені лічаться в `limits`, а рядок «що саме перевірено»
складається з судів, які бігли. Слова обома мовами. Контракт cli.
Проба.

## transform: the-root-copy-is-held

Проба `root_copy_test.rs`: тіло кореневої копії від першого розділу
дорівнює тілу `docs/en/METHODOLOGY-V2.md`.

## transform: the-concept-and-the-readme-tell-the-truth

`NEW-CONCEPT.md`: приклад keel.toml без «checksum опційно» (checksum-и
релізів перевіряє launcher, поля нема, парсер відмовляє на зайве);
ціна мови — зміряна (модуль, рядок мов і шість місць диспетчера);
рядок про `-C`/`--branch` називає `KEEL_BRANCH`. `README.md`: v1
заморожено — останній реліз v0.8.11, далі лише критичні латки, гілка
`skarha-nazyvaye-prychynu` — архів розбору; таблиця відповідності
команд v1 → v2; `KEEL_BRANCH` названо поруч із `--branch`.

## transform: journal

Записи журналу хвилі (V2-PROCESS), рядки BACKLOG, зняті рішеннями
2026-09-06 і замірами перед планом (чернетка знята 2026-09-04;
`-C`/`--branch` живуть із 0040; нотатки про щабель нема), і файл
рецензії.

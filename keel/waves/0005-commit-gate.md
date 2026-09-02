---
depends_on: [0004-scope-and-links]

scenarios:
  stale-tag-found:
    proves: tool-tags@ffb8af
    covers: [functional.correctness]
  old-revision-legal-when-historic:
    covers: [security.authenticity]
  red-commit-needs-failing-test:
    proves: tool-gate@ef42fc
    covers: [safety.operational-constraints, security.integrity]
  work-commit-needs-green:
    proves: tool-gate@ef42fc
    covers: [functional.completeness]
  build-break-is-not-red:
    proves: tool-adapter-cargo@77e38c
    covers: [reliability.faultlessness, interaction.user-error-protection]
  gate-modes-obeyed:
    proves: tool-config@2b1bf3
    covers: [interaction.operability, interaction.self-descriptiveness]
  hook-installed-aloud:
    proves: tool-gate@ef42fc
    covers: [flexibility.installability, compatibility.co-existence, safety.safe-integration]

transforms:
  tag-floor:
    implements:
      - stale-tag-found
      - old-revision-legal-when-historic
    contracts: [tool-tags@ffb8af, tool-adapter-cargo@77e38c, tool-rev@882dea, tool-docs@2ab9a9]
    files:
      - tool/src/tags.rs
      - tool/src/adapter.rs
      - tool/src/check.rs
      - tool/src/lib.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/tags_test.rs
      - tool/tests/check_test.rs
      - keel.toml
  gate-command:
    implements:
      - red-commit-needs-failing-test
      - work-commit-needs-green
      - build-break-is-not-red
      - gate-modes-obeyed
    contracts: [tool-gate@ef42fc, tool-adapter-cargo@77e38c, tool-tags@ffb8af, tool-config@2b1bf3, tool-scope@b8ada4, tool-rev@882dea, tool-docs@2ab9a9]
    files:
      - tool/src/gate.rs
      - tool/src/adapter.rs
      - tool/src/config.rs
      - tool/src/main.rs
      - tool/src/lib.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/gate_test.rs
      - tool/tests/check_test.rs
  hook-install:
    implements:
      - hook-installed-aloud
    contracts: [tool-gate@ef42fc]
    files:
      - tool/src/gate.rs
      - tool/src/main.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/gate_test.rs

decisions:
  functional.appropriateness: "свідомо без тесту: доречність суду commit-ів судитиме перша ж хвиля, що піде під hook-ом, — наступна"
  performance.time-behaviour: "свідомо не міряємо: gate жене рівно тести названих сценаріїв, не всю батарею; вимір прийде, коли заболить"
  performance.capacity: "не застосовується: сценаріїв у трансформі — одиниці"
  performance.resource-utilisation: "свідомо не міряємо: один процес cargo на біг gate"
  compatibility.interoperability: "свідомо без окремого тесту: зовнішні домовленості — git CLI (доведено 0004) і cargo CLI; словник наслідків cargo тримає сценарій build-break-is-not-red"
  interaction.appropriateness-recognisability: "свідомо не робимо: пізнаваність команд — CLI-хвиля (--json, next)"
  interaction.learnability: "свідомо без тесту: нові команди входять у рядок usage; довідники — скіл-хвилею"
  interaction.user-assistance: "свідомо без нового тесту: кожна відмова несе «натомість» — школа чотирьох хвиль; нові ключі йдуть тим самим шляхом"
  interaction.user-engagement: "не застосовується: інструмент перевірки"
  interaction.inclusivity: "свідомо без нового тесту: всі нові тексти — ключами через i18n, доведеними в 0002"
  reliability.availability: "не застосовується: локальний бінарник"
  reliability.fault-tolerance: "свідомо без окремого тесту: зіпсований документ не валить gate — тримає scan (0004); зламаний cargo — відмова вголос, не падіння (сценарій build-break-is-not-red)"
  reliability.recoverability: "не застосовується: стану нема; hook перевстановлюється тим самим викликом"
  security.confidentiality: "не застосовується: читає репозиторій, жене локальні тести, нікуди не шле"
  security.non-repudiation: "свідомо без тесту цим щаблем: «бачив червоним» стає фактом історії гілки — невідмовність дає git; судити повноту цієї історії буде закриття (§6.5)"
  security.accountability: "не застосовується: єдина дія зі станом — файл hook, і його поява видна в git status"
  security.resistance: "свідомо не робимо фаззингу: повідомлення commit-а — рядок; слаг-подібний початок звіряється зі словником шапки, невідомий — відмова (§8.4)"
  maintainability.modularity: "свідомо без тесту: tags не знає cargo, adapter не знає docs, gate складає обидва через contracts — межі по журналу А3"
  maintainability.reusability: "не застосовується: внутрішні модулі; адаптери інших мов прийдуть своїми хвилями поруч із adapter.rs"
  maintainability.analysability: "свідомо без нового тесту: кожна відмова називає сценарій, тест і файл поіменно — це assert-иться у сценаріях хвилі"
  maintainability.modifiability: "свідомо без тесту: adapter — одна функція «запусти і класифікуй»; режими — три гілки match"
  maintainability.testability: "свідомо без окремого тесту: gate-тести будують справжні cargo-проєкти з git-ом у пісочницях — та сама школа, що git-пісочниці 0004"
  flexibility.adaptability: "свідомо: адаптер лише cargo цим щаблем; Elixir/Ruby/Python/TS — окремі хвилі (NEW-CONCEPT); край А3 «мова не збирає тест без модуля» — їхній, не Rust-овий"
  flexibility.scalability: "не застосовується: обсяги малі"
  flexibility.replaceability: "свідомо без тесту: hook — плоский sh-файл, що кличе keel gate; заміна — перезапис тим самим keel hook"
  safety.risk-identification: "не застосовується як окрема робота: головні загрози щабля — фальшиве «бачив червоним» і тихий пропуск суду — і є сценаріями хвилі"
  safety.fail-safe: "свідомо без нового тесту: невідомість (гілка не хвиля, тега нема, cargo зламаний) веде до відмови або слова, не до тихого пропуску — тримають build-break-is-not-red і gate-modes-obeyed"
  safety.hazard-warning: "свідомо без нового тесту: режим soft — і є попередження до шкоди; тримає gate-modes-obeyed"
---

## Why

Щабель 4 самонаведення (журнал А3): «бачив червоним» перестає бути
словом автора в повідомленні commit-а і стає судом машини. Дотепер я
запускав тест руками, дивився на падіння і писав `red: <сценарій>` —
віри цьому запису рівно стільки, скільки моїй дисципліні. Тепер
commit-msg hook віддає повідомлення команді `keel gate`: заявлене
народження перевіряється справжнім бігом тесту (впав — пропуск,
зелений — відмова, «не зібрався» — не «впав»), а commit роботи
проходить, лише коли тести всіх implements-сценаріїв зелені зі
збіжними редакціями тегів. Floor тегів у `keel check` знімає з моїх
рук і звірку §5.5/§7.5: застарілий тег видно без мене. А3 каже
«pre-commit hook» — механіка чесніша словом commit-msg: саме цей hook
дістає файл повідомлення.

Разом floor посилань вчиться §5.6 по-справжньому: стара редакція
контракту в закритій хвилі — законна, якщо цей текст справді жив у
git-історії файлу; вигадана — знахідка, як досі. Без цього кожна
зміна контракту фарбувала б закриті хвилі в брехливе червоне.

Відступи bootstrap-у, названі вголос: хвиля їде робочою гілкою сесії;
план затверджується словом оператора в чаті (§8.6). План міняє текст
tool-config (поле mode): golden-пін у rev_test оновлює сам план (тест
тримає живий контракт — інакше зелена батарея бреше), а посилання
закритих хвиль 0002–0004 на tool-config@63406a стоять застарілими у
звіті check на час першої трансформи і зцілюються нею ж
(§5.6-floor); почервоніння звіту чесне, тимчасове і назване тут. Gate судить робоче дерево, а не
staged-знімок, — різниця можлива при частковому add і названа тут;
чесніший знімок — щабель попереду. І слово «брама» з рядка
«наступний крок» (моя вигадка всупереч правилу «індустрійні терміни
не перекладаються») зникає цією ж хвилею: hook зветься hook.

## scenario: stale-tag-found

**Дано** тест із тегом `proves: <сценарій>@<редакція>`, де редакція
не збігається з чинною редакцією сценарію, або сценарію з таким
слагом нема в жодній хвилі,
**коли** біжить `keel check`,
**тоді** кожне — знахідка поіменно (файл тесту, сценарій, записана і
чинна редакції; §5.5, §7.5), а збіжні теги пораховані у звіті рядком
«тегів звірено: N» — і рядок «ще не перевірено» більше не називає
теги. Теги знятих сценаріїв не судяться — обіцянка померла (§2.12).

## scenario: old-revision-legal-when-historic

**Дано** посилання хвилі на контракт зі старою редакцією, чий текст
справді жив у git-історії файлу контракту,
**коли** біжить `keel check`,
**тоді** це зелене зі словом «стара редакція, справжня в історії»
(§5.6) — а редакція, якої в історії файлу не було ніколи, лишається
знахідкою, як досі. Де історії нема або вона обрізана (shallow-клон,
як у CI) — вирок не виноситься: слово «законність не звірити,
історія неповна» замість знахідки, бо відсутність історії — не
провина хвилі.

## scenario: red-commit-needs-failing-test

**Дано** гілку, що зветься як хвиля, і повідомлення commit-а
`red: <сценарій>`,
**коли** біжить `keel gate` з файлом цього повідомлення,
**тоді** пропуск є лише тоді, коли тест сценарію знайдений тегом
proves і справді впав; зелений тест — відмова вголос: незароблене
«бачив червоним» не вʼїжджає в історію (§7.12, А3). `red:` на знятий
сценарій — відмова (§2.12: мертва обіцянка не народжується); кілька
тегів одного сценарію при народженні — відмова «скажи, котрий
народжується» (при роботі женуться всі).

## scenario: work-commit-needs-green

**Дано** повідомлення commit-а, що починається зі слагу трансформи
хвилі,
**коли** біжить `keel gate`,
**тоді** пропуск є лише тоді, коли для кожного implements-сценарію
знайдено тег зі збіжною редакцією і його тест зелений; червоний або
незнайдений — відмова, що називає сценарій (§8.4). Слаг-подібний
початок, невідомий хвилі, — відмова: одрук не читається як «поза
судом». Merge-повідомлення і записи рішень — пропуск зі словом.

## scenario: build-break-is-not-red

**Дано** `red: <сценарій>`, чий тест не компілюється, і `red:` на
тест, якого біг не виконує (одрук в імені — нуль виконаних),
**коли** біжить `keel gate`,
**тоді** обидва — відмови з причиною і словами cargo: «не зібрався»
і «не виконався» не читаються як «впав» (журнал А3).

## scenario: gate-modes-obeyed

**Дано** ті самі порушення при `mode = "strict"`, `"soft"`,
`"manual"` і без поля mode,
**коли** біжить `keel gate`,
**тоді** strict заслоняє commit кодом 1; soft каже ті самі слова
кодом 0; manual каже «суд вимкнено» і пропускає; відсутнє поле — як
strict, і звіт каже «mode: strict (типове)» — типове не видає себе за
прочитане. Гілка, що не зветься хвилею, — пропуск зі словом у
кожному режимі.

## scenario: hook-installed-aloud

**Дано** репозиторій без hook-а, той самий репозиторій із нашим же
hook-ом і репозиторій із чужим commit-msg,
**коли** біжить `keel hook`,
**тоді** перший дістає виконуваний `.git/hooks/commit-msg`, що кличе
`keel gate`; другий — тихо той самий файл (ідемпотентно); третій —
відмову вголос, і чужий файл лишається неторканим (§9.7).

## transform: tag-floor

Модуль `tags` (розбір тегів proves у файлах тестів, які називає
адаптер) і перші дві функції адаптера cargo (`crate_root`,
`test_files`). `check` дістає floor тегів: застарілі і сирітські теги
— знахідки, збіжні — пораховані. Floor посилань вчиться §5.6:
застаріла редакція звіряється з git-історією файлу контракту
(приватний помічник check, git як команда системи — школа 0004);
справжня в історії — зелена зі словом, вигадана — знахідка.

## transform: gate-command

Модуль `gate` (суд повідомлення проти хвилі гілки через tool-scope) і
решта адаптера (`run_test` з класифікацією Failed / Green /
BuildBroken / NotRun). `config` дістає поле mode зі строгим словником
і strict-типовим. `main` дістає команду `keel gate <файл-повідомлення>
[тека]`. Тести будують справжні cargo-проєкти з git-ом у пісочницях і
женуть gate крізь бінарник. Рядок «наступний крок» обох ftl
переписується без вигаданих слів: «щабель 5 — закриття хвиль (§6.5)».

## transform: hook-install

`gate` дістає `install_hook`; `main` — команду `keel hook [тека]`.
Ідемпотентність і незатирання чужого — по сценарію. Dogfood чесний:
на гілці bootstrap-у hook скаже «гілка не зветься хвилею — пропуск зі
словом», і це видно в звіті gate.

Застереження: gate судить робоче дерево (не staged-знімок) — названо
у Why; суд «рівно однієї» відповіді §10.3, дельта тегів у точці
розгалуження (§7.15 — «тег був і зник») і закриття (§6.5) — щаблі
попереду, рядок «ще не перевірено» їх називає далі.

Застереження після рецензії, вголос: §5.6-благословення історією
поки не відрізняє відкриту хвилю від закритої — закриття (§6.5)
звузить його, і поіменний список historic-редакцій — теж його
щабель; редакція тега при `red:`-народженні не судиться — її тримають
floor тегів і суд роботи; exports tool-docs вужчі за фактичну
поверхню (Wave, Transform — ними користується gate) — борг, названий
щаблю тримання контрактів (§7.6).

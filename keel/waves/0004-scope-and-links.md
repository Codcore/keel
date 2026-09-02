---
depends_on: [0003-revisions]

scenarios:
  unknown-cut-refused:
    proves: tool-graph@31feb7
    covers: [interaction.user-error-protection]
  silence-forbidden:
    proves: tool-graph@31feb7
    covers: [functional.completeness, safety.hazard-warning]
  broken-links-named:
    proves: tool-graph@31feb7
    covers: [reliability.faultlessness]
  scope-both-ways:
    proves: tool-scope@14384c
    covers: [functional.correctness, security.integrity]
  one-new-in-counted:
    proves: tool-scope@14384c
    covers: [safety.operational-constraints]
  scope-honest-when-unknown:
    proves: tool-scope@14384c
    covers: [interaction.self-descriptiveness, safety.fail-safe]

transforms:
  graph-checks:
    implements:
      - unknown-cut-refused
      - silence-forbidden
      - broken-links-named
    contracts: [tool-graph@31feb7, tool-docs@2ab9a9]
    files:
      - tool/src/graph.rs
      - tool/src/check.rs
      - tool/src/lib.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/graph_test.rs
      - tool/tests/check_test.rs
  scope-checks:
    implements:
      - scope-both-ways
      - one-new-in-counted
      - scope-honest-when-unknown
    contracts: [tool-scope@14384c, tool-docs@2ab9a9, tool-config@63406a]
    files:
      - tool/src/scope.rs
      - tool/src/check.rs
      - tool/src/lib.rs
      - tool/i18n/en.ftl
      - tool/i18n/uk.ftl
      - tool/tests/scope_test.rs
      - tool/tests/check_test.rs

decisions:
  functional.appropriateness: "свідомо без тесту: доречність судять хвилі, що підуть під цими перевірками, — першою ця сама"
  performance.time-behaviour: "свідомо не міряємо: один git diff і прохід по шапках; вимір прийде з hook-хвилею"
  performance.capacity: "не застосовується: хвиль і файлів у scope — десятки"
  performance.resource-utilisation: "свідомо не міряємо: разовий виклик git і память на списки імен"
  compatibility.co-existence: "не застосовується: читає, питає git, виходить — нічого не тримає"
  compatibility.interoperability: "свідомо без окремого тесту: єдина зовнішня домовленість — git CLI, і його відмова — відмова вголос (сценарії scope живуть на справжніх git-репо в пісочницях)"
  interaction.appropriateness-recognisability: "свідомо не робимо: пізнаваність команд — CLI-хвиля; нові знахідки їдуть у звичному звіті check"
  interaction.learnability: "не застосовується: нових команд і опцій хвиля не додає"
  interaction.operability: "свідомо не робимо: керування (--wave для явного вибору хвилі) — CLI-хвилею, поки чесне «не звірявся» покриває відступ bootstrap-у"
  interaction.user-assistance: "свідомо без нового тесту: кожна знахідка несе «натомість» — школа, доведена трьома хвилями; нові ключі йдуть тим самим шляхом"
  interaction.user-engagement: "не застосовується: інструмент перевірки"
  interaction.inclusivity: "свідомо без нового тесту: всі нові тексти — ключами через i18n, доведеними в 0002"
  reliability.fault-tolerance: "свідомо без нового тесту: зіпсовані документи не валять сусідів — тримає scan; graph і scope дістають лише прочитані хвилі"
  reliability.availability: "не застосовується: локальний бінарник"
  reliability.recoverability: "не застосовується: стану нема"
  security.confidentiality: "не застосовується: читає репозиторій і git-метадані того ж репозиторію, нікуди не шле"
  security.non-repudiation: "не застосовується: дій зі станом нема"
  security.accountability: "не застосовується: те саме"
  security.authenticity: "не застосовується: нікого не автентифікує; git кличеться як команда системи, не з файлів репо — TOFU (§7.16) не потрібен"
  security.resistance: "свідомо не робимо фаззингу: вхід — шапки після суворого розбору docs і вивід git; зловмисні імена файлів — просто рядки для порівняння"
  maintainability.modularity: "свідомо без тесту: graph без диска і git, scope без словників — межі модулів рівно по главах 3 і 4"
  maintainability.reusability: "не застосовується: внутрішні модулі"
  maintainability.analysability: "свідомо без нового тесту: кожна знахідка називає слаг/файл/хвилю поіменно — це і assert-иться в сценаріях"
  maintainability.modifiability: "свідомо без тесту: словник — один масив, перевірки — короткі функції"
  maintainability.testability: "свідомо без окремого тесту: механіка червоних народжень та сама; scope-тести будують справжні git-репо в пісочницях"
  flexibility.adaptability: "свідомо не робимо: git-виклик однаковий на підтримуваних платформах; CI-матриця — хвилею релізів"
  flexibility.scalability: "не застосовується: обсяги малі"
  flexibility.installability: "не застосовується: launcher-щабель; git уже є всюди, де є методика — вона живе в git за конституцією п.8"
  flexibility.replaceability: "свідомо без тесту: вшитий словник міняється лише релізом — це навмисно: розрізи і інструмент версіонуються разом, і зміна словника — зміна методики"
  safety.risk-identification: "не застосовується як окрема робота: головні загрози щабля — тихий дрейф і тиха тиша — і є самими сценаріями хвилі"
  safety.safe-integration: "свідомо без тесту: нові файли — graph.rs, scope.rs і їхні тести; check.rs росте лише новими поверхами; на нашому ж репо новий scope-поверх чесно скаже «не звірявся» (гілка — не хвиля), нічого не зламавши"
---

## Why

Щабель 3 самонаведення: «файли називаються до роботи» і звʼязки графа
відтепер тримає машина. Дотепер scope і повнота відповідей по
розрізах звірялись моїми руками у кожному чеку — тепер це поверхи
check. Словник сорока розрізів вшивається в бінарник: методика й
інструмент версіонуються разом (рішення оператора), тож слаг — частина
релізу, а не файл, який можна тихо підправити. Scope звіряється з
гілкою через git: гілка, що зветься як хвиля (§8.2), дістає порівняння
в обидва боки (§4.4) з чесним винятком для keel/ (§4.8) і строгим
рахунком one new in (§4.1); гілка, що хвилею не зветься — як наша
робоча гілка bootstrap-у, — дістає чесний рядок «scope не звірявся»
замість тиші або брехливого зеленого.

Відступ bootstrap-у, названий вголос: хвиля їде робочою гілкою сесії;
план затверджується словом оператора в чаті, записаним у commit
(§8.6). Звіт рецензента ляже в keel/reviews/0004-scope-and-links.md.

## scenario: unknown-cut-refused

**Дано** хвилю зі слагом у covers або decisions, якого у вшитому
словнику нема,
**коли** біжить `keel check`,
**тоді** це знахідка, що називає чужий слаг і хвилю (§3.4) — одрук у
розрізі не читається як «нова відповідь».

## scenario: silence-forbidden

**Дано** хвилю, де хоч один із сорока розрізів не має відповіді ані в
чиємусь covers, ані в decisions,
**коли** біжить `keel check`,
**тоді** це знахідка з переліком пропущених розрізів (§10.3): тиша
заборонена на рівні поля, і план без відповіді — неповний план.
Covers знятого сценарію відповіддю не рахуються — обіцянка померла
(§2.12), і розріз, який тримався лише нею, знову без відповіді.

## scenario: broken-links-named

**Дано** implements на сценарій, якого нема; superseded_by у нікуди;
depends_on на хвилю, якої нема; і цикл depends_on,
**коли** біжить `keel check`,
**тоді** кожне — знахідка з іменами (§7.1, §7.2), а не тихий пропуск
і не падіння.

## scenario: scope-both-ways

**Дано** git-гілку, що зветься як хвиля, з commit-ами, які і чіпають
файл поза оголошеними, і лишають оголошений файл неторканим,
**коли** біжить `keel check` на цій гілці,
**тоді** обидва — знахідки з іменами файлів (§4.4): дрейф видно, і
недороблене видно; файли теки keel/ у порівняння не входять (§4.8).

## scenario: one-new-in-counted

**Дано** трансформу з рядком `one new in <тека>/` і гілки, де в тій
теці зʼявилось нуль, один і два нові файли,
**коли** біжить `keel check`,
**тоді** нуль — знахідка, два — знахідка з іменами обох, рівно
один — тихо (§4.1): кількість зафіксована, glob-вільниці нема.

## scenario: scope-honest-when-unknown

**Дано** гілку, що не зветься як жодна хвиля (як робоча гілка цього
bootstrap-у), або теку без git,
**коли** біжить `keel check`,
**тоді** звіт каже вголос «scope не звірявся» з причиною — і це не
знахідка і не червоне: відступ названий, зелене про незвірене не
брешеться.

## transform: graph-checks

Застереження: цей поверх судить усі хвилі як v2 — міграційне
помʼякшення Додатка В («старі хвилі v1 заднім числом не судяться»)
прийде щаблем міграції разом із полем version у keel/keel-стані; наш
проєкт чистий v2, тож тут це чесна межа, а не діра.

Модуль `graph` (глава 3): вшитий словник сорока, суд хвилі зсередини
(слаги, тиша §10.3, implements, superseded_by) і між хвилями
(depends_on існує, циклів нема). `check` дістає graph-поверх; рядки
«перевірено/ще не перевірено» чесно переписуються.

## transform: scope-checks

Модуль `scope` (глава 4): гілка → хвиля (§8.2), порівняння файлів
гілки з обʼєднанням files трансформ в обидва боки (§4.4, по гілці за
§4.5), виняток keel/ (§4.8), строгий рахунок one new in (§4.1).
`check` дістає scope-поверх; тести будують справжні git-репозиторії в
пісочницях.

Застереження: точка порівняння — merge-base з main; де main-а нема
(голий тест-репозиторій), береться перший commit гілки, і звіт каже,
що взяв. Дельта тегів (§7.15) і закриття (§6.5) — щаблі попереду, і
рядок «ще не перевірено» це далі називає.

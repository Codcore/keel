---
depends_on: [0060-the-second-release]

scenarios:
  the-court-asks-git-once-for-the-same-answer:
    covers: [performance.time-behaviour, maintainability.testability]

transforms:
  the-git-hand-remembers:
    implements: [the-court-asks-git-once-for-the-same-answer]
    files:
      - tool/src/scope.rs
      - tool/src/check.rs
      - tool/tests/git_memory_test.rs
      - keel/contracts/tool-scope.md
decisions:
  functional.completeness: "зміряно перед планом на цьому дереві: 4604 виклики git за один `keel check`, з них унікальних 387 — тобто 92% повторів; 3546 із них `show`, і той самий `show <коміт>:tool-config.md` повторюється 72 рази. Хвиля бере рівно ці два джерела: вміст файлу на коміті і список комітів файлу"
  functional.correctness: "тримає ця хвиля: памʼять не міняє ЖОДНОЇ відповіді — вона лише не питає вдруге того, що git уже сказав; вироки суду до і після мусять збігатися дослівно, і це міряє батарея"
  interaction.user-error-protection: "не застосовується"
  functional.appropriateness: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "названо: кеш живе в памʼяті процесу і вмирає з ним — keel це CLI, один процес на запуск. Ціна памʼяті — вміст тих файлів, які суд і так читав, тільки по одному разу замість десятків"
  compatibility.co-existence: "тримає: кеш ключується коренем дерева разом зі шляхом, бо в одному процесі проба веде кілька пісочниць — без кореня відповідь одного дерева поїхала б у суд іншого"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "не застосовується"
  interaction.learnability: "не застосовується"
  interaction.operability: "тримає ця хвиля: суд, що триває хвилину, людина жене рідше, ніж треба — а keel просить його на кожен крок (§9.2)"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "не застосовується"
  interaction.self-descriptiveness: "не застосовується"
  reliability.faultlessness: "названо межу: кеш вірний рівно доти, доки git відповідає те саме — для вмісту коміта і для історії файлу це так за побудовою (коміт незмінний). Робоче дерево під час одного запуску не перечитується і доти, тож нового вікна це не відкриває"
  reliability.fault-tolerance: "не застосовується"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "не застосовується"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "тримає: памʼять живе в одному місці — руці, що кличе git, — а не в кожного суду свою"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "не застосовується"
  maintainability.modifiability: "не застосовується"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "не застосовується"
  safety.fail-safe: "не застосовується"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "не застосовується"
---

## Why

**Інструмент, який триває хвилину, жене рідше, ніж треба.** Рішення
оператора 2026-09-07: «швидкість понад усе — інструмент не потрібен
нікому, якщо він сповільнює роботу». Зміряно автором на цьому дереві
перед планом:

```
keel check .        60 с (release і debug однаково: 31 с user + 21 с sys)
викликів git        4604
з них унікальних     387   — 92% повторів
show                3546   — той самий show <коміт>:tool-config.md × 72
rev-parse --git-dir  225   — відповідь, що за запуск не змінюється
один процес git    ~13 мс   → 4604 × 13 мс ≈ 60 с, сходиться
```

Причина названа поіменно: `check::revision_in_history` на КОЖНЕ
посилання на контракт бере `git log` по файлу, а тоді `git show` на
КОЖЕН коміт його історії, доки не знайде збіг. Двадцять хвиль, що
посилаються на один контракт із сімдесятьма комітами історії, — це
тисяча чотириста процесів заради сімдесяти різних відповідей.

**Чого хвиля НЕ робить.** Не паралелить батарею: батарея довга тому,
що довгий інструмент, і три найдорожчі проби (44% часу) просто кличуть
`keel check` над справжнім деревом. Не міняє механіки читання на `git
cat-file --batch` — це інша дорога, і їй потрібні власні проби; рядок
про неї покладено в чергу цією ж хвилею (рецензія 0061 R-6: спершу він
був обіцяний і не покладений).

## scenario: the-court-asks-git-once-for-the-same-answer

**Дано** дерево з історією, де на один контракт посилається кілька
хвиль, **коли** біжить `keel check` під шимом `git`, який рахує
виклики, **тоді** процесів git не більше, ніж різних відповідей, яких
суд потребує: вміст (коміт, файл) питається один раз, історія файлу —
один раз, і `rev-parse` про це дерево — один раз. Той самий суд над
двома різними деревами в одному процесі не плутає їхніх відповідей.

**Чим сценарій закриває `maintainability.testability`** (рецензія 0061
R-7 слушно спитала). Батарея цього проєкту на 44% складається з трьох
проб, які кличуть `keel check` над СПРАВЖНІМ деревом; суд, що тривав
хвилину, робив кожен такий виклик дорогим, а `keel close` жене батарею
тричі (§7.13). Здатність випробувати цей інструмент прямо залежить від
ціни його власного суду — тому сценарій, що ту ціну зміряв і збив,
закриває саме цей розріз, а не лише time-behaviour.

## transform: the-git-hand-remembers

Памʼять живе в руці, що кличе git (`scope`), і ключується КОРЕНЕМ
разом із питанням. Три відповіді, які за один запуск не змінюються:
вміст файлу на коміті, список комітів файлу, і те, що каже `rev-parse`
про саме дерево. `check::revision_in_history` бере історію і вміст
через цю память.

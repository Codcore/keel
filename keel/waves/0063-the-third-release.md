---
depends_on: [0061-the-court-asks-git-once]

transforms:
  the-third-release:
    chore: "версія крейта і пін проєкту стають 1.1.1 одним комітом; теґ v1.1.1 їде на злитий коміт, щоб виправлення launcher-а доїхало до того, хто вже поставив keel (рішення оператора 2026-09-07)"
    files:
      - tool/Cargo.toml
      - tool/Cargo.lock
      - keel.toml
      - README.md
      - keel/reviews/0063-the-third-release.md
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
decisions:
  functional.completeness: "зміряно перед хвилею: у 1.1.1 входять дві хвилі — 0062 (launcher під вузьким PATH) і 0061 (суд питає git один раз). Обидві виправляють поведінку і жодного нового API не додають, тож за семвером це PATCH (рішення оператора 2026-09-07: версії тримати за семвером)"
  functional.correctness: "не застосовується"
  interaction.user-error-protection: "не застосовується"
  functional.appropriateness: "не застосовується"
  performance.time-behaviour: "тримає 0061, і реліз доносить це до людини: keel check над деревом keel — 60 с до, 7.5 с після; у чужому проєкті ціна та сама за природою (процес git на кожне питання)"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "названо повністю (рецензія 0063 R-4): нова версія стає поруч у своєму домі ~/.keel/versions/, а launcher перезаписується — саме тому виправлення й доїжджає. Друга половина, якої бракувало: після релізу збірка гілки і сам реліз відповідають ОДНИМ числом, тож на машині розробника keel два доми відповідають піну 1.1.1 — зміряно: exit 2 і поіменна відмова з порадою пінити ref"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "не застосовується"
  interaction.learnability: "не застосовується"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "названо межу і борг (рецензія 0063 R-2): інструмент НЕ каже людині, що вийшла новіша версія — `keel version` мовчить, `keel update` оновлює згенеровані інтеграції, а не сам інструмент, CHANGELOG у дереві нема, і нотатки релізу workflow пише рядком «keel v<число>». Тобто дорога доставки є, а слова про неї нема; рядок про це стоїть у черзі"
  interaction.self-descriptiveness: "не застосовується"
  reliability.faultlessness: "не застосовується"
  reliability.fault-tolerance: "не застосовується"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "тримає реліз: workflow збирає чотири цілі тим самим release.sh, кладе поруч .sha256 і attestation; launcher звіряє checksum до розпакування"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "названо межу: provenance доводить attestation GitHub і звіряє `gh attestation verify` — launcher цього не робить і каже про це"
  security.resistance: "не застосовується"
  maintainability.modularity: "не застосовується"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "тримає: число версії живе у двох місцях, які тримає суд піна — маніфест крейта і пін проєкту, — і вони рухаються одним комітом; решту, де число могло б застаріти, хвиля 0062 з контракту прибрала зовсім"
  maintainability.modifiability: "не застосовується"
  maintainability.testability: "не застосовується"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "тримає ця хвиля і саме заради цього вона є: launcher, зламаний під вузьким PATH, полагодиться в людини ЛИШЕ перевстановленням — він себе не оновлює (межа, названа хвилею 0062). Поки релізу нема, `sh install.sh 1.1.0` ставить зламаний"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "названо вікно: між merge і пушем теґа пін уже називає 1.1.1, а релізу ще нема — launcher у ньому відмовляє поіменно; закриває вікно пуш теґа одразу за злиттям"
  safety.risk-identification: "не застосовується"
  safety.fail-safe: "не застосовується"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "не застосовується"
---

## Why

**Виправлення, яке не доїхало, — не виправлення.** Хвиля 0062 полагодила
launcher, що падав під вузьким PATH: у того, хто комітить із графічного
клієнта, суд комміту мовчав. Launcher себе не оновлює — він файл, який
пише `install.sh` під час установки, тож людина мусить перевстановити.

**Чим саме реліз тут допомагає, названо точно** (рецензія 0063 R-1
виправила автора). Латка їде в `install.sh`, а не в бінарнику, тож
дорога `curl .../install.sh` доносить її і БЕЗ релізу — зміряно
рецензентом: цей `install.sh`, поставлений навіть із версією 1.1.0,
пише полагоджений launcher, і той біжить під `PATH=/usr/bin` з exit 0.
Реліз потрібен для іншого: **пін-версія обслуговує лише опублікований
теґ**. Поки v1.1.1 нема, `sh install.sh 1.1.1` відмовляє поіменно, а
проєкт, що пінує 1.1.1, не має чим бігти.

**Чому PATCH, а не MINOR.** За семвером (рішення оператора 2026-09-07
тримати версії за ним): 0062 — виправлення вади, 0061 — прискорення
без зміни поведінки (вироки збігаються дослівно). Жодного нового API,
жодної нової можливості для людини. Отже 1.1.**1**.

**Порядок — той, що записав контракт `tool-release`:** версія крейта
(`tool/Cargo.toml`, `tool/Cargo.lock`) і пін `version` у `keel.toml`
ОДНИМ комітом — між двома окремими кожен суд піна червоний (exit 2 з
обох боків, зміряно хвилею 0056 і переміряно рецензентом 0060); теґ
`v1.1.1` — на злитий коміт.

Хвиля легка (§6.8): одна chore, контрактів не чіпає.

## transform: the-third-release

Два числа і запис. Що опубліковано — перевіряється руками за теґом:
`shasum -c` над архівом, `sh install.sh 1.1.1` у чистій теці, і головне
для цього релізу — що поставлений launcher переживає вузький PATH.

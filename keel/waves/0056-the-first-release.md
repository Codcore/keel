---
depends_on: [0055-the-first-day-in-a-strangers-project]

transforms:
  the-first-release:
    chore: "the crate's version and the project's pin become 1.0.0 in one commit, and the two texts that describe a world with no release are rewritten (the operator's decision of 2026-09-06)"
    files:
      - tool/Cargo.toml
      - tool/Cargo.lock
      - keel.toml
      - README.md
      - install.sh
decisions:
  functional.completeness: "зміряно: `release.sh --tag` відмовляє дереву, чий бінарник відповідає іншим числом, тож теґ і версія не розійдуться"
  functional.correctness: "не застосовується"
  functional.appropriateness: "не застосовується"
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "зміряно: кожна версія має власний дім у ~/.keel/versions/, тож 1.0.0 стає поруч зі старими, не чіпаючи їх (контракт tool-launcher, хвиля 0041)"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "не застосовується"
  interaction.learnability: "не застосовується"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "не застосовується"
  interaction.self-descriptiveness: "не застосовується"
  reliability.faultlessness: "не застосовується"
  reliability.fault-tolerance: "не застосовується"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "тримає реліз: workflow збирає чотири цілі тим самим release.sh, кладе поруч .sha256 і attestation; launcher звіряє checksum до розпакування (контракт tool-release)"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "названо межу: provenance доводить attestation GitHub, і звіряє її `gh attestation verify` — launcher цього не робить і каже про це"
  security.resistance: "не застосовується"
  maintainability.modularity: "не застосовується"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "тримає: README каже стан релізу після цієї хвилі — що пін тепер версія, і що власний CI збирає інструмент із дерева гілки"
  maintainability.modifiability: "не застосовується"
  maintainability.testability: "не застосовується"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "тримає ця хвиля: після теґа v1.0.0 пін проєкту — версія, і launcher та install.sh беруть опублікований реліз (архів + .sha256) замість збірки з джерела; дорогу з джерела лишено як запасну"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "свідомо і названо: номер крейта і пін проєкту рухаються ОДНИМ коммітом — суд піна порівнює пін із бінарником, зібраним із цього ж дерева, тож роздільні комміти зробили б будь-який суд між ними червоним"
  safety.risk-identification: "не застосовується"
  safety.fail-safe: "не застосовується"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "не застосовується"
---

## Why

Рішення оператора 2026-09-06, поставлене через інструмент: **реліз
1.0.0 зараз**, теґ пушить агент. Хвиля 0055 закрила все, що падало або
брехало на першому дні в чужому проєкті; далі інструмент іде в
роботу, і брати його треба опублікованим релізом, а не збіркою з
гілки.

**Зміряно перед хвилею.** Версія крейта не рухалась 55 хвиль
(`0.1.0`), тож пін проєкту й досі називає коміт чи гілку, а не
версію: `keel version` каже «пін keel.toml: "0.1.0" — тримається»,
але жоден опублікований теґ цієї розкладки не несе (`KEEL_REF=v0.8.9`
відмовляє поіменно — крейт v1 жив поза `tool/`). Дорога релізу
(хвиля 0048) написана, зіграна пробами і **жодного разу не пройдена
по-справжньому**: релізів у репозиторії нуль.

**Чому одним коммітом.** Суд піна порівнює пін у `keel.toml` із
версією бінарника, зібраного з цього ж дерева: піднімеш номер крейта
окремо — червоніє кожен суд, поки не рухнеш пін; рухнеш пін окремо —
те саме. Тому обидва рядки їдуть одним коммітом, і це названо тут, а
не лишене здогадом.

**Чого хвиля НЕ робить.** Не чіпає контрактів (тому легка, §6.8) і не
переписує історичних рядків у `keel/reviews/` і `keel/waves/`, де
`0.1.0` — запис заміру свого часу. Контракти `tool-launcher` і
`tool-release` теж лишаються: вони описують заміри хвиль 0041 і 0048
з їхніми номерами.

## transform: the-first-release

`tool/Cargo.toml` і `tool/Cargo.lock` — `1.0.0`; `keel.toml` — пін
`1.0.0` тим самим коммітом. README: абзац про те, що жоден теґ не
несе цієї розкладки, стає абзацом про перший реліз. `install.sh`:
коментар «на самому keel КОЖЕН ref відповідає 0.1.0» був заміром
світу без релізу.

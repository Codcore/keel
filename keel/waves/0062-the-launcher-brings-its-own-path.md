---
depends_on: [0060-the-second-release]

scenarios:
  the-launcher-runs-where-the-hook-runs:
    covers: [reliability.availability, maintainability.testability]

transforms:
  the-launcher-brings-its-own-path:
    implements: [the-launcher-runs-where-the-hook-runs]
    files:
      - install.sh
      - tool/tests/launcher_test.rs
      - tool/tests/frame_tongue_test.rs
  the-contract-pays-its-debt:
    chore: "контракт launcher-а каже нову межу — і платить борг, який реліз 1.1.0 залишив у ньому двома уступами про 1.0.0 (черга після 0060, рецензія 0060 R-1)"
    files:
      - keel/contracts/tool-launcher.md
decisions:
  functional.completeness: "зміряно живим користуванням 2026-09-07: `PATH=/nonexistent sh ~/.local/bin/keel --version` → `dirname: command not found`. Launcher кличе dirname, grep, head, sed, cut, uname і решту без абсолютних шляхів, а hook, який keel сам і ставить, біжить саме у вузькому PATH графічного клієнта"
  functional.correctness: "тримає ця хвиля: launcher робить те саме, що робив, — просто там, де раніше не міг; жодна відповідь не змінюється, і це міряє батарея"
  interaction.user-error-protection: "тримає: людина не мусить знати, що її графічний клієнт дає вузький PATH — інструмент, який ставить хук, відповідає за те, щоб той хук працював"
  functional.appropriateness: "не застосовується"
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "тримає ця хвиля: launcher не міняє PATH людини — він лише додає /usr/bin і /bin ДО свого власного, і лише якщо їх там нема; те, що людина поклала в PATH сама, лишається попереду"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "не застосовується"
  interaction.learnability: "не застосовується"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "тримає ця хвиля: людина, що комітить із графічного клієнта, дістає суд, а не мовчазне `command not found` посеред чужого скрипта"
  interaction.self-descriptiveness: "не застосовується"
  reliability.faultlessness: "не застосовується"
  reliability.fault-tolerance: "названо межу: launcher не вміє працювати БЕЗ /usr/bin і /bin зовсім — він шелловий скрипт і кличе утиліти; хвиля робить те, що можна: не покладатись на PATH, який дав чужий процес"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "названо ризик і його межу: додавати теки в PATH — це вибір, ЯКИЙ бінарник побіжить. Додаються рівно дві системні теки і рівно в кінець власного PATH, тож підмінити ними нічого не можна: що людина поставила попереду, те й виграє"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "не застосовується"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "тримає: контракт tool-launcher каже цю межу своїм текстом — разом із боргом, який реліз 1.1.0 залишив у ньому (черга після 0060)"
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

**Суд комміту не біжить там, де його ставлять.** Знайдено живим
користуванням 2026-09-07: оператор поставив keel 1.1.0 з GitHub, і
проба `frame_tongue_test` — та, що існує рівно для цього випадку —
почервоніла вперше за весь час. Зміряно:

```
PATH=/nonexistent sh ~/.local/bin/keel --version
  → /Users/…/.local/bin/keel: line 211: dirname: command not found
```

Hook, який пише сам keel, свою частину роботи робить правильно: він
несе АБСОЛЮТНИЙ шлях до інструмента, бо «графічний клієнт жене хуки
своїм PATH, де keel часто не знайти» (рецензія 0055 R-4). Але той
абсолютний шлях веде до LAUNCHER-а, а launcher — шелловий скрипт, і
він кличе `dirname`, `grep`, `head`, `sed`, `cut`, `uname` іменами, не
шляхами. У вузькому PATH графічного клієнта їх нема, і суд не біжить
зовсім — людина бачить `command not found` посеред чужого файлу.

Проба цього не бачила, поки на машині автора не стояв launcher: без
нього hook брав `current_exe`, тобто сам бінарник, якому PATH не
потрібен. Тобто вада жила рівно в тій розкладці, яку має КОРИСТУВАЧ, і
не жила в тій, яку має CI.

**Борг, який ця хвиля платить тією самою рукою.** Хвиля повна (§6.8:
чіпає контракт), тож у ній законно сплатити те, що легка 0060 назвала
і не могла зробити: два уступи `tool-launcher.md` кажуть `1.0.0` про
дерево, яке відповідає `1.1.0` (рецензія 0060 R-1).

## scenario: the-launcher-runs-where-the-hook-runs

**Дано** hook, поставлений keel, і launcher на PATH, **коли** git жене
той hook із PATH, у якому нема ні keel, ні звичайних утиліт (`PATH=/
nonexistent` — те саме, що дає графічний клієнт), **тоді** суд комміту
відповідає своїм словом, а не `command not found`: launcher сам
приносить `/usr/bin` і `/bin`, і лише в кінець власного PATH — те, що
людина поставила попереду, лишається попереду.

## transform: the-launcher-brings-its-own-path

Перші рядки launcher-а: якщо в PATH нема `/usr/bin` — додати його і
`/bin` у кінець. Не заміна PATH, а доповнення; вибір людини не
чіпається.

**Дрейф (§4.6), названий тут.** `tool/tests/frame_tongue_test.rs` план
не називав. Це та проба, що вперше почервоніла на машині оператора, і
причина її червоного — не її обіцянка: вона ставила hook у середовищі,
де на PATH стоїть УЖЕ ВСТАНОВЛЕНИЙ launcher, і судила його. Тобто
міряла розкладку машини, а не те, що пише цей код (той самий клас, що
хвиля 0058 називала chore-трансформою). Тепер вона ставить hook без
keel на PATH — і судить рівно свою обіцянку: hook несе абсолютний шлях
і працює. Launcher має власну пробу в цій хвилі.

## transform: the-contract-pays-its-debt

Контракт каже нову межу і виправляє числа, які реліз 1.1.0 зробив
неправдою. Уступів виявилось ТРИ, а не два: рецензія 0060 назвала два,
третій — той, що автор написав власною рукою в хвилі 0058
(«теперішню розкладку несе теґ v1.0.0»). Тому він переписаний так, щоб
не старіти: число в ньому більше не стале, і сказано, чому.

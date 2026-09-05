---
depends_on: [0040-the-cli-contract]

scenarios:
  versions-live-side-by-side:
    covers: [flexibility.installability, compatibility.co-existence]
  the-launcher-runs-what-the-project-pinned:
    covers: [functional.correctness, safety.fail-safe]
  the-lamp-shows-what-stands-here:
    covers: [interaction.self-descriptiveness, maintainability.analysability]

transforms:
  each-version-its-own-home:
    implements:
      - versions-live-side-by-side
    files:
      - install.sh
      - tool/tests/versions_test.rs
  the-launcher:
    implements:
      - the-launcher-runs-what-the-project-pinned
    files:
      - install.sh
      - one new in tool/
      - keel/contracts/tool-launcher.md
      - tool/tests/launcher_test.rs
  the-lamp-counts-them:
    implements:
      - the-lamp-shows-what-stands-here
    files:
      - tool/src/version.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/installed_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0041-the-tool-lives-in-versions.md

decisions:
  functional.completeness: "тримає the-launcher-runs-what-the-project-pinned: щабель закривається тим, що є під рукою — git-ref і його sha; докачування релізних бінарників GitHub не входить, і це сказано в межах"
  functional.appropriateness: "тримає versions-live-side-by-side: два проєкти на різних пінах — це щоденна робота, а не куток; сьогодні другий перетирає першого"
  performance.time-behaviour: "свідомо без тесту, і ціна названа: launcher — це один exec на команду; вимірювати мілісекунди тут нема сенсу, а приховувати їх — нечесно"
  performance.capacity: "свідомо без тесту: кожна версія — власне дерево і власний target; місце росте лінійно, і lamp показує, скільки їх"
  performance.resource-utilisation: "не застосовується"
  compatibility.interoperability: "тримає versions-live-side-by-side: домовляється рівно з git і cargo, як і досі"
  interaction.appropriateness-recognisability: "свідомо без тесту: імена тек — самі версії, тож ls ~/.keel/versions читається без пояснень"
  interaction.learnability: "свідомо без тесту: команди ті самі; єдина нова річ — що keel сам бере потрібну версію"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає the-launcher-runs-what-the-project-pinned: launcher ніколи не жене іншу версію мовчки — нема пінованої, і він відмовляє з готовою командою"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "тримає the-launcher-runs-what-the-project-pinned: відмова несе саме ту команду, якою ставиться саме та версія"
  reliability.faultlessness: "тримає versions-live-side-by-side: постановка другої версії не чіпає жодного байта першої"
  reliability.fault-tolerance: "тримає the-launcher-runs-what-the-project-pinned: без мережі — чесна відмова словами git, ніколи не тихий відкат на ту версію, що є"
  reliability.availability: "не застосовується"
  reliability.recoverability: "свідомо без тесту: зіпсовану версію лікує видалення її теки — вона нічия, крім своєї"
  security.confidentiality: "не застосовується"
  security.integrity: "тримає the-launcher-runs-what-the-project-pinned: sha коміта записується при постановці і звіряється перед бігом — дерево, яке зрушили руками, launcher називає вголос"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "свідомо без окремої роботи, і межа названа гостро: sha доводить, що дерево — те саме, яке назвав ref; він НЕ доводить, що ref гідний довіри. Підпис і checksum релізу — рядок черги"
  security.resistance: "не застосовується"
  maintainability.modularity: "тримає the-lamp-shows-what-stands-here: розкладка версій — знання одного місця, і lamp питає його, а не вгадує шляхи"
  maintainability.reusability: "не застосовується"
  maintainability.modifiability: "свідомо без тесту: launcher — десяток рядків shell; змінити розкладку — змінити одну змінну в ньому й у lamp"
  maintainability.testability: "свідомо без тесту: проби жени справжній install.sh проти справжнього git-репозиторію зі стабом cargo — школа рецензії 0039 R-5"
  flexibility.scalability: "не застосовується"
  flexibility.adaptability: "тримає versions-live-side-by-side: KEEL_HOME, KEEL_BIN, KEEL_REPO лишаються переставними, і launcher їх шанує"
  flexibility.replaceability: "свідомо без тесту: проєкт, що ставить keel інакше, лишає власний бінарник на PATH — launcher не встановлюється поверх чужого"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий числом — сьогодні друга версія перетирає першу, тож два проєкти на різних пінах не працюють разом узагалі"
  safety.hazard-warning: "тримає the-lamp-shows-what-stands-here: lamp показує, що стоїть тут, тож розбіжність піна видно раніше, ніж суд відмовить"
  safety.safe-integration: "тримає versions-live-side-by-side: проєкт без піна працює як досі — launcher жене ту версію, що стоїть, і нічого не питає"
---

## Why

Концепт описує щабель дистрибуції дуже конкретно
(`NEW-CONCEPT.md:121-145`, `320-321`, `378-379`):

> «Сам інструмент — глобально, версіями: `~/.keel/versions/<версія>/`,
> один статичний бінарник на версію. Проєкт **пінить версію в
> конфізі**; різні проєкти живуть на різних версіях одночасно і не
> заважають одне одному.»

> «Запуск (**рішення оператора** — авто-докачування): команда `keel`
> читає пін проєкту і виконує потрібну версію; якщо її нема
> локально — сама качає, звіряє checksum і каже вголос, що і звідки
> взяла; **без мережі — чесна відмова з готовою командою**.»

> «`keel version` — яка версія працює, звідки пін, **які версії стоять
> локально**.»

Зміряно перед планом:

| обіцяне | сьогодні |
|---|---|
| `~/.keel/versions/<версія>/` | нема: `install.sh` кладе **одне** дерево в `~/.keel` і **один** бінарник у `~/.local/bin/keel` — друга версія перетирає першу |
| проєкти на різних пінах разом | **неможливо**: бінарник один на машину |
| launcher, що читає пін | нема: пін лише **забороняє** — суди відмовляють, доки він не збігся, а звести їх мусить людина руками |
| checksum | нема жодного |
| `version` показує, що стоїть локально | нема: показує лише бінарник, який біжить, і пін |

**Що можна зробити чесно і повністю — і що не можна.** Тут не
потрібні опубліковані релізи GitHub: **git і є дистрибуцією**. Ref
розвʼязується в sha коміта, і sha **і є** checksum — він доводить, що
дерево те саме, яке назвав ref. Отже щабель закривається локально й
наскрізь.

Чого це **не** дає, і сказано гостро: sha доводить тотожність дерева,
а не **гідність довіри** до ref-а. Підпис, опублікований реліз і
перевірений checksum архіву — окремий рядок черги, не ця хвиля.
І далі лишається правдою, що жоден опублікований теґ не несе
теперішньої розкладки (крейт v1 жив поза `tool/`), тож пін сьогодні
називає коміт або гілку.

**Рішення оператора, яке хвиля шанує:** shim-скриптів **у проєкті**
нема. Launcher — це глобальний `keel` на PATH, а не файл у чужому
репозиторії.

## scenario: versions-live-side-by-side

**Дано** дві версії keel, названі різними ref-ами,
**коли** обидві ставляться,
**тоді** кожна живе у власній теці `~/.keel/versions/<ref>/` зі своїм
деревом, своїм бінарником і записаним sha коміта, з якого зібрана.
Постановка другої **не чіпає жодного байта** першої. `KEEL_HOME`,
`KEEL_BIN` і `KEEL_REPO` лишаються переставними.

## scenario: the-launcher-runs-what-the-project-pinned

**Дано** проєкт, що пінить версію в `keel.toml`,
**коли** в ньому кличуть `keel`,
**тоді** біжить **саме та** версія з `~/.keel/versions/`. Її нема
локально — launcher каже, чого бракує, і дає **готову команду**, якою
її поставити; іншу версію він не жене **ніколи**, бо тихо не той
бінарник гірший за відмову. Дерево версії, чий записаний sha не
збігається з тим, що лежить, — відмова вголос, не мовчазний біг.
Проєкт **без** піна дістає ту версію, що стоїть, і нічого нового не
питається.

## scenario: the-lamp-shows-what-stands-here

**Дано** машину, де стоїть кілька версій,
**коли** питають `keel version`,
**тоді** він каже три речі: яка версія біжить, куди показує пін — і
**які версії стоять тут**, кожна своїм ref-ом. Машинна дорога
(`--json`) несе той самий перелік полем `installed`, а не прозою.

## transform: each-version-its-own-home

`install.sh` кладе кожен ref у власну теку і записує поруч sha, з
якого зібрано. Спільної теки `~/.keel`, куди все валиться, більше
нема.

## transform: the-launcher

`~/.local/bin/keel` стає launcher-ом: читає `version` із `keel.toml`
проєкту, знаходить цю версію, звіряє sha і віддає їй керування. Нема
такої — відмова з командою. Контракт `tool-launcher` описує його
поведінку, і тримається він бігом, а не формою: це shell.

## transform: the-lamp-counts-them

`keel version` перелічує те, що стоїть у `~/.keel/versions/`, прозою
і полем `installed` у пакеті.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — `docs/uk/V2-PROCESS.md` (§9.10). BACKLOG
втрачає важкий рядок про дистрибуцію, README дістає розділ про
версії. Сюди ж лягає звіт рецензії.

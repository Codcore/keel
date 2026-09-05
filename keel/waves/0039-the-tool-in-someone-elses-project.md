---
depends_on: [0038-the-tool-learns-a-second-tongue]

scenarios:
  the-block-names-what-holds-it:
    covers: [functional.correctness, interaction.self-descriptiveness]
  the-generated-ci-runs-where-it-is-born:
    covers: [functional.completeness, reliability.availability]
  the-pin-has-a-hand:
    covers: [flexibility.installability, interaction.user-assistance]

transforms:
  the-rule-is-the-machinery:
    implements:
      - the-block-names-what-holds-it
    contracts: [tool-generated@28c67d]
    files:
      - tool/src/generated.rs
      - keel/contracts/tool-generated.md
      - docs/uk/METHODOLOGY-V2.md
      - docs/en/METHODOLOGY-V2.md
      - METHODOLOGY.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/rule_truth_test.rs
  the-workflow-brings-its-own-tool:
    implements:
      - the-generated-ci-runs-where-it-is-born
    files:
      - tool/src/generated.rs
      - .github/workflows/keel.yml
      - keel.toml
      - tool/tests/workflow_runs_test.rs
  the-pin-can-be-fetched:
    implements:
      - the-pin-has-a-hand
    files:
      - install.sh
      - tool/src/version.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/pin_hand_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0039-the-tool-in-someone-elses-project.md

decisions:
  functional.appropriateness: "тримає the-block-names-what-holds-it: агент читає блок перше, ніж будь-що інше; блок, що описує не цей проєкт, полегшує не ту задачу"
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "свідомо без тесту: крок постановки в CI будує keel із сирців — це хвилини і мегабайти чужого раннера, і воно сказане в самому файлі, а не приховане"
  compatibility.co-existence: "тримає the-generated-ci-runs-where-it-is-born: крок постановки не чіпає нічого поза ~/.keel і ~/.local/bin, і жоден наявний крок проєкту не переписується"
  compatibility.interoperability: "тримає the-pin-has-a-hand: install.sh домовляється рівно з git і cargo — і відмовляє вголос, коли їх нема"
  interaction.appropriateness-recognisability: "свідомо без тесту: назви кроків у workflow лишаються тими самими словами, якими їх зве норма"
  interaction.learnability: "свідомо без тесту: команд не додається — install.sh той самий, лише вміє версію"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає the-pin-has-a-hand: пін на ref, якого нема, — відмова install.sh із тим самим ім'ям, а не мовчазний білд main"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  reliability.faultlessness: "тримає the-block-names-what-holds-it: mode і hooks — дві незалежні ручки, і всі чотири їхні поєднання дають правдивий абзац"
  reliability.fault-tolerance: "свідомо без тесту: мережі нема — install.sh відмовляє словами git, і суди від цього не змінюються"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "свідомо без тесту: install.sh пише лише в ~/.keel і ~/.local/bin, обидва названі й переставні змінними"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "свідомо без окремої роботи, і межа названа: постановка тягне git-ref за іменем, а не перевірений checksum — checksum концепту лишається в черзі окремим рядком, і version про це каже вголос"
  security.resistance: "не застосовується"
  maintainability.modularity: "тримає the-block-names-what-holds-it: правило народжується з однієї руки для блока і для skill-а, і ця рука питає обидві ручки"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "тримає the-generated-ci-runs-where-it-is-born: провал постановки називає себе кроком із власним ім'ям, а не «command not found» посеред чужого кроку"
  maintainability.modifiability: "свідомо без тесту: абзаци правила лишаються константами поруч, і нова ручка додає рядок у ту саму руку"
  maintainability.testability: "свідомо без тесту: усі три проби будують пісочниці спільною рукою 0030; workflow судиться текстом, бо ганяти GitHub Actions тут нічим — межа названа"
  flexibility.adaptability: "тримає the-pin-has-a-hand: KEEL_REPO, KEEL_HOME, KEEL_BIN і версія — усе переставне, бо середовища різні"
  flexibility.scalability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий числом — щабель дистрибуції з концепту (~/.keel/versions/, докачування, checksum) цією хвилею НЕ закривається; закривається лише те, щоб порада «візьми приколоту версію» мала руку"
  safety.fail-safe: "тримає the-block-names-what-holds-it: там, де машина не тримає нічого, абзац каже «тримають люди», а не мовчить — небезпечний бік брехні тут саме той, що правило подане як гарантоване"
  safety.hazard-warning: "тримає the-generated-ci-runs-where-it-is-born: файл сам каже, що крок постановки будує з сирців і скільки це коштує"
  safety.safe-integration: "свідомо без окремої роботи: проєкт, що вже поклав власний крок постановки, не дістає другого — межа сказана в самому файлі"
---

## Why

Аудит «обіцяно і не зроблено» (2026-09-05, двадцять знахідок) назвав
пʼять важких. Три з них — одна історія: **що робить keel, коли він
приїхав у чужий проєкт**. Кожна зміряна руками, не прочитана.

**Перша.** `keel init --no-hooks --mode strict` друкує «git hook не
поставлено» — і через два рядки пише в `AGENTS.md`:

> Two rules a machine holds here, so no memory has to: a scenario is
> born red — the commit `red: <scenario>` passes the commit-msg hook
> only when its test really fails.

`.git/hooks/commit-msg` не існує. `rule_for()` (`generated.rs:334`)
дивиться лише на `mode` і **не питає `config.hooks` ніколи**. Це
конституція п. 6 навиворіт: правило подане як гарантоване машиною
там, куди агент дивиться **першим**. `mode` і `hooks` — дві
незалежні ручки, і чотири їхні поєднання дають чотири різні правди;
рука знає одну.

Того ж класу — два менші рядки, зміряні тоді ж: `keel check` у
**чужому** проєкті друкує «наступний крок: контракт, що називає
неіснуючий модуль, має бути знахідкою поза план-гілкою» — внутрішню
нотатку keel-а про власний щабель, до того ж закриту хвилею 0035; а
**§4.13** каже, що заборону merge-у гілки `spike/*` тримає
«перевірка на PR», хоч зміряно протилежне: `keel check` на
`spike/probe` — **вихід 0**, тільки нотатка; відмовляє `keel close`.
Проєкт, що поставить у CI лише `check`, заборони не має, а норма
каже, що має.

**Друга.** Згенерований `.github/workflows/keel.yml` кличе `keel`,
**не ставлячи його**, і сам це визнає коментарем. Проєкт, що зробив
`init` і запушив, дістає `keel: command not found` — червоне CI, яке
не про його роботу. Файл, який рама породила, мусить бігти там, де
народився.

**Третя.** `keel version` на розбіжності каже: «пін keel.toml: "X" —
не цей бінарник: суди відмовляють, доки пін і бінарник не зійдуться»
— і **не називає жодної руки**, якою їх звести. `install.sh` версії
не приймає взагалі: завжди клонує і будує `main`. При тому теґи в
репозиторії є (до `v0.8.9`) — тобто рука можлива, її просто нема.
Порада вказує на дію, якої інструмент не вміє.

**Межа цієї хвилі, названа наперед.** Щабель дистрибуції з концепту
— `~/.keel/versions/`, launcher, докачування, checksum — цією хвилею
**не** закривається. Закривається рівно те, що порада мала руку:
`install.sh` бере названу версію, а `version` називає команду. Решта
щабля лишається рядком у черзі, а не мовчазною обіцянкою.

## scenario: the-block-names-what-holds-it

**Дано** проєкт, де `mode` і `hooks` стоять у будь-якому з чотирьох
поєднань,
**коли** `keel init` пише `AGENTS.md` і `SKILL.md`,
**тоді** абзац правила описує **те, що в цьому проєкті справді
стоїть**: машина тримає правило лише там, де hook поставлено і режим
не `manual`; де hook-а нема — сказано, що тут його тримають люди, і
названо, що досі судить (`keel close`, `keel check`). Абзац ніколи не
обіцяє hook, якого нема.

І ширше — **слова, які keel каже в чужому проєкті, про цей проєкт**:
`keel check` не друкує внутрішніх нотаток keel-а про власну чергу, а
норма не приписує заборони тому судові, який її не тримає (§4.13
називає `keel close`).

## scenario: the-generated-ci-runs-where-it-is-born

**Дано** проєкт, який щойно зробив `keel init` і запушив,
**коли** біжить згенерований workflow,
**тоді** `keel` є на PATH, бо workflow ставить його сам — окремим
названим кроком, який каже, що будує з сирців і чого це коштує.
Провал постановки — провал **свого** кроку з власним ім'ям, а не
`command not found` посеред судового. Проєкт, який уже має власний
крок постановки, другого не дістає.

## scenario: the-pin-has-a-hand

**Дано** `keel.toml` із `version`, що не збігається з бінарником,
**коли** людина питає `keel version`,
**тоді** вирок називає **команду**, якою взяти саме приколоту
версію, і ця команда справді працює: `install.sh` приймає версію
(git-ref) і ставить рівно її. Ref, якого нема, — відмова install.sh
із тим самим ім'ям, а не мовчазний білд `main`. Межа сказана вголос:
це git-ref за іменем, не перевірений checksum, і щабля
`~/.keel/versions/` тут ще нема.

## transform: the-rule-is-the-machinery

`rule_for()` питає обидві ручки, а не одну. Абзаців стає чотири на
мову замість трьох: до `strict`, `soft` і `manual` додається той,
що описує проєкт із вимкненими hook-ами. `check` перестає друкувати
чужому проєктові власну чергу. §4.13 називає суд, який заборону
справді тримає.

## transform: the-workflow-brings-its-own-tool

Згенерований workflow дістає крок постановки перед судовими. Він
каже про себе все: що будує з сирців, що потребує git і cargo, і що
проєкт може замінити його своїм.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — `docs/uk/V2-PROCESS.md` (§9.10). BACKLOG
втрачає три важкі рядки аудиту, README дістає правду про постановку
і про пін. Сюди ж лягає звіт рецензії.

## transform: the-pin-can-be-fetched

`install.sh` приймає версію — аргументом і змінною — і ставить рівно
її. `version` на розбіжності називає цю команду з підставленою
версією замість поради без руки.

---
depends_on: [0020-ignore-reminded]

scenarios:
  courts-deaf-to-the-environment:
    proves: tool-scope@5bf6e5
    covers: [functional.correctness, safety.fail-safe]

transforms:
  one-git-hand:
    implements:
      - courts-deaf-to-the-environment
    contracts: [tool-scope@5bf6e5, tool-gate@0a8613, tool-init@9fafaa, tool-plan@89aa74, tool-review@aa0a73, tool-next@ec56ff]
    files:
      - tool/src/scope.rs
      - tool/src/gate.rs
      - tool/src/init.rs
      - tool/src/check.rs
      - tool/src/plan.rs
      - tool/src/review.rs
      - tool/src/next.rs
      - tool/tests/git_hand_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md

decisions:
  functional.appropriateness: "свідомо без окремого тесту: хвиля не додає ручок — вона робить правдою те, що всі суди вже обіцяли: судити названий проєкт"
  functional.completeness: "тримає courts-deaf-to-the-environment: усі дванадцять місць виклику git ідуть однією рукою; grep за Command::new(\"git\") поза домом лишає нуль — це й перевіряє сценарій"
  performance.time-behaviour: "не застосовується: та сама кількість викликів git, лише з чистішим середовищем"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "свідомо без нового тесту: поза hook-ом середовища тих змінних нема — жоден звичайний біг не міняє поведінки ані на слово"
  compatibility.interoperability: "свідомо без нового тесту: git кличеться тими самими підкомандами; міняється лише те, чого він НЕ успадковує"
  interaction.appropriateness-recognisability: "не застосовується: вивід не міняється"
  interaction.learnability: "не застосовується: нових слів нема"
  interaction.operability: "не застосовується: нових ручок нема"
  interaction.user-error-protection: "тримає courts-deaf-to-the-environment: помилка, від якої береже хвиля, — не людська, а середовища: hook віддає дітям свій репозиторій, і суд мовчки судив чужий"
  interaction.user-assistance: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується: жодного нового тексту"
  interaction.self-descriptiveness: "свідомо без нового тесту: контракт scope каже вголос, що рука глуха до середовища і чому — сама поведінка мовчазна за задумом"
  reliability.faultlessness: "свідомо без окремого тесту: один дім замість дванадцяти сирих викликів — менше місць для розбіжности (школа 0015/0017)"
  reliability.fault-tolerance: "свідомо без нового тесту: шляхи відмов ті самі — мовчазний git лишається мовчазним git-ом"
  reliability.availability: "не застосовується: локальний бінарник"
  reliability.recoverability: "не застосовується: стану нема"
  security.confidentiality: "не застосовується"
  security.integrity: "тримає courts-deaf-to-the-environment: у репозиторій середовища не йде жодного байта — це і є зміст хвилі для того, хто пише (hook, план, довіра)"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "свідомо без фаззингу: вхід — імена змінних середовища, знятих поіменно"
  maintainability.modularity: "свідомо без тесту: рука git живе в scope — модулі, що вже знає git проєкту; gate, init, check, plan, review і next її питають, а не будують свою"
  maintainability.reusability: "свідомо без тесту: рука — звичайна Command, готова до будь-якої підкоманди"
  maintainability.analysability: "свідомо без нового тесту: контракт scope називає перелік знятих змінних і причину — читач не мусить розгадувати"
  maintainability.modifiability: "свідомо без тесту: нова змінна середовища — один рядок у списку одного дому"
  maintainability.testability: "свідомо без окремого тесту: пісочниці — справжні проєкти школи 0005–0020"
  flexibility.adaptability: "не застосовується: git той самий для всіх адаптерів"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "свідомо: знімаються лише змінні адресування репозиторію і механізм -c; GIT_CONFIG_GLOBAL/SYSTEM, GIT_AUTHOR_* і решта лишаються — то свідомий вибір людини, не витік hook-а, і це названо в Застереженні"
  safety.risk-identification: "не застосовується як окрема робота: загроза — суд, що мовчки судить чужий репозиторій, — і є сценарієм хвилі"
  safety.hazard-warning: "не застосовується: хвиля прибирає пастку, а не попереджає про неї"
  safety.safe-integration: "свідомо без тесту: рука переїжджає з gate у scope одним рухом, gate питає її як усі; жодного нового модуля"
---

## Why

Рецензія хвилі 0020 (R-1), заробивши це бігами, назвала ціну того,
що лишалось нечищеним. Hook віддає своїм дітям репозиторій, для
якого біжить — `GIT_DIR`, `GIT_WORK_TREE` і рідню, — і ці змінні
**старші за `-C`**. Хвиля 0020 почистила дві руки рами (рядок
ignore і установку hook-а), решта дванадцяти місць лишились сліпими.
Рецензент виміряв, чого це варте:

- `keel check` під чужим `GIT_DIR` дав **46 знахідок із чужого
  дерева** там, де на своєму нуль;
- `keel review` відмовив;
- `keel gate` — єдина команда, що **завжди** біжить із hook-а, —
  втратив предмет суду («гілка "?" не зветься як жодна хвиля»);
- найгірше: `keel close` **зазеленів на недоведеній хвилі** —
  вихід 0 замість 1. Суд закриття, що благословляє недоведене, —
  це саме те, проти чого стоїть уся методика.

Хвиля робить просту річ: **git кличеться однією рукою, і ця рука
глуха до середовища**. Суд судить проєкт, на який його навели.

Відступи bootstrap, названі вголос: хвиля їде робочою гілкою сесії;
план їде план-гілкою §8.3 (експорт scope росте наперед коду);
журнал їде chore-трансформою; хвіст `keel check` лишається на
щаблі 19 — ця хвиля вклинюється перед ним за важливістю, і це
сказано тут.

## scenario: courts-deaf-to-the-environment

**Дано** проєкт A з хвилею в роботі (сценарій доведений тегом і
зеленим тестом, а звіту рецензії §9.9 поруч нема — саме та нестача,
що блокує merge) і чужий репозиторій B, чиї `GIT_DIR`, `GIT_WORK_TREE`
і рідня стоять у середовищі — рівно так, як їх лишає git-hook,
**коли** над A біжать суди — `check`, `close`, `gate`, `review`,
`status`, `next`, `plan`, — а також `keel trust` і `keel init`,
**тоді** кожен судить A: `close` лишається червоним (вихід 1) і
називає нестачі A, ніколи не зеленіє; `check` дає знахідки самого A
(і жодної з B); `gate` називає гілку A; `review` збирає пакет A;
`plan` рахує числа A; у репозиторій B не йде **жодного байта**, і
жодна команда не читає з B ані рядка.

## transform: one-git-hand

Рука `git_at` переїжджає з gate у `scope` — модуль, що вже знає git
проєкту (гілка, історія, база порівняння), — і стає його експортом.
Усі дванадцять місць виклику git (scope ×3, check ×5, gate, plan,
review, next) кличуть її; init переходить із `gate::git_at` на неї
ж. Знімаються змінні адресування репозиторію (`GIT_DIR`,
`GIT_WORK_TREE`, `GIT_COMMON_DIR`, `GIT_INDEX_FILE`,
`GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`,
`GIT_PREFIX`, `GIT_CEILING_DIRECTORIES`) і механізм `-c`
(`GIT_CONFIG_PARAMETERS`, `GIT_CONFIG_COUNT`). Поведінка поза
hook-ом не міняється ані на слово: без цих змінних рука — звичайний
`git -C <корінь>`.

Застереження: знімаються саме змінні витоку, не всі GIT_* —
`GIT_CONFIG_GLOBAL`, `GIT_CONFIG_SYSTEM`, `GIT_AUTHOR_*` і подібні
лишаються, бо то свідомий вибір людини чи CI, а не спадок hook-а;
grep за сирим `Command::new("git")` поза домом стає нулем, і саме
це число хвиля тримає — нові виклики мусять іти рукою; тести самі
чистять середовище своїх пісочниць тією ж школою (0020).

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10).

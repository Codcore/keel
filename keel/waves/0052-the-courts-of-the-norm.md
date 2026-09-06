---
depends_on: [0051-the-readers-of-names]

scenarios:
  a-court-that-cannot-judge-does-not-pass:
    covers: [safety.fail-safe, reliability.fault-tolerance]
  the-weight-is-read-from-the-branch-too:
    covers: [functional.correctness, interaction.self-descriptiveness]
  furniture-is-known-by-its-digest:
    covers: [functional.completeness, maintainability.analysability]
  a-transform-is-closed-by-its-commit:
    covers: [security.accountability, interaction.operability]
  two-waves-with-one-number-are-red:
    covers: [reliability.faultlessness, safety.hazard-warning]

transforms:
  the-gate-refuses-what-it-cannot-judge:
    implements:
      - a-court-that-cannot-judge-does-not-pass
    files:
      - tool/src/gate.rs
      - keel/contracts/tool-gate.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/gate_cannot_judge_test.rs
      - tool/tests/adapter_name_test.rs
  the-weight-and-the-merge-are-facts-of-the-branch:
    implements:
      - the-weight-is-read-from-the-branch-too
    files:
      - tool/src/docs.rs
      - tool/src/close.rs
      - tool/src/status.rs
      - tool/src/next.rs
      - tool/src/check.rs
      - tool/src/scope.rs
      - keel/contracts/tool-docs.md
      - keel/contracts/tool-close.md
      - keel/contracts/tool-status.md
      - keel/contracts/tool-next.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/weight_of_the_branch_test.rs
      - tool/tests/close_test.rs
      - tool/tests/rev_test.rs
  the-scope-court-and-the-anchor:
    implements:
      - furniture-is-known-by-its-digest
    files:
      - tool/src/scope.rs
      - tool/src/review.rs
      - tool/src/check.rs
      - tool/src/generated.rs
      - keel/contracts/tool-scope.md
      - keel/contracts/tool-review.md
      - keel/contracts/tool-generated.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/furniture_test.rs
  the-slug-commit-is-read:
    implements:
      - a-transform-is-closed-by-its-commit
      - two-waves-with-one-number-are-red
    files:
      - tool/src/next.rs
      - tool/src/check.rs
      - tool/src/plan.rs
      - keel/contracts/tool-next.md
      - keel/contracts/tool-plan.md
      - keel/contracts/tool-docs.md
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/slug_commits_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - keel/reviews/0052-the-courts-of-the-norm.md
decisions:
  functional.appropriateness: "свідомо без тесту: жодного нового суду поза нормою — кожен новий рядок знахідки цитує параграф, який він тримає (§2.11, §4.6, §4.8, §6.2, §6.5, §6.8, §7.12, §8.8)"
  performance.time-behaviour: "свідомо без тесту: суд слагів читає `git log --format=%s base..HEAD` один раз; суд меблів — digest файлів, що вже читає `keel update`"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "свідомо без тесту: суд слагів — дисципліна гілки (§6.2), і після merge його ніхто не читає — main судиться за наслідками (§6.5), як і досі"
  compatibility.interoperability: "тримає a-court-that-cannot-judge-does-not-pass: повідомлення читається так, як його запише git за `commit.cleanup=strip` — порожні рядки і рядки з `#` спереду не є темою; за `verbatim` тема з `#` не є ні народженням, ні роботою, і суд лише суворіший"
  interaction.appropriateness-recognisability: "не застосовується: команд не додається"
  interaction.learnability: "не застосовується"
  interaction.user-error-protection: "тримає a-court-that-cannot-judge-does-not-pass: шапка хвилі, якої суд не читає, — відмова з її словами, а не «гілка не зветься як хвиля» і пропуск"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "свідомо без тесту: кожна нова відмова несе «натомість» — суд слів проти коду (хвиля 0035)"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "тримає furniture-is-known-by-its-digest: згенерований файл, чий записаний відбиток застарів, названо в `keel check`, навіть коли текст збігається з релізом — файл, переписаний без запису, більше не стоїть мовчки"
  security.non-repudiation: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "тримає a-court-that-cannot-judge-does-not-pass: коміт над червоним тестом не проходить ні під коментарем, ні під адаптером не цього релізу, ні під битою шапкою"
  maintainability.modularity: "свідомо без тесту: суд номера хвилі — одна рука для `keel plan` і `keel check` (`plan::taken`), не дві копії"
  maintainability.reusability: "тримає two-waves-with-one-number-are-red: та сама рука рахує номери на диску й у гілках для народження і для суду"
  maintainability.modifiability: "свідомо без тесту: жодного нового місця диспетчеризації мов"
  maintainability.testability: "свідомо без тесту: проби будують проєкти з git рукою 0030 і женуть суди; де історії нема — суди кажуть межу, як і досі"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "свідомо без окремої роботи, і названо: старий бінарник на PATH судить hook-ом, поки пін версії не пінить (усі 52 хвилі — 0.1.0); суд, який не може судити, тепер відмовляє, а версія крейта → теґ → пін — рядок оператора (BACKLOG)"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "свідомо без тесту, і названо: «закрита фактом merge» кажеться лише там, де файл хвилі стоїть у main; без main (перший коміт гілки) суд каже, що факту merge не бачить"
  safety.risk-identification: "тримає the-weight-is-read-from-the-branch-too: контракт, змінений на гілці легкої хвилі, — хвиля повна, і сказано до PR, не після"
  safety.safe-integration: "тримає furniture-is-known-by-its-digest: `keel update` на робочій гілці більше не робить із меблів дрейф — файл у формі, яку лишив інструмент, поза scope (§4.8), а правлений рукою — код"
---

## Why

Третя черга глобального ревʼю 2026-09-06 — **суди норми**: параграфи
методики, які інструмент заявляє, а тримає не так, як написано, або не
тримає зовсім. Кожен рядок зміряно наново перед планом, у пісочницях
на цьому бінарнику.

**Зміряно перед планом.**

**Суд, який не може судити, пускає.** `adapter = "go"` — gate над
`work:` і над `red:` каже «адаптер не цього релізу — комміт не
суджений» і виходить **0** (методика R-10): коміт над червоним тестом
лягає. Шапка хвилі з полем, якого суд не знає, — «гілка не зветься як
жодна прочитана хвиля — судити нічого, пропуск», exit 0 (баги R-13):
бита шапка = вимкнений суд коміту. І повідомлення, яке git ще
почистить: `\n# Please enter…\nwork: typed after an Enter` — gate
читає перший сирий рядок, «поза судом, пропуск», а git записує тему
`work: typed after an Enter` (баги R-5): так виглядає кожен коміт із
редактора і з `commit.template`.

**Вага не бачить гілки.** Легка хвиля (одна chore-трансформа), чия
гілка змінює контракт `keel/contracts/ext.md`, — `keel next` каже «час
PR — легка хвиля їде в свій один PR», а §6.8 вимагає повної (методика
R-3). Хвиля з двома chore-трансформами — check 0 знахідок, status
«закрита фактом merge», а §2.11 каже «мусить бути легкою» (R-4).
Chore-хвиля лише на гілці `feature-x`, ніколи не злитій, — status
«закрита фактом merge (§2.11)», close на її гілці «закрита (легка)»,
exit 0 (R-5): факт merge ніхто не питав, і параграф цитовано не той.

**Меблі й дрейф.** `keel update` на робочій гілці переписав
`.claude/settings.json` і `keel.toml` — check: «гілка чіпає …, якого
жодна трансформа не називає», два червоних над файлами у формі, яку
лишив інструмент (методика R-6, §4.8). План-гілка з двох комітів
(файл хвилі, потім `Cargo.toml` дописано до merge) — після merge
`keel review` каже «Cargo.toml — дописаний після якоря», бо якір —
перший коміт файлу, а норма §4.6 — merge план-PR (R-7). І записаний
відбиток `keel.yml` стояв чужим від хвилі 0044 до 0050, а `keel check`
мовчав, бо текст збігався з релізом (хвиля 0050).

**§6.2 не читається.** Гілка з усією роботою в одному коміті `wip: …`
без слага трансформи — `keel next`: «час PR» (методика R-9); суд
червоного коміту є (`red:` без коміту — знахідка), суду слага — нема.

**Два номери.** `0001-a-wave.md` і `0001-b-wave.md` поруч — check 0
знахідок (R-11); §8.8 судить лише `keel plan`.

**Дрейф (§4.6), названий уголос.** У першу трансформу дописано
`tool/tests/adapter_name_test.rs`: проба хвилі 0017 тримала слово
рецензії 0017 R-4 — «gate над невідомим адаптером пропускає зі
словом», — а сценарій 0017 каже «з невідомим — відмова вголос»; проба
тепер тримає сценарій, і текст 0017 не змінюється. У другу —
`tool/tests/close_test.rs` (проба 0037 звала легку хвилю закритою без
факту merge в пісочниці без main; тепер чекає файла в main) і
`tool/tests/rev_test.rs` (золота редакція tool-docs).

## scenario: a-court-that-cannot-judge-does-not-pass

**Дано** коміт `red:` або трансформи над проєктом, де суд судити не
може: адаптер, якого реліз не веде; шапку хвилі гілки, якої читач не
читає; і повідомлення з порожніми рядками та `#`-коментарями перед
темою над червоним тестом.
**Коли** біжить `keel gate` у режимі strict.
**Тоді** коміт не проходить: адаптер не цього релізу — відмова над
`red:` і роботою (коміт поза судом проходить зі словом, як і досі);
шапка, якої не прочитати, — відмова з її словами; тема читається так,
як її запише git — після порожніх рядків і коментарів, — і суд бачить
`work:`.

## scenario: the-weight-is-read-from-the-branch-too

**Дано** легку хвилю, чия гілка змінює контракт; хвилю з двома
chore-трансформами; chore-хвилю на гілці, якої main не бачив.
**Коли** біжать `keel check`, `keel status`, `keel next`, `keel close`.
**Тоді** перша — знахідка на гілці: контракт змінюється лише повною
хвилею (§6.8, §5.7), і `next` не веде до одного PR; друга — знахідка
§2.11: хвиля з самих chore мусить бути легкою — одна трансформа;
третя — «закриється фактом merge», доки файл хвилі не стоїть у main,
і «закрита фактом merge (§6.5)» — коли стоїть; параграф — §6.5.

## scenario: furniture-is-known-by-its-digest

**Дано** робочу гілку, де `keel update` переписав згенеровані файли й
`keel.toml`; той самий файл, правлений рукою; план-гілку з двох
комітів, злиту в main; і згенерований файл, чий записаний відбиток
застарів при тексті, що збігається з релізом.
**Коли** біжать `keel check` і `keel review`.
**Тоді** файл у формі, яку лишив інструмент (відбиток збігається із
записаним або з тим, що пише реліз), — поза scope на обох гілках
(§4.8); правлений рукою — код, і дрейф; якір дрейфу повної хвилі —
файл хвилі в точці розгалуження з main (після merge план-PR), легкої
— її перший коміт; застарілий записаний відбиток — рядок `keel check`
поіменно.

## scenario: a-transform-is-closed-by-its-commit

**Дано** гілку хвилі, де файли трансформи торкнуті, а коміту, чия тема
починається її слагом, нема.
**Коли** біжать `keel next` і `keel check` на гілці.
**Тоді** `next` веде до коміту трансформи під її слагом, а не до PR;
`check` називає трансформу без коміту зі слагом знахідкою §6.2 — на
гілці, і ніколи на main (§6.5).

## scenario: two-waves-with-one-number-are-red

**Дано** дві хвилі з одним номером у `keel/waves/`.
**Коли** біжить `keel check`.
**Тоді** знахідка §8.8 називає обидва файли й наступний вільний номер
тією самою рукою, що судить `keel plan`.

## transform: the-gate-refuses-what-it-cannot-judge

`gate.rs`: тема — перший рядок після порожніх і `#`-рядків; адаптер не
цього релізу — відмова над `red:`/роботою; нечитана шапка гілки —
відмова з її словами. Словник, контракт tool-gate, проба.

## transform: the-weight-and-the-merge-are-facts-of-the-branch

`docs.rs`/`scope.rs`: вага читає й зміни контрактів на гілці; `check`:
§2.11 над самими chore; `close.rs`/`status.rs`/`next.rs`: «закрита
фактом merge» лише при файлі хвилі в main, інакше «закриється»;
цитати §6.5. Контракти docs, close, status, next. Проба.

## transform: the-scope-court-and-the-anchor

`scope.rs`: меблі за відбитком на обох гілках; `review.rs`: якір —
файл хвилі в точці розгалуження, для легкої — перший коміт;
`check.rs`/`generated.rs`: застарілий записаний відбиток — рядок.
Контракти scope, review, generated. Проба.

## transform: the-slug-commit-is-read

`next.rs`: трансформа зроблена, коли є коміт зі слагом, а не лише
торкнуті файли; `check.rs`: суд §6.2 на гілці і суд §8.8 номерів рукою
`plan::taken`. Контракти next, plan, docs. Проба.

## transform: journal

Записи журналу цього покоління їдуть із хвилею (V2-PROCESS), разом із
чергою і звітом рецензії.

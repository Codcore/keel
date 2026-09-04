---
depends_on: [0036-the-holes-in-the-norm]

scenarios:
  every-wave-has-its-reviewer:
    covers: [functional.correctness, security.accountability]
  a-green-birth-is-named-and-proved:
    covers: [functional.appropriateness, security.non-repudiation]
  a-started-wave-can-be-cancelled:
    covers: [reliability.recoverability, flexibility.adaptability]
  a-write-that-lies-is-red:
    covers: [functional.completeness, reliability.faultlessness]
  the-closing-says-what-failed:
    covers: [interaction.user-assistance, reliability.fault-tolerance]
  the-answers-are-obeyed:
    covers: [interaction.user-error-protection, security.integrity]

transforms:
  the-reviewer-is-not-optional:
    implements:
      - every-wave-has-its-reviewer
    files:
      - docs/uk/METHODOLOGY-V2.md
      - docs/en/METHODOLOGY-V2.md
      - METHODOLOGY.md
      - tool/src/close.rs
      - tool/src/next.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/reviewer_test.rs
      - tool/tests/next_test.rs
      - tool/tests/status_test.rs
      - tool/tests/close_test.rs
  a-green-birth-carries-its-mutant:
    implements:
      - a-green-birth-is-named-and-proved
    files:
      - docs/uk/METHODOLOGY-V2.md
      - docs/en/METHODOLOGY-V2.md
      - METHODOLOGY.md
      - tool/src/gate.rs
      - tool/src/check.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/green_birth_test.rs
  a-wave-can-be-called-off:
    implements:
      - a-started-wave-can-be-cancelled
    files:
      - docs/uk/METHODOLOGY-V2.md
      - docs/en/METHODOLOGY-V2.md
      - METHODOLOGY.md
      - tool/src/docs.rs
      - tool/src/check.rs
      - tool/src/close.rs
      - tool/src/status.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/called_off_test.rs
  the-write-tells-the-truth:
    implements:
      - a-write-that-lies-is-red
    files:
      - tool/src/rev.rs
      - tool/src/main.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/rev_write_test.rs
  the-closing-names-the-red-test:
    implements:
      - the-closing-says-what-failed
    files:
      - tool/src/close.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/closing_red_test.rs
  the-hook-answer-is-obeyed:
    implements:
      - the-answers-are-obeyed
    files:
      - tool/src/init.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/hooks_off_test.rs
  the-draft-mark-comes-off:
    chore: "рішення оператора 2026-09-04: методика v2 чинна — за нею живуть 36 хвиль, увесь інструмент і всі суди, а текст досі казав, що чинною лишається v1"
    files:
      - docs/uk/METHODOLOGY-V2.md
      - docs/en/METHODOLOGY-V2.md
      - METHODOLOGY.md
      - docs/uk/QUALITY.md
      - QUALITY.md
  the-first-implementation-is-buried:
    chore: "перша реалізація (keel.py, tests/ на Python, install.sh, що кличе python3) лежить поруч із чинною і не має жодного суду: два інструменти в одному репозиторії, і жоден не каже, який чинний"
    files:
      - install.sh
      - README.md
  the-briefing-pays-its-debt:
    chore: "борг доручення рецензентові: дванадцять рядків, названих рецензіями 0033 і 0034, не дописані двічі"
    files:
      - tool/src/review.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/briefing_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - keel/reviews/0037-the-last-of-it.md

decisions:
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "свідомо без тесту: нові суди читають git тією самою рукою scope::git_at, що й наявні"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "свідомо без тесту: кожна нова відмова називає файл, хвилю або команду поіменно"
  interaction.learnability: "свідомо без тесту: нових команд не додається — ті самі keel check, close, next"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "свідомо без тесту: суд слів проти коду (хвиля 0035) тримає підставлення в кожному новому повідомленні"
  reliability.availability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "свідомо без тесту: суд рецензії живе у close.rs біля станів хвилі, скасування — у docs.rs біля читання хвилі"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "свідомо без тесту: кожна знахідка називає параграф норми, з якого вона"
  maintainability.modifiability: "не застосовується"
  maintainability.testability: "свідомо без тесту: усі шість проб будують пісочниці спільною рукою 0030 і дають git власну особу (школа рецензії 0036 R-11)"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "тримає the-first-implementation-is-buried через chore: install.sh складає бінарник із джерел замість запускати python-файл, якого нема"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: кожен рядок цієї хвилі — знахідка аудиту або рецензії, названа числом"
  safety.fail-safe: "тримає every-wave-has-its-reviewer: суд додає заслон там, де досі його не було — жоден рядок хвилі не робить зеленого з червоного"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "свідомо без окремої роботи: суди суворішають, і чужий проєкт може почервоніти — кожна нова відмова тому й каже, що робити"
---

## Why

Остання хвиля цієї серії: два рішення оператора, чотири болі, що
лишились у черзі, і борг, не сплачений двічі.

**Два рішення оператора, 2026-09-04.**

**1. Усі хвилі — з рецензентами.** §9.9 казав «повна хвиля без цього
файлу не зливається», і рецензія 0036 зміряла наслідок: коли вагу
почали рахувати за нормою, хвиля з однією трансформою і обіцянкою
поїхала б **одним PR без рецензента**. Оператор вирішив інакше:
рецензія — кожній хвилі. Вага лишається питанням §6.8 (скільки PR),
але заслон рецензента більше від неї не залежить.

**2. Чернетку знято.** Методика v2 позначена «чернеткою» і каже, що
чинною лишається `METHODOLOGY.md` першої версії. За v2 живуть **36
закритих хвиль**, увесь інструмент і всі його суди; кореневий
`METHODOLOGY.md` уже став англійським текстом самої v2. Позначка —
неправда, і зняти її може тільки оператор (§8.6). Він це зробив.

**Що ще лишалось у черзі.**

**§6.3 дістає названий виняток.** Рішення оператора, записане ще
2026-09-04: суд над власною батареєю або інструментарієм **може
народитись зеленим**, але тоді в коміті трансформи має бути записаний
**мутант** — яка саме поломка внесена і як проба її назвала. Гарантія
лишається: довести, що тест уміє падати, все одно треба. А лазівка
«перейменуй роботу на chore» закривається: виняток названий, вузький і
коштує роботи. Межа сказана вголос: **машина мутанта не перевіряє**,
він живе текстом у коміті, тож тримає його чесність автора плюс око
рецензента.

**Скасування початої хвилі.** Другий бік того ж рішення. Хвиля, яку
почали й вирішили не робити, не має способу вмерти: обіцянки можна
зняти `withdrawn`, а саму хвилю — ні, і `keel close` довіку зватиме її
незакритою. Відкат злитої хвилі й швидкий шлях для термінового фікса
чекають далі; тут — тільки скасування початої.

**`rev --write` бреше кодом виходу.** Аудит багів B5: команда друкує
червоне, каже «нічого не дрейфує» і виходить **нулем**. CI бачить
нуль і їде далі.

**`keel close` мовчить про червоний тест.** Аудит багів B6: суд жене
батарею тричі, бачить червоне і не каже, **який саме** тест упав —
людина мусить гнати батарею вручну, щоб дізнатись те, що суд щойно
бачив.

**`keel init --no-hooks` не слухає відповіді.** Рецензія 0035 («не
цієї хвилі»): відповідь `hooks = false` лягає в `keel.toml`, а
`.git/hooks/commit-msg` пишеться однаково. Питання, чия відповідь
нічого не змінює, — не питання.

**Борг доручення рецензентові.** Дванадцять рядків, названих
рецензіями 0033 і 0034, не дописані **двічі**. Кожен із них —
знахідка, яку рецензент знайшов лише тому, що здогадався зробити те,
чого доручення не просило.

## scenario: every-wave-has-its-reviewer

**Дано** хвилю будь-якої ваги,
**коли** `keel close` судить її закриття,
**тоді** без файла `keel/reviews/<хвиля>.md` вона **не закрита**, і
причина названа поіменно. `keel next` веде до рецензії так само —
хвилі без обіцянок теж, бо і в них є що читати. Вага (§6.8) лишається
про кількість PR і про це більше нічого не вирішує.

## scenario: a-green-birth-is-named-and-proved

**Дано** трансформу, чия робота — суд над власною батареєю або
інструментарієм,
**коли** її обіцянка не має червоного народження,
**тоді** commit проходить, **якщо** його повідомлення несе рядок
`mutant: <що зламано> -> <як проба це назвала>`; без такого рядка —
відмова, як і досі. `keel check` каже про такий виняток вголос
окремим рядком: він названий, а не мовчазний. Машина не перевіряє, що
мутант справжній, і каже це прямо.

## scenario: a-started-wave-can-be-cancelled

**Дано** почату хвилю, яку вирішили не робити,
**коли** її файл дістає в шапці `cancelled: <причина>`,
**тоді** `keel close` більше не зве її незакритою, `keel status` каже
«скасована» з причиною, а `keel check` не судить ані її обіцянок, ані
її scope. Скасована хвиля не зникає з репозиторію: причина лишається
читаною, як і зняття обіцянки (§2.12).

## scenario: a-write-that-lies-is-red

**Дано** `keel rev --write` на проєкті, де редакції розійшлися,
**коли** команда відпрацювала,
**тоді** код виходу каже те саме, що й текст: нуль лише тоді, коли
писати не було чого або запис удався цілком; будь-яка знахідка — не
нуль. Слова «нічого не дрейфує» не друкуються там, де щойно надруковано
червоне.

## scenario: the-closing-says-what-failed

**Дано** проєкт, чия батарея червона,
**коли** `keel close` жене її,
**тоді** він називає **кожен** упалий тест поіменно і каже, з якого
біга — не «батарея червона», а що саме. Те, що суд уже бачив, він не
ховає від людини.

## scenario: the-answers-are-obeyed

**Дано** `keel init --no-hooks` (або `hooks = false` у відповіді),
**коли** він відпрацював,
**тоді** `.git/hooks/commit-msg` **не написаний**, і звіт каже про це
рядком. Наявний чужий hook не чіпається ніколи; наш власний, якщо він
уже стояв, лишається з чесним словом, що його не прибрано.

## transform: the-reviewer-is-not-optional

§9.9 в обох текстах: заслон рецензента більше не залежить від ваги.
`close.rs` і `next.rs` питають наявність звіту для будь-якої хвилі.

## transform: a-green-birth-carries-its-mutant

§6.3 в обох текстах дістає названий виняток із мутантом у коміті;
`gate.rs` пускає такий commit, `check.rs` каже про виняток вголос.

## transform: a-wave-can-be-called-off

Поле `cancelled` у шапці хвилі: `docs.rs` його читає, `check.rs`,
`close.rs` і `status.rs` його слухають. §6 в обох текстах.

## transform: the-write-tells-the-truth

`rev.rs` віддає число знахідок разом зі звітом; `main.rs` робить із
нього код виходу.

## transform: the-closing-names-the-red-test

`close.rs` збирає імена впалих тестів із виводу батареї і називає їх.

## transform: the-hook-answer-is-obeyed

`init.rs` слухає `hooks = false` і для git-гачка, не тільки для
агентських.

## transform: the-draft-mark-comes-off

Позначку «чернетка» знято з методики v2 і з чекліста в обох мовах:
рішення оператора (§8.6). Кореневий `METHODOLOGY.md` іде за ними.

## transform: the-first-implementation-is-buried

`install.sh` складає бінарник із джерел замість запускати
`keel.py`, якого вже нема; README каже те саме. Сама перша реалізація
лишається в історії git — прибирання її файлів чекає окремої хвилі,
бо це видалення, а не заміна.

## transform: the-briefing-pays-its-debt

Дванадцять рядків боргу лягають у доручення, яке збирає `keel review`.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10). BACKLOG
втрачає рядки, які ця хвиля закриває. Сюди ж лягає звіт рецензії.

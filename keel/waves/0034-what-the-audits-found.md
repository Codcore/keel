---
depends_on: [0033-plain-words-and-a-true-number]

scenarios:
  setup-never-breaks-what-it-edits:
    covers: [reliability.recoverability, interaction.user-error-protection]
  a-court-that-cannot-fail-is-not-a-court:
    covers: [maintainability.testability]
  the-red-birth-is-judged-by-the-branch:
    covers: [security.non-repudiation, reliability.faultlessness]
  the-vocabulary-cannot-drift-from-the-norm:
    covers: [functional.correctness, maintainability.analysability]

transforms:
  the-wizard-does-no-harm:
    implements:
      - setup-never-breaks-what-it-edits
    files:
      - tool/src/ask.rs
      - tool/src/init.rs
      - tool/src/confedit.rs
      - tool/tests/setup_test.rs
      - tool/tests/plain_words_test.rs
  every-assert-can-fail:
    implements:
      - a-court-that-cannot-fail-is-not-a-court
    files:
      - tool/tests/dead_assert_test.rs
  the-branch-remembers-the-red:
    implements:
      - the-red-birth-is-judged-by-the-branch
    files:
      - tool/src/check.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/red_birth_test.rs
  the-vocabulary-says-what-the-norm-says:
    implements:
      - the-vocabulary-cannot-drift-from-the-norm
    files:
      - QUALITY.md
      - docs/uk/QUALITY.md
      - docs/uk/METHODOLOGY-V2.md
      - docs/en/METHODOLOGY-V2.md
      - tool/tests/vocabulary_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md

decisions:
  functional.completeness: "тримає всі чотири сценарії; решта знахідок трьох аудитів названа поіменно в BACKLOG, а не змовчана — їх чотирнадцять, і кожна окрема робота"
  functional.appropriateness: "свідомо без тесту: доречність зміряли три незалежні аудити і одна рецензія; дві найважчі знахідки збіглися у двох аудитах"
  performance.time-behaviour: "свідомо без тесту: суд червоного народження читає імена коммітів гілки — один git log на вирок"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "не застосовується"
  compatibility.interoperability: "свідомо без тесту: нічого нового поза git"
  interaction.appropriateness-recognisability: "свідомо без тесту: нова відмова каже імʼя сценарію і те, що бракує саме комміту red:"
  interaction.learnability: "не застосовується"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "не застосовується"
  interaction.user-assistance: "свідомо без тесту: кожна нова відмова несе «натомість»"
  reliability.fault-tolerance: "тримає setup-never-breaks-what-it-edits: конфіг, якого не прочитати, спиняє команду, а не переписується наосліп"
  reliability.availability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "тримає setup-never-breaks-what-it-edits: записи довіри — рішення людини (§7.16), і майстер їх не викидає"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "не застосовується"
  maintainability.reusability: "не застосовується"
  maintainability.modifiability: "не застосовується"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: загрози названі числами трьох аудитів"
  safety.fail-safe: "тримає setup-never-breaks-what-it-edits: там, де майстер не певен, він не пише"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "тримає the-red-birth-is-judged-by-the-branch: суд додається, наявні не слабшають; хвилі, закриті до цієї, лишаються закритими — інакше вирок став би червоним на власній історії, і це названо в тілі сценарію"
---

## Why

Три незалежні аудити (баги, код проти методики, дірки в самій нормі)
і одна рецензія. Разом — **шістнадцять важких знахідок**. Дві
найважчі збіглися у двох аудитах незалежно, і це саме ті, що йдуть у
цю хвилю. Решта — чотирнадцять — названа поіменно в BACKLOG: кожна
окрема робота, і скласти їх усі в одну хвилю означало б повторити
рівно ту ваду, яку аудити й знайшли, — обіцянку, якої не тримає
проба.

**1. `keel setup` цеглить проєкт.** Здоровий проєкт, один `keel
setup` — і `agents = []`, значення, яке інструмент **сам же**
відмовляє. Далі не працює нічого, зокрема сам `setup`. Причина моя:
взято сире поле `config.agents` замість `config.agents()`, який дає
типове. Заразом майстер стирає записи довіри до `verify`-команд
контрактів — це я вніс, лагодячи R-10 рецензії 0032, — і переписує
файл цілком, гублячи рукописні коментарі.

**2. Асерт, який не може впасти.** Хвиля 0033 перейменувала рядок і
лишила сусідній асерт шукати слова, яких більше нема. Він тримав
знахідку рецензії 0031 — і перестав тримати будь-що, мовчки.
Рецензент довів мутантом. Це **клас**, а не випадок: за дві хвилі він
виявився важким двічі.

**3. «Бачив червоним» не тримається нічим.** Несуча ідея методики —
«зелений тест, якого не бачили червоним, не доводить нічого» (§6.3).
§7.12 називає двох тримачів: гук і **перевірку гілки**. Перевірки
гілки **не існує**. Аудит довів це наскрізь у свіжому клоні без гука:

    git commit -m "the-probe: work with no test and no red commit"
    keel check .  →  0 знахідок
    keel close .  →  закрита — кожен живий сценарій доведений

А `.git/hooks/` не їде з git, тож кожен новий агент у новому клоні
працює без захисту. Рішення оператора: **зробити перевірку гілки**.

**4. Словник суперечить нормі.** §3.4 називає `QUALITY.md` словником
розрізів, а він каже «один прохід» проти §10.2 «двічі» і «одна з
**трьох** відповідей» проти §10.3 «третьої нема, тиша заборонена».
Машина цього не бачить: звіряються сорок слагів, проза — ніколи.
Рішення оператора: **правити словник під методику**. Заразом §2.4
«**рівно** один commit на трансформу» стає «щонайменше один» — теж
рішення оператора: правило порушується щохвилі, зокрема самою
розсилкою рецензій.

## scenario: setup-never-breaks-what-it-edits

**Дано** проєкт із написаним `keel.toml`,
**коли** кличуть `keel setup`,
**тоді** конфіг після нього **читається інструментом** — жодного
`agents = []`; записи довіри до **всіх** живих команд лишаються;
рукописні коментарі лишаються; а конфіг, якого прочитати не вдалося,
**спиняє команду**, а не переписується наосліп.

## scenario: a-court-that-cannot-fail-is-not-a-court

**Дано** батарею,
**коли** її читають,
**тоді** жоден асерт не шукає рядка, якого нема в жодному рядку
інструмента: асерт, що шукає відсутній літерал, зелений завжди, і це
не суд, а слова. Судиться самим текстом проб проти текстів
інструмента.

## scenario: the-red-birth-is-judged-by-the-branch

**Дано** гілку хвилі,
**коли** `keel check` судить її,
**тоді** в кожного доведеного сценарію на гілці є свій комміт
`red:` — і якщо нема, це знахідка, не порада. Суд не залежить від
гука: він читає імена коммітів гілки проти бази порівняння, отже
працює у свіжому клоні і в CI. Межа названа: історія, обрізана так,
що бази не видно, дає «не перевірено», а не хибне зелене. І
**хвилі, закриті до цієї**, судові не підлягають: їхні гілки давно
злиті, а вимагати червоного від історії, якої вже нема, означало б
почервонити вирок на власному минулому.

## scenario: the-vocabulary-cannot-drift-from-the-norm

**Дано** `QUALITY.md` і методику,
**коли** вони кажуть про одне,
**тоді** вони кажуть **те саме**: два проходи двома головами, дві
відповіді, тиша заборонена. Розбіжність — знахідка, а не мовчання;
судиться машиною, бо саме мовчання машини й дало словникові
відстати.

## transform: the-wizard-does-no-harm

`ask.rs` бере типові значення через власні читалки конфігу; `init.rs`
несе довіру всіх живих команд і спиняється на непрочитаному конфізі.

## transform: every-assert-can-fail

Проба читає всі `tests/*.rs`, витягає літерали з асертів і вимагає,
щоб кожен зустрічався в текстах інструмента.

## transform: the-branch-remembers-the-red

`check.rs` читає імена коммітів гілки і судить народження червоного.

## transform: the-vocabulary-says-what-the-norm-says

`QUALITY.md` (обидві мови) приводиться до глави 10; §2.4 методики
править «рівно» на «щонайменше»; проба звіряє твердження словника з
методикою.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10). BACKLOG несе
чотирнадцять знахідок, які ця хвиля свідомо не бере.

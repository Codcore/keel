---
depends_on: [0031-what-the-verdict-covers]

scenarios:
  the-reviewer-is-briefed-by-the-tool:
    covers: [interaction.user-assistance, maintainability.analysability]
  the-wizard-asks-what-a-project-needs:
    covers: [functional.completeness, interaction.learnability]
  answers-can-be-changed-after-init:
    covers: [flexibility.adaptability, interaction.operability]
  the-mouth-serves-every-text-it-carries:
    covers: [functional.appropriateness]
  the-texts-say-when-they-were-taken:
    covers: [interaction.self-descriptiveness]
  the-source-of-truth-is-a-numbered-rule:
    covers: [functional.correctness]

transforms:
  the-briefing-in-the-binary:
    implements:
      - the-reviewer-is-briefed-by-the-tool
    files:
      - tool/src/review.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/briefing_test.rs
  the-wizard-and-the-second-chance:
    implements:
      - the-wizard-asks-what-a-project-needs
      - answers-can-be-changed-after-init
    files:
      - tool/src/ask.rs
      - tool/src/init.rs
      - tool/src/main.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/setup_test.rs
  the-mouth-carries-all-it-holds:
    implements:
      - the-mouth-serves-every-text-it-carries
      - the-texts-say-when-they-were-taken
    files:
      - tool/src/speak.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/carried_texts_test.rs
  the-source-of-truth-numbered:
    implements:
      - the-source-of-truth-is-a-numbered-rule
    files:
      - docs/uk/METHODOLOGY-V2.md
      - docs/en/METHODOLOGY-V2.md
      - tool/tests/source_of_truth_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md

decisions:
  performance.time-behaviour: "свідомо без тесту: доручення — сталий текст у бінарнику; майстер питає стільки ж разів, скільки й питав"
  performance.capacity: "свідомо без тесту: доручення ~4 КБ на мову"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "не застосовується"
  compatibility.interoperability: "свідомо без тесту: keel setup пише той самий keel.toml, що й init — інших форматів не зʼявляється"
  interaction.appropriateness-recognisability: "свідомо без тесту: доручення йде під власним заголовком у кінці пакета, після даних, — рецензент читає його останнім і памʼятає найкраще"
  interaction.user-error-protection: "тримає answers-can-be-changed-after-init: keel setup показує чинну відповідь як типову і НЕ чіпає того, чого не питали — конфіг не гіршає від запуску"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  reliability.faultlessness: "свідомо без тесту: жоден наявний суд не міняється — додаються текст, питання і другий рот"
  reliability.fault-tolerance: "тримає answers-can-be-changed-after-init: setup на проєкті без keel.toml поводиться як init, а не падає"
  reliability.availability: "не застосовується"
  reliability.recoverability: "свідомо без тесту: setup переписує файл цілком з відповідей, тож зіпсований конфіг лікується повторним запуском"
  security.confidentiality: "не застосовується"
  security.integrity: "тримає the-source-of-truth-is-a-numbered-rule: правило про джерело правди стає нумерованим параграфом, тобто його редакцію можна тримати і на нього можна послатись — сьогодні воно єдине без номера"
  security.non-repudiation: "тримає the-texts-say-when-they-were-taken: рот каже, що тексти взяті в мить збірки бінарника, а не читаються з диска — інакше він тихо подає вчорашнє за сьогоднішнє"
  security.accountability: "тримає the-reviewer-is-briefed-by-the-tool: доручення в git означає, що кожен рецензент дістав ТЕ САМЕ — сьогодні вони дістають різне, і рецензент 0026 знищив 10128 чужих тек саме тому, що заборони в його дорученні не було"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "свідомо без тесту: доручення — рядки i18n, як методика і розрізи"
  maintainability.reusability: "свідомо без тесту: setup і init беруть той самий ask::Answers"
  maintainability.modifiability: "тримає the-reviewer-is-briefed-by-the-tool: доручення правиться хвилею і судиться, як усе інше — а не переписується щоразу в чаті"
  maintainability.testability: "тримає the-reviewer-is-briefed-by-the-tool: проба вимагає, щоб доручення називало лише команди, які інструмент справді має, і лише параграфи, які в методиці є — школа 0024, де блок радив те, чого інструмент не вміє"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "тримає the-wizard-asks-what-a-project-needs: version, ci і trust — саме те, що робить проєкт готовим до воріт, і саме те, чого майстер не питав"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "тримає the-reviewer-is-briefed-by-the-tool: гігієна рецензента (власний клон, власний target, ЧУЖОГО НЕ ЧІПАТИ) стає частиною релізу, а не памʼяті автора"
  safety.risk-identification: "свідомо без окремої роботи: загроза названа числом — 10128 чужих тек, знищених рецензентом, у якого тієї заборони не було"
  safety.fail-safe: "не застосовується"
  safety.hazard-warning: "тримає the-reviewer-is-briefed-by-the-tool: доручення несе заборони першими, а не наприкінці"
  safety.safe-integration: "тримає the-source-of-truth-is-a-numbered-rule: новий параграф зʼявляється в ОБОХ методиках однаково, тож суд кістяка (§9.9 хвилі 0029) лишається зеленим — і саме він це і доводить"
---

## Why

Останні шість рядків черги, однією хвилею — рішення оператора після
того, як я порадив дві і назвав ризик. Ризик приймається так: **по
окремому сценарію на кожну обіцянку**, щоб проби не злиплися в одну
розмиту.

**1. Доручення рецензентові живе тільки в чаті.** `keel review` подає
**пакет** (Why, сценарії з редакціями, дрейф §4.6, мапа якості, вплив
§5.7, повний diff) — і жодного слова про те, **що з ним робити**.
Гігієну, контрфакти, чотири питання, вимоги до звіту автор пише
руками щоразу заново. Ціна названа числом: рецензент 0026 знищив
**10 128 чужих тек**, бо заборони «чужого не чіпати» в його дорученні
не було — її дописано лише в наступне. Це конституція, п. 8: усе
потрібне для продовження має лежати в git.

**2 і 3. Майстер `keel init`** не питає `version`, `ci` і `[trust]` —
тобто саме того, що робить проєкт готовим до воріт. А **змінити
відповіді після init нема чим**: `keel.toml` правиться руками або не
правиться зовсім.

**4. `keel method` не подає `NEW-CONCEPT.md`** — текст, який бінарник
несе, але ротом не віддає.

**5. `include_str!` бере текст у мить збірки.** Це сказано у виводі,
але не в коді: рот подає те, що було на диску, коли бінарник
збирали, — і мовчить про це.

**6. Джерело правди — єдине правило перекладу без номера.** Питання
оператора з рецензії 0029 (П-2): сьогодні найважливіше правило стоїть
блок-цитатою над §1.1, тож `keel method §1.1` подає лише половину, і
послатися на нього нема як. Оператор сказав «закінчуй усе в одній
хвилі» — я читаю це як відповідь «так» і роблю; це **зміна норми**, і
якщо прочитано хибно, її знімає один revert.

## scenario: the-reviewer-is-briefed-by-the-tool

**Дано** `keel review` на гілці хвилі,
**коли** пакет зібрано,
**тоді** він несе **доручення**: гігієну (власний клон, не обрізаний,
власний target, **чужого не чіпати**), три обовʼязкові списки, чотири
питання §9.9, вимогу «числа тільки з бігів» і форму звіту
`keel/reviews/<хвиля>.md`. Заборони йдуть першими. Доручення
**судиться**: воно називає лише команди, які інструмент справді має,
і лише параграфи, які в методиці є.

## scenario: the-wizard-asks-what-a-project-needs

**Дано** `keel init` у порожньому проєкті,
**коли** майстер питає,
**тоді** серед питань є `version`, `ci` і `[trust]`, і відповіді
лягають у `keel.toml` у тій самій формі, яку читає `config`.

## scenario: answers-can-be-changed-after-init

**Дано** проєкт із уже написаним `keel.toml`,
**коли** кличуть `keel setup`,
**тоді** майстер показує **чинні** відповіді як типові, переписує
файл з нових і **не чіпає того, чого не питав**. На проєкті без
`keel.toml` він поводиться як `init`, а не падає.

## scenario: the-mouth-serves-every-text-it-carries

**Дано** бінарник, що несе методику, розрізи, чеклист і
`NEW-CONCEPT.md`,
**коли** питають рот,
**тоді** він віддає **кожен** із них, і жоден текст не лишається
несказаним. Це судиться списком, а не окремою згадкою: новий текст у
бінарнику без рота — відмова.

## scenario: the-texts-say-when-they-were-taken

**Дано** будь-який текст, що його подає рот,
**коли** його подано,
**тоді** сказано, що він **узятий у мить збірки бінарника**, а не
прочитаний із диска зараз, — разом із редакцією, з якої взято.

## scenario: the-source-of-truth-is-a-numbered-rule

**Дано** обидві методики,
**коли** читають главу 1,
**тоді** правило про джерело правди — **нумерований параграф**, а не
блок-цитата: його подає `keel method §1.8`, на нього можна послатись,
і суд кістяка бачить його в обох мовах однаково.

## transform: the-briefing-in-the-binary

Доручення рядками i18n у кінці пакета `review.rs`, і проба, що звіряє
його з тим, що інструмент справді вміє.

## transform: the-wizard-and-the-second-chance

`ask.rs` дістає три питання, `init.rs` їх ставить, `main.rs` вчиться
слову `setup`.

## transform: the-mouth-carries-all-it-holds

`speak.rs` віддає всі несені тексти і каже, коли їх узято.

## transform: the-source-of-truth-numbered

Новий §1.8 в обох методиках, однаковий за кістяком.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10).

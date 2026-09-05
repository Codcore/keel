---
depends_on: [0042-the-third-tongue]

scenarios:
  a-red-test-holds-the-wave-open:
    covers: [functional.correctness, safety.fail-safe]
  a-promise-alive-only-in-prose-does-not-hold:
    covers: [functional.appropriateness, reliability.faultlessness]

transforms:
  the-red-is-a-blocker:
    implements:
      - a-red-test-holds-the-wave-open
    files:
      - tool/src/close.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-close.md
      - tool/tests/close_red_test.rs
  prose-is-not-code:
    implements:
      - a-promise-alive-only-in-prose-does-not-hold
    files:
      - tool/src/holding.rs
      - keel/contracts/tool-holding.md
      - tool/tests/holding_prose_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0043-the-court-that-saw-red.md

decisions:
  functional.completeness: "свідомо без окремого сценарію: ця хвиля нічого не додає до переліку того, що суд уміє — вона робить обовʼязковим те, що він уже бачив і мовчки пропускав"
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "свідомо без тесту: батарея вже жене тричі (§7.13); ця хвиля читає її вирок, а не додає бігів"
  compatibility.co-existence: "тримає a-red-test-holds-the-wave-open: суд стає суворішим для всіх трьох мов однаково — жодна не дістає власного винятку"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "не застосовується"
  interaction.learnability: "свідомо без тесту: команд не додається"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає a-red-test-holds-the-wave-open: людина більше не може закрити хвилю над червоним, навіть не помітивши рядка про нього"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "тримає a-red-test-holds-the-wave-open: вирок називає не лише ЩО червоне, а й що саме через це не закривається — рядок про червоний тест перестає бути тільки прозою"
  interaction.user-assistance: "свідомо без тесту: нова відмова несе «натомість» — суд слів проти коду (хвиля 0035) тримає підставлення"
  reliability.fault-tolerance: "тримає a-red-test-holds-the-wave-open: суд, який бачив червоне і закрив, — гірший за суд, який не біг; напрям виправлення не робить із червоного зеленого ніде"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "не застосовується"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "тримає a-promise-alive-only-in-prose-does-not-hold: рядки й heredoc-и знає зрізач у holding, один на всі мови — не три гілки в трьох адаптерах"
  maintainability.reusability: "тримає a-promise-alive-only-in-prose-does-not-hold: одна рука на rust, ruby й elixir, бо ховати код у тексті всі три вміють однаково"
  maintainability.modifiability: "не застосовується"
  maintainability.analysability: "тримає a-red-test-holds-the-wave-open: хто читає вирок, бачить причину незакриття поруч із її іменем"
  maintainability.testability: "свідомо без тесту: проби будують справжні проєкти спільною рукою 0030 і жеуть справжні бігуни"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  flexibility.adaptability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий числом — обидві вади зміряно в УСІХ трьох мовах, тобто це не мовна дірка, а дірка суду"
  safety.hazard-warning: "тримає a-red-test-holds-the-wave-open: попередження, за яким нічого не стоїть, — це не попередження"
  safety.safe-integration: "тримає a-promise-alive-only-in-prose-does-not-hold: наявні контракти keel судяться так само — 103 сигнатури мусять лишитись зеленими, і батарея це міряє"
---

## Why

Дві вади, обидві **знайдені рецензією 0042**, обидві записані в
чергу з виміром — і обидві виявились **не про elixir**. Я зміряв
кожну в **усіх трьох мовах**, і саме тому вони тут окремою хвилею, а
не латкою в мовній: це дірки **суду**, а не адаптера.

**Перша, і вона найгірша, яку цей інструмент мав.** Суд закриття жене
батарею тричі, **бачить червоний тест**, називає його вголос — і
закриває хвилю:

```
  червоний тест: nobody_claims_me (toy_test) — падав у кожному бігу
0001-a-wave: закрита ...
блокерів нема ...                                              rc=0
```

Зміряно однаково в rust (`nobody_claims_me`), ruby
(`test_nobody_claims_me`) і elixir (`Toy.works/0 (1)` — той самий
`doctest`, з якого почалась рецензія 0042). Причина в коді одна:
`fell` збирається, друкується — і **нікуди не йде**. Блокери
рахуються лише з `State::Progress(lacks)` **власної** хвилі гілки,
тобто лише з ненакритих обіцянок. Червоний тест, якого не заявляє
жоден сценарій, не стає ніяким `lack`, і вирок його не помічає.

Це гірше за суд, який не біг. Суд, який не біг, каже «не судив» —
і людина знає, що не знає. Суд, який побачив червоне і вийшов з 0,
**видає себе за прочитане** рівно в тому місці, де методика (§7.8)
обіцяє, що зелене означає «існує, збігається і **проходить**».

**Друга: обіцянка, жива лише в прозі, тримає форму.** §7.6 звіряє
`exports` контракту з сирцем модуля, а зрізач коментарів ріже
**рядкові** коментарі і про багаторядкові рядки не знає. Тож
оголошення, написане всередині тексту, стоїть за живе. Зміряно втрьох:

| мова | де живе привид | вирок |
|---|---|---|
| elixir | `@moduledoc """ … def ghost(a, b) … """` | сигнатур звірено: 1, **0 знахідок** |
| ruby | `DOC = <<~TEXT … def ghost(a, b) … TEXT` | сигнатур звірено: 1, **0 знахідок** |
| rust | `const SHAPE: &str = r#"… pub fn phantom(b: u8) -> u8 …"#;` | сигнатур звірено: 1, **0 знахідок** |

І це не куток: `@doc`/`@moduledoc` із прикладом коду — **центральна
ідіома Elixir**, а приклад пишуть мовою, яку документують. Хвиля 0042
вже мусила навчити цього **читача тегів** (M28, дограна після
рецензії). Суд форми лишився з тією ж сліпотою.

**Чому однією хвилею.** Це один і той самий гріх у двох судах:
**зелене, яке нічого не означає**. Перший каже «доведено» над тим, що
падало; другий — «тримає» над тим, чого нема. Обидва зміряні в трьох
мовах, обидва правляться в одному місці кожен, і обидва перевіряються
тим самим питанням: чи стало **червоне червоним**.

## scenario: a-red-test-holds-the-wave-open

**Дано** проєкт, чия батарея має тест, що падає в кожному з трьох
бігів, і жоден сценарій жодної хвилі його не заявляє.
**Коли** біжить `keel close`.
**Тоді** вирок називає цей тест **і не закриває**: рядок про червоне
стає блокером, `rc` не нульовий, і слова «закрита» серед вироків
хвиль немає.

Це тримається **в усіх трьох мовах** — rust, ruby, elixir, — бо дірка
не мовна. «Хиткий» тест (падав не в кожному бігу) блокує так само:
§7.13 жене тричі саме для того, щоб хиткість було видно, і хиткий
тест — це не зелений тест.

Тег над іменем, якого бігун не знає, лишається «не бігло» (§7.12) і
не стає через цю хвилю червоним: не бігло — це не впало.

## scenario: a-promise-alive-only-in-prose-does-not-hold

**Дано** контракт, чий `exports` називає оголошення, яке в сирці
модуля існує **лише** всередині багаторядкового тексту — elixir-ового
`"""`, ruby-ового heredoc-а або rust-ового `r#"…"#`.
**Коли** біжить `keel check`.
**Тоді** контракт **не тримається**: знахідка каже, що оголошення не
знайдено, і називає файл, у якому шукали.

І зворотний бік, який мусить лишитись цілим: усі **103** сигнатури
самого keel зостаються зеленими, бо в них немає привидів — інакше це
не суворіший суд, а зламаний.

## transform: the-red-is-a-blocker

`fell` перестає бути прозою. Червоний (і хиткий) тест, який батарея
бачила, — блокер сам собою, незалежно від того, чи заявляє його
сценарій і чи зветься гілка як хвиля. Вирок каже, скільки їх і що
саме через них не закривається; словник дістає рядок обома мовами.
Контракт `tool-close.md` називає нове правило і його бік: сумнів іде
**в бік незакриття**.

## transform: prose-is-not-code

Зрізач у `holding.rs` вчиться багаторядкових текстів: elixir-ове
`"""`, ruby-ові `<<~`/`<<-`/`<<` і rust-ове `r"…"`/`r#"…"#`. Те, що
всередині, — не сирець. Контракт `tool-holding.md` перестає називати
межею лише «`#` усередині heredoc зрізається» і каже правило цілком.

## transform: journal

Записи журналу цього покоління їдуть із хвилею (V2-PROCESS), разом із
чергою, README і звітом рецензії.

---
depends_on: [0056-the-first-release]

scenarios:
  declared-and-touched-are-one-file:
    covers: [functional.correctness, interaction.user-error-protection]
  the-tongue-names-what-its-runner-leaves:
    covers: [functional.completeness, interaction.user-assistance]
  the-advice-leads-with-the-road-that-works:
    covers: [interaction.learnability, flexibility.installability]

transforms:
  one-name-for-one-file:
    implements: [declared-and-touched-are-one-file]
    files:
      - tool/src/scope.rs
      - tool/src/docs.rs
      - tool/src/check.rs
      - tool/src/next.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - tool/tests/scope_path_test.rs
      - keel/contracts/tool-scope.md
  the-tongue-names-its-leavings:
    implements: [the-tongue-names-what-its-runner-leaves]
    files:
      - tool/src/adapter.rs
      - tool/src/scope.rs
      - tool/src/init.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-scope.md
      - keel/contracts/tool-adapter-python.md
      - keel/contracts/tool-adapter-javascript.md
      - keel/contracts/tool-adapter-ruby.md
      - tool/tests/leavings_test.rs
  the-advice-leads-with-the-release:
    implements: [the-advice-leads-with-the-road-that-works]
    files:
      - tool/src/version.rs
      - tool/i18n/uk.ftl
      - tool/i18n/en.ftl
      - keel/contracts/tool-version.md
      - tool/tests/pin_hand_test.rs
  journal:
    chore: "запис хвилі: звіт свіжого рецензента, стаття журналу і зняті рядки черги (§4.11)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - keel/reviews/0057-the-paths-the-furniture-and-the-advice.md

decisions:
  functional.appropriateness: "не застосовується"
  performance.time-behaviour: "названо: перелік лишків питається РАЗ на порівняння, як і lock-файли (рецензія 0055 R-13: питання про мову коштує читання маніфесту і обходу теки, і `keel check` було сповільнився на 3–7%)"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "не застосовується"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "тримає ця хвиля: у відмові лишається САМЕ той рядок, який людина написала (`./src/a.rs`), а не його нормалізований двійник — інакше вона не знайде його очима у своєму файлі"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "не застосовується"
  reliability.faultlessness: "не застосовується"
  reliability.fault-tolerance: "не застосовується"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "не застосовується"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "тримає: перелік лишків живе в адаптері (`adapter::leavings`), поруч із lock-файлами і текою збірки — суд scope мов не знає і знати не мусить (хвиля 0038 R-5)"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "не застосовується"
  maintainability.modifiability: "тримає: нова мова додає свій рядок до одного переліку — суд scope і рада init читають той самий"
  maintainability.testability: "не застосовується"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "названо межу: `..` у рядку scope хвиля НЕ узаконює — нормалізація прибирає `./` і подвійні скісні, а шлях, що виходить за корінь, лишається відмовою, бо файл поза деревом не є файлом хвилі"
  safety.fail-safe: "не застосовується"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "названо: лишки мови стають меблями лише для суду scope; вони не додаються до .gitignore рукою інструмента — рама радить рядок, файлів проєкту не пише (правило хвилі 0045, тримається)"
---

## Why

Черга після 0055 і 0056: три вади, кожну зміряно в чужому проєкті на
цьому ж бінарнику (`keel 1.0.0`). Усі три кусають на **першому дні** —
там, де людина ще не знає інструмента і читає його вивід буквально.

**1. Один файл, дві знахідки-дзеркала (черга: баги R-22).** Хвиля
оголошує `./src/a.rs`, гілка чіпає `src/a.rs` — той самий файл. Зміряно
автором у пісочниці:

```
червоне  гілка чіпає "src/a.rs", якого жодна трансформа хвилі не називає
червоне  оголошений файл "./src/a.rs" гілка не чіпає
```

Дві відмови про один файл, і жодна не каже, що вони про один файл.
Рядки scope порівнюються як голі рядки (`declared: BTreeSet<&str>`
проти імен, які дає git), тож `./` — уже інший файл. Людина пише шлях
так, як звикла; інструмент відповідає їй загадкою.

**2. «Ця мова нічого не збирає» — неправда для трьох мов із пʼяти
(черга: баги R-21).** Зміряно автором: `keel init --adapter python`
каже «правила ігнорування: ця мова нічого не збирає, тож теки збірки,
яку варто було б ігнорувати, нема». Те саме слово дістають javascript і
ruby. А `pytest` лишає `__pycache__/` і `.pytest_cache/`, node —
`node_modules/`, rspec — `.rspec_status`. Далі зміряно: коли ці теки
потрапляють у коміт (а стороння людина комітить `git add -A`, бо рама
сама сказала, що ігнорувати нема чого), суд scope зве їх дрейфом:

```
червоне  гілка чіпає "tests/__pycache__/test_calc.cpython-311-pytest-9.0.2.pyc",
         якого жодна трансформа хвилі не називає
```

Elixir тут поводиться правильно — називає `_build/` і дає точний рядок
для `.gitignore`; решта мов дістала «нічого не збирає», бо `build_dir`
знає лише ОДНУ теку збірки, а лишки бігуна — це кілька шляхів.

**3. Порада піна веде довшою дорогою (черга: баги R-26).** Зміряно
автором над проєктом із піном `0.9.0`:

```
взяти саме її: KEEL_REF="0.9.0" sh install.sh — або curl … | sh -s -- 0.9.0
```

Перша дорога — збірка з джерела за git-ref-ом, де checksum не звіряє
ніхто. Після релізу v1.0.0 у числа піна є **своя** дорога: опублікований
архів і його `.sha256`, без git і без cargo (зміряно автором: над
`file://`-релізом launcher докачав, звірив sha256 і побіг, exit 0).
Лампа мусить вести нею першою, а `KEEL_REF` лишити тому, що релізу не
має.

**Що НЕ входить у хвилю.** Борг двох контрактів після релізу (рецензія
0056 R-2, R-3) — окрема хвиля: він упирається в суперечність самої
норми (§2.11 велить хвилі з самих chore бути легкою, §6.8 велить хвилі,
що змінює контракт, бути повною — а той борг є і те, й те), і рядок
норми — рішення оператора, не автора.

## scenario: declared-and-touched-are-one-file

**Дано** хвилю, чия трансформа оголошує файл рядком `./src/a.rs`, і
гілку, яка чіпає `src/a.rs`, **коли** біжить `keel check`, **тоді**
знахідок про цей файл нема жодної: обидва боки порівнюються
нормалізованими (прибрано `./`, подвійні скісні і сегменти `.`), а
шлях, що виходить за корінь дерева (`..`), лишається відмовою. У словах
відмови, які таки лишаються, стоїть рядок, як його написала людина.

## scenario: the-tongue-names-what-its-runner-leaves

**Дано** проєкт python (те саме для javascript і ruby), **коли** біжить
`keel init` і потім суд scope над гілкою, у якій лишки бігуна потрапили
в коміт, **тоді** рада ignore називає ці шляхи поіменно і дає точний
рядок для `.gitignore` (як уже робить elixir), а суд scope зве їх
меблями, а не дрейфом — так само, як зве lock-файл, який бігун написав
без прохання.

## scenario: the-advice-leads-with-the-road-that-works

**Дано** проєкт, чий пін — число версії, якого цей бінарник не
відповідає, **коли** біжить `keel version`, **тоді** перша порада —
дорога релізу (`sh install.sh <пін>`: архів і його `.sha256`, без git і
cargo), а `KEEL_REF` стоїть другим і названий тим, чим він є: збіркою з
джерела за іменем ref-а. Пін, що не має форми версії, лишає першою
дорогу `KEEL_REF`, бо релізу під нього не буває.

## transform: one-name-for-one-file

`scope.rs`: обидва боки порівняння — оголошені рядки і імена від git —
проходять одну нормалізацію; у словах відмови лишається оригінальний
рядок. Проба `scope_path_test.rs` грає рівно ту пісочницю, яку зміряно:
`./src/a.rs` проти `src/a.rs`, і поруч — `..`, який лишається відмовою.
Контракт `tool-scope.md` дістає рядок про нормалізацію і про її межу.

**Дрейф (§4.6), названий тут.** Нормалізація мусить жити в одному
місці, і це `docs::one_name` поруч із самим рядком scope, а не в
`scope.rs`: рядки порівнюють ЧОТИРИ суди — scope, «трансформа
зібрана» (`check.rs`), крок §9.2 (`next.rs`) і вивід ваги
(`docs.rs`, §6.8). Останній був окремою дірою: `./keel/contracts/x.md`
проносив контракт повз правило повної ваги. Тому в files трансформи
стоять `docs.rs`, `check.rs`, `next.rs` і обидві мови (`scope-outside`
— нові слова).

## transform: the-tongue-names-its-leavings

`adapter.rs`: `leavings(root) -> Vec<String>` — що бігун цієї мови
лишає в дереві (`__pycache__/`, `.pytest_cache/` для python;
`node_modules/` для javascript; `.rspec_status` для ruby; для rust і
elixir — тека збірки, яку вони вже мають). `scope.rs`: меблі питають
цей перелік раз на порівняння, поруч із lock-файлами. `init.rs`: рада
ignore називає всі шляхи мови, а не одну теку, і слово «ця мова нічого
не збирає» лишається тільки там, де це правда. Контракти адаптерів і
`tool-scope.md` кажуть це.

## transform: the-advice-leads-with-the-release

**Замір виправив саму обіцянку.** План казав: «дорога релізу — інша,
коротша за KEEL_REF». Зміряно проти релізу на `file://` — це ОДНА
дорога: позиційний аргумент `install.sh` і є `KEEL_REF`, а вибирає
дорогу **форма** значення (строгий semver із необовʼязковим `v` —
`release_tag` у скрипті). Тож вада не в тому, що лампа радить довшу
дорогу, а в тому, що вона **називає не ту**: над піном-версією казала
«KEEL_REF бере git ref … checksum там не звіряє ніхто», хоч та дорога
звіряє `.sha256`. Речення це я написав сам у хвилі 0056, виправляючи
сусіднє слово, — і рецензент хвилі 0056 його теж не спіймав.

`version.rs`: `release_shaped` — дзеркало `release_tag` зі скрипта; за
ним лампа друкує рядок дороги: реліз (архів і `.sha256`, без git і
cargo) або git ref (клон і збірка, checksum ніхто не звіряє). Порада
стала коротшою формою — `sh install.sh <пін>`. Межа лишається межею:
теґа може не бути, старе покоління не збирається, версія без релізу
збирається з гілки лише тоді, коли зібране відповідає тим числом.

**Дрейф (§4.6), названий тут.** Проба хвилі 0039 (`pin_hand_test`,
`the_pin_has_a_hand`) тримала форму `KEEL_REF="<пін>"` дослівно.
Обіцянка 0039 — «вирок називає команду, якою взяти саме приколоту
версію, і ця команда справді працює» — тримається й далі, тож проба
тепер питає команду (`sh install.sh <пін>`), а не одну її запис. Текст
сценарію 0039 лишається як є: його межа («git-ref за іменем, не
перевірений checksum») була правдою світу без релізів — тим самим
записом свого часу, що й числа `0.1.0` у хвилі 0056.

## transform: journal

Звіт свіжого рецензента (§9.9) лягає в історію гілки; стаття журналу в
`docs/uk/V2-PROCESS.md`; три рядки черги (баги R-21, R-22, R-26)
знімаються з `BACKLOG.md` як сплачені.

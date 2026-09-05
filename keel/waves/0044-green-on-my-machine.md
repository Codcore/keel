---
depends_on: [0043-the-court-that-saw-red]

scenarios:
  a-court-runs-where-the-crate-is:
    covers: [functional.correctness, compatibility.co-existence]
  a-probe-without-its-tool-stops-aloud:
    covers: [reliability.fault-tolerance, functional.appropriateness]
  a-court-names-the-toolchain-it-judged-with:
    covers: [reliability.faultlessness, maintainability.analysability]

transforms:
  the-step-that-can-run:
    implements:
      - a-court-runs-where-the-crate-is
      - a-court-names-the-toolchain-it-judged-with
    files:
      - tool/src/generated.rs
      - keel/contracts/tool-generated.md
      - tool/src/adapter.rs
      - tool/tests/generated_ci_test.rs
      - .github/workflows/keel.yml
      - .github/workflows/tool-ci.yml
      - rust-toolchain.toml
      - tool/tests/generated_stands_test.rs
  a-probe-that-knows-its-machine:
    implements:
      - a-probe-without-its-tool-stops-aloud
    files:
      - tool/tests/common/mod.rs
      - tool/tests/elixir_border_test.rs
      - tool/tests/elixir_tests_test.rs
      - tool/tests/write_truth_test.rs
      - tool/tests/machine_test.rs
  the-lint-of-a-newer-stable:
    chore: "a lint that exists on the runner's toolchain and not on mine"
    files:
      - tool/tests/generated_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0044-green-on-my-machine.md

decisions:
  functional.completeness: "свідомо без окремого сценарію: нічого нового суд не вміє — він починає бути правдою не лише на моїй машині"
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "свідомо без тесту: названий toolchain коштує одного завантаження на холодному раннері — ціна відома і сплачується раз"
  compatibility.interoperability: "тримає a-court-runs-where-the-crate-is: крок питає адаптер, де корінь крейта, а не припускає корінь репозиторію"
  interaction.appropriateness-recognisability: "не застосовується"
  interaction.learnability: "свідомо без тесту: команд не додається"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає a-court-runs-where-the-crate-is: людина, що зробила keel init і запушила, дістає крок, який біжить, а не «could not find Cargo.toml»"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "тримає a-court-names-the-toolchain-it-judged-with: вирок, який залежить від того, який сьогодні день на раннері, не каже, що саме він судив"
  interaction.user-assistance: "свідомо без тесту: нових відмов ця хвиля не додає — вона править згенерований артефакт і проби"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "не застосовується"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "тримає a-court-runs-where-the-crate-is: де лежить крейт, знає адаптер — генератор питає, а не знає сам"
  maintainability.reusability: "тримає a-probe-without-its-tool-stops-aloud: одна рука на всі проби, у спільному common/, а не той самий assert у трьох файлах"
  maintainability.modifiability: "не застосовується"
  maintainability.testability: "тримає a-probe-without-its-tool-stops-aloud: проба, яка червоніє від того, чого на машині нема, не судить нічого — вона просто шумить"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  flexibility.adaptability: "тримає a-court-runs-where-the-crate-is: розкладка «крейт у підтеці» — звичайна, і сам keel така"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий числом — CI цього репозиторію червона чотирма різними причинами, і жодну з них не видно на машині автора"
  safety.hazard-warning: "не застосовується"
  safety.fail-safe: "тримає a-probe-without-its-tool-stops-aloud: проба, яка не може судити, каже це вголос і не вдає ані зеленого, ані червоного"
  safety.safe-integration: "тримає a-court-names-the-toolchain-it-judged-with: наявні проєкти дістають той самий крок плюс названу ланку — поведінка судів не міняється"
---

## Why

Батарея зелена, `keel check` — нуль знахідок, `keel close` — закрито.
А **CI цього ж репозиторію червона**, і червона чотирма різними
причинами. Жодної з них не видно з машини, на якій я працюю.

Це не дрібниця про yaml. Три з чотирьох — це **суди й проби, які
тримаються лише в тому контейнері, де їх писали**, а четверта — це
**артефакт, який інструмент пише людям і який не біжить**. Тобто
твердження «зелене» в цьому проєкті означало «зелене в мене».

Зміряно на бігу
`https://github.com/Codcore/keel/actions/runs/33967843600`:

**Перше — згенерована CI не може бігти.**
```
крок «the battery»:  run: cargo test --no-fail-fast
error: could not find `Cargo.toml` in `/home/runner/work/keel/keel`
```
Це не про розкладку keel. Відтворено на **свіжій фікстурі**: корінь із
`keel.toml` (adapter=rust, ci=github), крейт у `tool/`, `keel update` —
і крок такий самий, і `cargo test` із кореня так само не біжить.
`generated.rs` бере `battery_command()` і **не питає**
`adapter::crate_root()`, хоч той поруч і вміє відповісти. Хвиля 0039
вже правила рівно цей клас: «згенерована CI кликала `keel`, не
поставивши його». Той самий гріх, сусідній рядок.

**Друге — проби, яким потрібен mix, падають там, де mix нема.**
```
tool/tests/elixir_border_test.rs:116
    assert!(have_mix(), "this probe runs a real mix; it is not on PATH");
```
Голова того самого файлу каже: «Where mix is not on the machine the
probe says so and **stops rather than pretending**». `assert!` — це
падіння, а не зупинка. На раннері elixir-а нема, і `keel close` там
називає два сценарії хвилі 0042 недоведеними.

**Третє — проба, яка тримається лише під root-ом.** `write_truth_test`
ставить теку в `0555` і жене `setpriv --reuid=65534 … keel rev --write`,
щоб довести, що незаписна тека дає ненульовий код. Не-root скинути
права не може. Зміряно тут:
```
$ setpriv --reuid=65534 --clear-groups /bin/true
setpriv: setgroups failed: Operation not permitted     rc=127
```
І це найгірша форма: **spawn удається**, тож `Command::output()` вертає
`Ok(..)`, гілка `Err` (яка ловить «setpriv не на PATH») не спрацьовує —
і проба судить **повідомлення setpriv** як вивід keel. Не-root-ові
setpriv узагалі не потрібен: режим `0555` сам зупиняє запис.

**Четверте — «clippy чистий» було твердженням про мою машину.**
```
error: this `if` can be collapsed into the outer `match`
   --> tests/generated_test.rs:563:17
   = note: `-D clippy::collapsible-match` implied by `-D warnings`
```
Лінта є в 1.98 на раннері й нема в 1.94 у мене, бо `tool-ci.yml`
робить `rustup update stable`. Вирок, що залежить від того, який
сьогодні день, — не вирок. Ланка мусить бути **названа**, а не та, що
трапилась.

**Спільна теза.** Кожен із чотирьох — той самий гріх у різних одежах:
**зелене в мене — ще не зелене**. І цю хвилю судить не батарея на моїй
машині, а той самий раннер, який усі чотири й показав.

## scenario: a-court-runs-where-the-crate-is

**Дано** проєкт, чий корінь несе `keel.toml` із `ci`, а крейт (чи
`Gemfile`, чи `mix.exs`) лежить **не в корені**, а в підтеці.
**Коли** біжить `keel update` (чи `keel init`).
**Тоді** згенерований крок батареї біжить **там, де лежить корінь
мови**: команду мови видно як була, а тека сказана явно — і той крок,
виконаний із кореня репозиторію, справді жене батарею.

І зворотний бік: проєкт, чий крейт **у корені**, дістає той самий крок,
що й досі — без зайвої теки й без зміни поведінки.

## scenario: a-probe-without-its-tool-stops-aloud

**Дано** машину, на якій нема бігуна, потрібного пробі (`mix`), або
нема права, потрібного пробі (скидання привілеїв).
**Коли** біжить батарея.
**Тоді** проба **не червоніє**: вона каже вголос, чого їй бракує, і не
судить нічого — ані зеленим, ані червоним. І **ніколи** не судить
повідомлення чужої програми як вивід keel: якщо помічна команда сама
впала, це не вирок keel, а відсутність вироку.

Там, де інструмент **є**, проба судить, як судила: пропуск не сміє
стати способом не судити на машині, де все стоїть.

## scenario: a-court-names-the-toolchain-it-judged-with

**Дано** rust-проєкт, який просить `ci`.
**Коли** біжить `keel update`.
**Тоді** згенерований файл **називає ланку**, якою судить, а не бере
ту, що трапилась на раннері того дня — і каже це людині словом, а не
лише полем: вирок, відтворюваний тільки випадково, не вирок.

## transform: the-step-that-can-run

`generated.rs` питає адаптер — `adapter::battery_dir` — де той жене
батарею, і пише `working-directory` лише там, де ця тека не збігається
з коренем репозиторію. Для ruby й elixir відповідь **завжди корінь**,
бо їхні адаптери женуть звідти; де адаптер не може сказати, файл каже
це вголос замість кроку, приреченого впасти без причини. Канал піна
читається TOML-читачем і мусить бути **іменем** — інакше пін міг би
вписати довільну команду в згенеровану CI. Плюс названа ланка для
rust. Контракт `tool-generated.md` каже всі три правила. Словника це не
чіпає: увесь згенерований `keel.yml` — англійський, шапкою і
коментарями, як і був. Власний `.github/workflows/keel.yml` цього
репозиторію переписується тим самим `keel update` — інакше ця хвиля
знову доводила б щось лише на фікстурі.

## transform: a-probe-that-knows-its-machine

Спільна рука в `tool/tests/common/`: «чи є на цій машині те, без чого
судити не можна» — і одна форма зупинки вголос замість трьох
`assert!`. `write_truth_test` більше не кличе `setpriv` там, де він
приречений: не-root жене keel **напряму**, бо режим `0555` уже тримає;
root — через `setpriv`, як і досі. І жодна гілка не судить чужого
`stderr` як свого.

Нова проба `machine_test.rs` тримає саме це правило: відсутність
інструмента дає **пропуск, сказаний уголос**, а не червоне і не зелене.

## transform: the-lint-of-a-newer-stable

Одна знахідка `clippy::collapsible-match` у `generated_test.rs`,
згорнута так, як просить лінта. Chore, бо поведінки не міняє: це
рядок, який на моїй ланці не судився взагалі.

## transform: journal

Записи журналу цього покоління їдуть із хвилею (V2-PROCESS), разом із
чергою, README і звітом рецензії.

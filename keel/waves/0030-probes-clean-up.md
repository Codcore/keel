---
depends_on: [0029-the-methodology-in-both]

scenarios:
  a-sandbox-does-not-outlive-its-test:
    covers: [maintainability.testability, compatibility.co-existence]

transforms:
  one-hand-that-cleans-up:
    implements:
      - a-sandbox-does-not-outlive-its-test
    files:
      - tool/tests/common/mod.rs
      - tool/tests/adapter_name_test.rs
      - tool/tests/ask_test.rs
      - tool/tests/battery_runs_test.rs
      - tool/tests/body_test.rs
      - tool/tests/check_test.rs
      - tool/tests/ci_run_test.rs
      - tool/tests/close_test.rs
      - tool/tests/config_test.rs
      - tool/tests/docs_test.rs
      - tool/tests/gate_test.rs
      - tool/tests/generated_test.rs
      - tool/tests/git_hand_test.rs
      - tool/tests/holding_test.rs
      - tool/tests/ignore_reminder_test.rs
      - tool/tests/init_test.rs
      - tool/tests/map_test.rs
      - tool/tests/next_test.rs
      - tool/tests/plan_test.rs
      - tool/tests/rev_test.rs
      - tool/tests/rev_write_test.rs
      - tool/tests/review_test.rs
      - tool/tests/scope_test.rs
      - tool/tests/status_test.rs
      - tool/tests/tags_test.rs
      - tool/tests/trust_test.rs
      - tool/tests/version_pin_test.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md

decisions:
  functional.completeness: "тримає a-sandbox-does-not-outlive-its-test: усі 26 файлів проб, що роблять пісочниці, беруть одну руку; жодного власного sandbox не лишається — це судиться грепом у самій пробі"
  functional.correctness: "тримає a-sandbox-does-not-outlive-its-test: пісочниця зникає, коли тест скінчився зелено, і ЛИШАЄТЬСЯ, коли тест упав — обидві дороги жене проба"
  functional.appropriateness: "не застосовується як окремий тест: доречність зміряна ціною — 11511 тек і 20 ГБ, і диск на 100% посеред роботи хвилі 0029"
  performance.time-behaviour: "свідомо без тесту: прибирання — один remove_dir_all на пісочницю, там само, де вона й народилась"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "тримає a-sandbox-does-not-outlive-its-test: саме про це хвиля — 20 ГБ, які трималися нізащо"
  compatibility.interoperability: "свідомо без тесту: tests/common/ — звичайний спільний модуль інтеграційних тестів Rust, без жодної залежності"
  interaction.appropriateness-recognisability: "не застосовується: людина цього не бачить"
  interaction.learnability: "не застосовується"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "свідомо без тесту: рука зветься sandbox, як і колишні двадцять шість; імени не міняємо, щоб дифи лишились читними"
  interaction.user-assistance: "не застосовується"
  reliability.faultlessness: "тримає a-sandbox-does-not-outlive-its-test: прибирання висить на Drop, тож воно стається і на звичайному виході, і на розкручуванні стека — а не на останньому рядку тіла, який падіння пропускає"
  reliability.fault-tolerance: "свідомо без тесту: remove_dir_all на неіснуючій теці ігнорується — пісочниця, прибрана вручну, не валить тесту"
  reliability.availability: "не застосовується"
  reliability.recoverability: "тримає a-sandbox-does-not-outlive-its-test: пісочниця впалого тесту ЛИШАЄТЬСЯ — це те, що людина відкриє, розбираючись; прибирання, яке зʼїдає доказ, гірше за протікання"
  security.confidentiality: "не застосовується"
  security.integrity: "свідомо без тесту: рука прибирає лише те, що сама створила, і шлях складає сама — чужої теки вона не бачить"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "тримає a-sandbox-does-not-outlive-its-test: одна рука замість двадцяти шести копій"
  maintainability.reusability: "свідомо без тесту: нова проба пише mod common; і бере ту саму руку"
  maintainability.analysability: "свідомо без тесту: імʼя теки далі несе pid і імʼя випадку, тож пісочниця, що лишилась, сама каже, чия вона"
  maintainability.modifiability: "тримає a-sandbox-does-not-outlive-its-test: правило прибирання живе в одному місці"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "свідомо без тесту: рука ніколи не чіпає теки поза власним префіксом — це та межа, яку рецензент 0026 переступив руками, коли знищив 10128 чужих тек"
  safety.risk-identification: "не застосовується як окрема робота: загроза — прибирання, що зʼїсть чуже або живе, — і вона названа в самій руці"
  safety.fail-safe: "тримає a-sandbox-does-not-outlive-its-test: коли тест падає, рука НЕ прибирає"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "тримає a-sandbox-does-not-outlive-its-test: жодна проба не міняє того, що доводить; змінюється лише те, що лишається на диску після неї"
---

## Why

Перший рядок BACKLOG, і він перестав бути боргом. Під час хвилі 0029
диск став **на 100% посеред розсилки знахідок**: у `/tmp` лежало
**11 511 тек на 20 ГБ**. Історія цього числа: рецензент 0026 знайшов
10 128 тек на 17 ГБ і — попри пряму заборону — знищив їх, визнавши це
власним порушенням; рецензенти 0028 і 0029 обидва звітували про
тісний диск і **свідомо не ганяли `keel close`**, тобто суд закриття
через це лишився неперевіреним двічі поспіль.

Причина проста: `sandbox()` живе **двадцятьма шістьма копіями**, і
кожна робить `remove_dir_all` **на вході** — тобто прибирає за
попереднім бігом того самого імені, але ніколи за собою. Дві проби
(0026 і 0028) дістали прибирання останнім рядком тіла — а падіння
той рядок пропускає.

Тому: **одна рука на всі проби, і прибирання на `Drop`**. Drop
стається і на звичайному виході, і на розкручуванні стека, тож
зелений тест лишає по собі порожньо. А впалий — **лишає пісочницю
навмисно**: саме її людина відкриє, розбираючись, і прибирання, що
зʼїдає доказ, гірше за протікання.

Межа, названа вголос: рука прибирає **лише те, що створила сама** —
шлях вона складає сама, з власного префікса і pid-а. Чужого вона не
бачить і бачити не може; це рівно та межа, яку рецензент 0026
переступив руками.

## scenario: a-sandbox-does-not-outlive-its-test

**Дано** пробу, що робить пісочницю спільною рукою,
**коли** тест закінчується,
**тоді** пісочниця **зникає з диска**, якщо тест зелений, і
**лишається**, якщо тест упав. Прибирання висить на `Drop`, тож воно
стається і на звичайному виході, і на розкручуванні стека — не на
останньому рядку тіла, який падіння пропускає. Рука прибирає лише
власну теку: шлях складено з її префікса і pid-а процесу, і жодного
іншого шляху вона не приймає. **Жоден із двадцяти шести файлів проб
не тримає власної копії** `sandbox` — це судиться самим текстом
проб.

## transform: one-hand-that-cleans-up

`tool/tests/common/mod.rs` — одна рука: `sandbox(name)` віддає
охоронця, що прибирає теку на `Drop`, крім випадку, коли потік
падає. Двадцять шість файлів проб беруть `mod common;` і власних
копій більше не тримають.

## transform: journal

Журнальні записи bootstrap їдуть своєю хвилею (школа 0009 R-1):
документ памʼяті лупа — docs/uk/V2-PROCESS.md (§9.10).

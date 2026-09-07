---
depends_on: [0058-the-norm-weighs-a-contract]

scenarios:
  a-rails-project-is-judged-out-of-the-box:
    covers: [functional.correctness, compatibility.interoperability, maintainability.testability]

transforms:
  the-reader-sees-the-rails-test:
    implements: [a-rails-project-is-judged-out-of-the-box]
    files:
      - tool/src/tags.rs
      - tool/tests/rails_tests_test.rs
  the-runner-speaks-rails:
    implements: [a-rails-project-is-judged-out-of-the-box]
    files:
      - tool/src/ruby.rs
      - tool/src/config.rs
  the-contract-says-the-third-reading:
    chore: "контракт адаптера каже третє читання: як Rails пізнається, чим біжить батарея і чим біжить один тест"
    files:
      - keel/contracts/tool-adapter-ruby.md
decisions:
  functional.completeness: "зміряно на справжньому додатку (rails new, Rails 8.1.3.1): вад рівно три — читач тегів, батарея і команда одного тесту; розпізнавання Rails і меблі зміряні окремо (меблі вже криє власний .gitignore Rails)"
  functional.appropriateness: "не застосовується"
  performance.time-behaviour: "названо: `bin/rails test` піднімає середовище додатка, тож батарея Rails повільніша за голий ruby — це ціна того, що вона взагалі завантажується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "тримає: адаптер кличе `bin/rails` проєкту і не пише в дерево нічого"
  interaction.appropriateness-recognisability: "не застосовується"
  interaction.learnability: "тримає: крок §9.2 дає команду, яку людина в Rails справді набирає"
  interaction.operability: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "не застосовується"
  interaction.self-descriptiveness: "тримає: контракт адаптера каже третє читання своїм текстом"
  interaction.user-error-protection: "тримає: проєкт без bin/rails і config/application.rb лишається на першому читанні — розпізнавання питає дві прикмети, не одну"
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
  maintainability.modularity: "тримає: третє читання живе в тому самому модулі ruby, бо Rails — розкладка мови, а не інша мова (прецедент хвилі 0047)"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "не застосовується"
  maintainability.modifiability: "названо: розгалужень усередині ruby стає три (minitest, rspec, Rails); четверте вимагатиме іншої будови, і це межа, названа тут"
  flexibility.adaptability: "тримає ця хвиля: адаптер питає прикмети проєкту, а не конфіг"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "названо: keel кличе `bin/rails` проєкту — тобто виконує скрипт із дерева, яке судить; це та сама межа, що з `ruby` і `mix`, і вона вже названа контрактом"
  safety.risk-identification: "названо: `test «…» do` є і в ExUnit; читання розділяє розширення файлу (.rb проти .exs), як і доти"
  safety.fail-safe: "не застосовується"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "не застосовується"
---

## Why

**У Rails-проєкті сценарій не довести взагалі.** Зміряно автором на
СПРАВЖНЬОМУ додатку (`rails new`, Rails 8.1.3.1, `bundle install`
пройдено), бінарником 1.0.0 — три вади, і перша з них смертельна:

```
відмова: тег proves: a-user-greets@… не має тест-функції одразу за собою
```

Читач тегів знає `def test_…`, а Rails пише `test "greets" do` в
`ActiveSupport::TestCase`. Це не кут — це типовий стиль фреймворку,
тож на Rails-проєкті керма нема з першого кроку.

Друге і третє — команди. Батарея ruby (`ruby -Itest -e 'Dir.glob…'`)
Rails не завантажить: додатку потрібне його середовище. Один тест
адаптер жене `ruby -Itest <файл> -n <метод>`, а Rails уміє `bin/rails
test <файл>:<рядок>` і `bin/rails test <файл> -n <метод>` (обидва
зміряні, exit 0 на зеленому і 1 на червоному).

**Рішення оператора 2026-09-07: третє читання всередині ruby**, а не
окрема мова. Rails — не інша мова, а інша розкладка ruby, і прецедент
стоїть: хвиля 0047 дала цій самій мові друге читання (rspec). `keel.toml`
Rails-проєкту лишається `adapter = "ruby"`, і людина не пише нічого.

Пізнається Rails двома прикметами разом — `bin/rails` і
`config/application.rb` (зміряно). Меблі (`log/`, `tmp/`) не болять:
власний `.gitignore` Rails їх уже криє, `git status` після бігу чистий.

## scenario: a-rails-project-is-judged-out-of-the-box

**Дано** проєкт із `bin/rails` і `config/application.rb`, чий тест
оголошений по-Rails — `test "greets" do` у `test/**/*_test.rb`, — і тег
`proves:` над ним, **коли** біжить `keel check`, один тест і батарея,
**тоді** тег читається (імʼям методу, яким його знає minitest —
`test_greets`), один тест жене `bin/rails test`, і батарея жене
`bin/rails test`; а проєкт без цих двох прикмет лишається на першому
читанні — `ruby -Itest` — незмінно.

## transform: the-reader-sees-the-rails-test

Читач `.rb` дістає другу декларацію: `test «…» do` поруч із `def
test_…`. Імʼя — те, яке будує ActiveSupport: `test_` плюс назва, де
кожен пробіл став підкресленням. Розділяє форми розширення файлу
(`.rb` проти `.exs`), як і доти: `test «…» do` пише й ExUnit.

## transform: the-runner-speaks-rails

`ruby::rails_root` питає дві прикмети. `run_test` і `run_all` над
Rails-проєктом кличуть `bin/rails test`; вироки читає той самий читач
minitest (`-v`, рядок `Клас#метод = <час> s = <позначка>`), бо Rails
жене той самий minitest. `config::battery_command` каже Rails-команду
там, де проєкт Rails.

## transform: the-contract-says-the-third-reading

Контракт `tool-adapter-ruby` каже третє читання і його межі.

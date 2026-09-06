---
depends_on: [0047-rspec-in-the-ruby-hand]

scenarios:
  a-release-is-built-by-one-script:
    covers: [functional.completeness, security.authenticity]
  the-launcher-fetches-a-missing-version-aloud:
    covers: [flexibility.installability, security.integrity]
  the-installer-takes-the-release-before-the-source:
    covers: [performance.time-behaviour, flexibility.replaceability]

transforms:
  one-script-builds-the-release:
    implements:
      - a-release-is-built-by-one-script
    files:
      - release.sh
      - .github/workflows/release.yml
      - keel/contracts/tool-release.md
      - keel.toml
      - tool/tests/release_test.rs
      - tool/tests/common/versions.rs
  the-launcher-fetches-aloud:
    implements:
      - the-launcher-fetches-a-missing-version-aloud
      - the-installer-takes-the-release-before-the-source
    files:
      - install.sh
      - keel/contracts/tool-launcher.md
      - tool/tests/launcher_fetch_test.rs
      - tool/tests/common/versions.rs
  journal:
    chore: "bootstrap journal entries of the wave ride with it (V2-PROCESS)"
    files:
      - docs/uk/V2-PROCESS.md
      - BACKLOG.md
      - README.md
      - keel/reviews/0048-a-published-release.md

decisions:
  functional.correctness: "тримає a-release-is-built-by-one-script: імʼя архіву несе версію крейта і target-трійку, і бінарник усередині відповідає тією самою версією — одне число в трьох місцях, звірене пробою"
  functional.appropriateness: "свідомо без тесту: реліз — це те, що концепт назвав «сама качає реліз, звіряє checksum»; форма архіву — tar.gz з одним файлом `keel`, без інсталятора всередині"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "тримає the-launcher-fetches-a-missing-version-aloud: докачаний бінарник лягає у versions/<тег>/, ~4 МіБ на версію; тимчасова тека докачування прибирається і при успіху, і при відмові, а TMPDIR, у якому теку не вдалось зробити, — відмова без файлів у / (рецензія R-3)"
  compatibility.co-existence: "тримає the-installer-takes-the-release-before-the-source: версія з релізу і версія, зібрана з git, стоять поруч у versions/ під різними тегами, і launcher обирає їх піном, як і досі"
  compatibility.interoperability: "свідомо без тесту: докачування — `curl` (або `wget`, коли curl нема), розпакування — `tar`, checksum — `sha256sum`/`shasum`; де інструмента нема, відмова називає котрого"
  interaction.appropriateness-recognisability: "свідомо без тесту: команд не додається; `release.sh` у корені — так само, як `install.sh`"
  interaction.learnability: "свідомо без тесту: README каже три рядки — як поставити реліз, як запінити, як зібрати реліз самому"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "тримає the-launcher-fetches-a-missing-version-aloud: checksum не зійшовся — відмова, і НІЧОГО не встановлено; машина з невідомим target — відмова, що називає її"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.self-descriptiveness: "тримає the-launcher-fetches-a-missing-version-aloud: launcher каже вголос, ЩО взяв і ЗВІДКИ, при кожному докачуванні — концепт вимагає саме цього, а хвиля 0041 відмовилась качати мовчки"
  interaction.user-assistance: "тримає the-launcher-fetches-a-missing-version-aloud: без мережі або без релізу для цього target — відмова з порадою, що працює: пін-ref або чекати релізу (рецензія R-13: команда установки йшла б до того самого 404)"
  reliability.faultlessness: "тримає the-launcher-fetches-a-missing-version-aloud: другий біг не качає знову — версія стоїть, `.keel-sum` звіряється перед exec, як для зібраної; бінарник, що відповідає іншим числом, ніж теґ, не ставиться (рецензія R-2 — інакше качався б на кожному бігу)"
  reliability.fault-tolerance: "тримає the-installer-takes-the-release-before-the-source: ref без опублікованого релізу (гілка, коміт, старий тег) збирається з сирців, як до хвилі, і сказано вголос чому"
  reliability.availability: "не застосовується"
  reliability.recoverability: "свідомо без тесту: відмова докачування лишає versions/ таким, як був; наполовину докачаної версії не буває, бо розпакування йде в тимчасову теку і переноситься лише після звірки"
  security.confidentiality: "не застосовується"
  security.non-repudiation: "свідомо без окремої роботи, і названо: workflow релізу підписує provenance артефакту (`actions/attest-build-provenance`) — звірити підпис може `gh attestation verify`; launcher звіряє sha256, а не підпис, бо `gh` на машині людини не обовʼязковий; це межа, сказана в контракті"
  security.accountability: "не застосовується"
  security.resistance: "тримає the-launcher-fetches-a-missing-version-aloud: імʼя тега з піна йде в URL лише як `v<число>.<число>.<число>` або як `v`-тег із безпечних знаків — пін із слешем, `..` чи пробілом у реліз не перетворюється, а йде старою дорогою git; checksum звіряється до розпакування, і архів із чужим sha не торкається versions/"
  maintainability.modularity: "свідомо без тесту, і сказано чесно: функція докачування написана двічі — в install.sh і в launcher-і, який install.sh пише; launcher мусить стояти сам, без клону і без install.sh поруч"
  maintainability.reusability: "свідомо без тесту: release.sh — той самий, що жене workflow і що жене людина; проба жене саме його зі стабом cargo"
  maintainability.analysability: "тримає a-release-is-built-by-one-script: `.sha256` на кожен архів окремо, бо matrix у workflow збирає кілька target-ів паралельно і спільний SHA256SUMS вони б перетирали"
  maintainability.modifiability: "свідомо без тесту: жодного нового місця диспетчеризації за мовою; версія крейта і пін keel НЕ підіймаються цією хвилею — число і теґ обирає оператор, і порядок записано в черзі: версія крейта → теґ на той коміт → пін (рецензія R-1: у зворотному порядку реліз під теґом не знайшов би жоден пін — тепер `release.sh --tag` це відмовляє)"
  maintainability.testability: "свідомо без тесту: проби женуть справжні install.sh, launcher і release.sh проти справжнього git і сервера релізів `file://` зі стабом cargo — як у хвилі 0041; докачування з GitHub не проходить у пробі жодного разу — світ проб ставить KEEL_RELEASES на порожню теку file:// (рецензія R-4: до того проби 0041 ходили на github.com девʼять разів за батарею)"
  flexibility.scalability: "не застосовується"
  flexibility.adaptability: "свідомо без тесту: target-трійка з `uname`; linux x86_64/aarch64 і macOS arm64/x86_64 названі — і workflow збирає всі чотири (рецензія R-7: перший matrix збирав два), решта — відмова з іменем машини"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "свідомо без окремої роботи: ризик названий — реліз існує лише після того, як оператор запушить теґ; до того launcher докачує лише з `KEEL_RELEASES`, який ставить проба"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "тримає the-installer-takes-the-release-before-the-source: наявні проби launcher-а і versions-у хвилі 0041 лишаються зеленими без змін — старий шлях не зрушив"
  safety.fail-safe: "тримає the-launcher-fetches-a-missing-version-aloud: будь-яка відмова докачування — це відмова, не біг іншої версії; закон 0041 «не той бінарник мовчки гірший за відмову» тримається і з мережею"
---

## Why

Хвиля 0041 поставила версії поруч і launcher, який жене рівно ту, що
запінено, — і назвала три дірки: **підписаний реліз із власним
checksum-ом**, **launcher не докачує сам** (концепт хотів «нема
локально — сама качає»), **жоден опублікований теґ не несе теперішньої
розкладки**. Ця хвиля закриває перші дві машиною і третю — до одного
рядка оператора.

**Зміряно перед планом.** Релізів GitHub у репозиторії нема жодного
(`releases: []`); теґи `v0.8.5…v0.8.9` — це keel v1 (`keel.py`), крейт
`tool/` там відсутній, і install.sh каже це вголос. Крейт відповідає
`0.1.0` 500 комітів поспіль; пін keel — `0.1.0`, і CI keel ставить
інструмент `KEEL_REF: "0.1.0"` крізь install.sh **з main upstream-у**.
Рецензія (R-14) доміряла те, чого я не доміряв: main upstream-у ще
старий, його install.sh `KEEL_REF` не читає взагалі, а `0.1.0` — ні
теґ, ні реліз; тож після злиття цієї гілки новий install.sh відмовив
би пінові `0.1.0` і CI keel став би червоним **без жодного підняття
версії**. Звідси третя дорога інсталятора: версія без релізу і без
теґа збирається з гілки, якою веде remote, і рахується лише коли
зібране відповідає нею. Число і теґ — рядок оператора, у порядку
«версія крейта → теґ на той коміт → пін» (R-1). На машині: `curl`
(уміє `file://`: відсутній файл — rc 37, 404 з GitHub — rc 22), `tar`,
`sha256sum` і `shasum`, host `x86_64-unknown-linux-gnu`; `gh` нема — тож
крок `gh release create` тримається текстом workflow-у, а не бігом.

**Одна форма релізу.** `release.sh` збирає `cargo build --release`,
питає в бінарника його версію, у `rustc -vV` — target, і кладе в
`dist/` архів `keel-<версія>-<target>.tar.gz` з одним файлом `keel` та
`<архів>.sha256` поруч. Один файл checksum-у на архів, а не спільний
`SHA256SUMS`: workflow збирає кілька target-ів matrix-ом паралельно,
і спільний файл вони перетирали б. Workflow `release.yml` на push теґа
`v*`: checkout, toolchain з `rust-toolchain.toml`, `sh release.sh`,
`actions/attest-build-provenance` (підпис походження артефакту — це
«підписаний реліз» у формі, яку GitHub дає без ключів), `gh release
create <теґ> dist/*`. Проба жене release.sh зі стабом cargo хвилі 0041
і тримає імена, checksum і бінарник усередині; текст workflow-у тримає
контракт `tool-release.md` командою `verify`.

**Launcher докачує — вголос.** Пін не стоїть локально → launcher
(1) перетворює пін на теґ релізу: `2.0.0` → `v2.0.0`, `v2.0.0` → як є;
пін, який не є `v?число.число.число`, у реліз не перетворюється — це
git-ref, стара дорога з готовою командою; (2) визначає target із
`uname`; (3) качає `<KEEL_RELEASES>/<теґ>/keel-<версія>-<target>.tar.gz`
і `.sha256` поруч у тимчасову теку, `KEEL_RELEASES` типово
`https://github.com/Codcore/keel/releases/download`; (4) звіряє sha256
**до** розпакування; (5) розпаковує, питає `keel --version`, пише
`.keel-version`, `.keel-ref` (теґ), `.keel-sha` (`release`), `.keel-sum`
(sha256 бінарника, як для зібраної), і лише тоді переносить теку у
`versions/`; (6) каже на stderr, що взяв і звідки; (7) `exec`. Кожна
відмова — без сліду у `versions/` і з готовою командою. Хвиля 0041
відмовилась качати, бо «тихо тягти код із мережі під час суду не
варто»; тут це не тихо — і це рішення оператора з концепту.

**Інсталятор бере реліз перед сирцями.** `KEEL_REF=v2.0.0 sh
install.sh` спершу пробує реліз (без cargo, без клону — секунди
замість збірки на холодному кеші); нема релізу для цього теґа і
target-у — збирає з git, як досі, і каже чому. Функція докачування
написана двічі — в install.sh і в launcher-і, — бо launcher мусить
стояти сам; це названо в рішеннях.

**Проби** — на світі хвилі 0041 (`common/versions.rs`): справжній git,
стаб cargo, і тепер ще сервер релізів `file://` з архівом, збудованим
самим release.sh. Що не проходить у пробі жодного разу: докачування з
GitHub (мережа) і `gh release create` — названо.

## scenario: a-release-is-built-by-one-script

**Дано** дерево keel і машина з cargo, tar, sha256sum/shasum.
**Коли** біжить `sh release.sh`.
**Тоді** у `dist/` лежить `keel-<версія>-<target>.tar.gz` з одним
файлом `keel`, що відповідає `keel <версія>`, і `<архів>.sha256`, який
`sha256sum -c` приймає; версія — та, яку каже бінарник; target — з
`rustc -vV`; скрипт каже вголос, що написав. Workflow `release.yml`
жене саме цей скрипт на push теґа `v*`, підписує provenance і кладе
`dist/*` у реліз GitHub — і це тримає контракт `tool-release.md`
командою `verify`.

## scenario: the-launcher-fetches-a-missing-version-aloud

**Дано** проєкт, чий `keel.toml` пінить версію, якої у `versions/`
нема, і сервер релізів (`KEEL_RELEASES`), де для теґа цієї версії
лежать архів і `.sha256`.
**Коли** біжить `keel <команда>`.
**Тоді** launcher докачує архів і `.sha256`, звіряє sha256 до
розпакування, ставить версію у `versions/<теґ>/` з усіма записами,
каже на stderr, що взяв і звідки, і жене саме її; другий біг не качає.
Checksum не зійшовся — відмова, і у `versions/` не зʼявляється нічого;
релізу нема (сервер недосяжний, 404, невідомий target) — відмова з
готовою командою; пін, який не є номером версії, у реліз не
перетворюється.

## scenario: the-installer-takes-the-release-before-the-source

**Дано** `install.sh`, теґ версії з опублікованим релізом і сервер
релізів.
**Коли** біжить `KEEL_REF=<теґ> sh install.sh`.
**Тоді** версія ставиться з архіву — без cargo і без збірки, — з
`.keel-sum` із sha256 бінарника, і launcher жене її; ref без релізу
(гілка, коміт, теґ без архіву) збирається з git, як до хвилі, і скрипт
каже вголос, що релізу не було; проби launcher-а і versions-у хвилі
0041 лишаються зеленими без змін.

## transform: one-script-builds-the-release

`release.sh` у корені; `.github/workflows/release.yml`; контракт
`tool-release.md` з `verify` над `release_test` і `launcher_fetch_test`
(і рядок довіри в `keel.toml`); `versions.rs` дістає сервер релізів
`file://` і архів, збудований release.sh.

## transform: the-launcher-fetches-aloud

`install.sh`: функція докачування (двічі — інсталятор і launcher),
пін → теґ, target із `uname`, звірка до розпакування, записи, слово
вголос; реліз перед сирцями в інсталяторі. Контракт `tool-launcher.md`
переписує розділ меж: докачування є, підпис — межа.

## transform: journal

Записи журналу цього покоління їдуть із хвилею (V2-PROCESS), разом із
чергою, README і звітом рецензії.

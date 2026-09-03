### Тексти інструмента keel — українська.
### Один файл на мову; нова мова = один перекладений файл + реліз
### (NEW-CONCEPT, «Конфіг → Мови виводу інструмента»).

## рамка відмови
word-refusal = відмова
word-reason = причина
word-instead = натомість
word-green = зелене
word-red = червоне

## підписи, що підставляються в повідомлення
what-waves = хвилі
what-contracts = контракти
what-wave-header = шапка хвилі
what-contract-header = шапка контракту
what-field = поле "{ $name }"
what-scenario = сценарій "{ $name }"
what-transform = трансформа "{ $name }"
what-decision-reason = причина в decisions "{ $name }"

## модуль docs
docs-keel-missing = теки keel/ тут нема — методика живе в keel/waves/ і keel/contracts/
docs-keel-missing-instead = створи keel/waves/ і keel/contracts/ або запусти keel з кореня проєкту
docs-dir-missing = теки для документів «{ $what }» нема
docs-dir-missing-instead = створи її — порожня тека краща за відсутню: відсутність не відрізнити від одруку в шляху
docs-dir-among = тека серед документів «{ $what }» — документи живуть пласко
docs-dir-among-instead = перенеси документи з неї прямо в цю теку і прибери її
docs-alien-file = чужий файл серед документів «{ $what }» — тут живуть лише .md
docs-alien-file-instead = прибери файл або перейменуй на .md, якщо це документ методики
docs-file-slug = імʼя файлу "{ $slug }" — не слаг
docs-file-slug-instead = імʼя документа стає гілкою і тегом (§1.2, §8.2): лише малі латинські літери, цифри і дефіс
docs-unreadable = файл не читається: { $error }
docs-unreadable-instead = перевір шлях і права доступу
docs-not-utf8 = файл не в UTF-8 — методика пише документи в UTF-8
docs-not-utf8-instead = перезбережи файл у кодуванні UTF-8
docs-file-empty = шапки нема: файл порожній
docs-no-header = шапки нема: файл не починається з рядка ---
docs-header-start-instead = почни файл шапкою — рядок ---, поля, знову --- (глава 2)
docs-header-unclosed = шапка не закрита: другий рядок --- не знайдено
docs-header-unclosed-instead = закрий шапку рядком --- після останнього поля
docs-field-twice = поле "{ $name }" оголошене двічі (рядки { $first } і { $second })
docs-field-twice-instead = лиши один запис: методика не вгадує, котрий із двох правий
docs-key-not-string = імʼя поля мусить бути рядком (рядок { $line })
docs-key-not-string-instead = запиши імʼя поля простим словом
docs-yaml-broken = шапка не читається як YAML: { $error }
docs-yaml-broken-instead = полагодь розмітку — методика пише лише поля, списки і рядки
docs-yaml-anchor = якір YAML у шапці (рядок { $line })
docs-yaml-anchor-instead = методика не пише якорів — повтори значення словами
docs-yaml-tag = тег YAML у шапці (рядок { $line })
docs-yaml-tag-instead = методика не пише тегів — прибери його
docs-header-empty = шапка порожня
docs-header-empty-instead = шапка мусить нести поля документа (глава 2)
docs-unknown-field = { $what }: невідоме поле "{ $name }" (рядок { $line })
docs-unknown-field-instead = { $what } знає лише: { $known }
docs-not-fields = { $what } мусить бути набором полів «імʼя: значення» (рядок { $line })
docs-not-fields-instead = подивись форму в прикладі README або в keel/waves/ поруч
docs-field-blank = { $what } — порожньо (рядок { $line })
docs-field-blank-instead = заповни поле або прибери його рядок зовсім
docs-value-blank = { $what } — порожньо (рядок { $line })
docs-value-blank-instead = заповни значення або прибери рядок зовсім
docs-not-string = { $what } мусить бути рядком (рядок { $line })
docs-not-string-instead = запиши значення одним рядком
docs-not-list = { $what } мусить бути списком (рядок { $line })
docs-not-list-instead = запиши як список: [a, b] або рядками з дефісом
docs-scenario-name-not-slug = імʼя сценарію "{ $name }" (рядок { $line }) — не слаг
docs-transform-name-not-slug = імʼя трансформи "{ $name }" (рядок { $line }) — не слаг
docs-name-not-slug-instead = імена стають кодом (§1.2): лише малі латинські літери, цифри і дефіс
docs-contract-ref-bad = { $what }: посилання на контракт мусить бути «slug@редакція», а не "{ $value }" (рядок { $line })
docs-contract-ref-bad-instead = редакція — 4–6 шістнадцяткових знаків, як-от session-run@7c40de (§5.1–§5.2)
docs-wave-no-transforms = шапка хвилі не має transforms — хвилі без роботи не буває
docs-wave-no-transforms-instead = оголоси хоч одну трансформу (§2.4) або chore (§2.11)
docs-scenario-bare = { $what } ні на що не спирається: ні proves, ні covers
docs-scenario-bare-instead = дай опору — контракт (proves) або розріз якості (covers), §3.3; знятий сценарій познач withdrawn
docs-transform-both = { $what } має і implements, і chore
docs-transform-both-instead = трансформа несе рівно одне: обіцянки — або chore з причиною (§2.11)
docs-transform-neither = { $what } не має ні implements, ні chore
docs-transform-neither-instead = назви, які сценарії вона наближає, — або chore: "<причина>" (§2.11)
docs-transform-no-files = { $what } не називає файлів
docs-transform-no-files-instead = файли перелічуються поіменно до роботи (§4.1)
docs-one-new-in-no-slash = { $what }: рядок "one new in" мусить називати теку зі скісною рискою в кінці (рядок { $line })
docs-one-new-in-no-slash-instead = напиши, наприклад: one new in priv/migrations/
docs-glob = { $what }: glob "{ $value }" у списку файлів (рядок { $line })
docs-glob-instead = файли називаються поіменно (§4.2); для файлу без відомого імені є one new in <тека>/
docs-exports-empty = exports порожній (рядок { $line })
docs-exports-empty-instead = перелічи сигнатури — або прибери поле і дай verify (§2.7–§2.8)
docs-exports-no-module = exports без module: не названо, хто обіцяє
docs-exports-no-module-instead = назви одиницю коду в полі module (§2.7)
docs-contract-empty = контракт нічого не обіцяє: ні exports, ні verify
docs-contract-empty-instead = дай сигнатури з module (§2.7) або команду verify (§2.8); слова без перевірки — застереження в хвилі, не контракт (§2.10)

## модуль rev
rev-missing-section = сценарій "{ $name }" оголошений у шапці, але секції "## scenario:" у тілі не має
rev-missing-section-instead = напиши секцію або прибери оголошення; редакції потрібне тіло, яке хешувати (§5.3)
rev-dup-section = секція "## scenario: { $name }" стоїть у тілі більш як один раз
rev-dup-section-instead = лиши одну секцію: методика не вгадує, котре тіло — обіцянка
rev-empty-section = секція "## scenario: { $name }" має порожнє тіло
rev-empty-section-instead = обіцянці потрібні слова: напиши тіло сценарію або зніми оголошення (§2.3)
rev-transform-no-body = трансформа "{ $name }" оголошена в шапці, але секції "## transform:" у тілі не має (§7.7)
rev-transform-no-body-instead = напиши секцію — слова роботи живуть у тілі — або прибери оголошення
rev-orphan-section = у тілі стоїть секція-сирота "## { $kind }: { $name }" — "{ $name }" не оголошений у шапці, а сирота не живе мовчки (§7.7)
rev-orphan-section-instead = оголоси її в шапці або прибери секцію свідомо
rev-nearmiss = заголовок "## { $heading }" пише слово секції без пробілу — не розпізнаний як секція, але й не вільна проза (§7.7)
rev-nearmiss-instead = напиши "## scenario: <імʼя>" / "## transform: <імʼя>" з пробілом або перейменуй заголовок геть від слів секцій
rev-dup-transform = секція "## transform: { $name }" стоїть у тілі більш як один раз — "{ $name }" не вгадується поміж них (§7.7)
rev-dup-transform-instead = лиши одну секцію: методика не вгадує, котре тіло несе слова роботи (§2.10)

rev-write-title = keel rev --write — розійшлі записи (NEW-CONCEPT)
rev-write-needs-adapter = рука перепису потребує адаптера rust, названого в keel.toml (старе написання cargo приймається): закритість судиться тегами
rev-write-needs-adapter-instead = постав adapter = "rust" — імʼям мови; "cargo" — прийнятий синонім (NEW-CONCEPT, Config); інші мови приїдуть своїми хвилями
rev-write-rewritten = { "  " }{ $wave }: { $contract }@{ $old } → { $contract }@{ $new } — запис тепер тримає чинну редакцію
rev-write-kept = { "  " }{ $wave }: закрита — лишаю її записи судові історії (§5.6)
rev-write-none = у відкритих хвилях нічого не розійшлось — кожен їхній запис чинний
rev-write-count = записів переписано: { $count }

## модуль graph
graph-unknown-cut = "{ $holder }" показує на розріз "{ $slug }", якого у словнику нема
graph-unknown-cut-instead = сорок розрізів їдуть з релізом (§3.4); вибери один із них або виправ одрук
graph-double-cover = розріз "{ $slug }" має { $count } живих covers: сценарії { $holders } (§10.3 — рівно одна відповідь)
graph-double-cover-instead = лиши один cover; другий сценарій стоїть на своєму proves або іншому розрізі (§3.3)
graph-double-decided = розріз "{ $slug }" закритий сценарієм "{ $holder }" і водночас вирішений (§10.3)
graph-double-decided-instead = прибери рядок decisions — відповідає сценарій; або зніми cover свідомо (§2.12)
graph-silence = розрізи без відповіді: { $missing }
graph-silence-instead = кожен розріз дістає рівно одну відповідь — рядок covers або decisions (§10.3); тиша заборонена
graph-implements-missing = трансформа "{ $transform }" реалізує "{ $scenario }", якого нема в шапці
graph-implements-missing-instead = назви наявний сценарій або прибери запис (§7.1)
graph-depends-missing = depends_on показує на "{ $target }" — такої хвилі тут нема
graph-depends-missing-instead = назви наявну хвилю або прибери ребро (§7.1)
graph-superseded-missing = сценарій "{ $scenario }" називає наступником "{ $successor }", невідомого жодній хвилі
graph-superseded-missing-instead = наступник мусить існувати (§2.12); спершу напиши його або виправ слаг
graph-superseded-self = сценарій "{ $scenario }" називає наступником самого себе
graph-superseded-self-instead = наступник — інший сценарій, що прийшов на зміну (§2.12); назви його або зніми позначку
graph-cycle = цикл depends_on: { $chain }
graph-cycle-instead = порядок виводиться з depends_on, тож петлі бути не може (§7.2); розірви цикл

## модуль scope
scope-drift = гілка чіпає "{ $file }", якого жодна трансформа хвилі не називає
scope-drift-instead = назви файл у files трансформи до роботи (§4.1) або прибери зміну (§4.6)
scope-untouched = оголошений файл "{ $file }" гілка не чіпає
scope-untouched-instead = робота ще попереду — або імʼя стояло даремно; судиться по всій гілці (§4.4, §4.5), тож дороби або прибери імʼя свідомо
scope-one-new-none = у теці "{ $dir }" не зʼявилось жодного нового файлу — обіцяно рівно один
scope-one-new-none-instead = створи обіцяний файл або прибери рядок `one new in`, якщо його не буде (§4.1)
scope-one-new-many = у теці "{ $dir }" нових файлів більш як один: { $files }
scope-one-new-many-instead = `one new in` обіцяє рівно один (§4.1); назви зайві файли у files поіменно
scope-one-new-count = рядки `one new in` обіцяють { $promised } нових файлів у теці "{ $dir }", гілка додає { $found }
scope-one-new-count-instead = зведи рахунки (§4.1): рядок на очікуваний файл, новий файл на рядок
scope-git-failed = git тут відмовляє: { $error }
scope-git-failed-instead = scope судиться по гілці (§4.5); зроби так, щоб git у цій теці відповідав, і повтори keel check

## модуль tags
tags-stale = тег тесту "{ $test }" тримає { $scenario }@{ $recorded }, а текст сценарію зараз дає { $actual }
tags-stale-instead = перечитай сценарій і онови тег свідомо (§5.1, §7.5) — або тест уже не тримає того, що змінилось
tags-orphan = тег тесту "{ $test }" доводить "{ $scenario }" — такого сценарію не знає жодна хвиля
tags-orphan-instead = назви наявний сценарій або прибери тег (§5.5)
tags-dangling = тег proves: { $scenario }@{ $rev } не має тест-функції одразу за собою
tags-dangling-instead = постав тег на його тест (§5.5); запис, який нічого не тримає, гірший за відсутній
tags-bad-rev = тег "{ $scenario }" тримає запис "{ $rev }" — редакція пишеться 4–6 hex-знаками (§5.2)
tags-bad-rev-instead = перерахуй через keel rev і запиши її префікс; кривий запис нічого не тримає
tags-vanished = тег сценарію "{ $scenario }" був у точці розгалуження і зник у HEAD — сценарій живий
tags-vanished-instead = поверни тест або зніми сценарій withdrawn свідомо (§7.15, §2.12); старі обіцянки не роззброюються мовчки
tags-vanished-gone = тег сценарію "{ $scenario }" був у точці розгалуження і зник у HEAD — разом із самим сценарієм: обіцянку стерто цілком
tags-vanished-gone-instead = документи не видаляються (§4.12): поверни хвилю і зніми сценарій withdrawn у її файлі (§2.12) — знищення без сліду заборонене

# -- trust: TOFU-суд команд (§7.16, §2.8) ---------------------------
trust-untrusted = команда "{ $command }" не довірена (§7.16 — нова чи змінена не виконується)
trust-untrusted-instead = запиши довіру: keel trust — рядок ляже в diff, який затверджує merge
trust-ci-empty = ci записаний порожнім — не вирішено
trust-ci-empty-instead = назви команду або скажи "none" вголос
trust-crooked = запис довіри команди "{ $command }" несе збитий відбиток
trust-crooked-instead = перезапиши через keel trust, якщо команді справді довіряєш
trust-door = запис довіри "{ $command }" не відповідає жодній живій команді — двері, відчинені наперед
trust-door-instead = прибери рядок: зміна чи зняття довіри не успадковує (§7.16)
trust-title = keel trust — запис довіри відбитком (§7.16)
trust-recorded-line = записано: "{ $command }" = { $fingerprint }
trust-nothing-new = нового нема: кожна команда verify/ci вже несе свій відбиток
trust-approves = рядки лягають у diff, який затверджує merge (§7.16)
trust-no-config = keel.toml нема — рядок довіри нема куди готувати
trust-no-config-instead = спершу створи конфіг: команда trust нічого не вигадує
trust-surgery-broken = хірургія не втримала форму цього файлу ({ $error }) — нічого не записано
trust-surgery-broken-instead = наведи лад у блоці [trust] руками і біжи keel trust знову

# -- holding: суд форми контрактів (§7.6, §2.9) ---------------------
holding-diverged = контракт "{ $contract }" обіцяє "{ $signature }" — "{ $name }" у коді з нею не сходиться (§2.9)
holding-diverged-instead = вирівняй код або обіцянку; зміна триманого контракту — повна хвиля зі списком впливу (§5.7)
holding-vanished = контракт "{ $contract }" обіцяє "{ $name }" — такої одиниці у файлі модуля нема (§7.6)
holding-vanished-instead = поверни одиницю або зміни/зніми контракт вголос (§2.12, §5.7)
check-holding-count = сигнатур звірено: { $count }
check-holding-uncompared = { $contract } — форму ніхто не порівнював: { $why } (§7.6)
holding-why-no-adapter = адаптер у keel.toml не названий
holding-why-unknown-adapter = названий адаптер не цього релізу (реліз обслуговує "rust")
holding-why-deep = шлях module глибший, ніж порівнює це покоління
holding-why-no-file = файл модуля в crate не знайдено
check-holding-plan = план-гілка: суд форми не біжить (§8.3) — exports ростуть наперед коду (§4.9)
check-holding-window = { $contract } — форма не судиться: обіцянку ростить затверджена, ще не почата хвиля { $wave } (§6.5); перший тег хвилі поверне суд

# -- review: пакет рецензента (§9.9) --------------------------------
review-title = keel review — пакет рецензента (§9.9)
review-wave = хвиля: { $wave }
review-why-header = ## Why, дослівно
review-why-missing = (у хвилі нема секції Why)
review-scenarios-header = ## Сценарії з редакціями (§5.3)
review-scenario-withdrawn = { " " }(знятий)
review-transforms-header = ## Трансформи, дослівно — застереження їдуть тут (§2.10)
review-chores-header = ## Причини chore (§2.11)
review-chores-none = нема
review-drift-header = ## Дрейф (§4.6) — файли, дописані в scope після якоря (перший комміт файлу хвилі: { $sha })
review-drift-line = { $file } — дописаний після якоря
review-drift-removed-line = { $file } — знятий зі scope після якоря
review-drift-empty = порожній — після якоря жодного файлу не дописано і не знято
review-drift-unverified = ## Дрейф (§4.6) не звірявся: історія не свідчить — git-а нема, або обрізаний клон не доведе якоря
review-drift-unreadable = файл хвилі на якорі не читається — дрейф суди руками
review-map-header = ## Мапа якості (§10.7)
review-impact-header = ## Вплив зміни контракту (§5.7)
review-impact-none = порожній — жоден триманий текст контракту не змінився проти точки розгалуження
review-impact-unverified = не звірявся: нема точки розгалуження для порівняння
review-impact-contract = контракт { $slug }: { $old } -> { $new }
review-impact-current = збіжна з новим текстом
review-impact-stale = стара проти нового тексту
review-diff-header = ## Повний diff гілки (проти { $base })
review-diff-empty = порожній
review-diff-unverified = ## Повний diff гілки: не звірявся — нема точки розгалуження
review-not-wave = гілка "{ $branch }" не зветься як хвиля (§8.2) — для якої хвилі збирати пакет, не вгадується
review-not-wave-instead = стань на гілку хвилі: пакет збирається для хвилі гілки (§9.9)
review-scenarios-none = нема — chore-хвиля сценаріїв не обіцяє (§6.8)
review-transform-no-body = (секції тіла нема — поверх §7.7 «шапка↔тіло» щаблем попереду; суди руками)
review-protocol-header = ## Що з цим робить рецензент (§9.9)
review-protocol-rows = кожен рядок кожного списку вище дістає відповідь — «гаразд, бо…» — або стає знахідкою; мовчки пропустити рядок не можна
review-protocol-questions = понад списками — чотири питання судження: про що ми змовчали; чи всі можливі сценарії враховані; чи реалізовано все обіцяне, без тихого звуження; чи покриває тест весь сценарій, а не його кут
review-protocol-report = звіт лягає файлом keel/reviews/{ $wave }.md поруч із хвилею — keel close тримає хвилю відкритою, доки його нема

## модуль adapter
adapter-no-crate = Cargo.toml нема ні в корені, ні рівно одного на першому рівні тек
adapter-no-crate-instead = адаптеру cargo потрібен крейт: поклади Cargo.toml у корінь або в одну теку першого рівня
adapter-many-crates = крейтів першого рівня кілька: { $found }
adapter-many-crates-instead = адаптер не вгадує; лиши один крейт на першому рівні або запусти keel із проєкту самого крейта
adapter-cargo-failed = cargo відмовляє: { $error }
adapter-cargo-failed-instead = судові потрібен робочий cargo (журнал А3); зроби так, щоб cargo тут відповідав, і повтори
adapter-battery-mismatch = cargo оголошує { $stems } цілей і друкує { $blocks } блоків вироків — зшивка не сходиться (ціль із harness = false?)
adapter-battery-mismatch-instead = суд не судить по зсунутому шву; віддай тій цілі harness або жени її окремо, тоді повтори keel close

## модуль gate
gate-mode = mode: { $mode }
gate-mode-default = mode: strict (типове — не видає себе за прочитане)
gate-manual = mode: manual — суд вимкнено, дисципліна руками (як у v1)
gate-not-wave = гілка "{ $branch }" не зветься як жодна прочитана хвиля — судити нічого, пропуск із цим словом
gate-outside = повідомлення — не народження і не робота трансформи; поза судом, пропуск із цим словом
gate-chore = трансформа — chore, обіцянок бігти нема (§2.11), пропуск
gate-red-pass = народження червоного "{ $scenario }": тест "{ $test }" справді падає — commit проходить (§7.12)
gate-red-green = заявлене народження червоного "{ $scenario }", але тест "{ $test }" зелений — незароблене «бачив червоним» в історію не вʼїжджає (§7.12)
gate-red-unknown = red: називає "{ $slug }" — це не сценарій хвилі { $wave }
gate-red-withdrawn = "{ $scenario }" знятий — мертва обіцянка не народжується (§2.12)
gate-red-untagged = заявлене народження червоного "{ $scenario }", але жоден тест не несе його тега proves (§5.5)
gate-red-many-tags = "{ $scenario }" несе { $count } тегів proves — котрий народжується, не вгадується
gate-red-broken = заявлене народження червоного "{ $scenario }", але тести не збираються — злам збірки не є падінням (А3): { $words }
gate-red-notrun = заявлене народження червоного "{ $scenario }", але біг не виконав жодного тесту "{ $test }" — нуль виконаних не є падінням (А3)
gate-work-pass = трансформа "{ $transform }": { $count } тестів сценаріїв зелені зі збіжними тегами — робота проходить (§8.4)
gate-work-red = трансформа "{ $transform }": тест "{ $test }" сценарію "{ $scenario }" падає — робота не зроблена
gate-work-stale = трансформа "{ $transform }": тег сценарію "{ $scenario }" тримає { $recorded }, а текст дає { $actual } (§7.5)
gate-work-untagged = трансформа "{ $transform }": сценарій "{ $scenario }" не має тега proves у тестах (§5.5)
gate-work-broken = трансформа "{ $transform }": тести не збираються: { $words }
gate-work-notrun = трансформа "{ $transform }": біг не виконав жодного тесту "{ $test }" сценарію "{ $scenario }"
gate-unknown-slug = "{ $slug }" — не red: і не трансформа хвилі { $wave }; одрук не проходить як «поза судом» (§8.4)
gate-case = "{ $head }" носить великі літери — red: і слаги пишуться малими (§1.2, §8.4); капіталізований двійник не проходить як «поза судом»
gate-work-vacuum = трансформа "{ $transform }": живих сценаріїв судити не лишилось — зняті поза судом (§2.12), пропуск із цим словом
gate-soft = mode: soft — ті самі слова, лише попередженням
gate-hook-installed = commit-msg hook тепер кличе keel gate — записано в { $path }
gate-adapter-unjudged = адаптер "{ $name }" не цього релізу (реліз обслуговує "rust") — комміт не суджений: слово стоїть уголос, суд чекає хвилі свого адаптера
gate-adapter-absent-name = не названий
gate-hook-already = hook уже наш — тихо той самий файл
gate-hook-foreign = commit-msg hook тут уже є, і він не наш
gate-hook-foreign-instead = keel не затирає чужий hook (§9.7); прочитай його і злий або прибери сам, тоді повтори keel hook

## команда rev
rev-title = keel rev — чинні редакції
rev-next = наступний крок: тримай ці редакції в proves/contracts і в тегах тестів (§5.5); застарілу онови, перечитавши текст (§5.1)

## команда check
check-title = keel check — документи (щабель 1)
check-config-present = конфіг: keel.toml (lang = { $lang })
check-config-absent = keel.toml нема — діють типові значення (lang = en); типове не видає себе за прочитане
check-config-lang-default = конфіг: keel.toml (lang не заданий — діє типове en; типове не видає себе за прочитане)
check-refs-count = посилань на контракти звірено: { $count }
check-refs-historic = старих редакцій, справжніх в історії файлу, у закритих хвиль: { $count } (§5.6)
check-refs-historic-item = { $wave }: { $contract }@{ $recorded } — стара, справжня в історії (§5.6)
check-refs-shallow = історія обрізана (shallow-клон) — законність старих редакцій не звірити, вирок не виноситься
check-refs-no-history = git-історії тут нема — законність старих редакцій не звірити, вирок не виноситься (§5.6)
check-tags-count = тегів тестів звірено: { $count }
check-trust-count = команд verify/ci звірено: { $count }
check-trust-ci-none = ; ci — відмова вголос: none
check-trust-ci-absent = ; ci не оголошений
check-trust-skipped-broken = команди verify/ci не суджено: битий документ може ховати саму команду — спершу полагодь названі файли
check-tags-skipped-no-adapter = теги тестів не звірялись: adapter у keel.toml не названий — названо вголос, зеленим не замальовано
check-tags-skipped-adapter = теги тестів не звірялись: адаптер "{ $name }" не цього релізу — реліз обслуговує "rust" (старе написання "cargo"); названо вголос, зеленим не замальовано
check-tags-skipped-refused = теги тестів не звірялись: адаптер відмовив посеред роботи — його відмова стоїть серед знахідок
check-scope-compared = scope: гілка "{ $branch }" і є хвиля — порівняно з { $base }
check-scope-base-main = merge-base з main @ { $sha }
check-scope-base-first = перший commit гілки @ { $sha } (main тут нема)
check-scope-skipped-not-wave = scope не звірявся: гілка "{ $branch }" не зветься як жодна прочитана хвиля (§8.2) — названо вголос, зеленим не замальовано
check-scope-skipped-no-git = scope не звірявся: git не дає гілки для цього кореня — названо вголос, зеленим не замальовано
check-scope-skipped-refused = scope не звірявся: git відмовив посеред порівняння — його відмова стоїть серед знахідок
check-header-reads = шапка читається
check-no-documents = документів ще нема
check-checked = перевірено цим поверхом: шапки — словник і форма (глави 2–4, §7.9); посилання на контракти і їхні редакції (§7.1, §7.3), стара редакція судиться по історії файлу для закритих хвиль (§5.6); звʼязки графа (глава 3: розрізи, тиша, implements, depends_on, наступники; §7.2, §10.3); scope гілки, що зветься як хвиля (§4.1, §4.4–§4.6, §4.8); редакції сценаріїв у тегах тестів (§5.5, §7.5) і зниклі теги проти точки розгалуження (§7.15); довіра команд verify/ci проти записаних відбитків (§7.16, §2.8); тримання форми контрактів (§7.6, §2.9); шапка↔тіло в обидва боки (§7.7); закриття судить keel close (§6.5)
check-adapter-synonym = adapter = "cargo" — прийнятий синонім; канонічне імʼя — мовою проєкту: adapter = "rust" (NEW-CONCEPT, Config; хвиля 0017)
check-borders = межа зеленого (§7.8): зелений тест означає «існує, збігається і проходить» — не «обіцянку доведено по суті»; зелена форма — ще не сенс. Цієї щілини механіка не закриває: її тримає свіжий рецензент чотирма питаннями (§9.9)
check-ref-missing = хвиля { $wave }: посилання { $contract }@{ $recorded } показує на контракт, файлу якого нема
check-ref-missing-instead = створи keel/contracts/{ $contract }.md або виправ слаг (§7.1)
check-ref-stale = хвиля { $wave }: записано { $contract }@{ $recorded }, а текст контракту зараз дає { $actual }
check-ref-stale-instead = перечитай контракт і онови посилання свідомо (§5.1); якщо ця хвиля вже закрита — стара редакція законна (§5.6)
check-summary = підсумок: { $docs ->
        [one] { $docs } документ
        [few] { $docs } документи
       *[many] { $docs } документів
    }, { $refusals ->
        [one] { $refusals } знахідка
        [few] { $refusals } знахідки
       *[many] { $refusals } знахідок
    }
check-next-fix = наступний крок: полагодь названі файли і повтори keel check
check-next-first-wave = наступний крок: створи першу хвилю в keel/waves/
check-next-rung = наступний крок: щабель 21 — контракт, що називає неіснуючий модуль, має бути знахідкою поза план-гілкою, а не порадою (R-13 рецензії 0022)

## команда close (§6.5)
close-title = keel close — суд закриття (§6.5)
close-battery = батарея: { $count } тестів × { $runs } біги (§7.13) — зелений лише зелений у кожному бігу
close-closed = { $wave }: закрита — кожен живий сценарій доведений, посилання сходяться, звіт рецензії поруч
close-closed-unjudged = { $wave }: закрита — кожен живий сценарій доведений, звіт рецензії поруч; посилань не звірено: { $count } — історія тут не свідчить (§5.6)
close-closed-light = { $wave }: закрита (легка) — самі chore, закрита фактом merge
close-plan = { $wave }: затверджена, ще не почата — план без тестів не червоне (§6.5)
close-progress = { $wave }: в роботі — нестачі поіменно:
close-lack-untagged = сценарій "{ $scenario }": тега proves у тестах нема (§5.5)
close-lack-stale = сценарій "{ $scenario }": тег тримає { $recorded }, а текст дає { $actual } (§7.5)
close-lack-red = сценарій "{ $scenario }": тест "{ $test }" червоний — не доведено (§6.3)
close-lack-notrun = сценарій "{ $scenario }": батарея не виконала тесту "{ $test }"
close-lack-flaky = сценарій "{ $scenario }": тест "{ $test }" зелений у { $green } з { $runs } бігів — не зелений (§7.13)
close-lack-ref = посилання { $contract }@{ $recorded } не сходиться (§6.4)
close-lack-review = звіту рецензії keel/reviews/<хвиля>.md поруч із хвилею нема (§9.9)
close-needs-adapter = судові закриття потрібен адаптер cargo, названий у keel.toml
close-needs-adapter-instead = постав adapter = "cargo" (NEW-CONCEPT, «Конфіг»); інші адаптери прийдуть своїми хвилями
close-blockers = блокери хвилі цієї гілки { $wave }: { $count } — повна хвиля не зливається недоведеною (§6.5, §9.9)
close-no-blockers = блокерів нема: гілка не зветься як незакрита хвиля — стани вище інформують
close-verify-count = verify-команд суджено: { $count }
close-verify-passed = verify "{ $command }" контракту { $contract } — пройшла
close-verify-failed = verify "{ $command }" контракту { $contract } — ВПАЛА ({ $words }) — зламана чужа обіцянка не зливається (§2.8)
close-verify-untrusted = verify "{ $command }" контракту { $contract } — не бігала: недовірена (§7.16); той вирок тримає check
close-verify-blockers = зламаних чужих обіцянок: { $count } — вихід червоний
close-verify-no-words = команда не лишила слів
close-ci-passed = ci "{ $command }" — пройшов: власний gate проєкту зелений
close-ci-failed = ci "{ $command }" — ВПАВ ({ $words }) — власний gate проєкту червоний, хвиля не зливається (§7.16); повтори команду сам, щоб побачити її слово повністю
close-ci-untrusted = ci "{ $command }" — не бігав: недовірений (§7.16); той вирок тримає check — запиши довіру рукою keel trust
close-ci-none = ci = "none" — відмова вголос, законна; нічого не біжить
close-ci-undecided = ci = "" — не вирішено; нічого не біжить (знахідка check-а)
close-ci-absent = ci не оголошено — нічого не біжить
close-ci-blocker = власний gate проєкту червоний: ci ВПАВ — вихід червоний
close-plan-own = хвиля цієї гілки — затверджена, ще не почата: план-PR зливається планом (§6.6), робота видається після

## команда map (§10.7)
map-title = keel map — мапа якості (§10.7)
map-view-wave = мапа хвилі { $wave }: гілка зветься нею (§8.2) — пункт пакета рецензента (§9.9); чесність кожного рядка лишається роботою рецензента
map-view-project = мапа проєкту: гілка "{ $branch }" не зветься хвилею — по кожному розрізу слово наймолодшої хвилі, що відповіла
map-covered = закрито: "{ $scenario }" — { $proof }
map-proof-proven = доведений (тег збіжний, §6.3; зелень тесту — суд keel close)
map-proof-unproven = ще не доведений (збіжного тега нема)
map-proof-unread = доведеність не читалась (адаптер у keel.toml не названий)
map-proof-unknown = доведеність не читалась (названий адаптер не цього релізу — реліз обслуговує "rust")
map-decided = вирішено: "{ $reason }"
map-unanswered = без відповіді — суд тиші в keel check (§10.3)
map-older = давніших відповідей: { $count }

## команда status (§6.5, §6.8, §9.2)
status-title = keel status — де ми (§6.5, §6.8)
status-branch-wave = гілка "{ $branch }" і є хвилею — ми стоїмо всередині неї (§8.2)
status-branch-plan = гілка "{ $branch }" — план-гілка: план пишеться (§8.2)
status-branch-other = гілка "{ $branch }" не зветься жодною хвилею — огляд без хвилі
status-branch-none = git не назвав гілки для цього кореня — огляд їде без неї, ніколи не здогад
status-branch-broken = гілка "{ $branch }" зветься як хвиля, чий документ відмовив — полагодь його; рядки відмов нижче
status-wave-closed = { "  " }{ $wave } — повна, закрита за структурою: теги збіжні, посилання сходяться, звіт поруч
status-wave-closed-unjudged = { "  " }{ $wave } — повна, закрита за структурою; посилань не звірено: { $count } — історія тут не свідчить (§5.6)
status-wave-closed-light = { "  " }{ $wave } — легка, закрита фактом merge (§6.8)
status-wave-light-own = { "  " }{ $wave } — легка, їде цією гілкою одним PR — закриється фактом merge (§6.8)
status-wave-plan = { "  " }{ $wave } — повна, затверджена, ще не почата (§6.5)
status-wave-progress = { "  " }{ $wave } — повна, в роботі; нестачі поіменно:
status-awaiting = { "  " }чекає старту: хвиля { $wave } — гілка "{ $wave }" (§8.2)
status-counts = пораховано: закритих { $closed }, у роботі { $working }, планів { $plans }
status-no-battery = стадія тут — структурна (теги, посилання, звіт) — батарея не бігла: зелень тестів судять close і hook (§9.2)
status-next = далі — keel next
status-needs-adapter = око стадій потребує адаптера rust, названого в keel.toml (старе написання cargo приймається): теги — памʼять стадій
status-needs-adapter-instead = постав adapter = "rust" — імʼям мови; "cargo" — прийнятий синонім (NEW-CONCEPT, Config); інші мови приїдуть своїми хвилями

## команда next (§9.2, §9.10, §8.4)
next-title = keel next — один крок (§9.2)
next-needs-adapter = рука кроку потребує адаптера rust, названого в keel.toml (старе написання cargo приймається): без тегів стадія була б здогадом
next-needs-adapter-instead = постав adapter = "rust" — імʼям мови; "cargo" — прийнятий синонім (NEW-CONCEPT, Config); інші мови приїдуть своїми хвилями
next-step-fix = крок: полагодь документ { $file } — { $reason }; натомість: { $instead }
next-step-fix-more = { "  " }і ще { $count ->
        [one] { $count } відмова
        [few] { $count } відмови
       *[many] { $count } відмов
    } — keel check назве всі
next-step-red = крок: напиши тест сценарію "{ $scenario }" і закоммить `red: { $scenario }` — він мусить упасти; хук пустить лише червоний (§7.12, §8.4)
next-body-label = { "  " }тіло сценарію (@{ $rev }), дослівно:
next-tag-line = { "  " }тег у тесті: /// proves: { $scenario }@{ $rev }
next-tests-dir = { "  " }адаптер cargo читає тести в { $dir }
next-step-stale = крок: редакція сценарію "{ $scenario }" розійшлась — тег тримає { $recorded }, тіло тепер дає { $actual }; онови тест під нове тіло і перепиши тег (§5.5)
next-step-transform = крок: трансформа "{ $name }" — працюй рівно в названих файлах, тоді закоммить `{ $name }: <слова>`; хук пустить лише зелений (§8.4)
next-step-chore = крок: chore "{ $name }" ({ $reason }) — працюй рівно в названих файлах, тоді закоммить `{ $name }: <слова>` (§2.11, §8.4)
next-files-label = { "  " }файли:
next-section-label = { "  " }секція "{ $name }", дослівно:
next-section-missing = { "  " }шапка її оголошує, а тіло хвилі не має секції "## transform: { $name }" — keel check червонить це (§7.7); полагодь тіло перед роботою
next-contract-label = { "  " }контракт { $contract }@{ $rev }, чинний текст дослівно:
next-contract-missing = файлу контракту "{ $contract }" нема — бите посилання назве keel check (§7.1)
next-run-label = { "  " }біг тестів його сценаріїв:
next-run-none = { "  " }тестів її сценаріїв ще нема — біг зʼявиться з тегами (знятий сценарій тега не дістане)
next-step-review = крок: хвиля зібрана — час рецензії (§9.9): збери пакет командою `keel review` свіжому агентові; звіт ляже в keel/reviews/{ $wave }.md
next-step-pr-light = крок: час PR — легка хвиля їде в свій один PR (§6.8), злитий кнопкою merge commit (§8.7); merge — її затвердження і закриття разом (§6.6, §6.5)
next-step-pr = крок: звіт рецензії поруч із хвилею — час PR, злитого кнопкою merge commit (§8.7); останнє слово про нестачі — за keel close
next-plan-branch = крок: це план-гілка хвилі { $wave } (§8.3) — доведи повноту плану (keel check, мапа), зливай план-PR; робота поїде гілкою "{ $wave }"
next-ready = { "  " }стартуй гілку "{ $wave }" — хвиля затверджена і не почата, її залежності закриті (§6.5, §8.2)
next-working = { "  " }гілка "{ $wave }" триває — хвиля в роботі
next-all-closed = всі хвилі закриті і жодна не чекає — час планувати нову хвилю: план цього покоління пишеться рукою, затвердження — merge файлу хвилі (§6.6)

## команда init (NEW-CONCEPT «Наскрізні», §8.7)
init-title = keel init — рама методики одним рухом
init-born = { "  " }народжено: { $piece }
init-stands = { "  " }вже стоїть: { $piece } — жоден байт не чіпається
init-fed = { "  " }догодовано: { $piece } — .gitkeep, щоб наявна порожня тека пережила git
init-failed = { "  " }не стало: { $piece } — { $error }; натомість: прибери заваду і повтори keel init
init-config-header = keel.toml — словник §2.9; розкоментуй, щоб увімкнути; типові значення лишаються словам keel
init-ignore-missing = правила ignore: git не ігнорує теки збірки адаптера ({ $path }) — допиши в .gitignore рівно цей рядок: { $rule } (рама радить; файлів самого проєкту вона не пише)
init-ignore-stands = правила ignore: тека збірки ({ $path }) ігнорується — правило дає { $source }, і воно їде з репозиторієм
init-ignore-exclude-only = правила ignore: { $path } ігнорує лише { $source }, а він з репозиторієм не їде — допиши в .gitignore рівно цей рядок: { $rule }
init-ignore-no-crate = правила ignore: адаптер не знайшов крейта, щоб назвати теку збірки ({ $error })
init-ignore-no-adapter = правила ignore: адаптера цього релізу в keel.toml не названо, тож і теки збірки нема кому назвати
init-ignore-unknown-adapter = правила ignore: адаптер названий — "{ $name }", — і цей реліз його не веде: свою теку збірки він принесе своєю хвилею
init-ignore-unjudged = правила ignore: git тут нічого не сказав ({ $error }) — правило не суджено
init-eight-seven = §8.7: вимкни squash і rebase у налаштуваннях репозиторію — правило тримає вимкнена кнопка, не памʼять
init-next = далі — keel plan <перша хвиля>

## команда plan (§10.2, §8.2, §8.5)
plan-created = народжено { $file } — риштування свідомо червоне: keel check веде (§3.3), поки план не стане повним, — тож недописане не зливається помилково
plan-branches = гілки §8.2: повна хвиля планується на "plan/{ $slug }" і працює на "{ $slug }"; легка (§6.8) вся їде "{ $slug }"
plan-branches-unread = git тут не назвав гілок — число суджено лише по хвилях диска (§8.8), сказано вголос
plan-cuts = прохід автора (§10.2): кожен із сорока розрізів дістає відповідь у covers або decisions ще до коду (§10.3); мовчання судить keel check
plan-next = далі: заповни скелет рукою — змісту плану інструмент не пише ніколи — і жени keel check (§8.3)
plan-no-number = імʼя хвилі "{ $slug }" не починається числом (§8.5)
plan-no-number-instead = почни імʼя числом хвилі, як-от 0042-session-loop — число є унікальним префіксом, ніколи не порядком
plan-number-taken = число { $number } уже тримає хвиля або гілка (§8.8)
plan-number-taken-instead = візьми наступне вільне число: { $next } — інструмент шукав по хвилях диска і всіх гілках, не лише по main
plan-number-taken-instead-disk = візьми наступне вільне число: { $next } — git тут не назвав гілок, суджено самі хвилі диска (§8.8)
plan-number-huge = число { $head } не влазить у лічбу цього покоління (§8.5)
plan-number-huge-instead = візьми коротше число — ширина цього покоління чотири знаки, і росте вона лише доки лічиться
plan-write-failed = файл не народився: { $error }
plan-write-failed-instead = перевір права на теки keel/ і повтори народження — обрубків по собі не лишається
plan-bad-slug = імʼя "{ $slug }" — не слаг
plan-bad-slug-instead = імʼя документа стає гілкою і тегом (§1.2, §8.2): лише малі латинські літери, цифри і дефіс
plan-exists = keel/waves/{ $slug }.md уже існує — ніщо ніколи не перезаписується
plan-exists-instead = заповни наявний файл або вибери нове імʼя для нової хвилі
plan-skel-header = риштування хвилі { $slug } — заповни обіцянки рукою і прибери ці слова (§10.2)
plan-skel-why = чому ця хвиля мусить існувати — своїми словами (§2.2)
plan-skel-scenario = **Дано** ..., **коли** ..., **тоді** ... (§2.3); кожна обіцянка спирається на proves або covers (§3.3)
plan-skel-transform = слова роботи; застереження живуть тут (§2.10)
newc-created = народжено { $file } — він свідомо ще не обіцяє нічого: keel check веде (§2.9), поки не стануть exports із module або verify
newc-next = далі: дай сигнатури з module (§2.7) або команду verify (§2.8) рукою — keel check скаже, чого бракує
newc-exists = keel/contracts/{ $slug }.md уже існує — ніщо ніколи не перезаписується
newc-exists-instead = заповни наявний файл або вибери нове імʼя для нового контракту
newc-skel-header = риштування контракту { $slug }: обіцянки ще нема — заповни §2.7 або §2.8 і прибери риштування
newc-skel-body = чиїм словам цей контракт дає пережити хвилю — і чому (§2.6)

## команда version (NEW-CONCEPT, таблиця команд; хвиля 0018)
version-running = keel { $version } — бінарник, що відповідає
version-pin-held = пін keel.toml: "{ $pin }" — тримається; суди судять саме цим бінарником
version-pin-mismatch = пін keel.toml: "{ $pin }" — НЕ цей бінарник: суди відмовляють, поки пін і бінарник не зійдуться
version-pin-none = поле version не задано — піна нема; концепт радить пін: version = "{ $version }"
version-no-file = keel.toml нема — піна нема, біжить бінарник вище
version-unread = keel.toml не прочитано ({ $reason }) — пін невідомий; відмову повністю скаже суд config

update-title = keel update — згенеровані інтеграції оновлено (NEW-CONCEPT, «Дистрибуція»)
generated-born = { $file } — народжено цим релізом
generated-appended = { $file } — блок keel дописано; текст над ним не чіпано
generated-refreshed = { $file } — оновлено цим релізом
generated-stands = { $file } — уже стоїть таким, яким його пише цей реліз
generated-removed = { $file } — прибрано рукою: то рішення, а не прогалина; нічого не дописується. Щоб мати його знову, прибери його рядок у [generated] keel.toml і повтори keel update
{ $snippet }
next-unknown-agent = агент "{ $agent }" не з тих, що знає цей реліз: { $known }
next-unknown-agent-instead = назви одного з { $known } — форма відповіді сесійного hook-а належить самому агентові, і в неназваного нема документованої форми, якою говорити
generated-guest-empty = { $file } — цей файл є і він порожній, а keel не пише поверх того, чого не писав. Прибери його і повтори keel update — або встав у нього цілий документ сам:
{ $snippet }
generated-guest-taken = { $file } — цей файл ваш, і keel не написав у нього нічого: не зрушено жодного байта. Якщо hook лупа потрібен — додай ці записи ВСЕРЕДИНУ ключа "{ $key }" свого файлу, а якщо такого ключа ще нема — додай ключ саме з цим вмістом. Не встромляй їх поруч із зовнішніми дужками файлу: це не буде JSON, а злиття поверх твоїх власних записів їх втратить:
{ $snippet }
generated-guest-edited = { $file } — цей файл написав keel, а потім його правила рука, тож keel не написав поверх нічого. Лиши як є або поверни ці записи в ключ "{ $key }"; а щоб віддати файл keel-ові цілком — прибери ФАЙЛ І його рядок у [generated] keel.toml, тоді повтори keel update:
{ $snippet }
generated-foreign-file = { $file } — файл із цим імʼям генерує keel, і те, що тут стоїть, не є тим, що пише цей реліз; про нього нічого не записано, тож він не наш: НЕ перезаписано (§9.7). Лиши як є або прибери ФАЙЛ, тоді повтори keel update (цей шлях — спільний простір імен, файл може належати іншому інструментові)
generated-changed-file = { $file } — цей файл генерує keel, і те, що тут стоїть, не є ні тим, що пише цей реліз, ні тим, що записано (записано { $recorded }, знайдено { $actual }): НЕ перезаписано (§9.7). Лиши як є або прибери ФАЙЛ І його рядок у [generated] keel.toml, тоді повтори keel update
generated-changed = { $file } — блок keel не є ні тим, що пише цей реліз, ні тим, що записано (записано { $recorded }, знайдено { $actual }): НЕ перезаписано (§9.7). Лиши як є або прибери блок І його рядок у [generated] keel.toml, тоді повтори keel update
generated-no-config = keel.toml тут нема: це не keel-проєкт, і нічого не вигадується. Зроби його ним командою keel init
generated-many-blocks = { $file } — тут стоїть більше ніж один блок keel: котрий наш, не вгадується. Лиши одну пару маркерів і повтори keel update
generated-none = нічого
generated-half-marked = { $file } — один маркер keel без другого: де кінчається блок, не вгадується; полагодь маркери або прибери обидва
generated-unjudged-config = згенеровані інтеграції не суджено: keel.toml не читається (причину каже суд config)
generated-unread = { $file } не прочитано ({ $error }) — не суджено
generated-write-failed = { $file } не записано ({ $error })
generated-config-failed = digest файлу { $file } не записано в keel.toml ({ $error })
generated-config-failed-instead = полагодь keel.toml, щоб він парсився, і повтори keel update

## рамка CLI
main-unknown-command = відмова: невідома команда "{ $command }"
main-unknown-command-reason = причина: такої команди keel не знає
main-no-command = відмова: не названо команди
main-no-command-reason = причина: keel не вгадує, що робити
main-gate-no-message = відмова: gate потребує файл повідомлення commit-а
main-gate-no-message-reason = причина: суд читає повідомлення, яке віддає commit-msg hook
main-plan-no-slug = відмова: plan потребує імени нової хвилі
main-plan-no-slug-reason = причина: скелет народжується під імʼям, що стане його файлом і гілками (§8.2)
main-new-unknown = відмова: keel new знає лише: contract
main-new-unknown-reason = причина: інші види документів народжуються своїми командами (хвилі — keel plan)
main-new-no-slug = відмова: new contract потребує імени контракту
main-new-no-slug-reason = причина: скелет народжується під імʼям, що стане його файлом (§1.4)
main-usage = натомість: keel check [тека] | keel rev [--write] [тека] | keel gate <файл-повідомлення> [тека] | keel close [тека] | keel map [тека] | keel review [тека] | keel status [тека] | keel next [тека] | keel plan <слаг> [тека] | keel new contract <слаг> [тека] | keel init [тека] | keel trust [тека] | keel hook [тека] | keel version [тека] | keel update [тека]

# The settings wizard (wave 0026)
ask-lang = Якою людською мовою говорить цей проєкт? (його проза і його відмови)
ask-adapter = Якою мовою написаний код? («-» лишає поле неназваним)
ask-mode = Наскільки суворий суд commit-ів? (strict заслоняє, soft попереджає, manual вимкнено)
ask-agents = Для яких агентів keel має генерувати інтеграції? (пробіл — відмітити, мінімум один)
ask-hooks = Поставити сесійні hook-и, щоб агент знав наступний крок одразу на старті?
ask-unknown-field = «{ $field }» — не з тих налаштувань, про які питає цей реліз
ask-unknown-field-instead = налаштування такі: { $known }
ask-unknown-value = «{ $value }» — не з тих значень, які бере налаштування «{ $field }»
ask-unknown-value-instead = { $field } бере одне з: { $known }
ask-nobody = «{ $field }» не називає нікого, а треба щонайменше одного
ask-nobody-instead = назви щонайменше одного з { $known } — відмітити можна кількох, але мінімум одного
ask-interrupted = питання про «{ $field }» лишилось без відповіді: { $error }
ask-interrupted-instead = відповідай, або дай відповіді прапорцями (--lang, --adapter, --mode, --agents, --hooks), або жени keel init --no-ask по типові значення
init-config-answered = keel.toml — народжено з твоїх відповідей

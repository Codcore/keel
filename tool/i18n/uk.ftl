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

## модуль graph
graph-unknown-cut = "{ $holder }" показує на розріз "{ $slug }", якого у словнику нема
graph-unknown-cut-instead = сорок розрізів їдуть з релізом (§3.4); вибери один із них або виправ одрук
graph-double-cover = розріз "{ $slug }" закритий двічі: сценарії { $holders } (§10.3 — рівно одна відповідь)
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
check-tags-skipped-no-adapter = теги тестів не звірялись: adapter у keel.toml не названий — названо вголос, зеленим не замальовано
check-tags-skipped-adapter = теги тестів не звірялись: адаптер "{ $name }" цим щаблем не обслуговується (лише cargo) — названо вголос, зеленим не замальовано
check-tags-skipped-refused = теги тестів не звірялись: адаптер відмовив посеред роботи — його відмова стоїть серед знахідок
check-scope-compared = scope: гілка "{ $branch }" і є хвиля — порівняно з { $base }
check-scope-base-main = merge-base з main @ { $sha }
check-scope-base-first = перший commit гілки @ { $sha } (main тут нема)
check-scope-skipped-not-wave = scope не звірявся: гілка "{ $branch }" не зветься як жодна прочитана хвиля (§8.2) — названо вголос, зеленим не замальовано
check-scope-skipped-no-git = scope не звірявся: git не дає гілки для цього кореня — названо вголос, зеленим не замальовано
check-scope-skipped-refused = scope не звірявся: git відмовив посеред порівняння — його відмова стоїть серед знахідок
check-header-reads = шапка читається
check-no-documents = документів ще нема
check-checked = перевірено цим поверхом: шапки — словник і форма (глави 2–4, §7.9); посилання на контракти і їхні редакції (§7.1, §7.3), стара редакція судиться по історії файлу для закритих хвиль (§5.6); звʼязки графа (глава 3: розрізи, тиша, implements, depends_on, наступники; §7.2, §10.3); scope гілки, що зветься як хвиля (§4.1, §4.4–§4.6, §4.8); редакції сценаріїв у тегах тестів (§5.5, §7.5) і зниклі теги проти точки розгалуження (§7.15); закриття судить keel close (§6.5)
check-unchecked = ще не перевірено: тримання контрактів (§7.6), шапка↔тіло (§7.7) — щаблі попереду
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
check-next-rung = наступний крок: щабель 7 — довіра TOFU (§7.16) і CLI

## команда close (§6.5)
close-title = keel close — суд закриття (§6.5)
close-battery = батарея: { $count } тестів суджено одним бігом
close-closed = { $wave }: закрита — кожен живий сценарій доведений, посилання сходяться, звіт рецензії поруч
close-closed-unjudged = { $wave }: закрита — кожен живий сценарій доведений, звіт рецензії поруч; посилань не звірено: { $count } — історія тут не свідчить (§5.6)
close-closed-light = { $wave }: закрита (легка) — самі chore, закрита фактом merge
close-plan = { $wave }: затверджена, ще не почата — план без тестів не червоне (§6.5)
close-progress = { $wave }: в роботі — нестачі поіменно:
close-lack-untagged = сценарій "{ $scenario }": тега proves у тестах нема (§5.5)
close-lack-stale = сценарій "{ $scenario }": тег тримає { $recorded }, а текст дає { $actual } (§7.5)
close-lack-red = сценарій "{ $scenario }": тест "{ $test }" червоний — не доведено (§6.3)
close-lack-notrun = сценарій "{ $scenario }": батарея не виконала тесту "{ $test }"
close-lack-ref = посилання { $contract }@{ $recorded } не сходиться (§6.4)
close-lack-review = звіту рецензії keel/reviews/<хвиля>.md поруч із хвилею нема (§9.9)
close-needs-adapter = судові закриття потрібен адаптер cargo, названий у keel.toml
close-needs-adapter-instead = постав adapter = "cargo" (NEW-CONCEPT, «Конфіг»); інші адаптери прийдуть своїми хвилями
close-blockers = блокери хвилі цієї гілки { $wave }: { $count } — повна хвиля не зливається недоведеною (§6.5, §9.9)
close-no-blockers = блокерів нема: гілка не зветься як незакрита хвиля — стани вище інформують
close-plan-own = хвиля цієї гілки — затверджена, ще не почата: план-PR зливається планом (§6.6), робота видається після

## команда map (§10.7)
map-title = keel map — мапа якості (§10.7)
map-view-wave = мапа хвилі { $wave }: гілка зветься нею (§8.2) — пункт пакета рецензента (§9.9); чесність кожного рядка лишається роботою рецензента
map-view-project = мапа проєкту: гілка "{ $branch }" не зветься хвилею — по кожному розрізу слово наймолодшої хвилі, що відповіла
map-covered = закрито: "{ $scenario }" — { $proof }
map-proof-proven = доведений (тег збіжний, §6.3; зелень тесту — суд keel close)
map-proof-unproven = ще не доведений (збіжного тега нема)
map-proof-unread = доведеність не читалась (адаптер у keel.toml не названий)
map-decided = вирішено: "{ $reason }"
map-unanswered = без відповіді — суд тиші в keel check (§10.3)
map-older = давніших відповідей: { $count }

## рамка CLI
main-unknown-command = відмова: невідома команда "{ $command }"
main-unknown-command-reason = причина: такої команди keel не знає
main-no-command = відмова: не названо команди
main-no-command-reason = причина: keel не вгадує, що робити
main-gate-no-message = відмова: gate потребує файл повідомлення commit-а
main-gate-no-message-reason = причина: суд читає повідомлення, яке віддає commit-msg hook
main-usage = натомість: keel check [тека] | keel rev [тека] | keel gate <файл-повідомлення> [тека] | keel close [тека] | keel map [тека] | keel hook [тека]

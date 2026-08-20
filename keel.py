#!/usr/bin/env python3
"""keel — the tool behind the Keel method.

One file, standard library only. It knows state; it writes no prose.
What it says to a person follows `lang` in the project's config, which defaults
to English: the messages are prose, not code, so they are translatable, and the
catalogue is keyed by the English text.

    keel new wave <slug>       skeleton of a wave
    keel new contract <slug>   skeleton of a contract
    keel gaps                  what the wave description is missing
    keel next                  package for the next move
    keel check                 every check, and the project's own gate
    keel rev                   revisions that have drifted apart
    keel show                  a wave as a person reads it
    keel hooks                 git hooks: pre-commit and pre-push
    keel skills                regenerate the skills from the methodology
    keel init                  put Keel into a project
    keel update                bring a project's copies up to date
    keel hook <event>          answer an agent hook; called by a config
"""

import argparse
import functools
import hashlib
import json
import os
import re
import subprocess
import sys

VERSION = "0.6.0"

# ─────────────────────────────────────────────────────────────────────────────
# What the tool says
#
# English is the key, not a slug: a missing translation degrades to readable
# English instead of to a lookup error, and the code stays legible without the
# catalogue open beside it. The language follows `lang` from the project's
# config — the same setting that decides what the agent writes and which phrases
# the skills catch, because all three are read by the same person.
#
# The command line itself stays English: flag names, metavars and --help are the
# interface's own vocabulary, like `--force` is.
# ─────────────────────────────────────────────────────────────────────────────

OUTPUT_LANG = "en"
UK = {
    # читач YAML
    "line {line}: {message}": "рядок {line}: {message}",
    "unclosed quote": "лапки не закриті",
    "a quote in the middle of a value: {value}":
        "лапка посеред значення: {value}",
    "a backslash at the end of a value": "зворотний слеш у кінці значення",
    "unknown escape: {escape}": "невідоме екранування: {escape}",
    "stray bracket": "зайва дужка",
    "unclosed bracket": "дужка не закрита",
    "a map inside a list is not supported: {item}":
        "мапа в списку не підтримується: {item}",
    "a list inside a list is not supported: {item}":
        "список у списку не підтримується: {item}",
    "a list on the key's own line is not supported: {item}":
        "список на тому ж рядку, що й ключ, не підтримується: {item}",
    "list is not closed by a bracket": "список не закритий дужкою",
    "map is not closed by a bracket": "мапа не закрита дужкою",
    "no colon in the map entry: {entry}": "у мапі немає двокрапки: {entry}",
    "duplicate key {key}": "ключ {key} повторюється",
    "indented with a tab": "відступ табуляцією",
    "unexpected indent": "несподіваний відступ",
    "a header has to be a set of keys, not a list":
        "шапка має бути набором ключів, а не списком",
    "a list where a key was expected": "список там, де очікується ключ",
    "no key before the colon: {line}": "немає ключа перед двокрапкою: {line}",
    "two colons on one line: {line}": "дві двокрапки в одному рядку: {line}",
    "empty key": "порожній ключ",
    "no header between --- markers": "немає шапки між рисками ---",
    "the test command could not run ({command}): {reason}":
        "команда тестів не запустилась ({command}): {reason}",
    "{command} could not run: {reason}": "{command} не запустився: {reason}",
    "{file} does not parse — fix it first. Regenerating on the defaults would "
    "rewrite the project in the wrong language.":
        "{file} не читається — спершу полагодьте його. Перегенерація за "
        "замовчуваннями переписала б проєкт не тією мовою.",
    "the contract promises exports and names no module to ask for them":
        "контракт обіцяє exports і не називає модуля, в якого їх спитати",
    "{kind} {slug} has to be a set of named fields, and this is {actual}":
        "{kind} {slug} має бути набором іменованих полів, а це {actual}",
    "{field} of transform {slug} has to be a list, and this is {actual}":
        "{field} трансформи {slug} має бути списком, а це {actual}",
    "proves of scenario {slug} has to be a list, and this is {actual}":
        "proves сценарію {slug} має бути списком, а це {actual}",
    "the root matches {count} languages ({names}), and {picked} was taken "
    "because it comes first. Say which in keel/keel.json: \"adapter\": \"{picked}\"":
        "корінь підходить під {count} мови ({names}), а взято {picked}, бо він "
        "перший у списку. Скажіть яку в keel/keel.json: \"adapter\": \"{picked}\"",
    "cannot be read: {reason}": "не читається: {reason}",
    "this link leaves the repository: {target}":
        "це посилання виходить за межі репозиторію: {target}",
    "keel: {file} does not parse, so scope is not being checked: {reason}":
        "keel: {file} не розбирається, тож обсяг не перевіряється: {reason}",
    "keel: wave {wave} declares no transforms, so nothing says which files "
    "belong to this work.":
        "keel: хвиля {wave} не оголошує трансформ, тож ніщо не каже, які файли "
        "належать цій роботі.",
    "{field} has to be {kind}, and this is {actual}":
        "{field} має бути {kind}, а це {actual}",
    "a list": "списком",
    "a set of named entries": "набором іменованих записів",
    "a name": "імʼям",
    "header does not parse: {reason}": "шапка не читається: {reason}",

    # назви перевірок
    "references lead somewhere": "посилання ведуть кудись",
    "depends_on without cycles": "depends_on без циклів",
    "contract revisions match": "редакції контрактів збіглися",
    "changed files match those declared": "змінені файли збігаються з оголошеними",
    "every scenario has a green test": "у кожного сценарію зелений тест",
    "contracts hold": "контракти справджуються",
    "names in the header match the headings": "імена в шапці збігаються із заголовками",

    # перевірки
    "scenario {slug}": "сценарій {slug}",
    "transform {slug}": "трансформа {slug}",
    "depends_on points at a wave that does not exist: {slug}":
        "depends_on показує на хвилю, якої немає: {slug}",
    "scenario {scenario} proves a contract that does not exist: {slug}":
        "сценарій {scenario} доводить контракт, якого немає: {slug}",
    "transform {transform} implements a scenario that does not exist: {scenario}":
        "трансформа {transform} наближає сценарій, якого немає: {scenario}",
    "transform {transform} implements a contract that does not exist: {slug}":
        "трансформа {transform} реалізує контракт, якого немає: {slug}",
    "this link leads nowhere: {target}": "посилання нікуди не веде: {target}",
    "cycle in depends_on: {cycle}": "цикл у depends_on: {cycle}",
    "{who} leans on {slug} without a revision; it is now {now}":
        "{who} спирається на {slug} без редакції; зараз {now}",
    "{who} holds {slug}@{held}, and the contract is now {now}":
        "{who} тримає редакцію {slug}@{held}, а контракт зараз {now}",
    "this is not a git repository — nothing to check scope against":
        "це не git-репозиторій — межі перевірити нічим",
    "the head is detached and git does not know the branch name — "
    "pass it with --branch":
        "HEAD відчеплений, імені гілки git не знає — передай його прапорцем --branch",
    "cannot tell where this branch left from: {main} is missing or the history "
    "is truncated. Scope was not compared, which is not the same as scope being "
    "intact.":
        "не знайшов, від чого відійшла гілка: {main} немає або історія обрізана. "
        "Межі не звірено — це не означає, що вони цілі.",
    "a plan branch is touching code: {name}": "гілка плану чіпає код: {name}",
    "branch {branch} is not named after a wave — there is nothing to compare "
    "scope against":
        "гілка {branch} не називається хвилею — немає з чим звіряти межі",
    "changed but not declared: {name}": "файл змінено, але не оголошено: {name}",
    "declared but not changed: {name}": "файл оголошено, але не змінено: {name}",
    "nothing to run the tests with: the root has none of {markers}":
        "не знайшов, чим запускати тести: у корені немає жодного з {markers}",
    "scenario {slug} has no test": "сценарій {slug} не має тесту",
    "scenario {slug} is declared by more than one wave ({waves}), and a test "
    "tag names only the slug — it cannot say which":
        "сценарій {slug} оголошено більш ніж однією хвилею ({waves}), а тег "
        "тесту називає лише слаг — він не може сказати, котрий",
    "the test for {slug} carries no revision; it is now {now}":
        "тест сценарію {slug} без редакції; зараз {now}",
    "the test holds {slug}@{held}, and the scenario is now {now}":
        "тест тримає редакцію {slug}@{held}, а сценарій зараз {now}",
    "the tests are red ({command}):": "тести червоні ({command}):",
    "verify has to be a command as a string, and this is {kind}: {value}":
        "verify має бути командою рядком, а тут {kind}: {value}",
    "the contract did not answer within {seconds}s: {command}":
        "контракт не відповів за {seconds} с: {command}",
    "the contract was not confirmed: {command}": "контракт не підтвердився: {command}",
    "no language adapter found — nothing to check exports with":
        "не знайшов адаптера мови — експорти перевірити нічим",
    "the modules did not build:": "модулі не зібралися:",
    "the module is missing or did not build: {module}":
        "модуля немає або він не зібрався: {module}",
    "this export is neither name/arity nor a spec: {promised}":
        "цей запис не є ні імʼям з арністю, ні специфікацією: {promised}",
    "{language} cannot be asked for types, so the promised shape of {promised} goes unchecked":
        "{language} не вміє відповідати про типи, тож обіцяна форма {promised} лишається неперевіреною",
    "{module} declares no @spec for {promised}":
        "{module} не оголошує @spec для {promised}",
    "the promised shape of {promised} is not what the module declares:":
        "обіцяна форма {promised} не збігається з тим, що оголошує модуль:",
    "{module} does not export what was promised: {promised}":
        "{module} не експортує обіцяне: {promised}",
    "the heading ## {title} appears twice — the first is read and the last is "
    "counted":
        "заголовок ## {title} трапляється двічі — читають перший, а рахується останній",
    "the header has {kind} {slug} and the body has no section for it":
        "у шапці є {kind} {slug}, у тілі секції немає",
    "the body has ## {kind}: {slug} and the header does not":
        "у тілі є ## {kind}: {slug}, у шапці його немає",
    "the translation names no source revision; it is now {now}":
        "переклад не називає редакції джерела; зараз {now}",
    "the translation holds {held}, and {name} is now {now}":
        "переклад тримає {held}, а {name} зараз {now}",

    # вивід команд
    '<what was done>': '<що зроблено>',
    'Wave {id} · {file}': 'Хвиля {id} · {file}',
    'Closed: {names}': 'Закрито: {names}',
    'After this one: {names}': 'Після цієї: {names}',
    'Why the wave': 'Навіщо хвиля',
    'This transform': 'Ця трансформа',
    'The files, and only these': 'Файли, і тільки вони',
    '(none declared — the plan is incomplete)': '(не оголошено — план неповний)',
    'Scenarios it brings closer': 'Сценарії, які вона наближає',
    '(no body)': '(тіла немає)',
    'Contracts it leans on': 'Контракти, на які вона спирається',
    'Exports: {names}': 'Експортує: {names}',
    '(no such contract)': '(контракту немає)',
    'The commit': 'Комміт',
    'Depends on:': 'Залежить від:',
    'Scenarios': 'Сценарії',
    'Transforms': 'Трансформи',
    'no body': 'тіла немає',
    'closed {sha}': 'закрита {sha}',
    'open': 'відкрита',
    'Brings closer: {names}': 'Наближає: {names}',
    'not there yet': 'ще немає',
    '(new)': '(нове)',
    'ours': 'наш',
    "another tool's": 'чужий',
    'missing': 'немає',
    'none': 'жодного',
    '(block between the markers)': '(блок між маркерами)',
    'wave {slug}': 'хвиля {slug}',
    'there is no wave file for {branch} yet': 'файла хвилі для {branch} ще немає',
    "(not run)": "(не запускалась)",
    "documents do not parse": "документи не читаються",
    "documents disagree with each other": "документи суперечать одне одному",
    "clean": "чисто",
    "problems: {count}": "проблем: {count}",
    "(not run: a plan branch has no code)": "(не запускалась: на гілці плану коду немає)",
    "no CI command: merges go with nothing of the project's own run. Name one "
    "in {file} (\"ci\": \"{example}\"), or say there is none (\"ci\": \"none\").":
        "команди CI немає: злиття йдуть без жодного власного прогону проєкту. "
        "Назвіть її у {file} (\"ci\": \"{example}\") або скажіть, що її не буде "
        "(\"ci\": \"none\").",
    "your command": "вашу команду",
    "CI did not finish within {seconds}s: {command}":
        "CI не вклався в {seconds} с: {command}",
    "CI could not run ({command}): {reason}":
        "CI не запустився ({command}): {reason}",
    "CI is not set up: {command} — there is no such command":
        "CI не налаштований: {command} — такої команди немає",
    "CI is red: {command}": "CI червоний: {command}",
    "no waves yet. The first one starts with a plan: keel new wave <slug>, then "
    "the branch plan/<that name>.":
        "хвиль поки немає. Перша починається з плану: keel new wave <слаг>, а "
        "тоді гілка plan/<те саме імʼя>.",
    "{count} tests did not run — skipped or excluded. The runner does not say "
    "which, so any scenario among them is unproven.":
        "{count} тестів не виконувались — пропущені або виключені. Раннер не "
        "каже, які саме, тож будь-який сценарій серед них недоведений.",
    "the whole hooks key": "весь ключ hooks",
    "{file}: {what} is somebody else's shape, so the write guard was not "
    "installed there — put it in by hand or move what is in the way":
        "{file}: {what} має чужу форму, тож заслон перед записом туди не "
        "встановлено — впишіть руками або приберіть те, що заважає",
    "{name} was deleted on this branch. A wave or a contract outlives the "
    "branch that removes it — say so in the pull request, or put it back.":
        "{name} видалено на цій гілці. Хвиля і контракт живуть довше за гілку, "
        "яка їх прибирає — скажіть про це в PR або поверніть на місце.",
    "wave {wave} has two scenarios that read as {slug} once dashes and "
    "underscores are levelled, and a tag names only that — rename one":
        "у хвилі {wave} два сценарії читаються як {slug}, щойно зрівняти дефіси "
        "з підкресленнями, а тег називає тільки це — перейменуйте один",
    "keel: the head is detached, so there is no wave to judge this write "
    "against. Scope is not being checked.":
        "keel: голова відчеплена, тож немає хвилі, за якою судити цей запис. "
        "Межі не перевіряються.",
    "contract {slug} promises nothing that can be checked: no exports to compare "
    "and no verify to run":
        "контракт {slug} не обіцяє нічого перевірного: ні експортів для звірки, "
        "ні команди verify",
    "(the tests and probes were not run)": "(тести й проби не запускались)",
    "the test for {slug} did not run: {name} is skipped. A test that does not "
    "run proves nothing.":
        "тест для {slug} не виконувався: {name} пропущено. Тест, який не "
        "біжить, нічого не доводить.",
    "keel: the hook payload carried no file path, so this write was not judged. "
    "This is {main}, where finished work arrives.":
        "keel: у виклику хука не було шляху до файла, тож цей запис ніхто не "
        "судив. Це {main}, куди готова робота приїжджає.",
    "the documents do not agree with themselves, so there is no saying what is "
    "done: {reason}":
        "документи не узгоджені між собою, тож сказати, що зроблено, не можна: "
        "{reason}",
    "transform {name} is declared by {others} as well, and a commit naming it "
    "would close both. A slug is the only link between a commit and its "
    "transform, so it belongs to one wave.":
        "трансформу {name} оголошує ще й {others}, і комміт із цим імʼям закрив "
        "би обидві. Слаг — єдиний звʼязок між коммітом і трансформою, тож він "
        "належить одній хвилі.",
    "{name} was added to wave {wave} on this branch, not in the plan that was "
    "approved. Allowed — say in the pull request what widened and why.":
        "{name} додано до хвилі {wave} на цій гілці, а не в схваленому плані. "
        "Дозволено — скажіть у PR, що саме розширили й чому.",
    "contract {slug} arrives with this wave, and no transform or scenario leans "
    "on it — deliberate?":
        "контракт {slug} приходить із цією хвилею, і жодна трансформа чи "
        "сценарій на нього не спираються — це навмисно?",
    "{file} differs from what was approved: +{added} -{removed}. Allowed, and "
    "it stays a line in the diff — say in the pull request what changed and why.":
        "{file} відрізняється від схваленого: +{added} -{removed}. Дозволено, і "
        "воно лишається рядком у diff — скажіть у PR, що змінилось і чому.",
    "wave {other} declares {name} too, and depends_on does not name it — "
    "deliberate?":
        "хвиля {other} теж оголошує {name}, а depends_on його не називає — "
        "це навмисно?",
    "every wave is finished. The next one starts with a plan: keel new wave "
    "<slug>, then the branch plan/<that name>.":
        "усі хвилі завершені. Наступна починається з плану: keel new wave "
        "<слаг>, а тоді гілка plan/<те саме імʼя>.",
    "{waves}: the plan is not written yet — no transforms, so there is no work "
    "to hand out. keel gaps says what is missing.":
        "{waves}: план ще не написаний — трансформ немає, тож і роздавати "
        "нічого. keel gaps каже, чого бракує.",
    "every unfinished wave is waiting on another that is not done yet: {waves}. "
    "Finish what they lean on, or plan the wave that is missing.":
        "кожна незавершена хвиля чекає на іншу, яка ще не зроблена: {waves}. "
        "Закінчіть те, на що вони спираються, або заплануйте хвилю, якої бракує.",
    "wave {wave} is approved and {open} of {total} transforms are not closed. "
    "The work goes on its own branch:\n"
    "  git checkout -b {wave}":
        "хвиля {wave} схвалена, і {open} з {total} трансформ не закриті. Робота "
        "йде на власній гілці:\n"
        "  git checkout -b {wave}",
    "{name}: this is {main}, where finished work arrives — it is not where work "
    "is written. Code belongs on a branch named after a wave: check out the wave "
    "you are working on, or plan a new one with keel new wave.":
        "{name}: це {main}, куди готова робота приїжджає, — а не там, де її "
        "пишуть. Кодові місце на гілці, названій за хвилею: перейдіть на хвилю, "
        "над яким працюєте, або заплануйте новий через keel new wave.",
    "the plan is missing things": "планові дечого бракує",
    "{waves}: this branch did not come to write that wave. Somebody else's wave "
    "is not moved, renamed or deleted to get a check green — leave it and say it "
    "is there.":
        "{waves}: ця гілка писати ту хвилю не приходила. Чужу хвилю не пересувають, "
        "не перейменовують і не видаляють заради зеленої перевірки — лишіть його "
        "на місці й скажіть, що він там є.",
    "bad slug: {slug}": "поганий слаг: {slug}",
    "already there: {path}": "вже є: {path}",
    "no such wave: {wave}": "хвилі немає: {wave}",
    "branch {branch}": "гілка {branch}",
    "the Why section is empty": "секція «Навіщо» порожня",
    "no scenarios at all": "жодного сценарію",
    "no transforms at all": "жодної трансформи",
    "transform {slug} declared no files": "трансформа {slug} не оголосила файлів",
    "transform {slug} implements no scenario":
        "трансформа {slug} не наближає жодного сценарію",
    "transform {slug} has no body: what it does and where its edges are":
        "трансформа {slug} без тіла: що робить і де межі",
    "scenario {slug} has no proves": "сценарій {slug} не має proves",
    "no transform implements scenario {slug}":
        "сценарій {slug} не наближає жодна трансформа",
    "scenario {slug} has no body: given/when/then":
        "сценарій {slug} без тіла: given/when/then",
    "the plan is complete: {names}": "план повний: {names}",
    "the plan is missing things ({names}):": "плану бракує ({names}):",
    "in total: {count}": "всього: {count}",
    "{file}: does not parse as JSON, leaving it alone":
        "{file}: не читається як JSON, не чіпаю",
    "{file}: not an object, leaving it alone": "{file}: не обʼєкт, не чіпаю",
    "the skills did not change": "скіли не змінились",
    "{file}: the keel markers are out of balance — fix them by hand, this "
    "block was not touched":
        "{file}: маркери keel розбалансовані — полагодьте руками, блок не "
        "чіпався",
    "the tests did not finish within {seconds}s ({command}). Nothing was proved, "
    "which is not the same as nothing being wrong.":
        "тести не завершились за {seconds}с ({command}). Нічого не доведено, а це "
        "не те саме, що «нічого не зламано».",
    "{command} did not answer within {seconds}s":
        "{command} не відповів за {seconds}с",
    "path/to/file": "шлях/до/файлу",
    "What exactly is promised, and to whom.": "Що саме обіцяно й кому.",
    "Why": "Навіщо",
    "why this wave exists and what is missing without it":
        "навіщо ця хвиля і чого без неї бракує",
    "What it does.": "Що робить.",
    "Boundaries": "Межі",
    "what it does not do.": "чого не робить.",
    "A module that promises something:": "Модуль, який щось обіцяє:",
    "A name with an arity or a whole signature — what is written is what gets "
    "checked:":
        "Імʼя з арністю або ціла сигнатура — перевіряється те, що написано:",
    "Or a promise that is not a module — a command whose success is the proof:":
        "Або обіцянка, що не є модулем — команда, успіх якої і є доказом:",
    "{file}: not what Keel wrote, leaving it in place — the hooks in it still run":
        "{file}: це не те, що писав Keel, лишаю на місці — хуки в ньому далі "
        "працюють",
    "{file} removed": "{file} прибрано",
    "{file} removed — no longer part of the methodology":
        "{file} прибрано — його вже немає в методиці",
    "{file}: no longer part of the methodology, and not what Keel wrote — "
    "leaving it in place":
        "{file}: у методиці його вже немає, а писав його не Keel — лишаю на місці",
    "{file} (our hook entries taken out)": "{file} (наші записи хуків вилучено)",
    "Implements: [{slug}](../contracts/{slug}.md)@{rev}":
        "Виконує: [{slug}](../contracts/{slug}.md)@{rev}",
    "Proves: {proves} · revision `{rev}`":
        "Доводить: {proves} · ревізія `{rev}`",
    "Test tag: `{tag}`": "Тег тесту: `{tag}`",
    "The skills /keel-plan, /keel-work and /keel-review are in place. Start the "
    "agent in the project directory itself:\n  cd {root} && <agent>\nIf it answers "
    "\"Unknown skill\" they have not been picked up yet: /reload-skills, or simply "
    "call again. A session opened before the install has to be restarted; /clear "
    "does not register the directory.":
        "Скіли /keel-plan, /keel-work і /keel-review на місці. Запускай агента в "
        "самій теці проєкту:\n  cd {root} && <агент>\nЯкщо він відповідає "
        "\"Unknown skill\" — вони ще не підхопились: /reload-skills або просто "
        "поклич ще раз. Сесію, відкриту до встановлення, треба перезапустити; "
        "/clear теки не реєструє.",
    "The transform slug in the message is the only link between the work and the plan.":
        "Слаг трансформа в повідомленні — єдиний звʼязок роботи з планом.",
    "branch {branch} is not named after a wave. Work happens on a branch named "
    "after the wave, planning on plan/<wave>.":
        "гілка {branch} не названа за хвилею. Робота йде на гілці, названій за "
        "хвилею, планування — на plan/<хвиля>.",
    "every transform of wave {wave} is closed by a commit. Next: keel check, "
    "then the PR.":
        "кожен трансформ хвилі {wave} закрито комітом. Далі: keel check, потім PR.",
    "keel: the hook payload carried no file path, so scope was not checked. "
    "Files the wave declares: {declared}":
        "keel: у виклику хука не було шляху до файла, тож обсяг не перевірено. "
        "Файли, які оголошує хвиля: {declared}",
    "keel: {target} is outside the repository, so the wave's scope does not "
    "apply to it. Judge for yourself whether it should be written to.":
        "keel: {target} лежить поза репозиторієм, тож обсяг хвилі на нього не "
        "поширюється. Чи варто туди писати — вирішуй сам.",
    "wave {wave} is not on {main} yet: the plan is not approved and there is no work.":
        "хвилі {wave} ще немає на {main}: план не затверджено, і роботи немає.",
    "Keel: wave {wave} is not on {main} yet: the plan is not approved and there "
    "is no work.":
        "Keel: хвилі {wave} ще немає на {main}: план не затверджено, і роботи "
        "немає.",
    "this is a plan branch: the wave is written here, not code. keel gaps says "
    "what is missing.":
        "це гілка плану: тут пишеться хвиля, а не код. Чого бракує — каже keel gaps.",
    "Keel files with uncommitted changes: {count}. Commit them separately from "
    "the work:\n  git add {paths}\n  git commit -m \"Keel in the project\"":
        "Файли Keel із незакоміченими змінами: {count}. Закоміть їх окремо від "
        "роботи:\n  git add {paths}\n  git commit -m \"Keel у проєкті\"",
    "{name} is not declared in wave {wave}. Declared: {declared}. If this file "
    "is the one you need, add it to the transform in {file}: drift is not "
    "forbidden, it has to stay a line in the diff.":
        "{name} не оголошено в хвилі {wave}. Оголошено: {declared}. Якщо потрібен "
        "саме цей файл, додай його до трансформа в {file}: відхилення не "
        "заборонене, воно має лишитись рядком у diff.",
    "⚠ the wave holds {held}, the contract is now {now} — keel rev first":
        "⚠ хвиля тримає {held}, контракт тепер {now} — спершу keel rev",
    "Take the {name} skill.": "Візьми скіл {name}.",
    "Keel is in manual mode: ask the person to type /{name}.":
        "Keel у ручному режимі: попроси людину набрати /{name}.",
    "no agent hooks: mode is {mode}": "агентських хуків немає: режим {mode}",
    "Keel: plan branch {branch}, {where}. The plan is written here, not code.\n"
    "{take} What is missing is what `python3 {tool} gaps` says.":
        "Keel: гілка плану {branch}, {where}. Тут пишеться план, а не код.\n"
        "{take} Чого бракує — каже `python3 {tool} gaps`.",
    "Keel: branch {branch} is not named after a wave, so there is no planned "
    "work here.\nA new wave: first `python3 {tool} new wave <slug>` — it prints "
    "the file name with its number — and only then the branch "
    "`plan/<that same name>`. {take}":
        "Keel: гілка {branch} не названа за хвилею, тож запланованої роботи тут "
        "немає.\nНова хвиля: спершу `python3 {tool} new wave <слаг>` — він друкує "
        "імʼя файла з номером — і аж тоді гілка `plan/<те саме імʼя>`. {take}",
    "Keel: {file} does not parse: {reason}":
        "Keel: {file} не розбирається: {reason}",
    "Keel: every transform of wave {slug} is closed by a commit.\n{take} Then "
    "`python3 {tool} check` and the PR.":
        "Keel: кожен трансформ хвилі {slug} закрито комітом.\n{take} Тоді "
        "`python3 {tool} check` і PR.",
    "Keel: {take} Here is the package for the next move — work from it, nothing "
    "around it needs opening.":
        "Keel: {take} Ось пакет для наступного руху — працюй із нього, довкола "
        "нічого відкривати не треба.",
    "{root} is not a git repository, and Keel reads all of its state from\n"
    "git — transform closure, scope, the approval of a plan.\nFirst:\n  git init":
        "у {root} немає git-репозиторію, а Keel увесь свій стан читає з git —\n"
        "закриття трансформ, межі, схвалення плану.\nСпершу:\n  git init",
    "there are no references in {lang}: {missing}. Run init from the "
    "methodology repository.":
        "довідників мовою {lang} немає: {missing}. init запускають із "
        "репозиторію методики.",
    "committed separately: {count} files": "закомічено окремо: {count} файлів",
    "this is not a git repository — there is nowhere to put the hooks":
        "це не git-репозиторій — хуки нема куди класти",
    "{name}: another tool owns this hook, leaving it alone (--force to overwrite)":
        "{name}: чужий хук, не чіпаю (--force щоб перезаписати)",
    "keel hooks --install puts them in place": "keel hooks --install щоб поставити",
    "both are in place": "стоять обидва",
    "update compares the project against the methodology home, and there are no "
    "sources beside this copy. Run it from the keel repository:\n"
    "  python3 <keel>/keel.py -C {root} update":
        "update звіряє проєкт із домом методики, а поруч із цією копією джерел "
        "немає. Запусти з репозиторію keel:\n"
        "  python3 <keel>/keel.py -C {root} update",
    "translations have fallen behind the source:": "переклади відстали від джерела:",
    "no difference": "різниці немає",
    "updated: {what}": "оновлено: {what}",
    "edited by hand, leaving it alone: {what}": "правлено руками, не чіпаю: {what}",
    "everything is in place": "усе на місці",
    "keel update --diff shows the difference, --force overwrites":
        "keel update --diff покаже різницю, --force перепише",
    "every revision matches": "усі редакції збігаються",
    "drifted apart: {count}. keel rev --write records the new ones, once you "
    "have reread the text you lean on.":
        "розійшлося: {count}. keel rev --write впише нові — після того, як "
        "перечитаєш текст, на який спираєшся.",
    "recorded: {count}": "вписано: {count}",
    "recorded {written} of {count} — the rest were reported and not found where "
    "the rewrite looked":
        "вписано {written} з {count} — решту звітовано, але там, де шукав "
        "перепис, їх немає",
    "no such directory: {path}": "теки немає: {path}",
    "nothing": "нічого",
    "test {slug}": "тест {slug}",
    "{root} has no keel/ directory — Keel is not installed here":
        "у {root} немає теки keel/ — тут Keel не поставлений",
}


def t(text, **fields):
    template = UK.get(text, text) if OUTPUT_LANG == "uk" else text
    return template.format(**fields) if fields else template

REV_LEN = 6          # how many hex digits keel rev writes
REV_MIN = 4          # a shorter revision in a reference is not accepted
VERIFY_TIMEOUT = 30
# The project's own gate is the slowest thing Keel runs: a build, a linter and
# a suite, not one probe. Bounded all the same — a hung command holds pre-push
# and says nothing while it does.
CI_TIMEOUT = 900
# The command is free text, so the two words that are not commands have to be
# spelled: nothing decided yet, and deliberately no gate.
CI_UNDECIDED = ""
CI_REFUSED = "none"
# A test run is longer work than a promise being probed, but not unbounded: the
# same reasoning applies. Everything here executes the project's own code, and a
# suite that waits on input or on a socket would hold pre-push and CI for as
# long as they are allowed to run.
TEST_TIMEOUT = 600
PROBE_TIMEOUT = 120  # a contract's proof is a probe, not a build


# ─────────────────────────────────────────────────────────────────────────────
# YAML: a narrow subset
#
# Indented block maps, block lists, flow [a, b] and {k: v}, quotes, comments.
# No anchors, no multi-line scalars, no types. Exactly as much as a Keel
# document header needs — and not one pip install.
# ─────────────────────────────────────────────────────────────────────────────

class YamlError(Exception):
    def __init__(self, line, message):
        super().__init__(t("line {line}: {message}", line=line, message=message))
        self.line = line
        self.message = message


def _strip_comment(text):
    out = []
    quote = None
    escaped = False
    for index, ch in enumerate(text):
        if quote:
            out.append(ch)
            if escaped:
                escaped = False
            elif quote == '"' and ch == "\\":
                # A double-quoted string escapes with a backslash, so `\"` is a
                # quote inside the value, not its end. Missing this closed the
                # string early and stripped the rest as a comment — breaking the
                # round trip of a value yaml_string itself wrote.
                escaped = True
            elif ch == quote:
                if quote == "'" and text[index + 1: index + 2] == "'":
                    # A single-quoted string escapes a quote by doubling it, so
                    # the first of a '' pair is not the end either. _scalar has
                    # always unescaped ''; the scanners have to see it the same
                    # way, or `'it''s # note'` loses its tail to the comment.
                    escaped = True
                else:
                    quote = None
        elif ch in "\"'" and (not out or out[-1] in " \t:[{,"):
            # A quote opens a scalar at the start of a value, not mid-word:
            # otherwise an apostrophe inside a word swallows the comment after it.
            quote = ch
            out.append(ch)
        elif ch == "#" and (not out or out[-1] in " \t"):
            break
        else:
            out.append(ch)
    return "".join(out).rstrip()


def _scalar(text, line):
    text = text.strip()
    if len(text) >= 2 and text[0] == text[-1] and text[0] in "\"'":
        body = text[1:-1]
        # `"a" and "b"` has matching outer quotes and is still not one scalar:
        # an unescaped inner quote means the writer meant something this reader
        # does not read. Swallowing it moved the quotes into the value.
        if text[0] == '"' and re.search(r'(?<!\\)"', body.replace('\\\\', "")):
            raise YamlError(line, t("a quote in the middle of a value: {value}",
                                    value=repr(text)))
        if text[0] == "'" and "'" in body.replace("''", ""):
            raise YamlError(line, t("a quote in the middle of a value: {value}",
                                    value=repr(text)))
        if text[0] == '"':
            # One pass, not three chained replaces: chaining turns an escaped
            # backslash followed by n into a real newline, so a value written by
            # yaml_string would not survive the round trip.
            return unescape(body, line)
        return body.replace("''", "'")
    if text.startswith(("\"", "'")):
        raise YamlError(line, t("unclosed quote"))
    return text


ESCAPES = {'"': '"', "\\": "\\", "n": "\n", "t": "\t", "r": "\r"}


def unescape(body, line):
    out, index = [], 0
    while index < len(body):
        char = body[index]
        if char != "\\":
            out.append(char)
            index += 1
            continue
        if index + 1 >= len(body):
            raise YamlError(line, t("a backslash at the end of a value"))
        following = body[index + 1]
        if following not in ESCAPES:
            raise YamlError(line, t("unknown escape: {escape}",
                                    escape=repr("\\" + following)))
        out.append(ESCAPES[following])
        index += 2
    return "".join(out)


def _split_flow(text, line):
    """Split the inside of [..] or {..} on top-level commas."""
    parts, depth, quote, cur, escaped = [], 0, None, [], False
    for index, ch in enumerate(text):
        if quote:
            cur.append(ch)
            if escaped:
                escaped = False
            elif quote == '"' and ch == "\\":
                escaped = True      # `\"` is a quote in the value, not its end
            elif ch == quote:
                if quote == "'" and text[index + 1: index + 2] == "'":
                    escaped = True  # the first of a '' pair — same rule
                else:
                    quote = None
            continue
        if ch in "\"'" and (not cur or cur[-1] in " \t:[{,"):
            quote = ch
        elif ch in "[{":
            depth += 1
        elif ch in "]}":
            depth -= 1
            if depth < 0:
                raise YamlError(line, t("stray bracket"))
        elif ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
            continue
        cur.append(ch)
    if quote:
        raise YamlError(line, t("unclosed quote"))
    if depth:
        raise YamlError(line, t("unclosed bracket"))
    tail = "".join(cur).strip()
    if tail:
        parts.append(tail)
    return [p.strip() for p in parts if p.strip()]


MAP_ITEM = re.compile(r"^[^\s\"'\[{][^:]*:(\s|$)")


def _list_item(text, line):
    """One element of a list. A map here is valid YAML we do not read."""
    stripped = text.strip()
    # Both spellings: `- a: b` and `- {a: b}`. The braced form parsed into a
    # dict, and Ref(dict) later surfaced as a bogus "wave that does not exist"
    # far from the line that caused it.
    if MAP_ITEM.match(stripped) or stripped.startswith("{"):
        raise YamlError(line, t("a map inside a list is not supported: {item}", item=repr(stripped)))
    # A list inside a list — flow `[a, b]` or block `- - a` — is the same story:
    # it coerced through str() into a Python repr that surfaced far away as a
    # file literally named "['g.py']", or silently became the scalar "- a".
    if stripped.startswith(("[", "- ")):
        raise YamlError(line, t("a list inside a list is not supported: {item}", item=repr(stripped)))
    return _flow(text, line)


def _flow(text, line):
    text = text.strip()
    if text.startswith("["):
        if not text.endswith("]"):
            raise YamlError(line, t("list is not closed by a bracket"))
        return [_list_item(p, line) for p in _split_flow(text[1:-1], line)]
    if text.startswith("{"):
        if not text.endswith("}"):
            raise YamlError(line, t("map is not closed by a bracket"))
        out = {}
        for part in _split_flow(text[1:-1], line):
            if ":" not in part:
                raise YamlError(line, t("no colon in the map entry: {entry}", entry=repr(part)))
            key, _, value = part.partition(":")
            key = _scalar(key, line)
            if key in out:
                # Before the empty-value branch below, not after: skipping the
                # guard for `{s1: , s1: }` dropped one entry in silence, and a
                # silently dropped entry reads as never declared.
                raise YamlError(line, t("duplicate key {key}", key=repr(key)))
            # `{s1: }` — the block spelling of the same thing yields None, and
            # the shape checks exempt None only, so one spelling killed the wave
            # while the other passed.
            out[key] = _flow(value, line) if value.strip() else None
        return out
    return _scalar(text, line)


def parse_yaml(text):
    """Parse a document header. Returns a dict."""
    lines = []
    for number, raw in enumerate(text.splitlines(), 1):
        if "\t" in raw[: len(raw) - len(raw.lstrip())]:
            raise YamlError(number, t("indented with a tab"))
        body = _strip_comment(raw)
        if not body.strip():
            continue
        lines.append((number, len(body) - len(body.lstrip(" ")), body.strip()))

    value, index = _parse_block(lines, 0, 0)
    if index != len(lines):
        raise YamlError(lines[index][0], t("unexpected indent"))
    if not isinstance(value, dict):
        # Returning {} here would turn a malformed header into a wave with no
        # transforms — which reads as "nothing declared" and switches the write
        # hook off without a word.
        raise YamlError(1, t("a header has to be a set of keys, not a list"))
    return value


def _parse_block(lines, index, indent):
    if index >= len(lines):
        return {}, index
    number, _, text = lines[index]
    if text.startswith(("[", "{")):
        # A flow value wrapped onto its own line under a key. Reading it as a
        # block map ate `{proves: c@1}` into the key '{proves' — no error, and
        # a scenario that silently proved nothing.
        return _flow(text, number), index + 1
    if text.startswith("- "):
        return _parse_list(lines, index, indent)
    return _parse_map(lines, index, indent)


def _parse_list(lines, index, indent):
    items = []
    while index < len(lines):
        number, own, text = lines[index]
        if own < indent:
            break
        if own > indent:
            raise YamlError(number, t("unexpected indent"))
        if not text.startswith("- "):
            # A sibling key after a same-indent list — `depends_on:` written the
            # ordinary way. Raising here made the natural form of a header die
            # while the same list at document end parsed.
            break
        items.append(_list_item(text[2:], number))
        index += 1
    return items, index


def _parse_map(lines, index, indent):
    out = {}
    while index < len(lines):
        number, own, text = lines[index]
        if own < indent:
            break
        if own > indent:
            raise YamlError(number, t("unexpected indent"))
        if text.startswith("- "):
            raise YamlError(number, t("a list where a key was expected"))
        match = re.match(r"^([^:]+):(\s|$)", text)
        if not match:
            raise YamlError(number, t("no key before the colon: {line}", line=repr(text)))
        key, rest = match.group(1), text[match.end(1) + 1:]
        if re.match(r"^\s*[^\s\"'\[{][^:]*:(\s|$)", rest):
            raise YamlError(number, t("two colons on one line: {line}", line=repr(text)))
        key = _scalar(key, number)
        if not key:
            raise YamlError(number, t("empty key"))
        if key in out:
            raise YamlError(number, t("duplicate key {key}", key=repr(key)))
        index += 1
        rest = rest.strip()
        if rest:
            if rest.startswith("- "):
                # `files: - a.py` — the classic mis-indent. Coercing it to the
                # scalar "- a.py" planted a phantom file and sent every later
                # message away from the typo. Lists guard this family already.
                raise YamlError(number, t(
                    "a list on the key's own line is not supported: {item}",
                    item=repr(rest)))
            out[key] = _flow(rest, number)
            continue
        if index < len(lines) and lines[index][1] > indent:
            nested, index = _parse_block(lines, index, lines[index][1])
            out[key] = nested
        elif index < len(lines) and lines[index][2].startswith("- ") and lines[index][1] == indent:
            out[key], index = _parse_list(lines, index, indent)
        else:
            out[key] = None
    return out, index


# ─────────────────────────────────────────────────────────────────────────────
# Documents: header, body, sections, revisions
# ─────────────────────────────────────────────────────────────────────────────

# `[ \t]+`, not `\s+`: \s matches a newline, so a bare `##` line captured
# the next prose line as its title — truncating the section above it, whose
# revision then covered less text than the document holds.
SECTION_RE = re.compile(r"^##[ \t]+(.+?)[ \t]*$", re.M)
LINK_RE = re.compile(r"\]\(([^)\s]+\.md)\)")


def revision(text):
    """Short hash of a text. Only repeated spaces and newlines are collapsed."""
    return full_revision(text)[:REV_LEN]


def full_revision(text):
    return hashlib.sha256(re.sub(r"\s+", " ", text).strip().encode("utf-8")).hexdigest()


def rev_matches(recorded, text):
    if not recorded or len(recorded) < REV_MIN:
        return False
    return full_revision(text).startswith(recorded.lower())


class Ref:
    """A reference of the form `slug` or `slug@a3f1c0`."""

    __slots__ = ("slug", "rev", "raw")

    def __init__(self, raw):
        self.raw = raw = str(raw).strip()
        self.slug, _, self.rev = raw.partition("@")
        self.slug = self.slug.strip()
        self.rev = self.rev.strip() or None

    def __repr__(self):
        return f"Ref({self.raw!r})"


def shape_names():
    """Spelled out as literals so the catalogue guard can see them."""
    return {list: t("a list"),
            dict: t("a set of named entries"),
            str: t("a name")}


class Doc:
    def __init__(self, path, root):
        self.path = path
        self.rel = os.path.relpath(path, root).replace(os.sep, "/")
        self.slug = os.path.splitext(os.path.basename(path))[0]
        self.error = None
        self.front = {}
        self.body = ""
        self.sections = {}          # heading -> the text under it
        self.section_lines = {}     # heading -> line number
        self.repeated = []          # headings written more than once
        try:
            # utf-8-sig: reads plain UTF-8 unchanged and strips a BOM when one
            # is there. A file saved by a Windows editor used to report "no
            # header between --- markers" with the header right on line one.
            with open(path, encoding="utf-8-sig") as handle:
                text = handle.read()
        except OSError as exc:
            # A broken symlink, a permission change, a file removed while an
            # agent was writing beside it. Every other bad input here becomes a
            # named problem; this one used to be a traceback out of startup.
            self.text = ""
            self.error = t("cannot be read: {reason}", reason=exc.strerror or exc)
            return
        self.text = text
        front_text, self.body, self.body_offset = split_front_matter(text)
        if front_text is None:
            self.error = t("no header between --- markers")
            return
        try:
            self.front = parse_yaml(front_text) or {}
        except YamlError as exc:
            self.error = t("header does not parse: {reason}", reason=exc)
            return
        self.error = self._wrong_shape()
        if self.error:
            return
        self._split_sections()

    # A field of the wrong shape used to fall back to an empty default, and an
    # empty default reads as "nothing declared" — which is what disarms the
    # write hook and leaves every check green over a wave that guards nothing.
    # The reader was hardened against this one level up; this is the same rule
    # one level down.
    SHAPES = {}

    def _wrong_shape(self):
        for field, kind in self.SHAPES.items():
            value = self.front.get(field)
            if value is None or isinstance(value, kind):
                continue
            return t("{field} has to be {kind}, and this is {actual}",
                     field=field, kind=shape_names()[kind],
                     actual=type(value).__name__)
        return None

    def _split_sections(self):
        # Only real headings: a `## ` line shown inside a ``` example is content,
        # not a section. Counting it would truncate the section it sits in and
        # invent a phantom one — and the methodology's own docs quote example
        # headings.
        marks = [mark for mark in SECTION_RE.finditer(self.body)
                 if not self._in_fence(mark.start())]
        seen = {}
        for order, mark in enumerate(marks):
            end = marks[order + 1].start() if order + 1 < len(marks) else len(self.body)
            title = mark.group(1).strip()
            # The reader resolves a heading by (kind lower-cased, slug as written),
            # so the duplicate check has to collapse the same way: otherwise
            # `## scenario: s` and `## Scenario:  s` look distinct here while the
            # reader keeps only the second — the revision comes from text nobody
            # approved, and check 7 stays quiet because the name sets still match.
            key = self._section_key(title)
            if key in seen:
                self.repeated.append(title)
            seen[key] = True
            self.sections[title] = self.body[mark.end():end].strip()
            self.section_lines[title] = self.body[: mark.start()].count("\n") + self.body_offset

    def _in_fence(self, pos):
        return sum(line.lstrip().startswith(("```", "~~~"))
                   for line in self.body[:pos].split("\n")) % 2 == 1

    @staticmethod
    def _section_key(title):
        head, sep, slug = title.partition(":")
        return head.strip().lower() + ":" + slug.strip() if sep else title.strip().lower()

    def named_sections(self, kind):
        """Sections of the form `## scenario: slug` -> {slug: text}."""
        out = {}
        for title, text in self.sections.items():
            head, _, slug = title.partition(":")
            if head.strip().lower() == kind and slug.strip():
                out[slug.strip()] = text
        return out

    def line_of(self, needle):
        """The first line holding the needle, or 1.

        Repeated contract references need their own line each, and that lives in
        ref_line: it knows where a reference may legally appear, which plain
        substring search cannot.
        """
        for number, line in enumerate(self.text.splitlines(), 1):
            if needle in line:
                return number
        return 1


def split_front_matter(text):
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return None, text, 1
    for index in range(1, len(lines)):
        if lines[index].strip() == "---":
            front = "\n".join(lines[1:index])
            body = "\n".join(lines[index + 1:])
            return front, body, index + 2
    return None, text, 1


class Contract(Doc):
    # `verify` is left out on purpose: check 6 already names a wrong shape
    # there, with the value in the message, and that is a promise about a
    # command rather than about the document's own structure.
    SHAPES = {"module": str, "exports": list}

    @property
    def module(self):
        value = self.front.get("module")
        return value.strip() if isinstance(value, str) else None

    @property
    def exports(self):
        value = self.front.get("exports")
        if isinstance(value, list):
            return [str(item).strip() for item in value if str(item).strip()]
        return []

    @property
    def verify(self):
        """A command whose success is the proof, for a promise that is not a module."""
        value = self.front.get("verify")
        return value.strip() if isinstance(value, str) and value.strip() else None

    @property
    def revision(self):
        """A contract's revision covers the whole file, header along with body."""
        return revision(self.text)

    def rev_ok(self, recorded):
        return rev_matches(recorded, self.text)


class Wave(Doc):
    SHAPES = {"depends_on": list, "scenarios": dict, "transforms": dict}

    def _wrong_shape(self):
        problem = super()._wrong_shape()
        if problem:
            return problem
        # One level down: the top-level table proves scenarios and transforms
        # are maps, but each entry has fields of its own, and a wrong shape
        # there used to fall back to an empty list — which reads as "declared
        # no files" over a transform whose files are right there, mis-shaped.
        for kind, entries in (("scenario", self.scenarios),
                              ("transform", self.transforms)):
            for slug, spec in entries.items():
                if spec is not None and not isinstance(spec, dict):
                    return t("{kind} {slug} has to be a set of named fields, "
                             "and this is {actual}", kind=kind, slug=slug,
                             actual=type(spec).__name__)
        for slug, spec in self.transforms.items():
            for field in ("files", "implements", "contracts"):
                value = (spec or {}).get(field)
                if value is not None and not isinstance(value, (list, str)):
                    return t("{field} of transform {slug} has to be a list, "
                             "and this is {actual}", field=field, slug=slug,
                             actual=type(value).__name__)
        for slug, spec in self.scenarios.items():
            value = (spec or {}).get("proves")
            if value is not None and not isinstance(value, (list, str)):
                return t("proves of scenario {slug} has to be a list, and this "
                         "is {actual}", slug=slug, actual=type(value).__name__)
        return None

    @property
    def depends_on(self):
        value = self.front.get("depends_on")
        return [Ref(item) for item in value] if isinstance(value, list) else []

    @property
    def scenarios(self):
        value = self.front.get("scenarios")
        return value if isinstance(value, dict) else {}

    @property
    def transforms(self):
        value = self.front.get("transforms")
        return value if isinstance(value, dict) else {}

    def scenario_body(self, slug):
        return self.named_sections("scenario").get(slug)

    def transform_body(self, slug):
        return self.named_sections("transform").get(slug)

    def scenario_revision(self, slug):
        body = self.scenario_body(slug)
        return revision(body) if body is not None else None

    def proves(self, slug):
        spec = self.scenarios.get(slug)
        if isinstance(spec, dict) and spec.get("proves"):
            value = spec["proves"]
            values = value if isinstance(value, list) else [value]
            return [Ref(item) for item in values]
        return []

    def declared_files(self):
        """Every file the wave's transforms declare — the fact check 4 and the
        write hook must agree on, so it is computed in exactly one place."""
        declared = set()
        for slug in self.transforms:
            declared.update(self.transform_files(slug))
        return declared

    def transform_files(self, slug):
        spec = self.transforms.get(slug)
        if not isinstance(spec, dict):
            return []
        value = spec.get("files")
        if isinstance(value, list):
            return [str(item).strip() for item in value if str(item).strip()]
        if isinstance(value, str) and value.strip():
            return [value.strip()]
        return []

    def transform_contracts(self, slug):
        spec = self.transforms.get(slug)
        if not isinstance(spec, dict):
            return []
        value = spec.get("contracts")
        if isinstance(value, list):
            return [Ref(item) for item in value]
        if isinstance(value, str) and value.strip():
            return [Ref(value)]
        return []

    def transform_implements(self, slug):
        spec = self.transforms.get(slug)
        if not isinstance(spec, dict):
            return []
        value = spec.get("implements")
        if isinstance(value, list):
            return [str(item).strip() for item in value if str(item).strip()]
        if isinstance(value, str) and value.strip():
            return [value.strip()]
        return []

    @property
    def why(self):
        for title, text in self.sections.items():
            if title.strip().lower() in WHY_HEADINGS:
                return text
        return ""


# ─────────────────────────────────────────────────────────────────────────────
# git
# ─────────────────────────────────────────────────────────────────────────────

class Git:
    def __init__(self, root):
        self.root = root

    def run(self, *args):
        # core.quotePath off: without it diff/log return octal escapes for any
        # non-ASCII path, so a file named файл.txt never matches its declared
        # name and check 4 reports gibberish twice.
        proc = subprocess.run(
            ["git", "-C", self.root, "-c", "core.quotePath=false", *args],
            capture_output=True, text=True,
        )
        return proc.returncode, proc.stdout, proc.stderr

    def out(self, *args, default=""):
        code, stdout, _ = self.run(*args)
        return stdout.strip() if code == 0 else default

    @functools.cached_property
    def available(self):
        return self.run("rev-parse", "--git-dir")[0] == 0

    @functools.cached_property
    def branch(self):
        # symbolic-ref as well: in a repository with no commits yet, HEAD points
        # at a branch that does not exist, rev-parse fails, and an empty answer
        # read as a detached head — the first thing a new project saw was check 4
        # reporting a detachment that was not there.
        name = self.out("rev-parse", "--abbrev-ref", "HEAD")
        return name or self.out("symbolic-ref", "--short", "HEAD")

    @functools.cached_property
    def has_commits(self):
        return self.run("rev-parse", "--verify", "--quiet", "HEAD")[0] == 0

    ORIGIN = "refs/remotes/origin/"

    @functools.cached_property
    def tracks_whole_remote(self):
        """Positive evidence that this clone follows the remote's every branch.

        `git clone --single-branch` narrows the fetch refspec to one branch; a
        full clone keeps the wildcard. Later fetches may add refs, so counting
        them proves nothing — the refspec is what was narrowed, and it stays
        narrowed. Absent evidence the answer is no: trusting an origin/HEAD that
        names the current branch makes that branch its own baseline, and a
        silent green is the one outcome worth erring away from.
        """
        code, spec, _ = self.run("config", "--get-all", "remote.origin.fetch")
        if code != 0:
            return False
        return any("*" in line for line in spec.splitlines())

    @functools.cached_property
    def main_branch(self):
        """The main branch. On CI it is not local — there it is origin/main."""
        head = self.out("symbolic-ref", "--quiet", "refs/remotes/origin/HEAD")
        # Strip the ref prefix, do not take the last path segment: a default
        # branch named release/2024 would otherwise become "2024", a ref that
        # does not exist, and the scope check would find no baseline.
        short = head[len(self.ORIGIN):] if head.startswith(self.ORIGIN) else ""
        # In a single-branch clone origin/HEAD names the branch under test, and
        # believing it makes a branch its own baseline: the diff covers nothing
        # and the scope check reports green having compared nothing. Standing on
        # the default branch of a full clone looks identical from here, so the
        # two have to be told apart by something other than the name — guessing
        # from names cost three attempts. Counting branches fails too: CI
        # routinely fetches the base into a single-branch clone. The refspec is
        # the honest signal, because it is what --single-branch narrows:
        # a wildcard means the clone tracks the whole remote.
        if short and short == self.branch and self.tracks_whole_remote:
            return short
        # The remote-tracking ref first, and this order matters. A local `main`
        # sits wherever it sat when the repository was cloned: nobody checks it
        # out, colleagues keep merging, and using it as the baseline made check
        # 4 report their files as changed by this branch — in the pre-commit
        # hook, naming files the branch never opened. What the branch will be
        # merged into is `origin/main`, so that is what it is compared against.
        if short and short != self.branch:
            for name in (f"origin/{short}", short):
                if self.run("rev-parse", "--verify", "--quiet", name)[0] == 0:
                    return name
            return f"origin/{short}"
        for name in ("origin/main", "origin/master", "main", "master"):
            if self.run("rev-parse", "--verify", "--quiet", name)[0] == 0:
                return name
        return "main"

    @functools.cached_property
    def main_short(self):
        # The local name of the main branch: origin/main → main, but
        # release/2024 stays release/2024 (only the remote prefix comes off).
        name = self.main_branch
        return name[len("origin/"):] if name.startswith("origin/") else name

    def merge_base(self, other):
        return self.out("merge-base", other, "HEAD")

    def changed_files(self, base):
        """All the branch changed: commits since the base plus what is not committed."""
        files = set()
        if base:
            # --no-renames: a committed rename otherwise collapses to its
            # destination while the uncommitted half of this set reports both
            # names — the verdict of check 4 would flip at commit time.
            code, stdout, _ = self.run("diff", "--name-only", "--no-renames",
                                       base, "HEAD")
            if code == 0:
                files.update(name for name in stdout.splitlines() if name)
        code, stdout, _ = self.run("status", "--porcelain", "-z", "--untracked-files=all")
        if code == 0:
            fields = [item for item in stdout.split("\0") if item]
            index = 0
            while index < len(fields):
                entry = fields[index]
                status, name = entry[:2], entry[3:]
                index += 1
                # R/C carries a second path (the origin), and it sits in either
                # column: staged renames put it in X, worktree renames in Y. A
                # one-column test left the origin field to be misread as its own
                # entry, injecting a phantom path into scope.
                if ("R" in status or "C" in status) and index < len(fields):
                    files.add(fields[index])
                    index += 1
                if name:
                    files.add(name)
        return files

    def messages_on(self, ref):
        """Every commit message reachable from a ref, newest first."""
        code, stdout, _ = self.run("log", "--format=%x1e%B", ref)
        if code != 0:
            return []
        return [chunk.strip() for chunk in stdout.split("\x1e") if chunk.strip()]

    def commits_since(self, base):
        """[(sha, message, {files})], oldest first.

        One `git log --name-only`, not one `git show` per commit: the session
        hook and `next` call this, and a spawn per commit made them O(branch).
        """
        if not base:
            return []
        code, stdout, _ = self.run("log", "--format=%x1e%H%x1f%B%x1f",
                                   "--name-only", "--no-renames", f"{base}..HEAD")
        if code != 0:
            return []
        commits = []
        for chunk in stdout.split("\x1e"):
            if not chunk.strip():
                continue
            sha, _, rest = chunk.partition("\x1f")
            message, _, listing = rest.partition("\x1f")
            files = {name for name in listing.splitlines() if name.strip()}
            commits.append((sha.strip(), message.strip(), files))
        commits.reverse()
        return commits

    @functools.cached_property
    def top_level(self):
        """The repository root. Cached like every other repeated lookup here:
        it cannot change during one invocation, and check_scope asks per file —
        one scope check was spawning a git process for each one."""
        return self.out("rev-parse", "--show-toplevel")

    def relative_to_root(self, name, root):
        """A top-level-relative git path, seen from the project root.

        None when it falls outside — a sibling directory in the same repository
        belongs to whoever owns it, not to this wave.
        """
        if not self.top_level:
            return name
        absolute = os.path.join(self.top_level, name)
        relative = os.path.relpath(os.path.realpath(absolute),
                                   os.path.realpath(root))
        relative = relative.replace(os.sep, "/")
        return None if relative == ".." or relative.startswith("../") else relative

    def in_tree(self, path):
        """A keel-root path as `<ref>:<path>` needs it: from the repository top.

        Pathspecs honour `git -C`; `<ref>:<path>` does not — it always resolves
        from the repository root. The two coincide only when the keel root is
        the repository root, and a keel root nested in a bigger repository is a
        layout find_root supports on purpose. Without this, every lookup of the
        form `main:keel/waves/x.md` answered false there, and `next` refused all
        work with "the plan is not approved" while the plan sat on main.
        """
        if not self.top_level:
            return path
        absolute = os.path.realpath(os.path.join(self.root, path))
        relative = os.path.relpath(absolute, os.path.realpath(self.top_level))
        return relative.replace(os.sep, "/")

    def file_in_branch(self, branch, path):
        return self.run("cat-file", "-e", f"{branch}:{self.in_tree(path)}")[0] == 0


# ─────────────────────────────────────────────────────────────────────────────
# Language adapters
#
# Two of the checks depend on the language: what runs the tests and where
# a module's exports come from. The adapter is chosen by a marker in the root.
# ─────────────────────────────────────────────────────────────────────────────

# Printed by the test runner for every test that did not actually execute.
SKIP_MARK = "keel-skipped|"
EXPORT_MARK = "keel-exports|"
SPEC_MARK = "keel-spec|"
# A module reference is letters, digits, dots and underscores — nothing that can
# escape a string literal or open an interpolation in a generated probe script.
MODULE_NAME = re.compile(r"[A-Za-z0-9_.]+\Z")   # \Z, not $: $ allows a trailing newline


class Probe:
    """What a finished subprocess looks like to parse_export_output."""

    def __init__(self, returncode, stdout, stderr):
        self.returncode, self.stdout, self.stderr = returncode, stdout, stderr


def run_probe(command, root, env=None):
    """Ask the project what it declares — under a bound, and without a stdin.

    Loading a module runs whatever that module runs at load: a connection, a
    prompt, a retry loop. Check 6 is what pre-push and CI run, so an unbounded
    wait here is an unbounded wait on every push.
    """
    try:
        return subprocess.run(command, cwd=root, capture_output=True, text=True,
                              stdin=subprocess.DEVNULL, timeout=PROBE_TIMEOUT,
                              **({"env": env} if env else {}))
    except subprocess.TimeoutExpired:
        return Probe(1, "", t("{command} did not answer within {seconds}s",
                              command=command[0], seconds=PROBE_TIMEOUT))
    except OSError as exc:
        # FileNotFoundError and PermissionError alike: a broken shim on PATH
        # must come back as a failed probe, not a traceback.
        return Probe(1, "", t("{command} could not run: {reason}",
                              command=command[0], reason=exc.strerror or exc))


class Adapter:
    name = "?"
    marker = ()
    test_dirs = ()
    test_suffix = ()
    tag_re = None

    @classmethod
    def detect(cls, root):
        return any(os.path.exists(os.path.join(root, item)) for item in cls.marker)

    def test_command(self, root):
        raise NotImplementedError

    def test_label(self, root):
        """The command as a report names it — argv joined, unless it is a
        generated script that would make the message unreadable."""
        return " ".join(self.test_command(root))

    def test_roots(self, root):
        """Where this language keeps its tests. Elixir adds apps/*/test."""
        return [os.path.join(root, directory) for directory in self.test_dirs]

    def is_test_file(self, name):
        return name.endswith(tuple(self.test_suffix))

    def test_files(self, root):
        """One walk, parameterised: three copies of it drifted apart twice."""
        found = []
        for base in self.test_roots(root):
            for current, _, names in os.walk(base):
                for name in sorted(names):
                    if self.is_test_file(name):
                        found.append(os.path.join(current, name))
        return sorted(found)

    def tags(self, root):
        """{scenario -> [(file, line, revision)]}, slug normalised."""
        out = {}
        for path in self.test_files(root):
            text = read_text(path)
            if not text:
                continue
            for match in self.tag_re.finditer(text):
                slug = normalise_slug(match.group(1))
                line = text[: match.start()].count("\n") + 1
                out.setdefault(slug, []).append(
                    (os.path.relpath(path, root).replace(os.sep, "/"), line, match.group(2))
                )
        return out

    supports_specs = False    # can this language be asked what a function promises

    def exports(self, root, modules):
        raise NotImplementedError

    def ci_steps(self, root):
        """Workflow lines that install the language. Without them CI is mute."""
        return []

    # The command that stands for "this project is in order" — a build, a
    # linter, its own suite. Only where the language has a convention for it:
    # proposing one where there is none would be inventing a fact, and the
    # operator would inherit a command that was never true.
    ci_command = ""

    def not_run(self, output):
        """(names, count) — tests the run did not execute.

        Names where the runner can say which, a bare count where it can only
        say how many. A skipped test leaves the suite successful, so without
        this check 5 says "every scenario has a green test" over one that never
        ran; and a language whose runner will not name them still must not
        report the silence as proof.
        """
        return [], 0

    def ci_guard(self):
        """The condition under which this language's waves make sense at all.

        The settings may name a language before its marker file exists — an
        adapter written into keel.json ahead of the work, or `init --adapter`
        in a repository that has no code yet. Then the install wave runs on a
        branch with no marker, which is every plan branch by design, and CI
        goes red for a reason that has nothing to do with what is on the
        branch. Found live: the agent had to work out on its own that the
        adapter must be named in the same commit that creates mix.exs.
        """
        patterns = ", ".join(f"'{name}'" for name in self.marker)
        return f"        if: hashFiles({patterns}) != ''"


def normalise_slug(text):
    return re.sub(r"[^a-z0-9]+", "-", str(text).strip().lower()).strip("-")


class ElixirAdapter(Adapter):
    name = "elixir"
    marker = ("mix.exs",)
    # `mix ci` is an alias projects define themselves; mix ships no such task.
    # Proposed, not assumed — an alias that does not exist says so on the first
    # run, which is the point of proposing it at all.
    ci_command = "mix ci"
    test_dirs = ("test",)
    test_suffix = ("_test.exs",)

    def test_roots(self, root):
        # An umbrella keeps its tests in apps/*/test — mix test runs them, and a
        # collector that only walked test/ called every tagged, passing test
        # missing: a permanent false red on check 5.
        roots = super().test_roots(root)
        apps = os.path.join(root, "apps")
        if os.path.isdir(apps):
            roots += [os.path.join(apps, app, "test")
                      for app in sorted(os.listdir(apps))]
        return roots
    # rev is captured whatever it looks like, not only hex: rubbish in a
    # revision should turn a check red rather than pass unnoticed.
    tag_re = re.compile(
        r"@tag\s+proves:\s*:([A-Za-z0-9_?!]+)"
        r"(?:\s*,\s*rev:\s*[\"']([^\"']*)[\"'])?"
    )

    @staticmethod
    def tag_example(slug, rev):
        return f'@tag proves: :{slug.replace("-", "_")}, rev: "{rev}"' 

    def test_command(self, root):
        return ["mix", "test"]

    # `mix test` prints "5 tests, 0 failures, 1 excluded, 2 skipped" and names
    # none of them. A count is all this runner will say, and a count is still
    # better than reporting the silence as proof — which is what happened while
    # only the Python side reported anything, on the very language this method
    # is developed against.
    not_run_re = re.compile(r"(\d+)\s+(?:excluded|skipped|invalid)")

    def not_run(self, output):
        return [], sum(int(number) for number in self.not_run_re.findall(output))

    def ci_steps(self, root):
        elixir, otp = self.versions()
        return [
            "      - uses: erlef/setup-beam@v1",
            self.ci_guard(),
            "        with:",
            f"          elixir-version: '{elixir}'",
            f"          otp-version: '{otp}'",
            "      - run: mix deps.get",
            self.ci_guard(),
        ]

    @staticmethod
    def versions():
        """Versions from the machine running init. We ask rather than guess."""
        try:
            # stdin closed like every other spawn: a version manager's shim can
            # prompt (or install) on first call, and a prompt here hangs init.
            proc = subprocess.run(["elixir", "--version"],
                                  capture_output=True, text=True, timeout=60,
                                  stdin=subprocess.DEVNULL)
        except (OSError, subprocess.SubprocessError):
            return "1.18", "27"
        otp = re.search(r"Erlang/OTP (\d+)", proc.stdout)
        elixir = re.search(r"Elixir (\d+\.\d+)", proc.stdout)
        return (elixir.group(1) if elixir else "1.18",
                otp.group(1) if otp else "27")

    supports_specs = True

    def exports(self, root, modules):
        # The name is interpolated into a generated Elixir script, so anything
        # that is not a plain module reference is kept out of it: a quote would
        # break the string and take the whole probe down, and #{...} would run
        # as code from a data file. A rejected name reads as missing, which is
        # what check 6 then reports.
        modules = [name for name in modules if MODULE_NAME.match(name)]
        if not modules:
            return {}
        listing = ", ".join(f'"{name}"' for name in modules)
        script = (
            f"for name <- [{listing}] do\n"
            "  mod = Module.concat([name])\n"
            "  if Code.ensure_loaded?(mod) do\n"
            "    funs = mod.__info__(:functions) ++ mod.__info__(:macros)\n"
            "    body = Enum.map_join(funs, \",\", fn {f, a} -> \"#{f}/#{a}\" end)\n"
            f"    IO.puts(\"{EXPORT_MARK}\" <> name <> \"|\" <> body)\n"
            "    case Code.Typespec.fetch_specs(mod) do\n"
            "      {:ok, specs} ->\n"
            # Every clause, not the first: a function may carry several @spec
            # lines, and comparing a contract against an arbitrary one of them
            # would call a kept promise broken.
            "        for {{f, a}, list} <- specs, spec <- list do\n"
            "          text = Code.Typespec.spec_to_quoted(f, spec) |> Macro.to_string()\n"
            # One line per clause is the protocol this is read back through, so
            # the line breaks Macro.to_string puts in a long spec have to go
            # here. Everything else about spacing is settled in flatten_spec.
            "          flat = String.replace(text, ~r/\\s+/, \" \")\n"
            f"          IO.puts(\"{SPEC_MARK}\" <> name <> \"|#{{f}}/#{{a}}|\" <> flat)\n"
            "        end\n"
            "      _ -> :ok\n"
            "    end\n"
            "  else\n"
            f"    IO.puts(\"{EXPORT_MARK}\" <> name <> \"|__missing__\")\n"
            "  end\n"
            "end\n"
        )
        return parse_export_output(
            run_probe(["mix", "run", "--no-start", "-e", script], root), modules)


class PythonAdapter(Adapter):
    name = "python"
    marker = ("pyproject.toml", "setup.py", "setup.cfg")
    test_dirs = ("tests", "test")
    test_suffix = ("_test.py",)
    tag_re = re.compile(
        r"#\s*proves:\s*([A-Za-z0-9_-]+)"
        r"(?:\s*,\s*rev:\s*[\"']?([^\"'\s,]*)[\"']?)?"
    )

    @staticmethod
    def tag_example(slug, rev):
        # No leading colon: written as dictated, the Elixir form is invisible
        # to this adapter's own recogniser.
        return f'# proves: {slug}, rev: "{rev}"' 

    def is_test_file(self, name):
        # Both spellings the ecosystem uses; the runner loads exactly this list,
        # so collector and runner cannot disagree about what a test is.
        return name.endswith("_test.py") or (
            name.startswith("test_") and name.endswith(".py"))

    def test_command(self, root):
        # The runner is handed the collector's own list, so "the files the tags
        # were read from" and "the files that run" are one fact, not two rules
        # kept in wave by hand — they drifted apart twice when they were two.
        # Loading by path also reaches a nested directory that is not a package,
        # which unittest discover silently skips.
        paths = [os.path.relpath(path, root).replace(os.sep, "/")
                 for path in self.test_files(root)]
        script = (
            "import importlib.util, os, sys, unittest\n"
            "sys.path.insert(0, os.getcwd())\n"
            "suite = unittest.TestSuite()\n"
            "loader = unittest.TestLoader()\n"
            f"for path in {paths!r}:\n"
            "    spec = importlib.util.spec_from_file_location(\n"
            "        os.path.splitext(path)[0].replace(os.sep, '.'), path)\n"
            "    module = importlib.util.module_from_spec(spec)\n"
            # Registered before it runs: mock.patch('tests.x.thing') resolves
            # through the import system, and an unregistered module means patch
            # imports a second copy and patches that one while this copy runs.
            "    sys.modules[spec.name] = module\n"
            "    spec.loader.exec_module(module)\n"
            "    suite.addTests(loader.loadTestsFromModule(module))\n"
            "result = unittest.TextTestRunner().run(suite)\n"
            # A skipped test is not a proof. The suite is successful with it,
            # and check 5 used to print "every scenario has a green test" over a
            # test marked skip whose body was a bare failure. Name them so the
            # check can tell a promise that ran from one that did not.
            "for case, _why in list(result.skipped) + [\n"
            "        (c, None) for c, *_ in result.expectedFailures]:\n"
            f"    print('{SKIP_MARK}' + case.id())\n"
            "sys.exit(0 if result.wasSuccessful() else 1)\n"
        )
        # -B for the same reason the probe sets PYTHONDONTWRITEBYTECODE: the
        # runner imports every test file, and the bytecode it would leave
        # behind is what check 4 blamed the branch for a moment later.
        return [sys.executable, "-B", "-c", script]

    def not_run(self, output):
        return ([line[len(SKIP_MARK):].strip() for line in output.splitlines()
                 if line.startswith(SKIP_MARK)], 0)

    def test_label(self, root):
        count = len(self.test_files(root))
        return f"{os.path.basename(sys.executable)}, {count} test files"

    def ci_steps(self, root):
        return [
            "      - uses: actions/setup-python@v5",
            self.ci_guard(),
            "        with:",
            f"          python-version: '{sys.version_info.major}."
            f"{sys.version_info.minor}'",
        ]

    def exports(self, root, modules):
        if not modules:
            return {}
        script = (
            "import importlib, inspect, sys\n"
            f"for name in {list(modules)!r}:\n"
            "    try:\n"
            "        mod = importlib.import_module(name)\n"
            "    except Exception:\n"
            f"        print('{EXPORT_MARK}' + name + '|__missing__')\n"
            "        continue\n"
            "    names = getattr(mod, '__all__', None)\n"
            "    if names is None:\n"
            "        names = [n for n in dir(mod) if not n.startswith('_')]\n"
            "    out = []\n"
            "    for n in names:\n"
            "        obj = getattr(mod, n, None)\n"
            "        out.append(n)\n"
            "        if callable(obj):\n"
            "            try:\n"
            "                params = inspect.signature(obj).parameters\n"
            "            except (TypeError, ValueError):\n"
            "                continue\n"
            "            count = len([p for p in params.values()\n"
            "                         if p.kind in (p.POSITIONAL_ONLY, p.POSITIONAL_OR_KEYWORD)])\n"
            "            out.append(n + '/' + str(count))\n"
            f"    print('{EXPORT_MARK}' + name + '|' + ','.join(out))\n"
        )
        return parse_export_output(run_probe(
            [sys.executable, "-c", script], root,
            env={**os.environ,
                 # Without this the probe writes __pycache__ into the project,
                 # and check 4 — which runs after it on purpose — then reports
                 # the tool's own litter as files the branch changed and never
                 # declared. It was the cause as well as the thing the ordering
                 # was meant to catch.
                 "PYTHONDONTWRITEBYTECODE": "1",
                 "PYTHONPATH": root + os.pathsep + os.environ.get("PYTHONPATH", "")}),
            modules)


def parse_export_output(proc, modules):
    """{module: {exports}} plus {("specs", module): {"name/arity": [spec, ...]}}.

    A list because one function may declare several specs, and every one of them
    is a shape the module honestly promises.
    """
    out = {}
    for line in proc.stdout.splitlines():
        if line.startswith(SPEC_MARK):
            name, _, rest = line[len(SPEC_MARK):].partition("|")
            signature, _, text = rest.partition("|")
            out.setdefault(("specs", name), {}).setdefault(signature, []).append(
                text.strip())
            continue
        if not line.startswith(EXPORT_MARK):
            continue
        name, _, body = line[len(EXPORT_MARK):].partition("|")
        out[name] = None if body == "__missing__" else {
            item for item in body.split(",") if item
        }
    for name in modules:
        if name not in out:
            out[name] = None
    if proc.returncode != 0 and not any(
            value for key, value in out.items() if isinstance(key, str)):
        out["__error__"] = (proc.stderr or proc.stdout).strip()[:400]
    return out


ADAPTERS = (ElixirAdapter, PythonAdapter)


BY_NAME = {adapter.name: adapter for adapter in ADAPTERS}


def matching_adapters(root):
    return [adapter for adapter in ADAPTERS if adapter.detect(root)]


def detect_adapter(root, chosen="", found=None):
    """The language adapter, chosen by name or by the markers in the root.

    A polyglot repository has more than one marker, and picking the first in a
    hard-coded order is a decision made in silence about which language's tests
    count. Naming it in the settings settles it; leaving it unset is reported
    rather than guessed.
    """
    if chosen in BY_NAME:
        return BY_NAME[chosen]()
    if found is None:
        found = matching_adapters(root)
    return found[0]() if found else None


def adapter_problem(project, check):
    """One line when the root says two languages and nobody said which."""
    if project.settings.get("adapter"):
        return []
    found = project.adapter_candidates
    if len(found) < 2:
        return []
    return [Problem(check, t(
        "the root matches {count} languages ({names}), and {picked} was taken "
        "because it comes first. Say which in keel/keel.json: "
        "\"adapter\": \"{picked}\"",
        count=len(found), names=", ".join(one.name for one in found),
        picked=found[0].name))]


# ─────────────────────────────────────────────────────────────────────────────
# Project
# ─────────────────────────────────────────────────────────────────────────────

class Problem:
    def __init__(self, check, message, where=None, line=None):
        self.check = check
        self.message = message
        self.where = where
        self.line = line

    def render(self):
        place = self.where or ""
        if place and self.line:
            place = f"{place}:{self.line}"
        return f"  {place}  {self.message}".rstrip() if place else f"  {self.message}"

    def as_dict(self):
        return {"check": self.check, "message": self.message,
                "file": self.where, "line": self.line}


class Project:
    def __init__(self, root, settings=None):
        self.root = root
        self.keel = os.path.join(root, "keel")
        self._transform_state = {}
        self.git = Git(root)
        self.settings = read_config(root) if settings is None else settings
        self.adapter_candidates = matching_adapters(root)
        self.adapter = detect_adapter(root, self.settings["adapter"],
                                      self.adapter_candidates)
        self.waves = {}
        self.contracts = {}
        self.broken = []
        # On CI the head is detached and git cannot name the branch — there the
        # name arrives in a flag.
        self.branch_override = None
        self._load()

    @property
    def branch(self):
        return self.branch_override or self.git.branch

    def _load(self):
        for folder, cls in (("waves", Wave), ("contracts", Contract)):
            target = getattr(self, folder)
            base = os.path.join(self.keel, folder)
            if not os.path.isdir(base):
                continue
            for name in sorted(os.listdir(base)):
                if not name.endswith(".md"):
                    continue
                path = os.path.join(base, name)
                doc = cls(path, self.root)
                if doc.error:
                    self.broken.append(doc)
                target[doc.slug] = doc

    @property
    def ready(self):
        return os.path.isdir(self.keel)

    def wave_for_branch(self, branch=None):
        branch = branch or self.branch
        if not branch or branch in ("HEAD", self.git.main_short):
            return None
        name = branch.split("/", 1)[1] if branch.startswith("plan/") else branch
        return self.waves.get(name)

    def is_plan_branch(self, branch=None):
        return (branch or self.branch or "").startswith("plan/")

    def transform_state(self, wave):
        """{transform -> (commit sha or None, {files of that commit})}.

        Cached per wave: check 4, `next` and the session hook each ask within
        one run, and every ask used to resolve the merge base and walk the
        branch's commits again.
        """
        if wave.slug in self._transform_state:
            return self._transform_state[wave.slug]
        state = self._transform_state[wave.slug] = self._transform_state_of(wave)
        return state

    def _transform_state_of(self, wave):
        base = self.git.merge_base(self.git.main_branch)
        found = {}
        for sha, message, files in self.git.commits_since(base):
            for slug in wave.transforms:
                if message_closes(message, slug) and slug not in found:
                    found[slug] = (sha, files)
        return {slug: found.get(slug, (None, set())) for slug in wave.transforms}

    @functools.cached_property
    def arriving_contracts(self):
        """Contracts not yet on the main branch — the ones a wave is bringing.

        One `ls-tree` for the whole directory rather than a `cat-file` per
        contract per wave: the old shape spawned a git process for every
        contract of every wave `gaps` looked at, and reported the same orphan
        once per wave on top of it.
        """
        if not self.git.available or not self.git.has_commits:
            return set()
        # --full-tree: ls-tree takes the current directory as an implicit
        # pathspec, and git runs with -C at the keel root — which is the
        # repository root only when the two coincide. Nested, it listed nothing
        # and every contract read as newly arrived.
        listing = self.git.out(
            "ls-tree", "--full-tree", "--name-only",
            f"{self.git.main_branch}:{self.git.in_tree('keel/contracts')}")
        there = {os.path.splitext(name)[0] for name in listing.splitlines()
                 if name.strip()}
        return {slug for slug in self.contracts if slug not in there}

    @functools.cached_property
    def main_messages(self):
        """Every commit message on the main branch, read once.

        `transform_state` counts closure over `merge_base..HEAD`, which is the
        right range while a wave is being worked. Standing on main that range is
        empty — main *is* the baseline — so closure has to be read from its own
        history instead, or every finished wave would look untouched.
        """
        return self.git.messages_on(self.git.main_branch)

    def unclosed_on_main(self, wave):
        """Transforms of a wave that no commit on the main branch closes."""
        return [slug for slug in wave.transforms
                if not any(message_closes(message, slug)
                           for message in self.main_messages)]

    def ready_waves(self):
        """(ready, blocked, unplanned) — where the work stands, read from git.

        The order of work is derived from `depends_on`, never from the numbers
        in the names, so "what now" is answered by walking the graph rather than
        taking the next file in the directory. Ready means every wave it leans
        on is finished and its own transforms are not.

        A wave with no transforms declares no work, so nothing can close it and
        it would otherwise count as finished — a skeleton nobody filled in would
        report the project complete.
        """
        done, unfinished, unplanned = {}, [], []
        for slug, wave in sorted(self.waves.items()):
            if wave.error:
                continue
            if not wave.transforms:
                unplanned.append(wave)
                done[slug] = False
                continue
            open_now = self.unclosed_on_main(wave)
            done[slug] = not open_now
            if open_now:
                unfinished.append((wave, len(open_now)))
        ready, blocked = [], []
        for wave, open_count in unfinished:
            laid = all(done.get(need, False) for need in wave.depends_on)
            (ready if laid else blocked).append((wave, open_count))
        return ready, blocked, unplanned


def message_closes(message, slug):
    """Whether this commit message closes that transform.

    The first word of the message, not anywhere in it: otherwise a commit for
    `add-more` also closes `add`, and a passing mention in the body closes
    whatever it names.
    """
    return bool(re.match(rf"\s*{re.escape(slug)}(?![\w-])", message))


def find_root(start):
    current = os.path.abspath(start)
    while True:
        if os.path.isdir(os.path.join(current, "keel", "waves")):
            return current
        if os.path.exists(os.path.join(current, ".git")):
            # A file, not a directory, in a linked worktree and in a submodule.
            # Testing for a directory walks straight past the root and lands the
            # tool in whatever repository happens to sit further up.
            return current
        parent = os.path.dirname(current)
        if parent == current:
            return os.path.abspath(start)
        current = parent


# ─────────────────────────────────────────────────────────────────────────────
# The checks — §7
# ─────────────────────────────────────────────────────────────────────────────

CHECK_NAMES = {
    1: "references lead somewhere",
    2: "depends_on without cycles",
    3: "contract revisions match",
    4: "changed files match those declared",
    5: "every scenario has a green test",
    6: "contracts hold",
    7: "names in the header match the headings",
}

FAST_CHECKS = (1, 2, 3, 4, 7)
KEEL_DIR_PREFIX = "keel/"

# Keel's own furniture in a project. A plan branch may carry it: it is not the
# project's code, and refusing it walls off the very first plan commit whenever
# `init` or `update` has just refreshed something.
# Defined here, before the ownership list that uses them, and nowhere else:
# a path restated as a literal can drift from the constant that names it.
CLAUDE_SETTINGS = ".claude/settings.json"
CURSOR_HOOKS = ".cursor/hooks.json"
KEEL_AGENT_SETTINGS = ".keel-agent/settings.json"
CI_FILE = ".github/workflows/keel.yml"

# Whom Keel can equip, and whom it equips unless told otherwise. `keel-agent`
# is opt-in because it is ours and nobody else's yet: writing .keel-agent/ into
# a stranger's repository would leave a folder for a tool they do not have.
AGENT_NAMES = ("claude", "cursor", "keel-agent")
DEFAULT_AGENTS = ("claude", "cursor")
# Those that speak Claude's hook and skill contract rather than their own.
CLAUDE_CONTRACT = ("claude", "keel-agent")

# `.keel-agent/skills/` and not `.keel-agent/`: sessions live in that folder too,
# and they are the project's, written at runtime, never ours to sweep.
KEEL_OWNED_DIRS = (KEEL_DIR_PREFIX, ".claude/skills/", ".cursor/skills/",
                   ".keel-agent/skills/")
KEEL_OWNED_FILES = (CURSOR_HOOKS, CI_FILE, "AGENTS.md")
# Neither settings file is on that list, and that is the point. They belong to
# the project and hold its own keys beside our hook entries, and whether we ever
# wrote in one depends on a setting — so ownership here is answered by evidence:
# our mark is in the file, or the file is not ours.
KEEL_MERGED_FILES = (CLAUDE_SETTINGS, KEEL_AGENT_SETTINGS)
def carries_our_hooks(path):
    """Whether one of our hook entries is actually in this settings file.

    Unreadable, absent, or somebody else's shape all answer no: ownership is a
    claim, and a claim we cannot see the grounds for is one we do not make.
    """
    try:
        data = json.loads(read_text(path))
    except (ValueError, OSError):
        return False
    hooks = data.get("hooks") if isinstance(data, dict) else None
    if not isinstance(hooks, dict):
        return False
    return any(is_ours(entry) for entries in hooks.values()
               if isinstance(entries, list) for entry in entries)


def keel_owns(name, root=None):
    """Ours, by whole path — not by anything that merely starts the same way.

    A bare prefix test claimed AGENTS.mdx and .claude/settings.json.bak, which
    let a plan branch modify somebody's unrelated file and let `init` sweep it
    into its own commit — against the one promise that commit makes.

    Two settings files are answered differently, by evidence rather than by
    list. They are the project's, not ours; we merge entries into them and only
    where a setting asked for it. On the list they would be exempt from the
    scope check in every project, including the ones we never wrote in — an
    exemption nobody asked for and nobody would see. Without a root there is no
    evidence to read, so the answer is no.
    """
    if name.startswith(KEEL_OWNED_DIRS) or name in KEEL_OWNED_FILES:
        return True
    if name in KEEL_MERGED_FILES and root is not None:
        return carries_our_hooks(os.path.join(root, name))
    return False


def check_structure(project):
    return [Problem(0, doc.error, doc.rel) for doc in project.broken]


def shared_transform_slugs(project):
    """A transform slug used by two waves at once.

    The slug in a commit message is the only link between the work and the
    plan — no hash is recorded anywhere, precisely so that nobody writes a
    status by hand. That makes the slug an identifier, and an identifier two
    waves share identifies neither: a commit closing one wave's `setup` closes
    the other's as well, and the tool reports work as done that nobody did.
    Found by review after `next` announced a finished project with a wave whose
    every transform was untouched.
    """
    owners = {}
    for slug, wave in sorted(project.waves.items()):
        if wave.error:
            continue
        for name in wave.transforms:
            owners.setdefault(name, []).append(wave)
    problems = []
    for name, waves in sorted(owners.items()):
        if len(waves) < 2:
            continue
        others = ", ".join(wave.slug for wave in waves[1:])
        problems.append(Problem(
            0, t("transform {name} is declared by {others} as well, and a "
                 "commit naming it would close both. A slug is the only link "
                 "between a commit and its transform, so it belongs to one "
                 "wave.", name=name, others=others), waves[0].rel,
            waves[0].line_of(name)))
    return problems


def check_refs(project):
    problems = []
    for wave in project.waves.values():
        if wave.error:
            continue
        for ref in wave.depends_on:
            if ref.slug not in project.waves:
                problems.append(Problem(
                    1, t("depends_on points at a wave that does not exist: {slug}", slug=ref.slug),
                    wave.rel, wave.line_of(ref.slug)))
        for slug in wave.scenarios:
            for ref in wave.proves(slug):
                if ref.slug not in project.contracts:
                    problems.append(Problem(
                        1, t("scenario {scenario} proves a contract that does not exist: {slug}", scenario=slug, slug=ref.slug),
                        wave.rel, wave.line_of(ref.raw)))
        for slug in wave.transforms:
            for name in wave.transform_implements(slug):
                if name not in wave.scenarios:
                    problems.append(Problem(
                        1, t("transform {transform} implements a scenario that does not exist: {scenario}", transform=slug, scenario=name),
                        wave.rel, wave.line_of(name)))
            for ref in wave.transform_contracts(slug):
                if ref.slug not in project.contracts:
                    problems.append(Problem(
                        1, t("transform {transform} implements a contract that does not exist: {slug}", transform=slug, slug=ref.slug),
                        wave.rel, wave.line_of(ref.raw)))

    for doc in list(project.waves.values()) + list(project.contracts.values()):
        for match in LINK_RE.finditer(doc.body):
            if doc._in_fence(match.start()):
                # The section splitter already treats fenced `## ` as content
                # because the methodology's own docs quote examples; a link in
                # the same example is content too, not a broken reference.
                continue
            target = match.group(1)
            if target.startswith(("http://", "https://")):
                continue
            resolved = os.path.normpath(os.path.join(os.path.dirname(doc.path), target))
            if os.path.exists(resolved):
                continue
            # A link out of keel/ used to be skipped here, by a condition whose
            # name said the opposite of its value. A broken link leads nowhere
            # wherever it points, and this check is called exactly that.
            outside = os.path.relpath(resolved, project.root).startswith("..")
            problems.append(Problem(
                1, t("this link leaves the repository: {target}", target=target)
                   if outside else
                   t("this link leads nowhere: {target}", target=target),
                doc.rel, doc.line_of(target)))
    return problems


def check_cycles(project):
    problems, state = [], {}

    def walk(slug, trail):
        if state.get(slug) == "done":
            return
        if state.get(slug) == "open":
            cycle = " → ".join(trail[trail.index(slug):] + [slug])
            problems.append(Problem(2, t("cycle in depends_on: {cycle}", cycle=cycle),
                                    project.waves[slug].rel))
            return
        state[slug] = "open"
        wave = project.waves.get(slug)
        if wave and not wave.error:
            for ref in wave.depends_on:
                if ref.slug in project.waves:
                    walk(ref.slug, trail + [slug])
        state[slug] = "done"

    for slug in sorted(project.waves):
        walk(slug, [])
    seen = set()
    return [p for p in problems if not (p.message in seen or seen.add(p.message))]


def contract_refs(wave):
    """Everything in a wave that leans on a contract: (who leans, reference).

    Yielded in the file's own order — the parser keeps key order, so iterating
    the header keeps scenarios and transforms in whichever sequence they were
    written. Line numbers for repeated references are assigned by occurrence,
    and a fixed scenarios-first order handed the transform's line to the
    scenario whenever the file was written transforms-first.
    """
    for key in wave.front:
        if key == "scenarios":
            for slug in wave.scenarios:
                for ref in wave.proves(slug):
                    yield t("scenario {slug}", slug=slug), ref
        elif key == "transforms":
            for slug in wave.transforms:
                for ref in wave.transform_contracts(slug):
                    yield t("transform {slug}", slug=slug), ref


def ambiguous_scenarios(project):
    """Scenario slugs declared by more than one wave.

    A test tag names the slug and nothing else, so it cannot say which wave's
    scenario it proves. Sharing the pool between them suppressed one wave's
    "has no test" and reddened the other wave's correct tag — and `rev --write`
    would then restamp the good tag to the other wave's revision, moving the
    red rather than clearing it. Named instead of guessed.
    """
    seen = {}
    for wave in project.waves.values():
        if wave.error:
            continue
        for slug in wave.scenarios:
            # Counted per scenario, not per wave: two scenarios in one wave
            # that normalise alike collide exactly as hard — the tags get
            # swapped, both read as drifted, and `rev --write` cannot fix it
            # because each is already stamped with what the other now hashes to.
            seen.setdefault(normalise_slug(slug), []).append((wave.slug, slug))
    return {flat: sorted({wave for wave, _ in owners})
            for flat, owners in seen.items() if len(owners) > 1}


def scenario_tags(project, tags=None, shared=None):
    """(wave, scenario, body, [(file, line, revision)]) — scenarios and their tags."""
    if tags is None:
        tags = project.adapter.tags(project.root) if project.adapter else {}
    if shared is None:
        shared = ambiguous_scenarios(project)
    for wave in project.waves.values():
        if wave.error:
            continue
        for slug in wave.scenarios:
            body = wave.scenario_body(slug)
            if body is None:
                continue  # check 7 catches this
            if normalise_slug(slug) in shared:
                continue  # the collision is reported; attribution is not guessed
            yield wave, slug, body, tags.get(normalise_slug(slug), [])


def drifted_contract_refs(project):
    """(wave, who, ref, contract) for every reference not matching its contract.

    One definition of "drifted", consumed by check 3 to report and by `rev` to
    restamp: two hand-kept copies of this loop could disagree about what needs
    fixing and what got fixed.
    """
    for wave in project.waves.values():
        if wave.error:
            continue
        for who, ref in contract_refs(wave):
            contract = project.contracts.get(ref.slug)
            if contract is None or contract.error:
                continue
            if ref.rev and contract.rev_ok(ref.rev):
                continue
            yield wave, who, ref, contract


def drifted_tags(project, tags=None, shared=None):
    """(wave, slug, body, path, line, rev) for every tag not matching its scenario."""
    for wave, slug, body, found in scenario_tags(project, tags, shared):
        for path, line, rev in found:
            if rev and rev_matches(rev, body):
                continue
            yield wave, slug, body, path, line, rev


def ref_line(wave, raw, skip=0):
    """The line holding this exact reference, not a longer one it prefixes.

    A bare `auth` substring-matches the stamped `auth@d0c229` line, so the
    report "leans on auth without a revision" pointed at the line that visibly
    carries one. The same whole-reference boundary rewrite_ref uses.
    """
    token = re.compile(rf"(?<![\w@./-]){re.escape(raw)}(?![\w@./-])")
    found, in_block, block_indent = 1, False, 0
    for number, line in enumerate(wave.text.splitlines(), 1):
        # Only where a contract reference can live. `depends_on: [auth]` names a
        # WAVE that may share the contract's slug, and counting it sent the
        # report to the depends_on line. Same reading rewrite_ref uses when it
        # writes: inline values, and items of a block list under one of the two
        # keys.
        stripped = line.lstrip()
        indent = len(line) - len(stripped)
        if in_block and stripped.startswith("- ") and indent > block_indent:
            hits = len(token.findall(line))
        else:
            in_block = bool(re.match(r"(proves|contracts)\s*:\s*$", stripped))
            block_indent = indent if in_block else block_indent
            hits = sum(len(token.findall(value))
                       for value in REF_VALUE.findall(line))
        if not hits:
            continue
        found = number
        if skip < hits:
            return number
        skip -= hits
    return found


def check_revisions(project):
    problems, seen = [], {}
    for wave, who, ref, contract in drifted_contract_refs(project):
        # Identical references share their drift status, so counting only the
        # drifted ones still lands each report on its own line.
        key = (wave.rel, ref.raw)
        line = ref_line(wave, ref.raw, skip=seen.get(key, 0))
        seen[key] = seen.get(key, 0) + 1
        if not ref.rev:
            problems.append(Problem(
                3, t("{who} leans on {slug} without a revision; it is now {now}",
                  who=who, slug=ref.slug, now=contract.revision),
                wave.rel, line))
        else:
            problems.append(Problem(
                3, t("{who} holds {slug}@{held}, and the contract is now {now}",
                  who=who, slug=ref.slug, held=ref.rev, now=contract.revision),
                wave.rel, line))
    return problems


def deleted_documents(project, base):
    """Waves and contracts this branch removed.

    Not covered by anything else: the scope check waves all of `keel/` through
    so that `update` can refresh our own files mid-work, and the drift note
    reads only modifications. So `git rm keel/waves/0002-other.md` left every
    gate green — the quietest way past the guard written to stop exactly that,
    because a document that is gone is in no list the checks walk.
    """
    if not base:
        return []
    listing = project.git.out("diff", "--name-only", "--diff-filter=D", base,
                             "--", "keel/waves", "keel/contracts")
    return [Problem(4, t("{name} was deleted on this branch. A wave or a "
                         "contract outlives the branch that removes it — say so "
                         "in the pull request, or put it back.", name=name))
            for name in sorted(listing.splitlines()) if name.strip()]


def check_scope(project):
    if not project.git.available:
        return [Problem(4, t("this is not a git repository — nothing to check scope against"))]
    if not project.git.has_commits:
        # A repository with nothing in it yet: there is no baseline to compare
        # against and nothing committed to compare. Saying so beats inventing a
        # detachment, which is what the very first `keel check` used to report.
        return []
    branch = project.branch
    if branch == project.git.main_short:
        return []
    if not branch or branch == "HEAD":
        # Passing silently would be a green check where none ever ran.
        return [Problem(4, t("the head is detached and git does not know the branch name — "
                        "pass it with --branch"))]
    main = project.git.main_branch
    base = project.git.merge_base(main)
    if not base:
        # Without a base the diff covers nothing but uncommitted work, so every
        # committed file would pass unseen. Green here would be a lie.
        return [Problem(4, t("cannot tell where this branch left from: {main} is missing or "
                        "the history is truncated. Scope was not compared, which "
                        "is not the same as scope being intact.", main=main))]
    # Git speaks in paths from the repository top level; declarations speak in
    # paths from the keel root. A keel root nested in a bigger repo — a layout
    # find_root explicitly supports — made every file fail scope in both
    # directions at once, and the write hook allowed exactly what check 4 then
    # condemned. Rebase git's answers onto the keel root, and drop what lies
    # outside it: another team's directory is not this wave's business.
    changed = set()
    for name in project.git.changed_files(base):
        inside = project.git.relative_to_root(name, project.root)
        if inside is not None:
            changed.add(inside)

    if project.is_plan_branch(branch):
        stray = sorted(name for name in changed
                       if not keel_owns(name, project.root))
        return (deleted_documents(project, base)
                + [Problem(4, t("a plan branch is touching code: {name}", name=name))
                   for name in stray])

    # Keel's own furniture is out of scope on a work branch too. `update` may
    # refresh a skill in the middle of the work, and telling the person to
    # declare our own generated file in their transform is the same mine the
    # plan branch was cleared of.
    # Before the exemption: a deleted wave or contract is not drift and not
    # furniture. Keel's own files are waved through so that `update` may refresh
    # them mid-work, and removing somebody's approved plan slipped through the
    # same door — quieter than moving it, because what is gone cannot be named
    # by anything that walks the documents that remain.
    problems = deleted_documents(project, base)
    changed = {name for name in changed if not keel_owns(name, project.root)}

    wave = project.wave_for_branch(branch)
    if wave is None:
        return [Problem(4, t("branch {branch} is not named after a wave — there is nothing "
                        "to compare scope against", branch=branch))]
    if wave.error:
        return []

    # The same exemption on both sides: keel_owns is filtered out of changed,
    # and a declared AGENTS.md would otherwise earn "declared but not changed"
    # over a diff that plainly changed it — a message stating a falsehood.
    declared = {name for name in wave.declared_files()
                if not keel_owns(name, project.root)}

    # The two directions do not read the same list, and that is the whole point.
    #
    # "Changed but not declared" asks about every transform of the wave: reach
    # outside the wave's files and it shows, whichever transform you are on.
    #
    # "Declared but not changed" may only ask about transforms already closed by
    # a commit. A wave is worked one transform at a time — that is what `next`
    # hands out — so on the first of five commits the other four have touched
    # nothing yet, by design. Asking about them there made pre-commit refuse
    # every commit until each declared file had been touched at least once —
    # that is, the early ones, exactly when the wave is least finished. An agent
    # that meets a gate it cannot pass honestly learns `--no-verify`, which is
    # worse than no gate at all. Verified live: it took one refusal.
    #
    # Nothing is lost at the end: once every transform is closed the two lists
    # are the same, so the branch is still held to everything it declared.
    closed = project.transform_state(wave)
    promised = set()
    for slug in wave.transforms:
        if closed[slug][0] is not None:
            promised.update(wave.transform_files(slug))
    promised = {name for name in promised if not keel_owns(name, project.root)}

    for name in sorted(changed - declared):
        problems.append(Problem(4, t("changed but not declared: {name}", name=name), wave.rel))
    for name in sorted(promised - changed):
        problems.append(Problem(4, t("declared but not changed: {name}", name=name),
                                wave.rel, wave.line_of(name)))
    return problems


def check_scenarios(project, run_tests=True):
    waves = [wave for wave in project.waves.values() if not wave.error and wave.scenarios]
    if not waves:
        return []
    problems = adapter_problem(project, 5)
    # Before the adapter guard: which slugs are ambiguous is a fact about the
    # wave documents, knowable from keel/waves alone. Behind the guard, a
    # project would set its waves up, add a language marker months later, and
    # only then learn the slugs were never distinguishable.
    shared = ambiguous_scenarios(project)
    for slug, owners in sorted(shared.items()):
        if len(owners) > 1:
            problems.append(Problem(
                5, t("scenario {slug} is declared by more than one wave ({waves}), "
                     "and a test tag names only the slug — it cannot say which",
                     slug=slug, waves=", ".join(owners))))
        else:
            problems.append(Problem(
                5, t("wave {wave} has two scenarios that read as {slug} once "
                     "dashes and underscores are levelled, and a tag names only "
                     "that — rename one", wave=owners[0], slug=slug)))
    if project.adapter is None:
        return problems + [Problem(
            5, t("nothing to run the tests with: the root has none of {markers}",
                 markers=", ".join(item for cls in ADAPTERS
                                   for item in cls.marker)))]

    # One walk of the test tree, shared by both loops: each call to
    # adapter.tags re-walks and re-reads every test file, and this runs in the
    # pre-commit hook.
    tags = project.adapter.tags(project.root)
    for wave, slug, body, found in scenario_tags(project, tags, shared):
        if not found:
            problems.append(Problem(
                5, t("scenario {slug} has no test", slug=slug), wave.rel,
                wave.section_lines.get(f"scenario: {slug}")))
    for _, slug, body, path, line, rev in drifted_tags(project, tags, shared):
        if not rev:
            problems.append(Problem(
                5, t("the test for {slug} carries no revision; it is now {now}",
                 slug=slug, now=revision(body)), path, line))
        else:
            problems.append(Problem(
                5, t("the test holds {slug}@{held}, and the scenario is now {now}",
                 slug=slug, held=rev, now=revision(body)), path, line))

    if run_tests:
        command = project.adapter.test_command(project.root)
        label = project.adapter.test_label(project.root)
        try:
            proc = subprocess.run(command, cwd=project.root, capture_output=True,
                                  text=True, stdin=subprocess.DEVNULL,
                                  timeout=TEST_TIMEOUT)
        except OSError as exc:
            # mix off PATH, a non-executable shim: red with the reason, not a
            # traceback out of the pre-push hook.
            return problems + [Problem(
                5, t("the test command could not run ({command}): {reason}",
                     command=label, reason=exc.strerror or exc))]
        except subprocess.TimeoutExpired:
            return problems + [Problem(
                5, t("the tests did not finish within {seconds}s ({command}). "
                     "Nothing was proved, which is not the same as nothing "
                     "being wrong.", seconds=TEST_TIMEOUT,
                     command=label))]
        if proc.returncode != 0:
            tail = (proc.stdout or proc.stderr).strip().splitlines()[-12:]
            problems.append(Problem(
                5, t("the tests are red ({command}):", command=label)
                + "\n" + "\n".join("      " + line for line in tail)))
        else:
            problems += skipped_proofs(project, proc.stdout or "")
    return problems


def skipped_proofs(project, output):
    """Scenarios whose test was there, was named, and did not run.

    A skipped test leaves the suite successful, so `wasSuccessful()` — the only
    thing greenness used to mean — is true over a test whose body is a bare
    failure. That put "every scenario has a green test" on top of a promise
    nothing had proved, which is the deepest silence this tool can produce.

    Attribution is by name, the way a person would read it: the test proving
    `runs-ok` is called `test_runs_ok`, and both normalise to the same thing.
    A skipped test that names no scenario says nothing here — a platform test
    that cannot run on this machine is not a broken promise.
    """
    skipped, count = project.adapter.not_run(output)
    if not skipped:
        # No names, only a number: say what is unproven without pretending to
        # know which promise it was.
        return [Problem(5, t("{count} tests did not run — skipped or excluded. "
                             "The runner does not say which, so any scenario "
                             "among them is unproven.", count=count))] if count else []
    # Anchored to the whole method name, not searched for inside it. A substring
    # match made a scenario called `ok` claim an unrelated `test_runs_ok_on_windows`
    # — the scenario proved, the skip irrelevant, and the branch red with no way
    # out. Under-matching here costs less than a red nobody can clear: the tag
    # scan still requires a test to exist, this only asks whether it ran.
    flattened = [normalise_slug(name.rsplit(".", 1)[-1]) for name in skipped]
    problems = []
    for wave in project.waves.values():
        if wave.error:
            continue
        for slug in wave.scenarios:
            wanted = normalise_slug(slug)
            for name, flat in zip(skipped, flattened):
                if wanted and flat in (wanted, f"test-{wanted}"):
                    problems.append(Problem(
                        5, t("the test for {slug} did not run: {name} is "
                             "skipped. A test that does not run proves nothing.",
                             slug=slug, name=name), wave.rel,
                        wave.section_lines.get(f"scenario: {slug}")))
                    break
    return problems


def spec_head(entry):
    """The part before the `::` that separates a signature from its return type.

    The separator is the one at bracket depth zero, not the first one in the
    string. Elixir names its arguments — `run(text :: binary()) :: :ok` is what
    the compiler itself hands back — so splitting on the first `::` would cut
    the entry in the middle of an argument and leave an unbalanced paren.
    """
    depth = 0
    for index, char in enumerate(entry):
        if char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
        elif char == ":" and depth == 0 and entry[index:index + 2] == "::":
            return entry[:index].strip()
    return None


def count_arguments(inside):
    """Commas at depth zero, plus one. A comma inside a type is part of that type.

    Bitstrings count as depth too: `<<_::binary, _::8>>` is one argument, and
    without tracking << >> its comma read as a separator — a kept promise
    reported as the wrong arity.
    """
    if not inside.strip():
        return 0
    depth, count, index = 0, 1, 0
    while index < len(inside):
        pair = inside[index:index + 2]
        if pair == "<<":
            depth += 1
            index += 2
            continue
        if pair == ">>":
            depth -= 1
            index += 2
            continue
        char = inside[index]
        if char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
        elif char == "," and depth == 0:
            count += 1
        index += 1
    return count


def promised_signature(entry):
    """What an `exports:` entry names: ("run", 3), from `run/3` or from a spec.

    A contract may promise as little as a name and an arity, or as much as the
    whole signature. Both are checkable, and what is written is what gets
    checked — the short form loses nothing by the long form existing.
    """
    entry = entry.strip()
    head = spec_head(entry)
    if head is None:
        name, _, arity = entry.partition("/")
        # isdigit() is wider than what int() accepts — it is true of a
        # superscript, which int() then refuses. A stray character in a contract
        # has to turn a check red, not end the run in a traceback.
        return (name.strip(), int(arity)) if arity.strip().isdecimal() else None
    match = re.match(r"^([a-z_][A-Za-z0-9_]*[?!]?)\s*\((.*)\)$", head, re.S)
    if not match:
        # `@spec run :: :ok` is legal Elixir: a bare head is zero arity.
        bare = re.fullmatch(r"[a-z_][A-Za-z0-9_]*[?!]?", head)
        return (head, 0) if bare else None
    name, inside = match.group(1), match.group(2).strip()
    if not balanced(inside):
        # `run(a)(b)` matches the pattern because `.*` is greedy. Accepting it
        # would let a garbled line count as a kept promise.
        return None
    return (name, count_arguments(inside))


def balanced(text):
    depth = 0
    for char in text:
        if char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
            if depth < 0:
                return False
    return depth == 0


def flatten_spec(text):
    """One line, and spacing that both sides agree on.

    The compiler renders a spec its own way — `run( binary() )` where a person
    writes `run(binary())`. Comparing those raw would report a difference in
    whitespace as a broken promise, and the message would print a line the eye
    cannot tell from what is already in the contract. So both sides are squeezed
    the same way first, and the squeezing is symmetric: no space hugging a
    bracket, exactly one after a comma, exactly one around a union bar.
    """
    text = re.sub(r"\s+", " ", text).strip()
    text = re.sub(r"([(\[{]) ", r"\1", text)
    text = re.sub(r" ([)\]}])", r"\1", text)
    text = re.sub(r"\s*,\s*", ", ", text)
    text = re.sub(r"\s*\|\s*", " | ", text)
    # The map and function-type arrows get the same treatment as the bar: the
    # compiler always spaces them, people write them tight.
    text = re.sub(r"\s*=>\s*", " => ", text)
    text = re.sub(r"\s*->\s*", " -> ", text)
    # `::` after the arrows, and it matters: named arguments put one inside the
    # head, where people write it tight and the compiler writes it spaced.
    text = re.sub(r"\s*::\s*", " :: ", text)
    # `@spec run :: :ok` is legal Elixir, but the compiler renders zero arity
    # with parens — give the bare head its parens so the two forms compare equal.
    return re.sub(r"^([a-z_][A-Za-z0-9_]*[?!]?) ::", r"\1() ::", text)


def check_exports(project, run_tests=True):
    problems = []
    # A promise that is not a module carries the command that proves it: a
    # service answering, a binary on PATH, a dependency of the right version.
    # Whoever makes the promise does not have to be our code for it to be checkable.
    for contract in project.contracts.values():
        if contract.error:
            continue
        if not run_tests and contract.verify:
            continue      # --no-tests means run nothing, and this is a run too
        if "verify" in contract.front and not contract.verify:
            # Written, but not as a command we can run — a list, a number, an
            # empty string. Skipping it would print a green check over a promise
            # nobody proved, and that is worse than having no verify at all.
            problems.append(Problem(
                6, t("verify has to be a command as a string, and this is {kind}: {value}",
                     kind=type(contract.front["verify"]).__name__,
                     value=repr(contract.front["verify"])),
                contract.rel, contract.line_of("verify")))
            continue
        if not contract.verify:
            continue
        try:
            proc = subprocess.run(contract.verify, shell=True, cwd=project.root,
                                  capture_output=True, text=True,
                                  stdin=subprocess.DEVNULL,
                                  timeout=VERIFY_TIMEOUT)
        except subprocess.TimeoutExpired:
            # Unbounded, a hung command holds pre-push and CI for as long as
            # they are allowed to run, and says nothing while it does.
            problems.append(Problem(
                6, t("the contract did not answer within {seconds}s: {command}", seconds=VERIFY_TIMEOUT, command=contract.verify),
                contract.rel, contract.line_of("verify")))
            continue
        if proc.returncode != 0:
            tail = (proc.stderr or proc.stdout).strip().splitlines()[-3:]
            problems.append(Problem(
                6, t("the contract was not confirmed: {command}", command=contract.verify)
                   + ("\n" + "\n".join("      " + line for line in tail) if tail else ""),
                contract.rel, contract.line_of("verify")))

    # A contract that promises nothing checkable is not a contract. §2.10 is
    # explicit: "a promise nothing can check is not a contract but a boundary,
    # and it lives as a paragraph inside a transform." Left alone, it collected
    # a green sixth check and a scenario could prove it — a tick over a promise
    # nobody ever compared against anything.
    for contract in project.contracts.values():
        # The key's presence, not its value: a verify that is empty or of the
        # wrong type is already named above, and saying it twice would make one
        # mistake read as two.
        if contract.error or "verify" in contract.front:
            continue
        if not contract.exports:
            problems.append(Problem(
                6, t("contract {slug} promises nothing that can be checked: no "
                     "exports to compare and no verify to run", slug=contract.slug),
                contract.rel))

    # Exports without a module are a concrete, checkable promise compared
    # against nothing. Filtering them out silently was green over unproven.
    for contract in project.contracts.values():
        if not contract.error and contract.exports and not contract.module:
            problems.append(Problem(
                6, t("the contract promises exports and names no module to ask "
                     "for them"),
                contract.rel, contract.line_of("exports")))

    contracts = [c for c in project.contracts.values()
                 if not c.error and c.module and c.exports]
    if not contracts:
        return problems
    problems += adapter_problem(project, 6)
    if project.adapter is None:
        return problems + [
            Problem(6, t("no language adapter found — nothing to check exports with"))]
    if not run_tests:
        # The probe imports the project's modules, and importing runs whatever
        # they run at load. --no-tests promises to run nothing; the verify loop
        # above honours that, and so must this. The skip is silent, as verify's
        # is: the flag is the operator's own informed choice.
        return problems

    modules = sorted({c.module for c in contracts})
    actual = project.adapter.exports(project.root, modules)
    if actual.get("__error__"):
        problems.append(Problem(6, t("the modules did not build:") + "\n      " + actual["__error__"]))
    for contract in contracts:
        have = actual.get(contract.module)
        if have is None:
            problems.append(Problem(
                6, t("the module is missing or did not build: {module}", module=contract.module),
                contract.rel, contract.line_of("module")))
            continue
        specs = actual.get(("specs", contract.module), {})
        for promised in contract.exports:
            signature = promised_signature(promised)
            if signature is None:
                problems.append(Problem(
                    6, t("this export is neither name/arity nor a spec: {promised}", promised=promised),
                    contract.rel, contract.line_of(promised)))
                continue
            named = "{0}/{1}".format(*signature)
            if named not in have:
                problems.append(Problem(
                    6, t("{module} does not export what was promised: {promised}", module=contract.module, promised=named),
                    contract.rel, contract.line_of(promised)))
                continue
            if "::" not in promised:
                continue
            # From here on the contract promises a shape, not just a name. If we
            # cannot read the shape, saying so beats a green check over a
            # promise nothing compared.
            if not getattr(project.adapter, "supports_specs", False):
                problems.append(Problem(
                    6, t("{language} cannot be asked for types, so the promised shape of {promised} goes unchecked", language=getattr(project.adapter, "name", "?"), promised=named),
                    contract.rel, contract.line_of(promised)))
            elif named not in specs:
                problems.append(Problem(
                    6, t("{module} declares no @spec for {promised}", module=contract.module, promised=named),
                    contract.rel, contract.line_of(promised)))
            else:
                declared = [flatten_spec(one) for one in specs[named]]
                if flatten_spec(promised) not in declared:
                    problems.append(Problem(
                        6, t("the promised shape of {promised} is not what the module declares:", promised=named)
                           + "".join("\n      " + one for one in declared),
                        contract.rel, contract.line_of(promised)))
    return problems


def check_headings(project):
    problems = []
    for doc in list(project.waves.values()) + list(project.contracts.values()):
        for title in sorted(set(doc.repeated)):
            problems.append(Problem(
                7, t("the heading ## {title} appears twice — the first is read and the "
                     "last is counted", title=title),
                doc.rel, doc.section_lines.get(title)))
    for wave in project.waves.values():
        if wave.error:
            continue
        for kind, declared in (("scenario", wave.scenarios), ("transform", wave.transforms)):
            in_body = set(wave.named_sections(kind))
            in_head = set(declared)
            for slug in sorted(in_head - in_body):
                problems.append(Problem(
                    7, t("the header has {kind} {slug} and the body has no "
                         "section for it", kind=kind, slug=slug),
                    wave.rel, wave.line_of(slug)))
            for slug in sorted(in_body - in_head):
                problems.append(Problem(
                    7, t("the body has ## {kind}: {slug} and the header does not",
                         kind=kind, slug=slug),
                    wave.rel, wave.section_lines.get(f"{kind}: {slug}")))
    return problems


def foreign_waves(project, problems):
    """Slugs of blamed waves that this branch did not come to write.

    Said out loud because of what happens when it is not: an unfinished skeleton
    somebody left behind holds `check` red, the agent finds its own plan commit
    walled off by a file that is not its business, and the shortest way past is
    to move the operator's wave out of the project.
    """
    mine = project.wave_for_branch()
    if mine is None:
        # On the main branch nobody is writing a wave, and a repo-wide check is
        # the operator's own business: every blamed wave would be "foreign", and
        # the advice would be noise on the one branch that has no work to unblock.
        return []
    blamed = {problem.where for problem in problems if problem.where}
    return sorted(wave.slug for wave in project.waves.values()
                  if wave.rel in blamed and wave.rel != mine.rel)


PLAN_BLIND = (5, 6)


def plan_wave(project):
    """The wave this plan branch came to write, or None where it is not one.

    Checks 5 and 6 want a green test for every scenario and a module that
    exports what was promised. A plan branch carries neither, deliberately —
    it holds documents and no code. So on every plan branch that ever existed
    those two were red, pre-push refused, CI went red, and the operator learned
    that red on a plan PR means nothing. A gate that is always shut is not a
    gate; what the plan actually promises is read by `gaps`, and that is what
    runs here instead.
    """
    return project.wave_for_branch() if project.is_plan_branch() else None


def drifted_from_main(project):
    """[(file, added, removed)] — documents this branch changed after approval.

    Approval is derived rather than written: the wave reached the main branch,
    so a person read it and let it through. Nothing then stopped the branch from
    rewriting it — `keel/` is out of scope by design, so that `update` may
    refresh our own files mid-work — and walking the cycle produced a wave
    amended three times after it was approved. Every amendment was right, and
    every one was named only because the agent chose to name it.

    Drift is not forbidden here either: §4.6 says extending a transform's
    file list stays a line in the diff. It is the silence that ends — the
    difference is stated, and whoever opens the pull request knows to look.

    Only modifications, and only on a work branch: a document this branch
    created is new work, not drift, and a plan branch exists to write one.
    """
    if not project.git.available or not project.git.has_commits:
        return []
    branch = project.branch
    if (not branch or branch == "HEAD" or branch == project.git.main_short
            or project.is_plan_branch(branch)):
        return []
    # The merge base, not the tip: main moves on while a branch is open, and
    # diffing against the tip reported somebody else's merged amendment as this
    # branch's drift — a note that fires when nothing happened, which is how a
    # note stops being read.
    base = project.git.merge_base(project.git.main_branch)
    if not base:
        return []
    stdout = project.git.out("diff", "--numstat", "--diff-filter=M",
                             base, "--", "keel/waves", "keel/contracts")
    drifted = []
    for line in (stdout or "").splitlines():
        parts = line.split("\t")
        if len(parts) != 3:
            continue
        added, removed, name = parts
        drifted.append((name, added, removed))
    return sorted(drifted)


def ci_verdict(project, run=True):
    """(problems, note, ran) — the project's own gate, named by whoever owns it.

    `ran` exists because the tick was printed from "no problems", and a gate
    that was skipped also has no problems: `check --no-tests` reported a green
    CI over a command it never started.

    Keel checks documents against facts. Whether the project itself builds,
    lints and passes its own suite is the project's business, and only the
    project knows the command — so it names one, and the condition is simply
    that the command succeeds. The same shape a contract's `verify` already has,
    for the same reason: who makes the promise does not matter, that it can be
    checked does.

    Three states, and the middle one is the whole point. A command is run. The
    word `none` is the decision to have no gate, said out loud, and it is
    silent. An empty setting is nobody having decided, and that is said on every
    run — because a merge going through with nothing checked, and nobody knowing
    it, is the silence this tool exists against. Refusal is not silence; that is
    the rule the quality cuts already live by.
    """
    command = (project.settings.get("ci") or CI_UNDECIDED).strip()
    if command == CI_REFUSED:
        return [], None, False
    if not command:
        # The example is the adapter's own proposal where there is one: telling
        # a Python project to write "mix ci" is a small untruth in a message
        # whose whole job is to be acted on.
        adapter = project.adapter
        example = (adapter.ci_command if adapter and adapter.ci_command
                   else t("your command"))
        return [], t("no CI command: merges go with nothing of the project's "
                     "own run. Name one in {file} (\"ci\": \"{example}\"), or "
                     "say there is none (\"ci\": \"none\").",
                     file=CONFIG_FILE, example=example), False
    if not run:
        return [], None, False
    try:
        proc = subprocess.run(command, shell=True, cwd=project.root,
                              capture_output=True, text=True,
                              stdin=subprocess.DEVNULL, timeout=CI_TIMEOUT)
    except subprocess.TimeoutExpired:
        return [Problem(0, t("CI did not finish within {seconds}s: {command}",
                             seconds=CI_TIMEOUT, command=command))], None, True
    except OSError as exc:
        return [Problem(0, t("CI could not run ({command}): {reason}",
                             command=command,
                             reason=exc.strerror or exc))], None, True
    if proc.returncode == 0:
        return [], None, True
    tail = (proc.stdout or proc.stderr).strip().splitlines()[-10:]
    # 127 is the shell saying the thing is not there at all. Worth its own
    # sentence: "the command is missing" and "the command failed" are different
    # states, and a raw runner error makes the reader work that out.
    reason = (t("CI is not set up: {command} — there is no such command")
              if proc.returncode == 127 else
              t("CI is red: {command}"))
    return [Problem(0, reason.format(command=command)
                    + ("\n" + "\n".join("      " + line for line in tail)
                       if tail else ""))], None, True


def run_checks(project, only=None, run_tests=True):
    only = set(only or CHECK_NAMES)
    if plan_wave(project) is not None:
        only -= set(PLAN_BLIND)
    results = {}
    structural = check_structure(project)
    runners = {
        1: lambda: check_refs(project),
        2: lambda: check_cycles(project),
        3: lambda: check_revisions(project),
        4: lambda: check_scope(project),
        5: lambda: check_scenarios(project, run_tests=run_tests),
        6: lambda: check_exports(project, run_tests=run_tests),
        7: lambda: check_headings(project),
    }
    # Check 4 runs after 5 and 6, though it is displayed in place: the test run
    # and the probes may drop files into the tree (_build/ without a gitignore),
    # and reading git before they run made two consecutive checks describe the
    # same tree in opposite words.
    for number in sorted(runners, key=lambda n: (n in (4,), n)):
        results[number] = runners[number]() if number in only else None
    return structural, results


# ─────────────────────────────────────────────────────────────────────────────
# Commands
# ─────────────────────────────────────────────────────────────────────────────

# The skeletons are written into somebody's project, so they follow `lang` like
# every other line the tool produces. The heading is the one structural word in
# them, and the reader accepts either spelling — a project may change language
# without its existing waves becoming unreadable.
STEP_SKELETON = """---
depends_on: []

scenarios:
  # <slug>: {{proves: <contract>@<rev>}}

transforms:
  # <slug>:
  #   implements: [<scenario>]
  #   contracts:  [<contract>@<rev>]
  #   files:      [<{path}>]
---

## {why}

{slug}: {why_hint}

## scenario: <slug>

**Given** ...,
**When** ...,
**Then** ...

## transform: <slug>

{does}

{bounds}: {bounds_hint}
"""

CONTRACT_SKELETON = """---
# {module_hint}
module: <Module.Name>
# {exports_hint}
exports: []
#   - "run(binary(), keyword()) :: {{:ok, term()}}"
#   - "halt/1"
# {verify_hint}
# verify: curl -sf http://localhost:11434/api/tags
---

{prose}
"""


WHY_HEADINGS = ("why", "навіщо")


def wave_skeleton(slug):
    """One formatting pass: two would need the literal braces escaped twice."""
    return STEP_SKELETON.format(
        slug=slug,
        path=t("path/to/file"),
        why=t("Why"),
        why_hint=t(WHY_HINT),
        does=t("What it does."),
        bounds=t("Boundaries"),
        bounds_hint=t("what it does not do."))


def contract_skeleton():
    return CONTRACT_SKELETON.format(
        module_hint=t("A module that promises something:"),
        exports_hint=t("A name with an arity or a whole signature — what is "
                       "written is what gets checked:"),
        verify_hint=t("Or a promise that is not a module — a command whose "
                      "success is the proof:"),
        prose=t("What exactly is promised, and to whom."))


WHY_HINT = "why this wave exists and what is missing without it"


def unfilled_why(wave):
    """The skeleton's own placeholder, in whatever language it was written in.

    Derived from the catalogue entry the skeleton writes, not restated: reword
    the hint and this recogniser follows, instead of silently going blind.
    """
    text = wave.why.strip()
    if not text.startswith(f"{wave.slug}:"):
        return False
    tail = text[len(wave.slug) + 1:].strip().lower()
    hints = (WHY_HINT, UK.get(WHY_HINT, WHY_HINT))
    return any(tail.startswith(hint.lower()) for hint in hints)


def cmd_new(project, args):
    kind, slug = args.kind, args.slug
    clean = normalise_slug(slug)
    if not clean:
        fail(t("bad slug: {slug}", slug=repr(slug)))

    if kind == "wave":
        folder = os.path.join(project.keel, "waves")
        numbers = [int(m.group(1)) for name in os.listdir(folder)
                   if (m := re.match(r"(\d{4})-", name))] if os.path.isdir(folder) else []
        number = max(numbers, default=0) + 1
        name = f"{number:04d}-{clean}.md"
        # The numbered name, as the file is called: the Why placeholder starts
        # with the slug, and the recogniser compares against wave.slug — the
        # unnumbered form made unfilled_why blind to every tool-created wave.
        text = wave_skeleton(os.path.splitext(name)[0])
    else:
        folder = os.path.join(project.keel, "contracts")
        name = f"{clean}.md"
        text = contract_skeleton()

    path = os.path.join(folder, name)
    if os.path.exists(path):
        fail(t("already there: {path}", path=os.path.relpath(path, project.root)))
    os.makedirs(folder, exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    print(os.path.relpath(path, project.root))
    return 0


def depends_closure(project, wave):
    """Every wave this one reaches through depends_on, however far."""
    seen, queue = set(), [ref.slug for ref in wave.depends_on]
    while queue:
        slug = queue.pop()
        if slug in seen:
            continue
        seen.add(slug)
        other = project.waves.get(slug)
        if other and not other.error:
            queue.extend(ref.slug for ref in other.depends_on)
    return seen


def missing_edges(project, wave):
    """Waves this one leans on without saying so.

    `depends_on` is the only order there is — the numbers in the names are
    unique prefixes and nothing more — so an empty list reads the same whether
    the dependency is absent or forgotten. Check 2 looks for cycles, and an
    empty graph has none, so nothing ever noticed.

    Asked, never demanded: two waves may legitimately touch one file without
    either leaning on the other, and a fabricated failure is worse than silence.

    Files only. A shared contract was asked about too and had to go: leaning on
    a common promise is not depending on the wave that first wrote it, so a
    logger or a config contract produced a question in every wave naming every
    other — N waves, N×(N−1) questions, none of them a missing edge. A question
    that is usually wrong is one an agent learns to answer "deliberate" without
    reading it, which costs more than the silence it replaced.
    """
    known = depends_closure(project, wave)
    files = wave.declared_files()

    problems = []
    for slug, other in sorted(project.waves.items()):
        if slug == wave.slug or slug in known or other.error:
            continue
        if wave.slug in depends_closure(project, other):
            # It leans on us, so we do not lean on it. The direction is not
            # guessed from the numbers in the names — those are unique
            # prefixes, not an order — it is read from the graph itself.
            continue
        shared = sorted(files & other.declared_files())
        if shared:
            problems.append(Problem(
                0, t("wave {other} declares {name} too, and depends_on does not "
                     "name it — deliberate?", other=slug, name=shared[0]), wave.rel))
    return problems


def unclaimed_contracts(project, wave):
    """Contracts this wave brought that none of its transforms leans on.

    Check 1 asks the one direction — every slug named in a header has its file.
    Nothing asked the other, so a contract could be written, referenced by
    nobody, its `verify` file declared in no transform, and `gaps` would still
    say the plan is complete. Seen live in wave 0003; the agent noticed and
    wired it up itself, which is not a guard.

    Only contracts this wave is bringing: one that already lives on the main
    branch belongs to whoever put it there, and a wave is free to leave it
    alone. A question, like the forgotten edge — a contract may honestly be
    written a wave ahead of the work that leans on it.
    """
    if not project.git.available or not project.git.has_commits:
        return []
    leaned_on = {ref.slug for slug in wave.transforms
                 for ref in wave.transform_contracts(slug)}
    leaned_on |= {ref.slug for slug in wave.scenarios
                  for ref in wave.proves(slug)}
    return [Problem(
        0, t("contract {slug} arrives with this wave, and no transform or "
             "scenario leans on it — deliberate?", slug=slug), wave.rel)
        for slug in sorted(project.arriving_contracts - leaned_on)]


def gaps_problems(project, waves):
    """What a plan is missing mechanically. Read by `gaps` and by `check`.

    One body, because the plan branch is gated by both: `gaps` while it is being
    written, `check` before it is pushed and merged. Two spellings of the same
    list would drift, and the half that drifted would be the gate.
    """
    problems = []
    for wave in waves:
        if wave.error:
            problems.append(Problem(0, wave.error, wave.rel))
            continue
        if not wave.why.strip() or unfilled_why(wave):
            problems.append(Problem(0, t("the Why section is empty"), wave.rel))
        if not wave.scenarios:
            problems.append(Problem(0, t("no scenarios at all"), wave.rel))
        if not wave.transforms:
            problems.append(Problem(0, t("no transforms at all"), wave.rel))

        implemented = set()
        for slug in wave.transforms:
            implemented.update(wave.transform_implements(slug))
            if not wave.transform_files(slug):
                problems.append(Problem(
                    0, t("transform {slug} declared no files", slug=slug), wave.rel,
                    wave.line_of(slug)))
            if not wave.transform_implements(slug):
                problems.append(Problem(
                    0, t("transform {slug} implements no scenario", slug=slug), wave.rel,
                    wave.line_of(slug)))
            if not (wave.transform_body(slug) or "").strip():
                problems.append(Problem(
                    0, t("transform {slug} has no body: what it does and where its edges are", slug=slug), wave.rel))
        for slug in wave.scenarios:
            if not wave.proves(slug):
                problems.append(Problem(
                    0, t("scenario {slug} has no proves", slug=slug), wave.rel, wave.line_of(slug)))
            if slug not in implemented:
                problems.append(Problem(
                    0, t("no transform implements scenario {slug}", slug=slug), wave.rel,
                    wave.line_of(slug)))
            if not (wave.scenario_body(slug) or "").strip():
                problems.append(Problem(
                    0, t("scenario {slug} has no body: given/when/then", slug=slug), wave.rel))

        problems += missing_edges(project, wave)
        problems += unclaimed_contracts(project, wave)

    mine = {wave.rel for wave in waves}
    problems += [p for p in check_headings(project) if p.where in mine]
    problems += [p for p in check_refs(project) if p.where in mine]
    return problems


def cmd_gaps(project, args):
    waves = ([project.waves[args.wave]] if args.wave and args.wave in project.waves
             else [project.wave_for_branch()] if not args.wave and project.wave_for_branch()
             else list(project.waves.values()))
    if args.wave and args.wave not in project.waves:
        fail(t("no such wave: {wave}", wave=args.wave))
    waves = [wave for wave in waves if wave]

    # The same disagreement `check` refuses: gaps is what the planning skill
    # runs until it comes back clean, so a slug two waves share must not slip
    # through the plan and surface on a work branch, where fixing it means
    # amending an approved plan.
    problems = shared_transform_slugs(project) + gaps_problems(project, waves)
    names = ", ".join(wave.slug for wave in waves) or t("nothing")
    if not problems:
        print(t("the plan is complete: {names}", names=names))
        return 0
    print(t("the plan is missing things ({names}):", names=names) + "\n")
    for problem in problems:
        print(problem.render())
    print("\n" + t("in total: {count}", count=len(problems)))
    return 1


def cmd_check(project, args):
    only = FAST_CHECKS if args.fast else None
    structural, results = run_checks(project, only, run_tests=not args.no_tests)

    # The plan's own gate, and only in the full run: a commit on a plan branch
    # may be half-written, a push and a merge may not.
    planning = plan_wave(project)
    plan_gaps = ([] if planning is None or args.fast
                 else gaps_problems(project, [planning]))
    # The project's own gate, and only in the full run — same reasoning as the
    # plan's: a commit may be half-written, a push and a merge may not.
    ci_problems, ci_note, ci_ran = ci_verdict(
        project, run=not args.fast and not args.no_tests)
    drift = drifted_from_main(project)
    disagreements = shared_transform_slugs(project)

    if args.json:
        payload = {
            "ok": (not structural and not disagreements and not plan_gaps
                   and not ci_problems
                   and not any(results.get(n) for n in results)),
            "structure": [p.as_dict() for p in structural],
            "disagreement": [p.as_dict() for p in disagreements],
            "plan": [p.as_dict() for p in plan_gaps],
            "ci": {"command": project.settings.get("ci", ""),
                   "problems": [p.as_dict() for p in ci_problems],
                   "note": ci_note, "ran": ci_ran},
            # Drift belongs here too: the whole point of naming it is that
            # somebody learns of it, and scripts read this payload, not the
            # prose underneath.
            "drift": [{"file": name, "added": added, "removed": removed}
                      for name, added, removed in drift],
            "checks": {
                str(number): {
                    "name": t(CHECK_NAMES[number]),
                    "run": results[number] is not None,
                    # Distinct from "run": the check ran, its expensive half
                    # did not, and a script could not tell the two apart.
                    "fully": (results[number] is not None
                              and not (args.no_tests and number in PLAN_BLIND)),
                    "problems": [p.as_dict() for p in (results[number] or [])],
                }
                for number in sorted(results)
            },
        }
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return 0 if payload["ok"] else 1

    total = len(structural) + len(disagreements)
    if structural:
        print("✗ " + t("documents do not parse"))
        for problem in structural:
            print(problem.render())
        print()

    # Its own heading: these documents parse perfectly, they disagree with each
    # other, and calling that a parse error sends the reader hunting for a YAML
    # mistake that is not there.
    if disagreements:
        print("✗ " + t("documents disagree with each other"))
        for problem in disagreements:
            print(problem.render())
        print()

    blind = t("(not run: a plan branch has no code)")
    # The expensive half of 5 and 6 — the suite, the module probe, the verify
    # commands — is what --no-tests turns off, and the tick was printed from
    # "no problems", which a check that was skipped also has. Say so on the tick
    # itself rather than let it read as proof.
    partial = t("(the tests and probes were not run)") if args.no_tests else ""
    for number in sorted(results):
        problems = results[number]
        if problems is None:
            why = blind if planning is not None and number in PLAN_BLIND else t("(not run)")
            print(f"– {number}. " + t(CHECK_NAMES[number]) + " " + why)
            continue
        total += len(problems)
        if not problems:
            qualifier = " " + partial if partial and number in PLAN_BLIND else ""
            print(f"✓ {number}. " + t(CHECK_NAMES[number]) + qualifier)
            continue
        print(f"✗ {number}. " + t(CHECK_NAMES[number]))
        for problem in problems:
            print(problem.render())
    if plan_gaps:
        total += len(plan_gaps)
        print("\u2717 " + t("the plan is missing things"))
        for problem in plan_gaps:
            print(problem.render())
    if ci_problems:
        total += len(ci_problems)
        for problem in ci_problems:
            print("\u2717 " + problem.message)
    elif ci_ran:
        # Not through the catalogue: a proper noun and a command have nothing
        # to translate, and an entry that renders to itself reads as a
        # forgotten translation to the guard that watches for exactly that.
        print("\u2713 CI: " + project.settings["ci"].strip())
    print()
    print(t("clean") if total == 0 else t("problems: {count}", count=total))

    if ci_note:
        print("\u2013 " + ci_note)

    for name, added, removed in drift:
        print("\u2013 " + t("{file} differs from what was approved: +{added} "
                            "-{removed}. Allowed, and it stays a line in the "
                            "diff — say in the pull request what changed and "
                            "why.", file=name, added=added, removed=removed))

    everything = (list(structural) + list(plan_gaps)
                  + [p for found in results.values() if found for p in found])
    strays = foreign_waves(project, everything)
    if strays:
        print("\n" + t("{waves}: this branch did not come to write that wave. "
                       "Somebody else's wave is not moved, renamed or deleted to "
                       "get a check green — leave it and say it is there.",
                       waves=", ".join(strays)))
    return 0 if total == 0 else 1


def next_transform(project, wave):
    state = project.transform_state(wave)
    for slug in wave.transforms:
        if state[slug][0] is None:
            return slug, state
    return None, state


def main_branch_answer(project):
    """What to do while standing where finished work lands.

    The tool knows the state and the agent has the judgement — but this answer
    used to be "the branch is not named after a wave" and nothing more, while
    the documents and git together plainly said which wave is approved, which
    of its transforms are open, and what its branch has to be called.
    """
    # Not from ambiguous documents. Closure is read out of commit messages by
    # slug, so a slug two waves share makes every answer here a guess — and the
    # guess it made was "every wave is finished" over a wave nobody had started.
    broken = check_structure(project) + shared_transform_slugs(project)
    if broken:
        return t("the documents do not agree with themselves, so there is no "
                 "saying what is done: {reason}", reason=broken[0].message)
    ready, blocked, unplanned = project.ready_waves()
    if not project.waves:
        # Not the same sentence as "everything is finished": in a project that
        # has just taken Keel nothing is finished, because nothing exists — and
        # this is the first line its agent ever reads.
        return t("no waves yet. The first one starts with a plan: keel new wave "
                 "<slug>, then the branch plan/<that name>.")
    if not ready and not blocked and not unplanned:
        return t("every wave is finished. The next one starts with a plan: "
                 "keel new wave <slug>, then the branch plan/<that name>.")
    if not ready:
        if unplanned:
            names = ", ".join(wave.slug for wave in unplanned)
            return t("{waves}: the plan is not written yet — no transforms, so "
                     "there is no work to hand out. keel gaps says what is "
                     "missing.", waves=names)
        waiting = ", ".join(wave.slug for wave, _ in blocked)
        return t("every unfinished wave is waiting on another that is not done "
                 "yet: {waves}. Finish what they lean on, or plan the wave that "
                 "is missing.", waves=waiting)
    wave, open_count = ready[0]
    return t("wave {wave} is approved and {open} of {total} transforms are not "
             "closed. The work goes on its own branch:\n"
             "  git checkout -b {wave}", wave=wave.slug,
             open=open_count, total=len(wave.transforms))


def cmd_next(project, args):
    wave = project.wave_for_branch()
    branch = project.branch
    if wave is None:
        if branch and branch == project.git.main_short:
            return emit_next_error(args, main_branch_answer(project))
        message = t("branch {branch} is not named after a wave. Work happens on "
                    "a branch named after the wave, planning on plan/<wave>.",
                    branch=branch)
        return emit_next_error(args, message)
    if project.is_plan_branch(branch):
        return emit_next_error(args, t("this is a plan branch: the wave is written "
                                       "here, not code. keel gaps says what is "
                                       "missing."))
    if not project.git.file_in_branch(project.git.main_branch, wave.rel):
        return emit_next_error(
            args, t("wave {wave} is not on {main} yet: the plan is not approved "
                    "and there is no work.", wave=wave.slug,
                    main=project.git.main_branch))
    if wave.error:
        return emit_next_error(args, f"{wave.rel}: {wave.error}")

    slug, state = next_transform(project, wave)
    if slug is None:
        message = t("every transform of wave {wave} is closed by a commit. "
                    "Next: keel check, then the PR.", wave=wave.slug)
        return emit_next_error(args, message, code=0)

    package = next_package(project, wave, slug, state)
    if args.json:
        print(json.dumps(package, ensure_ascii=False, indent=2))
    else:
        print(render_next(package))
    return 0


def next_package(project, wave, slug, state):
    """Everything needed for one move, and nothing beyond it."""
    contracts = []
    for ref in wave.transform_contracts(slug):
        contract = project.contracts.get(ref.slug)
        contracts.append({
            "slug": ref.slug,
            "rev": ref.rev,
            "rev_ok": bool(contract and not contract.error and contract.rev_ok(ref.rev)),
            "rev_now": contract.revision if contract and not contract.error else None,
            "module": contract.module if isinstance(contract, Contract) else None,
            "exports": contract.exports if isinstance(contract, Contract) else [],
            "body": contract.body.strip() if contract else None,
        })

    scenarios = []
    for name in wave.transform_implements(slug):
        body = wave.scenario_body(name)
        rev = wave.scenario_revision(name)
        scenarios.append({
            "slug": name,
            "rev": rev,
            "proves": [ref.raw for ref in wave.proves(name)],
            "body": (body or "").strip(),
            # The dictated tag in the adapter's own dialect: dictating the
            # Elixir form to a Python project made the operator's obedient tag
            # invisible to the collector. No adapter — no dictation. No revision
            # either: a scenario with no body has none, and interpolating it
            # dictated the literal `rev: "None"`, which rev_matches can never
            # accept — a permanent red planted by the tool's own instruction.
            "tag": (project.adapter.tag_example(name, rev)
                    if rev and project.adapter
                    and hasattr(project.adapter, "tag_example")
                    else None),
        })

    return {
        "wave": {"id": wave.slug, "file": wave.rel, "why": wave.why.strip()},
        "transform": {
            "slug": slug,
            "body": (wave.transform_body(slug) or "").strip(),
            "files": wave.transform_files(slug),
        },
        "scenarios": scenarios,
        "contracts": contracts,
        "done": [name for name, (sha, _) in state.items() if sha],
        "left": [name for name in wave.transforms
                 if state[name][0] is None and name != slug],
        "commit": f"{slug}: " + t("<what was done>"),
        "tag_hint": [
            {"scenario": item["slug"], "rev": item["rev"]} for item in scenarios
        ],
    }


def emit_next_error(args, message, code=1):
    if args.json:
        print(json.dumps({"error": message, "done": code == 0}, ensure_ascii=False, indent=2))
    else:
        print(message)
    return code


def render_next(package):
    wave, transform = package["wave"], package["transform"]
    out = [f"# {transform['slug']}", ""]
    out.append(t("Wave {id} · {file}", id=wave["id"], file=wave["file"]))
    if package["done"]:
        out.append(t("Closed: {names}", names=", ".join(package["done"])))
    if package["left"]:
        out.append(t("After this one: {names}", names=", ".join(package["left"])))
    out.append("")
    if wave["why"]:
        out += ["## " + t("Why the wave"), "", wave["why"], ""]
    if transform["body"]:
        out += ["## " + t("This transform"), "", transform["body"], ""]
    out += ["## " + t("The files, and only these"), ""]
    out += [f"- {name}" for name in transform["files"]] or ["- " + t("(none declared — the plan is incomplete)")]
    out.append("")

    if package["scenarios"]:
        out += ["## " + t("Scenarios it brings closer"), ""]
        for item in package["scenarios"]:
            out.append(f"### {item['slug']}")
            out.append("")
            out.append(item["body"] or t("(no body)"))
            out.append("")
            if item.get("tag"):
                out.append(t("Test tag: `{tag}`", tag=item["tag"]))
                out.append("")

    if package["contracts"]:
        out += ["## " + t("Contracts it leans on"), ""]
        for item in package["contracts"]:
            head = f"### {item['slug']}"
            if item["module"]:
                head += f" — `{item['module']}`"
            out.append(head)
            out.append("")
            if item["exports"]:
                out.append(t("Exports: {names}", names=", ".join(item["exports"])))
                out.append("")
            out.append(item["body"] or t("(no such contract)"))
            out.append("")
            if not item["rev_ok"]:
                out.append(t("⚠ the wave holds {held}, the contract is now "
                             "{now} — keel rev first",
                             held=item["rev"], now=item["rev_now"]))
                out.append("")

    out += ["## " + t("The commit"), "", f"    {package['commit']}", ""]
    out.append(t("The transform slug in the message is the only link between "
                 "the work and the plan."))
    return "\n".join(out)


INIT_DIRS = ("keel/waves", "keel/contracts")
AGENTS_START = "<!-- keel:start -->"
AGENTS_END = "<!-- keel:end -->"
VENDORED = "keel/keel.py"
# References travel as copies: AGENTS.md points at them, and you can only point
# at what sits in the same repository. Methodology, tool, quality cuts.
REFERENCES = ("METHODOLOGY.md", "README.md", "QUALITY.md")

# Two settings, and they are deliberately separate: someone may well want the
# reference in English while the agent writes and listens in their own language.
#
#   docs — which translation of the references lands in the project
#   lang — what the agent writes (waves, commits) and what phrases the skills catch
#
# Neither can be guessed, so they are the one thing Keel keeps in a config file.
CONFIG_FILE = "keel/keel.json"
SOURCE_LANG = "uk"          # the language the methodology is written in
PUBLISHED_LANG = "en"       # the one at the root: the face of the repository
LANGS = ("uk", "en")
# How much of itself Keel installs. One word rather than a row of switches,
# because the three answers people actually give are whole positions, not
# independent bits: let the method drive, let it advise, or let it wait to be
# called. `strict` is the default because a method nobody starts is not a method.
MODES = ("strict", "soft", "manual")
# "" means: work it out from the markers in the root, and say so when the root
# is ambiguous. Derived, not restated: a third adapter added to ADAPTERS must
# not leave the settings layer disagreeing about which adapters exist.
ADAPTER_NAMES = ("",) + tuple(adapter.name for adapter in ADAPTERS)
# agent_hooks: "" lets the mode decide; True/False is the operator's override,
# stored so the first routine update does not quietly revert their choice.
DEFAULTS = {"docs": PUBLISHED_LANG, "lang": PUBLISHED_LANG, "mode": "strict",
            "adapter": "", "agent_hooks": "", "ci": CI_UNDECIDED,
            "agents": list(DEFAULT_AGENTS)}
ALLOWED = {"docs": LANGS, "lang": LANGS, "mode": MODES,
           "adapter": ADAPTER_NAMES, "agent_hooks": ("", True, False)}
# Keys whose value is a list drawn from a known set. Empty is a real answer:
# a project may want the method and none of the agent files.
LISTS = {"agents": AGENT_NAMES}
# Keys whose value is a command rather than one of a known set. Validating them
# against a list would mean Keel deciding what a project may run.
FREE_TEXT = ("ci",)
REVISIONS = "docs/revisions.json"


def generated_files(root, settings):
    """{path in the project: what the methodology would put there now}.

    Only files Keel owns whole. AGENTS.md and .claude/settings.json are shared
    with the project — Keel owns a block inside them, not the file — so they are
    merged rather than tracked, and `update` refreshes them the same way `init`
    does.
    """
    out = {VENDORED: read_text(os.path.abspath(__file__))}
    for name in REFERENCES:
        source = doc_source(name, settings["docs"])
        if os.path.exists(source):
            out[f"keel/{name}"] = strip_front_matter(read_text(source))
    for skill in SKILLS:
        for agent, relative in skill_targets(skill, settings["agents"]):
            out[relative] = render_skill(skill, agent, settings["lang"],
                                         settings["mode"])
    adapter = detect_adapter(root, settings["adapter"])
    out[CI_FILE] = CI_TEMPLATE.format(
        tool=VENDORED,
        setup="".join(line + "\n" for line in (adapter.ci_steps(root) if adapter else [])))
    if agent_hooks_wanted(settings) and "cursor" in settings["agents"]:
        out[CURSOR_HOOKS] = json.dumps(cursor_hook_config(), ensure_ascii=False,
                                       indent=2) + "\n"
    return out


def agent_hooks_wanted(settings, args=None):
    """Whether to install the hooks that watch what the agent writes.

    The mode decides, a stored override outlives it, and a flag overrules both.
    `manual --agent-hooks` is the combination the three words alone would lose:
    a person who starts every procedure by hand may still want the guard that
    refuses a write outside the declared files.
    """
    chosen = getattr(args, "agent_hooks", None)
    if chosen is not None:
        return chosen
    stored = settings.get("agent_hooks", "")
    if stored != "":
        return stored
    return settings["mode"] == "strict"


def load_json_object(path, label):
    """The file as a dict, or None with the reason said once.

    Both editors of a shared JSON file refuse the same two ways — unreadable
    and not-an-object — and stating that twice is how the two came to validate
    differently.
    """
    if not os.path.exists(path):
        return {}
    try:
        found = json.loads(read_text(path))
    except ValueError:
        print("  " + t("{file}: does not parse as JSON, leaving it alone", file=label))
        return None
    if not isinstance(found, dict):
        print("  " + t("{file}: not an object, leaving it alone", file=label))
        return None
    return found


def refuse_broken_config(root):
    """The three commands that regenerate files stop here, in one sentence."""
    if config_broken(root):
        fail(t("{file} does not parse — fix it first. Regenerating on the "
               "defaults would rewrite the project in the wrong language.",
               file=CONFIG_FILE))


def config_broken(root):
    """The settings file exists and does not read as an object.

    read_config falls back to the defaults then — right for a hook, which must
    answer something — but a command that regenerates files would act on those
    defaults: rewrite a Ukrainian project's AGENTS block in English and label
    the pristine copies hand-edited. Those commands refuse instead.
    """
    path = os.path.join(root, CONFIG_FILE)
    if not os.path.exists(path):
        return False
    try:
        return not isinstance(json.loads(read_text(path)), dict)
    except ValueError:
        return True


def read_config(root):
    # Copied one level down: a list in DEFAULTS would otherwise be the same
    # object in every settings dict in the process, and one in-place edit
    # anywhere would rewrite the default for everybody after it.
    settings = {key: (list(value) if isinstance(value, list) else value)
                for key, value in DEFAULTS.items()}
    path = os.path.join(root, CONFIG_FILE)
    if os.path.exists(path):
        try:
            stored = json.loads(read_text(path))
        except ValueError:
            return settings
        if isinstance(stored, dict):
            for key in DEFAULTS:
                value = stored.get(key)
                if key in FREE_TEXT:
                    if isinstance(value, str):
                        settings[key] = value.strip()
                elif key in LISTS:
                    # Stored order and duplicates are somebody's typing; the
                    # canonical order is ours, so the generated set is the same
                    # whichever way the list was written.
                    if isinstance(value, list) and all(v in LISTS[key] for v in value):
                        settings[key] = [n for n in LISTS[key] if n in value]
                elif value in ALLOWED[key]:
                    settings[key] = value
    return settings


def write_config(root, settings, done, manifest=None):
    path = os.path.join(root, CONFIG_FILE)
    # Overwriting an unreadable config would silently reset docs and lang to the
    # defaults and regenerate the skills with the wrong triggers; valid JSON that
    # is not an object is just as much somebody's file.
    found = load_json_object(path, CONFIG_FILE)
    if found is None:
        return
    # Merge rather than replace: this file lives in somebody's repository, and
    # dropping a key we do not recognise is destroying their data during an
    # operation whose whole design elsewhere is to refuse rather than destroy.
    stored = dict(found) if isinstance(found, dict) else {}
    stored.update(settings)
    if manifest is not None:
        stored["generated"] = manifest
    text = json.dumps(stored, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    if os.path.exists(path) and read_text(path) == text:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    if CONFIG_FILE not in done:
        done.append(CONFIG_FILE)


def read_manifest(root):
    path = os.path.join(root, CONFIG_FILE)
    if not os.path.exists(path):
        return {}
    try:
        stored = json.loads(read_text(path))
    except ValueError:
        return {}
    found = stored.get("generated") if isinstance(stored, dict) else None
    return found if isinstance(found, dict) else {}


def survey(project, wanted, manifest):
    """(fresh, stale, touched, absent) — what update would do to each file."""
    fresh, stale, touched, absent = [], [], [], []
    for relative, wanted in wanted.items():
        path = os.path.join(project.root, relative)
        if not os.path.exists(path):
            absent.append(relative)
            continue
        now = read_text(path)
        if now == wanted:
            fresh.append(relative)
        elif relative in manifest and digest(now) == manifest[relative]:
            # Exactly what we last wrote, so the methodology moved on.
            stale.append(relative)
        else:
            # Edited, or we have no record. Both mean: do not overwrite.
            touched.append(relative)
    return fresh, stale, touched, absent


def doc_source(name, lang):
    """Where a reference lives at home.

    English sits at the root because that is what a visitor to the repository
    opens first; Ukrainian is the source and lives under docs/. Which one is the
    source and which one is on the front page are separate questions.
    """
    if lang == PUBLISHED_LANG:
        return os.path.join(home(), name)
    return os.path.join(home(), "docs", lang, name)


def translations(lang):
    """Translated references, with the source revision each one claims to follow.

    The bookkeeping lives in one file rather than in each document's header:
    the translated README is the repository's front page, and a YAML header
    would render as a table above the first line of it.
    """
    if lang == SOURCE_LANG:
        return {}
    try:
        stored = json.loads(read_text(os.path.join(home(), REVISIONS)))
    except (ValueError, OSError):
        return {}
    if not isinstance(stored, dict):
        # Checked before .items(), not inside the comprehension: a JSON list
        # parsed fine and then crashed update with a traceback.
        return {}
    return {name: str(rev) for name, rev in stored.items()
            if os.path.exists(doc_source(name, lang))}


LOGO_RE = re.compile(r'^<p align="center">.*?</p>\s*', re.S)


def strip_front_matter(text):
    """A reference carries its dressing at home; the project gets the prose.

    Front matter is bookkeeping, and the logo block points at an image that does
    not travel — a vendored copy carrying that pointer would render a broken
    image over the first line of every project's reference.
    """
    front, body, _ = split_front_matter(text)
    text = body.lstrip("\n") if front is not None else text
    return LOGO_RE.sub("", text, count=1)


def check_translations(project):
    """A translation that stopped following its source is worse than none.

    Same rule as everywhere else in Keel: whoever leans on a text holds its
    revision. Here the English page leans on the Ukrainian one. This lives with
    `update` rather than with `check`: the checks are about a project's own
    graph, and this one is about the methodology's copies of itself.
    """
    lang = project.settings["docs"]
    if lang == SOURCE_LANG:
        return []
    problems = []
    for name, recorded in translations(lang).items():
        source = doc_source(name, SOURCE_LANG)
        if not os.path.exists(source):
            continue
        text = read_text(source)
        where = os.path.relpath(doc_source(name, lang), home()) + f" ({REVISIONS})"
        if not recorded:
            problems.append(Problem(
                8, t("the translation names no source revision; it is now {now}",
                     now=revision(text)),
                where))
        elif not rev_matches(recorded, text):
            problems.append(Problem(
                8, t("the translation holds {held}, and {name} is now {now}",
                     held=recorded, name=name, now=revision(text)),
                where))
    return problems

AGENTS_BLOCK_EN = """{start}
## Keel

This project's method: two kinds of document — wave and contract — and the
checks that hold them. Waves live in `keel/waves/`, contracts in `keel/contracts/`.

{principles}

Two commands:

- `python3 {tool} next` — what to do next: the transform, its files and
  boundaries, the scenarios it brings closer, the contracts it leans on.
- `python3 {tool} check` — what is wrong right now. Before a commit and before a PR.

Three references — open them when something is unclear:

- `keel/METHODOLOGY.md` — the method: what goes in a wave's header, how revisions work,
  what each check looks at.
- `keel/README.md` — the tool: every command with its flags, language adapters,
  hooks, skills.
- `keel/QUALITY.md` — forty quality cuts. Walked once per wave, where the
  scenarios are written.

This block is generated; edits between the markers are overwritten on the next update.
{end}"""

AGENTS_BLOCK = """{start}
## Keel

Методика цього проєкту: два типи документів — хвиля і контракт — і перевірки,
що їх тримають. Хвилі лежать у `keel/waves/`, контракти в `keel/contracts/`.

{principles}

Дві команди:

- `python3 {tool} next` — що робити далі: трансформа, її файли й межі,
  сценарії, які вона наближає, тіла контрактів, на які спирається.
- `python3 {tool} check` — що не так зараз. Перед коммітом і перед PR.

Три довідники — відкривай, коли не ясно:

- `keel/METHODOLOGY.md` — методика: що йде в шапку хвилі, як влаштовані редакції,
  що саме перевіряє кожна перевірка.
- `keel/README.md` — інструмент: усі команди з прапорцями, адаптери мов,
  хуки, скіли.
- `keel/QUALITY.md` — сорок розрізів якості. Проходяться раз на хвилю, там,
  де пишуться сценарії.

Цей блок породжений; правки між маркерами затре наступне оновлення.
{end}"""

CI_TEMPLATE = """name: keel
# Generated by `keel init`. Edits are overwritten by the next update.
on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0   # the scope check compares the branch against main
{setup}      - run: python3 {tool} check --branch "${{{{ github.head_ref || github.ref_name }}}}"
"""


# Skills: one source, two renderings
#
# The judgement in Keel is planning; work and review only need pointing at the
# tool. Three texts written by hand would drift apart within a month — that
# already happened to the old method — so both agents are rendered from here.

# Both agents read SKILL.md with the same required fields, and both let the
# operator run one by typing /<name>. Cursor rules stay what they are meant for —
# short standing constraints — and Keel's texts are procedures, so: skills.
SKILL_DIRS = {"claude": ".claude/skills", "cursor": ".cursor/skills",
              "keel-agent": ".keel-agent/skills"}
DESCRIPTION_CAP = 1536      # Claude truncates the skill listing at this

PLAN_BODY = """\
## Start here

Planning and work are separate on purpose: this branch writes the wave and not a
line of code. A person reads the wave and lets it through, which is why it is its
own pull request.

Create the branch and the skeleton:

    python3 keel/keel.py new wave <slug>

It prints the file it made, and the number in that name is part of the wave's
identity. Branch after the file, not after the slug you typed:

    git checkout -b plan/0007-session-loop

Get this the wrong way round and nothing links the branch to the wave: the tool
looks the wave up by branch name, finds nothing, and the session hook will tell
you the wave does not exist while you are looking straight at it.

The slug may arrive as an argument to `/keel-plan`. If it did not, ask in one
sentence which wave we are writing rather than inventing it for the person.

**Header fields are English; the prose is the project's own language.** The
fields become code, test tags and file names, so they stay English everywhere.
The prose is read and approved by a person, so it follows whatever language the
existing waves in `keel/waves/` are written in. If there are none yet, ask which
language this project writes in — and write commit messages the same way.

## Order

**The Why section** — the heading `keel new wave` wrote, one or two sentences on
what is missing without this wave. Not a retelling of what you will do: the
reason it is worth doing.

**Scenarios** — what we promise. Each one becomes a test, so word them so that it
is visible when one fails: **Given** a state, **When** an action, **Then** an
observable consequence. Name them; do not number them. A scenario nobody can
check is a wish — either reword it or admit it is a boundary.

**Transforms** — what does the work. Each becomes exactly one commit. List the
files by name before the work starts. Globs do not exist here: under a glob an
agent creates ten files where one was meant, and nothing objects.

For the exact header shape — which fields exist, how `proves`, `implements` and
`contracts` are written, where a revision comes from — read `keel/METHODOLOGY.md`. The
skeleton shows the bones; the reference explains the fields.

## Ask, do not guess

A person will read and approve this wave, so a guess written into the plan costs
more than a question. When you reach a fork, ask through the **question tool** if
there is one — in Claude Code that is `AskUserQuestion` — and offer ready options
with the consequence of each. Prose with the options written out is not the same
thing: it hands the person nothing to pick, and it travels as ordinary text,
which is the first thing anything between you and them will shorten. Without
such a tool, ask in chat and stop until there is an answer.

Ask about:

- **the choice of approach**, when several exist and they lead to different work;
- **the edges of the wave** — does the neighbouring thing belong here or in the
  next wave;
- **a quality cut you were silent on** — do we write a scenario, or say "no" out
  loud;
- **a promise something makes to us** — a library, a service, a binary: does it
  earn a contract of its own;
- **contract names**, when the wave draws a new boundary between modules.

Write every question so it stands on its own, as if the person had just walked in
with no context.

Do not ask about anything readable: the shape of the code, whether a file exists,
anything `keel gaps` already answers. Questions are for judgement, not for facts.

An open question that blocks nothing does not stop the work: write everything
that does not depend on the answer, and raise the question where it belongs.

**A question refused, interrupted or left unanswered is not permission to
guess.** Where the answer decides what gets written, write everything that does
not depend on it, leave the rest unwritten, and do not commit the plan — say
which question is still open. A guess committed as a plan reads as a decision to
whoever opens the pull request, and the chat where you called it a guess is not
in the diff.

**And the same for an intent cut short by something that is not a person.** When
a guard, a sandbox or a permission refuses the command you were going to prove
something with, and you narrow what you prove rather than pressing on, the
narrowing belongs in the document and the commit, not only in the chat. From
outside, an agent that honestly narrowed and an agent that quietly stopped
checking are the same picture, and only one of them is fine.

## Quality cuts

Before treating the list of scenarios as complete, walk `keel/QUALITY.md` — forty
questions under nine headings. One pass per wave, here, while the scenarios are
being written.

Every cut gets exactly one of three answers:

- **does not apply** — with the reason. A cut about the person at the interface
  does not apply to a build file;
- **answered** — name the scenario. A scenario that proves something narrower
  than the cut asks is not an answer; it is the next case;
- **silent** — the cut is relevant, nothing closes it. Say what specifically can
  go wrong here, and write the scenario that closes it.

A cut that is relevant and deliberately answered "no" is a decision, and it gets
said out loud. Silence is what this list ends; refusal is not.

One pass, and that is not a promise nothing surfaces afterwards. A cut noticed
once the plan is already committed is not a failed pass: put it where it belongs
— a scenario if a test can prove it, a boundary if the answer is "no" — and say
in the commit that it arrived late. Walking the list a second time to be sure
costs more than the two lines that admission takes.

Check what arrived with the library before claiming something is missing.

## Where the line runs

**Scenario or boundary.** If a test can prove it, it is a scenario. If it is "we
deliberately do not do this", it is the boundaries paragraph inside the
transform — the one the skeleton opens for you. A boundary without a scenario is
honest; a scenario without a test is not.

**A transform is not yet atomic** if you cannot name its files in advance. That
is not a reason to write a glob, it is a reason to cut further. The other tell:
you want to write the commit message with an "and" in it.

**A contract appears** when a promise outlives the wave that created it. Ours:
module, exported functions, meaning. Somebody else's — a library, a service, a
binary — is the same thing, and it carries `verify`, a command whose success is
the proof. A promise nothing can check is not a contract; it is a boundary.

**There are no decision files.** What outlives a wave and promises something is a
contract; "we deliberately do not do this" is the boundaries paragraph; a rule
about architecture belongs to the linter's config.

## Before handing the plan over

Run this until it comes back clean:

    python3 keel/keel.py gaps

It reports what is missing mechanically: slugs without sections, transforms
without files, scenarios without `proves`. If you lean on a contract, a fresh
revision comes from `python3 keel/keel.py rev --write`.

**A wave this branch did not come to write is not yours.** An unfinished
skeleton somebody left behind holds `check` red and can wall off your own
commit, and moving, renaming or deleting it is not the way past: leave it where
it is and tell the operator it is there. The same goes for every other file the
branch did not come to touch.

Then commit on the `plan/<wave>` branch and open the PR. **No code goes in this
PR** — a plan branch touches Keel's own files and nothing else: `keel/`, the
skills, `AGENTS.md`, the CI file and the hook configs. Anything the project
itself owns is code, and the scope check refuses it here.

**Whoever decided what goes in the commit message, in its own paragraph.** The
documents say what the wave promises; they do not say which fork you stood at,
what the choices were, and who picked. Git records who and when, the diff
records what — why and on whose call is recorded nowhere, and the chat where it
was settled does not travel with the repository. So write it down: the question,
the options, the answer, and whether the answer came from the operator or from
you. Two or three lines. A guess and a decision look identical six months later,
and the difference is the whole of it.

**And what you refused out loud, if you then did it, with what changed your
mind.** Those four lines only ever look forward, and a position you abandoned is
the purest case of what lives in the chat alone: it never became a file, so the
diff cannot carry it. The line runs at said out loud — told to the operator, or
written into a document. A thought nobody heard is thinking, not a reversal, and
needs no trace; list everything you ever weighed and this paragraph becomes a
diary, which is the one thing nobody reads.

Approval is written nowhere: it is the fact that the wave file reached the main
branch. Until then `keel next` hands out no work, and that is deliberate.
"""

WORK_BODY = """\
## The move

One invocation, one transform. Do not take two: a commit has to answer to a
single transform, otherwise the slug in its message stops meaning anything.

    python3 keel/keel.py next

This hands over one transform — what it does, which files, where its boundaries
run, which scenarios it brings closer, and the bodies of the contracts it leans
on. It is the whole slice for one move; nothing around it needs opening.

If `next` refuses, it says why: the branch is not named after a wave, or the plan
has not reached the main branch yet. Neither is a reason to start by hand — both
are a reason to go back to `/keel-plan`.

Do exactly that work, then run

    python3 keel/keel.py check

and commit with the **transform slug as the first word** of the message:

    drive-turns-on-reqllm: keep turning while the model calls tools

The slug is English because it comes from the wave header; the rest of the
message follows the project's prose language, like the wave files do. The slug is
the only link between the work and the plan — without it the transform stays open
no matter what the code says.

Repeat until `next` reports nothing open. The wave is then ready for review —
that is `/keel-review`.

## Boundaries

The files on the list, and only those. If you need a file that is not there, add
it to the transform in the wave file. Drift is not forbidden — it is named, and
it shows up as a line in the diff. Leaving the list silently is what is
forbidden.

Know how far the guard actually reaches. The write hook refuses a write outside
the list when the write goes through an editing tool; a file written through the
shell never reaches it. `keel check` closes most of that gap at commit time,
because git shows a tracked file changed and an untracked one appearing. What
neither of them sees is an untracked file that **disappears**: git has no record
of what was never recorded. So on that last edge the rule holds because you keep
it, and for no other reason.

**A file in your way that is not yours stays where it is.** An abandoned
skeleton, somebody's scratch file, anything you did not write and the transform
does not name — it may well be the thing holding `check` red. Say so and stop.
Moving it aside turns the check green over a repository nobody agreed to change,
and from outside that is the same picture as the work having been done. Naming
it costs a sentence; the operator can move their own file in seconds.

Every scenario the transform brings closer needs a test carrying its name and its
revision in the tag. `next` prints the revision; `keel rev --write` records a new
one once the scenario text has changed — after you have reread it, which is the
entire point of the mechanism.
"""

REVIEW_BODY = """\
## What happens

    python3 keel/keel.py check

The full gate: references, cycles, revisions, scope, scenarios with green tests,
module exports — and the project's own CI command. Red gets fixed, not explained.

**The CI line is the project's, not Keel's.** When it is red, the project's own
build, linter or suite is failing and there is nothing here to argue with. When
the gate says no CI command is named, say so to the operator and ask what the
project's own run should be — do not invent one, and do not write `none` on
their behalf: that is their decision to make out loud, not yours to make for
them.

Then the part no check can see. Reread the wave and ask not "what else should be
true" but **what did we stay silent about**. Most often the silence is about
failure: what happens when a dependency does not answer, when there is more data
than anyone expected, when the thing is called twice. Whatever you find gets
closed before the PR, like anything else.

Compare the text of each scenario with what its test actually proves. A test that
goes green without checking the promise is worse than a missing one, because it
stays quiet.

Look at scope in both directions. The check reports not only "touched something
undeclared" but also "declared and never touched" — and the second usually means
the transform was described differently from how it was built, or that a piece of
the work was forgotten.

## After that

When it is clean, push and open the PR. The wave stands whole: every transform
closed by a commit, every scenario proved by a test, every check green. Nothing
else needs marking — the statuses are derived.
"""

SKILLS = (
    {
        "name": "keel-plan",
        "description": ("Write a Keel wave: why it exists, scenarios drawn through "
                        "the quality cuts, transforms with exact file lists. In a "
                        "project that has a keel/ directory, use this skill "
                        "whenever any new work begins — even when nobody says the "
                        "word wave and the request is {triggers}, or just a "
                        "description of what is missing. Use it as well when "
                        "keel/waves/*.md is being edited, when asked how to split "
                        "work into transforms or how a scenario differs from a "
                        "boundary, and when keel gaps reports an incomplete wave."),
        "triggers": {
            "uk": "«додай», «зроби це», «реалізуй», «давай спланую»",
            "en": "\"add\", \"build\", \"implement\", \"let's plan this\"",
        },
        # No paths. It scopes a skill to the files it is about, and the
        # skill that writes the FIRST wave would be scoped to wave files
        # that do not exist yet — verified live: with keel/waves/ empty,
        # /keel-plan was absent from the menu entirely, while its two
        # unscoped siblings were there. The field hides a skill, not just
        # its auto-loading, so nothing that bootstraps may carry it.
        "paths": None,
        "argument_hint": {"uk": "[слаг нової хвилі]",
                          "en": "[slug of the new wave]"},
        "body": PLAN_BODY,
    },
    {
        "name": "keel-work",
        "description": ("Do the next transform of a Keel wave: keel next, work "
                        "strictly inside the named files, keel check, commit "
                        "carrying the transform slug. In a project with a keel/ "
                        "directory, use this skill whenever the request is to write "
                        "code, continue the work, {triggers} — and whenever the "
                        "branch is named after a wave, even if Keel was never "
                        "mentioned. Use it before committing too: the message has "
                        "to carry the transform slug or the transform stays open."),
        "triggers": {
            "uk": "«зроби наступне», «що далі», «продовжуй»",
            "en": "\"what's next\", \"keep going\", \"carry on\"",
        },
        "paths": None,
        "argument_hint": None,
        "body": WORK_BODY,
    },
    {
        "name": "keel-review",
        "description": ("Check a Keel wave before the pull request: the full keel "
                        "check plus the question of what we stayed silent about. In "
                        "a project with a keel/ directory, use this skill when asked "
                        "to open a PR or merge a branch, and when the question is "
                        "{triggers} — and whenever every transform of the wave is "
                        "already closed by a commit. Use it before claiming the work "
                        "is finished."),
        "triggers": {
            "uk": "«готово?», «можна мержити», «все на місці?»",
            "en": "\"is this done\", \"ready to merge\", \"can we ship it\"",
        },
        "paths": None,
        "argument_hint": None,
        "body": REVIEW_BODY,
    },
)

GENERATED_NOTE = ("Generated by `keel skills` from the methodology. Editing this "
                  "file does not last — the next run overwrites it; edit the "
                  "source instead.")

SKILL_FILE = """---
name: {name}
description: {description}
{extra}---

# {name}

{note}

{body}"""


def yaml_string(text):
    """Quote a scalar. Descriptions carry colons, and bare colons break YAML.

    A newline is escaped rather than written through: a real line break inside
    quotes is a scalar our own reader would meet as two lines and never put back
    together. The reader undoes exactly these four.
    """
    for raw, escaped in (("\\", "\\\\"), ('"', '\\"'), ("\n", "\\n"),
                         ("\t", "\\t"), ("\r", "\\r")):
        text = text.replace(raw, escaped)
    return '"' + text + '"'


# ─────────────────────────────────────────────────────────────────────────────
# Agent hooks
#
# Event names match across agents; the replies do not. One command answers in
# whichever dialect the flag names, so the configs stay thin and cannot drift
# apart. Codex is left out until its apply_patch payload is worked out.
# ─────────────────────────────────────────────────────────────────────────────

# What marks a hook entry as ours in a shared config. The vendored path, not
# "keel.py hook": quoting the command put a " between the script and the
# subcommand, and a space-joined tag would no longer match — so is_ours would
# fail to recognise our own entries and merge would duplicate them every run.
HOOK_TAG = VENDORED

# Claude documents file_path for Write, Edit and NotebookEdit. Cursor documents
# it for beforeReadFile and afterFileEdit, and its Write tool takes the same key.
# The rest of the list is defence: a hook that cannot find the path must say so,
# never wave the write through in silence.
PATH_KEYS = ("file_path", "notebook_path", "filePath", "path", "target_file",
             "absolute_path")


def hook_command(event, agent):
    # Claude documents a variable for the project root; Cursor's own examples
    # use a relative path, so that is the best available there. The path is
    # quoted: ${CLAUDE_PROJECT_DIR} expands to a real directory, and a space in
    # it would otherwise split the command and run python3 on half a path — the
    # hook failing silently, which is the one thing it must not do.
    root = "${CLAUDE_PROJECT_DIR}/" if agent == "claude" else "./"
    return f'python3 "{root}{VENDORED}" hook {event} --agent {agent}'


def claude_hook_config(agent="claude"):
    return {
        "SessionStart": [{
            "matcher": "startup|resume|clear",
            "hooks": [{"type": "command",
                       "command": hook_command("session", agent),
                       "timeout": 30}],
        }],
        "PreToolUse": [{
            "matcher": "Write|Edit|NotebookEdit",
            "hooks": [{"type": "command",
                       "command": hook_command("write", agent),
                       "timeout": 10}],
        }],
    }


def cursor_hook_config():
    return {
        "version": 1,
        "hooks": {
            "sessionStart": [{"command": hook_command("session", "cursor")}],
            "preToolUse": [{"command": hook_command("write", "cursor")}],
        },
    }


def find_path(payload):
    """Dig the target path out of a hook payload. None when nothing looks like one."""
    def walk(node, depth=0):
        if depth > 5:
            return None
        # Cursor hands tool_input over as a JSON string for some tools.
        if isinstance(node, str):
            stripped = node.strip()
            if stripped.startswith("{"):
                try:
                    return walk(json.loads(stripped), depth + 1)
                except ValueError:
                    return None
            return None
        if isinstance(node, list):
            for item in node:
                found = walk(item, depth + 1)
                if found:
                    return found
            return None
        if not isinstance(node, dict):
            return None
        for key in PATH_KEYS:
            value = node.get(key)
            if isinstance(value, str) and value.strip():
                return value.strip()
        for value in node.values():
            found = walk(value, depth + 1)
            if found:
                return found
        return None

    return walk(payload)


def take(project, name):
    """How to reach a procedure — which is not the same sentence in every mode.

    In manual mode the model is not allowed to load the skill at all, so telling
    it to would produce a blocked call and a confused agent. There the sentence
    addresses the person instead, through the agent.
    """
    if project.settings["mode"] == "manual":
        return t("Keel is in manual mode: ask the person to type /{name}.", name=name)
    return t("Take the {name} skill.", name=name)


def session_context(project):
    """What the agent needs at session start — and which skill answers it."""
    branch = project.branch or "?"
    if project.is_plan_branch(branch):
        wave = project.wave_for_branch(branch)
        where = (t("wave {slug}", slug=wave.slug) if wave
                 else t("there is no wave file for {branch} yet", branch=branch))
        return t("Keel: plan branch {branch}, {where}. The plan is written here, "
                 "not code.\n{take} What is missing is what "
                 "`python3 {tool} gaps` says.",
                 branch=branch, where=where, tool=VENDORED,
                 take=take(project, "keel-plan"))

    wave = project.wave_for_branch(branch)
    if wave is None:
        # On the main branch, the same answer `next` gives — it walks the graph
        # and names the wave waiting to be worked. The hook used to say "there
        # is no planned work here" while `next`, in the same repository at the
        # same moment, named an approved wave with every transform open. The
        # hook is what a fresh agent reads first, so the wrong answer was the
        # one that arrived first.
        if branch == project.git.main_short:
            return "Keel: " + main_branch_answer(project) + " " + take(
                project, "keel-plan")
        # Order matters and is easy to get backwards: the number only exists
        # after `new wave`, and a branch named before it never links to the wave.
        return t("Keel: branch {branch} is not named after a wave, so there is no "
                 "planned work here.\nA new wave: first `python3 {tool} new wave "
                 "<slug>` — it prints the file name with its number — and only then "
                 "the branch `plan/<that same name>`. {take}",
                 branch=branch, tool=VENDORED, take=take(project, "keel-plan"))
    if wave.error:
        return t("Keel: {file} does not parse: {reason}",
                 file=wave.rel, reason=wave.error)
    # The same gate `next` enforces: a wave that has not reached the main branch
    # is not approved, and there is no work. Without this the hook dictated
    # exactly the work the method's own gate refuses — the guard arguing with
    # the tool it was installed to enforce.
    if not project.git.file_in_branch(project.git.main_branch, wave.rel):
        return t("Keel: wave {wave} is not on {main} yet: the plan is not "
                 "approved and there is no work.", wave=wave.slug,
                 main=project.git.main_branch)

    slug, state = next_transform(project, wave)
    if slug is None:
        return t("Keel: every transform of wave {slug} is closed by a commit.\n"
                 "{take} Then `python3 {tool} check` and the PR.",
                 slug=wave.slug, tool=VENDORED, take=take(project, "keel-review"))

    package = next_package(project, wave, slug, state)
    return (t("Keel: {take} Here is the package for the next "
              "move — work from it, nothing around it needs opening.",
              take=take(project, "keel-work")) + "\n\n"
            + render_next(package))


def hook_reply(agent, event, kind, message):
    """The same verdict, in the dialect of one agent."""
    if agent in CLAUDE_CONTRACT:
        if event == "session":
            return {"hookSpecificOutput": {"hookEventName": "SessionStart",
                                           "additionalContext": message}}
        if kind == "deny":
            return {"hookSpecificOutput": {"hookEventName": "PreToolUse",
                                           "permissionDecision": "deny",
                                           "permissionDecisionReason": message}}
        return {"systemMessage": message}

    if event == "session":
        return {"additional_context": message}
    if kind == "deny":
        return {"permission": "deny",
                "agent_message": message,
                "user_message": message}
    # No permission field: the normal flow stays, the note is still seen.
    return {"agent_message": message}


def cmd_hook(project, args):
    payload = read_stdin_json()
    if args.event == "session":
        reply = hook_reply(args.agent, "session", "context", session_context(project))
    else:
        verdict = write_verdict(project, payload)
        if verdict is None:
            return 0
        reply = hook_reply(args.agent, "write", *verdict)
    print(json.dumps(reply, ensure_ascii=False))
    return 0


def read_stdin_json():
    try:
        raw = sys.stdin.read() if not sys.stdin.isatty() else ""
    except (OSError, ValueError):
        return {}
    try:
        return json.loads(raw) if raw.strip() else {}
    except ValueError:
        return {}


def repo_relative(project, target):
    """The path as the repository sees it, or None when it lies outside.

    realpath on both sides: on macOS /tmp is a symlink to /private/tmp, and the
    agent hands over the path the user sees. Comparing the two unresolved turns
    every write into "outside the repository" — that is, into silence.
    """
    absolute = target if os.path.isabs(target) else os.path.join(project.root, target)
    relative = os.path.relpath(os.path.realpath(absolute),
                               os.path.realpath(project.root))
    relative = relative.replace(os.sep, "/")
    if relative == ".." or relative.startswith("../"):
        return None
    return relative


def approved_files(project, wave):
    """The files the wave declared when it reached the main branch, or None.

    Read out of git rather than through a Doc: this is the text as it was
    approved, not as it sits on disk now, and there is no file to open.
    """
    if not project.git.available or not project.git.has_commits:
        return None
    # Against the branch point for the same reason as the drift note: the plan
    # this branch was approved under is the one it left main with.
    base = project.git.merge_base(project.git.main_branch)
    if not base:
        return None
    # `show` alone: it fails the same way `cat-file -e` would, so asking twice
    # bought a second process per write for an answer we already had. The hook
    # runs as its own process for every write, so this is the only saving
    # available — caching between writes is not.
    text = project.git.out("show", f"{base}:{project.git.in_tree(wave.rel)}",
                           default=None)
    if not text:
        return None
    front, _, _ = split_front_matter(text)
    if front is None:
        return None
    transforms = parse_yaml(front).get("transforms")
    if not isinstance(transforms, dict):
        return None
    files = set()
    for spec in transforms.values():
        if not isinstance(spec, dict):
            continue
        value = spec.get("files")
        if isinstance(value, list):
            files.update(str(item).strip() for item in value if str(item).strip())
        elif isinstance(value, str) and value.strip():
            files.add(value.strip())
    return files


def widened_here(project, wave, relative):
    """Whether this file entered the wave's scope after the plan was approved.

    Extending the list is allowed — §4.6 says so, and it stays a line in the
    diff. But the hook used to wave such a write through in silence: amend the
    wave, then write anything, and nothing said a word until `check` ran, which
    is three moves later and somewhere else. Said here instead, at the moment
    the judgement is actually made.
    """
    approved = approved_files(project, wave)
    return approved is not None and relative not in approved


def main_branch_verdict(project, payload):
    """The main branch takes finished work; it is not where work is written.

    Nothing guarded it. Check 4 compares a branch against main and so has
    nothing to compare while standing on main, and this hook used to return
    early — leaving the one branch with no planned work as the one branch where
    anything could be written. Found by walking the cycle, not by a test.

    Only once a wave exists: a repository still being set up, or one that has
    just taken Keel and planned nothing yet, has no plan to work from and no
    business being walled in.
    """
    if not project.waves:
        return None
    target = find_path(payload)
    if target is None:
        # The same unknown is loud on a wave branch and used to be silent here,
        # on the one branch where code is not supposed to be written at all.
        return ("note", t("keel: the hook payload carried no file path, so this "
                          "write was not judged. This is {main}, where finished "
                          "work arrives.", main=project.git.main_short))
    relative = repo_relative(project, target)
    if relative is None or keel_owns(relative, project.root):
        return None
    return ("deny", t("{name}: this is {main}, where finished work arrives — it "
                      "is not where work is written. Code belongs on a branch "
                      "named after a wave: check out the wave you are working "
                      "on, or plan a new one with keel new wave.",
                      name=relative, main=project.git.main_short))


def write_verdict(project, payload):
    """(kind, message), or None when there is nothing to say."""
    branch = project.branch
    if not branch or branch == "HEAD":
        # Every other unknown here is loud, and check 4 reports this same state
        # loudly too. A detached head — an interrupted rebase, a bisect, a
        # checkout by sha — used to turn the guard off without a word.
        return ("note", t("keel: the head is detached, so there is no wave to "
                          "judge this write against. Scope is not being checked."))
    if branch == project.git.main_short:
        return main_branch_verdict(project, payload)
    if project.is_plan_branch(branch):
        return None
    wave = project.wave_for_branch(branch)
    if wave is None:
        return None
    if wave.error:
        # Unreadable is not the same as unrestricted. Waving the write through
        # in silence is how a broken header turns the guard off without a word.
        return ("note", t("keel: {file} does not parse, so scope is not being "
                          "checked: {reason}", file=wave.rel, reason=wave.error))
    if not wave.transforms:
        return ("note", t("keel: wave {wave} declares no transforms, so nothing "
                          "says which files belong to this work.", wave=wave.slug))

    declared = wave.declared_files()

    target = find_path(payload)
    if target is None:
        return ("note", t("keel: the hook payload carried no file path, so scope "
                          "was not checked. Files the wave declares: {declared}",
                          declared=", ".join(sorted(declared)) or t("none")))

    relative = repo_relative(project, target)
    if relative is None:
        return ("note", t("keel: {target} is outside the repository, so the "
                          "wave's scope does not apply to it. Judge for yourself "
                          "whether it should be written to.", target=target))
    if keel_owns(relative, project.root):
        return None      # the same exemption check 4 applies, so the hook is
                         # never stricter than the gate
    if relative in declared:
        if widened_here(project, wave, relative):
            return ("note", t("{name} was added to wave {wave} on this branch, "
                              "not in the plan that was approved. Allowed — say "
                              "in the pull request what widened and why.",
                              name=relative, wave=wave.slug))
        return None

    return ("deny", t("{name} is not declared in wave {wave}. Declared: "
                      "{declared}. If this file is the one you need, add it to the "
                      "transform in {file}: drift is not forbidden, it has to stay "
                      "a line in the diff.",
                      name=relative, wave=wave.slug,
                      declared=", ".join(sorted(declared)) or t("none"),
                      file=wave.rel))


def remove_retired(root, wanted, manifest, done):
    """Take back a file the methodology has stopped generating.

    Renaming a reference used to leave the old copy in every project that had
    ever been updated — `keel/KEEL.md` beside `keel/METHODOLOGY.md`, both looking
    authoritative, one of them frozen at whatever the methodology said the day it
    was retired. A stale copy of the rules is worse than none: it answers.

    Same answer as everywhere else for a file somebody edited: name it and leave
    it. The manifest entry goes either way, so the next run has nothing to say
    about a file that is no longer ours.
    """
    for relative in sorted(set(manifest) - set(wanted)):
        path = os.path.join(root, relative)
        if os.path.exists(path):
            if digest(read_text(path)) != manifest[relative]:
                print("  " + t("{file}: no longer part of the methodology, and not "
                               "what Keel wrote — leaving it in place",
                               file=relative))
                del manifest[relative]
                continue
            os.remove(path)
            done.append(t("{file} removed — no longer part of the methodology",
                          file=relative))
        del manifest[relative]


def remove_hook_configs(root, done):
    """Take back the hooks a narrower mode no longer wants.

    Installing is not the whole job. Switching a project from strict to manual
    and leaving the guard running would print a line saying the guard is gone
    while it still refuses writes — a report the filesystem contradicts, which is
    the one thing this tool exists to stop.

    A file we did not write is not ours to delete: if `.cursor/hooks.json` no
    longer matches what we put there, it is named and left, the same answer
    `update` gives a hand-edited file.
    """
    strip_claude_settings(os.path.join(root, CLAUDE_SETTINGS), done)
    strip_claude_settings(os.path.join(root, KEEL_AGENT_SETTINGS), done,
                          KEEL_AGENT_SETTINGS)
    path = os.path.join(root, CURSOR_HOOKS)
    if not os.path.exists(path):
        return
    if digest(read_text(path)) != read_manifest(root).get(CURSOR_HOOKS):
        print("  " + t("{file}: not what Keel wrote, leaving it in place — the "
                       "hooks in it still run", file=CURSOR_HOOKS))
        return
    os.remove(path)
    done.append(t("{file} removed", file=CURSOR_HOOKS))


def edit_claude_settings(path, change, done, note, create=False, file=None):
    """Load, hand the file to `change`, write it back if anything moved.

    Both the adding and the removing pass go through here. Two copies of this
    were how the two ended up validating the file differently, and only one of
    them noticed that a hook event holding something other than a list is not
    ours to rewrite.
    """
    if not os.path.exists(path) and not create:
        return
    # `note` is the line that goes into the report and `file` is what to call
    # the file if it will not parse. For the removing pass the note is a whole
    # sentence, and a refusal naming ".claude/settings.json (our hook entries
    # taken out)" points at nothing anybody can open.
    data = load_json_object(path, file or note)
    if data is None:
        return

    before = json.dumps(data, ensure_ascii=False, sort_keys=True)
    change(data)
    if json.dumps(data, ensure_ascii=False, sort_keys=True) == before:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(json.dumps(data, ensure_ascii=False, indent=2) + "\n")
    done.append(note)


def ours_only(entries):
    """Our entries out of one hook event, or None when it is not a list of them.

    A value we do not recognise stays exactly as it is: this file belongs to the
    project, and rewriting a shape we did not expect would destroy somebody's
    configuration while reporting that we only took our own out.
    """
    if not isinstance(entries, list):
        return None
    return [item for item in entries if not is_ours(item)]


def strip_claude_settings(path, done, label=None):
    """Ours out of a file that is not ours: the rest of it stays untouched."""
    label = label or CLAUDE_SETTINGS
    def change(data):
        hooks = data.get("hooks")
        if not isinstance(hooks, dict):
            return
        touched = False
        for event in list(hooks):
            kept = ours_only(hooks[event])
            if kept is None or len(kept) == len(hooks[event]):
                # Nothing of ours in it — a foreign event, even an empty one,
                # is not ours to tidy away. Deleting it reported "our hook
                # entries taken out" over an edit to somebody else's shape.
                continue
            touched = True
            if kept:
                hooks[event] = kept
            else:
                del hooks[event]
        if touched and not hooks:
            del data["hooks"]

    edit_claude_settings(path, change, done,
                         t("{file} (our hook entries taken out)", file=label),
                         file=label)


def merge_claude_settings(path, done, label=None, agent="claude"):
    """Settings.json belongs to the project; we own only our own entries in it.

    Refusing to rewrite a shape that is not ours is right. Refusing in silence
    is not: with `"hooks": []` already in the file, strict mode installed
    Cursor's guard, listed it, exited zero — and Claude's write guard was simply
    absent, with nothing said. The operator believed strict mode was on while
    half of it was not.
    """
    label = label or CLAUDE_SETTINGS
    refused = []

    def change(data):
        hooks = data.setdefault("hooks", {})
        if not isinstance(hooks, dict):
            refused.append(t("the whole hooks key"))
            return
        for event, entries in claude_hook_config(agent).items():
            existing = ours_only(hooks.get(event, []))
            if existing is None:
                refused.append(event)   # somebody else's shape; adding would break it
                continue
            hooks[event] = existing + entries

    edit_claude_settings(path, change, done, label, create=True)
    if refused:
        done.append(t("{file}: {what} is somebody else's shape, so the write "
                      "guard was not installed there — put it in by hand or "
                      "move what is in the way",
                      file=label, what=", ".join(refused)))


def merge_agent_settings(root, settings, done):
    """The files Keel owns entries inside rather than whole, for whoever is asked for.

    Cursor's file is generated and travels with the rest; these two are merged,
    because they belong to the project and hold more than our hooks.

    Adding for whoever is asked for is half the job. An agent dropped from the
    list keeps its entries otherwise, and they keep firing — the same failure the
    mode already had, one level down.
    """
    for agent, relative in (("claude", CLAUDE_SETTINGS),
                            ("keel-agent", KEEL_AGENT_SETTINGS)):
        path = os.path.join(root, relative)
        if agent in settings["agents"]:
            merge_claude_settings(path, done, relative, agent)
        else:
            strip_claude_settings(path, done, relative)


def is_ours(entry):
    if not isinstance(entry, dict):
        return False
    return any(HOOK_TAG in str(item.get("command", ""))
               for item in entry.get("hooks", []) if isinstance(item, dict))


def agents_block(lang, principles):
    """The block follows the reference language: a Ukrainian frame around
    English principles reads as a bug, because it is one."""
    template = AGENTS_BLOCK if lang == SOURCE_LANG else AGENTS_BLOCK_EN
    return template.format(start=AGENTS_START, end=AGENTS_END, tool=VENDORED,
                           principles="\n".join(principles))


def home():
    return os.path.dirname(os.path.abspath(__file__))


def render_skill(skill, agent, lang=DEFAULTS["lang"], mode=DEFAULTS["mode"]):
    """One skill. Both agents take the same fields; only extras differ.

    In manual mode one line in the header turns a skill into a plain command:
    both Claude Code and Cursor read `disable-model-invocation`, and both stop
    offering the procedure to the model — the description leaves the agent's
    context entirely, and only `/keel-plan` typed by a person brings it back.
    """
    extra = ""
    if mode == "manual":
        extra += "disable-model-invocation: true\n"
    if skill.get("paths"):
        extra += "paths:\n" + "".join(
            f"  - {yaml_string(item)}\n" for item in skill["paths"])
    if agent in CLAUDE_CONTRACT and skill.get("argument_hint"):
        # Keyed by lang, like the triggers: the hint is shown to the operator,
        # and an English project was getting a Ukrainian one.
        extra += f"argument-hint: {yaml_string(skill['argument_hint'][lang])}\n"
    return SKILL_FILE.format(
        name=skill["name"],
        description=yaml_string(skill_description(skill, lang)),
        extra=extra,
        note=GENERATED_NOTE,
        body=skill["body"].strip())


def skill_description(skill, lang=DEFAULTS["lang"]):
    """The description in English, with example phrases in the project's language.

    Triggering is decided from this text, so the phrases have to be the ones the
    operator actually types — that is what `lang` is for.
    """
    return " ".join(
        skill["description"].format(triggers=skill["triggers"][lang]).split())


def skill_targets(skill, agents=None):
    """Where this skill lands, per agent. Each accepts /<name> from the operator."""
    chosen = DEFAULT_AGENTS if agents is None else agents
    return tuple((agent, f"{SKILL_DIRS[agent]}/{skill['name']}/SKILL.md")
                 for agent in sorted(chosen) if agent in SKILL_DIRS)


def cmd_skills(project, args=None):
    refuse_broken_config(project.root)
    done = []
    # With the manifest: regenerated skills that keep their old digests read as
    # "edited by hand" to the next update, which then refuses to refresh them —
    # keel wedged on its own output.
    manifest = read_manifest(project.root)
    write_skills(project.root, project.settings["lang"], done,
                 mode=project.settings["mode"], manifest=manifest)
    write_config(project.root, project.settings, done, manifest)
    for line in done:
        print(f"  {line}")
    if not done:
        print("  " + t("the skills did not change"))
    return 0


def write_skills(root, lang, done, manifest=None, mode=DEFAULTS["mode"]):
    for skill in SKILLS:
        for agent, relative in skill_targets(skill):
            write_if_changed(os.path.join(root, relative),
                             render_skill(skill, agent, lang, mode), done,
                             relative, manifest)


def principles_lines(lang=SOURCE_LANG):
    """The seven statements from PRINCIPLES.md — headings, without bodies."""
    path = doc_source("PRINCIPLES.md", lang)
    if not os.path.exists(path):
        return None
    found = re.findall(r"^##\s+(\d+)\.\s+(.+?)\s*$", read_text(path), re.M)
    return [f"{number}. {title}." for number, title in found] or None


def cmd_init(project, args):
    refuse_broken_config(project.root)
    # Keel reads every bit of its state from git: closure, scope, branch
    # comparison, the approval of a plan. Without a repository almost nothing
    # works, and creating one is a bigger decision than installing a method.
    if not project.git.available:
        fail(t("{root} is not a git repository, and Keel reads all of its state from\n"
               "git — transform closure, scope, the approval of a plan.\n"
               "First:\n  git init", root=project.root))
    settings = dict(project.settings)
    for key in DEFAULTS:
        chosen = getattr(args, key, None)
        if chosen:
            settings[key] = chosen
    # Separately, because the loop keeps only truthy values and this override
    # is meaningful when it is False.
    if getattr(args, "agent_hooks", None) is not None:
        settings["agent_hooks"] = args.agent_hooks
    # Same reason, and one more: the empty list is a real answer — equip nobody —
    # and the loop above would read it as "nothing was asked for".
    if getattr(args, "agents", None) is not None:
        settings["agents"] = args.agents

    # A proposal, only where the language has a convention for one, and only
    # into an empty setting: whoever already named a command keeps it, and a
    # project that said `none` is not talked out of its decision.
    if settings.get("ci", CI_UNDECIDED) == CI_UNDECIDED:
        adapter = detect_adapter(project.root, settings["adapter"])
        if adapter and adapter.ci_command:
            settings["ci"] = adapter.ci_command

    # The command that establishes lang speaks it: OUTPUT_LANG was set from the
    # config as it stood before init, so `init --lang uk` reported in English.
    global OUTPUT_LANG
    OUTPUT_LANG = settings["lang"]

    principles = principles_lines(settings["docs"])
    sources = {name: doc_source(name, settings["docs"]) for name in REFERENCES}
    missing = [name for name, path in sources.items() if not os.path.exists(path)]
    if principles is None or missing:
        fail(t("there are no references in {lang}: {missing}. Run init from the "
               "methodology repository.", lang=settings["docs"],
               missing=", ".join(missing + ([] if principles else ["PRINCIPLES.md"]))))

    done, manifest = [], {}
    for folder in INIT_DIRS:
        os.makedirs(os.path.join(project.root, folder), exist_ok=True)
    done.append(", ".join(INIT_DIRS))
    project.settings = settings

    # One owner of "which files, with what content": the same table update and
    # survey read. Init rendering its own copies by hand is how the CI block
    # once honoured --adapter here and not there.
    wanted = generated_files(project.root, settings)
    for relative, text in wanted.items():
        target = os.path.join(project.root, relative)
        if relative == VENDORED and os.path.abspath(target) == os.path.abspath(__file__):
            continue      # init run from the vendored copy itself
        write_if_changed(target, text, done, relative, manifest)

    # The same answer update gives, and reachable here since `--agents` can
    # narrow: init run twice used to leave the dropped agent's skills and hook
    # file on disk, working, while keel.json said that agent was not equipped.
    # Read before write_config, which replaces the record below.
    remove_retired(project.root, wanted, read_manifest(project.root), done)

    # The two shared files generated_files cannot express: Keel owns entries
    # inside them, not the files.
    if agent_hooks_wanted(settings):
        merge_agent_settings(project.root, settings, done)
    else:
        remove_hook_configs(project.root, done)
        done.append(t("no agent hooks: mode is {mode}", mode=settings["mode"]))

    block = agents_block(settings["docs"], principles)
    if update_agents(os.path.join(project.root, "AGENTS.md"), block):
        done.append("AGENTS.md " + t("(block between the markers)"))

    write_config(project.root, settings, done, manifest)

    for line in done:
        print(f"  {line}")
    code = cmd_hooks(project, args)
    if not getattr(args, "no_commit", False):
        committed = commit_own(project)
        if committed:
            print("  " + t("committed separately: {count} files", count=committed))
    for line in closing_hint(project, restart=True):
        print(line)
    return code


def digest(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()[:12]


COMMIT_MESSAGE = {"uk": "Keel у проєкті", "en": "Keel in the project"}


def pending_keel_paths(project):
    """Keel-owned paths with uncommitted changes — one definition for the
    commit that stages them and the hint that names them."""
    code, stdout, _ = project.git.run("status", "--porcelain", "--untracked-files=all")
    if code != 0:
        return None
    return sorted({row[3:] for row in stdout.splitlines() if keel_owns(row[3:], project.root)})


def commit_own(project):
    """Commit what init just wrote — and nothing else.

    Staging only Keel's own paths is what makes this safe: whatever the person
    had uncommitted next to it stays theirs, and the commit stays a separate,
    revertable thing. Coming into somebody's existing repository, that is the
    most a tool should take.
    """
    mine = pending_keel_paths(project)
    if not mine:
        return None
    if project.git.run("add", "--", *mine)[0] != 0:
        return None
    message = COMMIT_MESSAGE.get(project.settings["lang"], COMMIT_MESSAGE["en"])
    # --only: a bare commit takes the whole index, and anything the person had
    # staged would be swallowed into Keel's commit — against the one promise
    # this function makes.
    if project.git.run("commit", "--no-verify", "--only", "-m", message,
                       "--", *mine)[0] != 0:
        return None      # no identity configured, mid-merge — the hint still stands
    return len(mine)


def closing_hint(project, restart=False):
    """What to do with what we just wrote.

    Not a check and not a wall: an uncommitted setup does no harm by itself, and
    stopping the run over it would be the same mistake as asking. But it bites
    later — at the first plan commit, or at a session that never sees the skills —
    so it gets said once, here, with the command attached.
    """
    lines = []
    if project.git.available:
        pending = pending_keel_paths(project) or []
        if pending:
            # "with uncommitted changes", not "not in git yet": a tracked file
            # with a pending edit was described as absent from git entirely.
            lines.append("\n" + t("Keel files with uncommitted changes: {count}. "
                                  "Commit them separately from the work:\n  git add {paths}"
                                  "\n  git commit -m \"Keel in the project\"",
                                  count=len(pending),
                                  paths=" ".join(sorted(
                                      {name.split("/")[0] for name in pending}))))
    if restart:
        lines.append("\n" + t(
            "The skills /keel-plan, /keel-work and /keel-review are in place. Start "
            "the agent in the project directory itself:\n  cd {root} && <agent>\n"
            "If it answers \"Unknown skill\" they have not been picked up yet: "
            "/reload-skills, or simply call again. A session opened before the "
            "install has to be restarted; /clear does not register the directory.",
            root=project.root))
    return lines


def write_if_changed(path, text, done, label, manifest=None):
    """Write, and remember what we wrote.

    The manifest is what lets `update` tell "the methodology moved on" apart from
    "a person edited this by hand". Without it the two look identical, and the
    only safe move would be to never overwrite anything.
    """
    if manifest is not None:
        manifest[label] = digest(text)
    if os.path.exists(path) and read_text(path) == text:
        return False
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    done.append(label)
    return True


def update_agents(path, block):
    """Add the block to AGENTS.md without touching the rest of the file.

    Markers out of balance — an end lost in a merge, a start pasted twice —
    make the block's edges unknowable. Appending anyway compounded it: the
    first run added a second block, the second run swallowed everything a
    person had written between the stray markers. Named and left instead.
    """
    old = read_text(path) if os.path.exists(path) else ""
    starts, ends = old.count(AGENTS_START), old.count(AGENTS_END)
    if starts != ends or starts > 1 or (
            ends == 1 and old.find(AGENTS_END) < old.find(AGENTS_START)):
        print("  " + t("{file}: the keel markers are out of balance — fix them "
                       "by hand, this block was not touched", file="AGENTS.md"))
        return False
    if AGENTS_START in old and AGENTS_END in old:
        head, _, rest = old.partition(AGENTS_START)
        _, _, tail = rest.partition(AGENTS_END)
        new = head + block + tail
    elif old.strip():
        new = old.rstrip("\n") + "\n\n" + block + "\n"
    else:
        new = block + "\n"
    if new == old:
        return False
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(new)
    return True


# Fast on commit, slow on push: the agent commits often and must not wait
# minutes, and red will not reach the main branch either way.
HOOKS = {
    "pre-commit": ("check", "--fast"),
    "pre-push": ("check",),
}

HOOK_MARK = "# keel hook:"

HOOK_SCRIPT = """#!/bin/sh
{mark} {name}. Generated by `keel hooks --install`.
# Edits to this file are overwritten by the next install.
set -eu

run() {{
  case "$1" in
    *.py) exec python3 "$1" {args} ;;
    *)    exec "$1" {args} ;;
  esac
}}

# The tool is looked for in this order: the KEEL variable, PATH, the copy in the
# project, and only then the path this machine had when the hook was installed.
if [ -n "${{KEEL:-}}" ] && [ -f "${{KEEL}}" ]; then run "${{KEEL}}"; fi

tool=$(command -v keel 2>/dev/null || true)
if [ -n "$tool" ]; then run "$tool"; fi

root=$(git rev-parse --show-toplevel)
if [ -f "$root/keel/keel.py" ]; then run "$root/keel/keel.py"; fi

if [ -f "{baked}" ]; then run "{baked}"; fi

echo "keel: no tool found. Set KEEL=/path/to/keel.py" >&2
exit 1
"""


def hook_script(name, baked):
    return HOOK_SCRIPT.format(
        mark=HOOK_MARK, name=name, baked=baked, args=" ".join(HOOKS[name]))


def cmd_hooks(project, args):
    # --git-path hooks, not <git-dir>/hooks: git honours core.hooksPath, and a
    # linked worktree's own git dir is not where hooks are read from. Writing to
    # the wrong folder and reporting success is the silent green this whole tool
    # exists against — the commit then passes with no guard at all.
    folder = project.git.out("rev-parse", "--path-format=absolute",
                             "--git-path", "hooks")
    if not folder:
        folder = project.git.out("rev-parse", "--git-path", "hooks")
        if folder and not os.path.isabs(folder):
            folder = os.path.join(project.root, folder)
    if not folder:
        fail(t("this is not a git repository — there is nowhere to put the hooks"))
    baked = os.path.abspath(__file__)

    problems, missing = 0, 0
    for name in HOOKS:
        path = os.path.join(folder, name)
        present = os.path.exists(path)
        mine = present and HOOK_MARK in read_text(path)

        if not args.install:
            state = (t("ours") if mine
                     else (t("another tool's") if present else t("missing")))
            missing += 0 if mine else 1
            print(f"  {name}: {state}")
            continue
        if present and not mine and not args.force:
            print("  " + t("{name}: another tool owns this hook, leaving it alone "
                     "(--force to overwrite)", name=name))
            problems += 1
            continue
        os.makedirs(folder, exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(hook_script(name, baked))
        os.chmod(path, 0o755)
        print(f"  {name}: {' '.join(('keel',) + HOOKS[name])}")

    if not args.install:
        print("\n" + (t("keel hooks --install puts them in place") if missing
                  else t("both are in place")))
        return 0
    return 1 if problems else 0


def cmd_show(project, args):
    """A wave as a person reads it: links that resolve, and derived state.

    The header is YAML because a machine reads it, and a preview renders that
    badly. Rather than split the file — which would let the slug and its text
    drift apart in two places — this view is built on the fly and stored nowhere.
    """
    wave = project.waves.get(args.wave) if args.wave else project.wave_for_branch()
    if wave is None:
        fail(t("no such wave: {wave}",
               wave=args.wave or t("branch {branch}", branch=project.branch)))
    if wave.error:
        fail(f"{wave.rel}: {wave.error}")

    state = project.transform_state(wave) if project.git.available else {}
    # Every link is relative to the wave file, so they resolve wherever the
    # rendered text is read from — including the file's own directory.
    out = [f"# {wave.slug}", "",
           f"[{wave.rel}]({os.path.basename(wave.path)})", ""]
    if wave.why.strip():
        out += [wave.why.strip(), ""]

    depends = [f"[{ref.slug}](../waves/{ref.slug}.md)" for ref in wave.depends_on]
    if depends:
        out += ["**" + t("Depends on:") + "** " + ", ".join(depends), ""]

    out += ["## " + t("Scenarios"), ""]
    for slug in wave.scenarios:
        proves = []
        for ref in wave.proves(slug):
            contract = project.contracts.get(ref.slug)
            ok = contract and not contract.error and contract.rev_ok(ref.rev)
            proves.append(f"[{ref.slug}](../contracts/{ref.slug}.md)"
                          f"@{ref.rev or '—'} {'✓' if ok else '✗'}")
        out.append(f"### {slug}")
        out.append("")
        out.append(t("Proves: {proves} · revision `{rev}`",
                     proves=", ".join(proves) or "—",
                     rev=wave.scenario_revision(slug) or "—"))
        out.append("")
        out.append((wave.scenario_body(slug) or "_" + t("no body") + "_").strip())
        out.append("")

    out += ["## " + t("Transforms"), ""]
    for slug in wave.transforms:
        sha = state.get(slug, (None, set()))[0]
        out.append(f"### {slug} — " + (t("closed {sha}", sha=sha[:7]) if sha
                                       else t("open")))
        out.append("")
        near = ", ".join(f"[{name}](#{name})"
                         for name in wave.transform_implements(slug))
        out.append(t("Brings closer: {names}", names=near or "—"))
        for ref in wave.transform_contracts(slug):
            out.append(t("Implements: [{slug}](../contracts/{slug}.md)@{rev}",
                         slug=ref.slug, rev=ref.rev or "—"))
        out.append("")
        for name in wave.transform_files(slug):
            here = os.path.exists(os.path.join(project.root, name))
            out.append(f"- [{name}](../../{name})"
                       + ("" if here else " — " + t("not there yet")))
        out.append("")
        out.append((wave.transform_body(slug) or "_" + t("no body") + "_").strip())
        out.append("")

    print("\n".join(out).rstrip() + "\n")
    return 0


def cmd_update(project, args):
    """Bring the project's copies up to the methodology, without clobbering work.

    The open question was what to do with a generated file somebody edited by
    hand: asking stops an autonomous run, and not asking destroys the edit. The
    answer here is neither — refuse that one file, say so, and carry on with the
    rest. Nothing is lost and nothing waits for a human.
    """
    # Run from the copy inside a project, every source would be compared against
    # itself and the answer would always be "nothing to do" — a no-op wearing the
    # face of a clean result.
    refuse_broken_config(project.root)
    if not os.path.exists(os.path.join(home(), "PRINCIPLES.md")):
        fail(t("update compares the project against the methodology home, and there "
               "are no sources beside this copy. Run it from the keel repository:\n"
               "  python3 <keel>/keel.py -C {root} update", root=project.root))

    problems = check_translations(project)
    if problems:
        print(t("translations have fallen behind the source:"))
        for problem in problems:
            print(problem.render())
        print()

    # Rendered once: survey compares against it and the writes below reuse it,
    # instead of reading every source and probing the adapter twice.
    wanted = generated_files(project.root, project.settings)
    manifest = read_manifest(project.root)
    fresh, stale, touched, absent = survey(project, wanted, manifest)

    if args.diff:
        for relative in stale + touched:
            print_diff(project.root, relative, wanted[relative])
        if not stale and not touched:
            print(t("no difference"))
        return 0

    done = []
    for relative in fresh:
        manifest[relative] = digest(wanted[relative])   # heal a lost record
    for relative in absent + stale + (touched if args.force else []):
        write_if_changed(os.path.join(project.root, relative), wanted[relative],
                         done, relative, manifest)
    # AGENTS.md and settings.json are shared: they get merged, never replaced.
    principles = principles_lines(project.settings["docs"])
    if principles:
        block = agents_block(project.settings["docs"], principles)
        if update_agents(os.path.join(project.root, "AGENTS.md"), block):
            done.append("AGENTS.md " + t("(block between the markers)"))
    if agent_hooks_wanted(project.settings):
        merge_agent_settings(project.root, project.settings, done)
    else:
        # Refusing to add them back is half the job: a mode narrowed by hand in
        # keel.json would otherwise leave the old entries firing forever, and
        # generated_files no longer lists the cursor file, so survey never
        # mentions it either.
        remove_hook_configs(project.root, done)
    remove_retired(project.root, wanted, manifest, done)
    write_config(project.root, project.settings, done, manifest)

    for line in done:
        print("  " + t("updated: {what}", what=line))
    for relative in touched:
        if not args.force:
            print("  " + t("edited by hand, leaving it alone: {what}", what=relative))
    if not done and not touched:
        print(t("everything is in place"))
    for line in closing_hint(project):
        print(line)
    if touched and not args.force:
        print("\n" + t("keel update --diff shows the difference, --force overwrites"))
        return 1
    return 0 if not problems else 1


def print_diff(root, relative, wanted):
    import difflib
    now = read_text(os.path.join(root, relative)).splitlines(keepends=True)
    for line in difflib.unified_diff(now, wanted.splitlines(keepends=True),
                                     fromfile=relative,
                                     tofile=relative + " " + t("(new)")):
        print(line, end="" if line.endswith("\n") else "\n")


def read_text(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    except OSError:
        return ""


def cmd_rev(project, args):
    """Show revisions that have drifted apart; --write records the new ones."""
    edits = {}   # path -> [(old, new)]
    report = []

    for wave, who, ref, contract in drifted_contract_refs(project):
        fresh = contract.revision
        report.append((wave.rel, who, f"{ref.slug}@{ref.rev or '—'}",
                       f"{ref.slug}@{fresh}"))
        edits.setdefault(wave.path, []).append((ref.raw, f"{ref.slug}@{fresh}"))

    for _, slug, body, path, line, rev in drifted_tags(project):
        fresh = revision(body)
        report.append((f"{path}:{line}", t("test {slug}", slug=slug),
                       rev or "—", fresh))
        edits.setdefault(os.path.join(project.root, path), []).append(
            (("TAG", slug), fresh))

    if not report:
        print(t("every revision matches"))
        return 0

    for where, who, was, now in report:
        print(f"  {where}  {who}: {was} → {now}")

    if not args.write:
        print("\n" + t("drifted apart: {count}. keel rev --write records the new ones, "
                       "once you have reread the text you lean on.",
                       count=len(report)))
        return 1

    for path, changes in edits.items():
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        for old, new in changes:
            if isinstance(old, tuple):
                text, _ = rewrite_tag(text, old[1], new,
                                      project.adapter.name
                                      if project.adapter else "elixir")
            else:
                text = rewrite_ref(text, old, new)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    # Measured, not assumed: re-read the project and count what still drifts.
    # A claim of success over a write that did not happen — a capitalised tag
    # the rewrite could not see — is the one thing this tool must never print.
    after = Project(project.root)
    remaining = (sum(1 for _ in drifted_contract_refs(after))
                 + sum(1 for _ in drifted_tags(after)))
    if remaining:
        print("\n" + t("recorded {written} of {count} — the rest were reported "
                       "and not found where the rewrite looked",
                       written=len(report) - remaining, count=len(report)))
        return 1
    print("\n" + t("recorded: {count}", count=len(report)))
    return 0


REF_VALUE = re.compile(r"\b(?:proves|contracts)\s*:\s*(\[[^\]]*\]|[^\n,}]*)")


def rewrite_ref(text, raw, new):
    """Restamp one contract reference — only where a reference can appear.

    A contract reference is the value of `proves:` or `contracts:`, inline or as
    a block list under one of them. It is never a mapping key and never an
    `implements` item, so restamping the bare slug across the whole header would
    rename a scenario or transform that happens to share the contract's name —
    and a second run could not undo it. The body, the file's final newline
    included, is left byte for byte.
    """
    match = re.match(r"(---[ \t]*\n)(.*?\n)(---[ \t]*(?:\n|$))", text, re.S)
    if not match:
        return text
    slug = Ref(raw).slug
    token = re.compile(
        rf"(?<![\w@./-]){re.escape(slug)}(@[0-9a-fA-F]*)?(?![\w@./-])")

    def in_value(line_match):
        value = line_match.group(1)
        head = line_match.group(0)[:line_match.start(1) - line_match.start()]
        return head + token.sub(new, value)

    out, in_block, block_indent = [], False, 0
    for line in match.group(2).split("\n"):
        stripped = line.lstrip()
        indent = len(line) - len(stripped)
        if in_block and stripped.startswith("- ") and indent > block_indent:
            out.append(token.sub(new, line))
            continue
        in_block = False
        if re.match(r"(proves|contracts)\s*:\s*$", stripped):
            in_block, block_indent = True, indent
            out.append(line)
            continue
        out.append(REF_VALUE.sub(in_value, line))
    front = "\n".join(out)
    return text[:match.start(2)] + front + text[match.end(2):]


def rewrite_tag(text, slug, fresh, dialect):
    """Write a fresh revision into a test tag — the current adapter's form only.

    One dialect, because applying both rewrote what was never reported: a
    Python test holding an Elixir tag inside a string fixture had the fixture
    restamped. The slug is bounded on the right, as rewrite_ref's is, and
    matched case-blind, as the collector matches it — a capitalised tag was
    reported drifted, "recorded", and never actually written.

    Returns (text, how many tags changed), so the caller can tell a write that
    happened from one it merely claimed.
    """
    atom = slug.replace("-", "_")
    # Case-blind on the slug alone, spelled inline — never on the directive.
    # A whole-pattern re.I matched `# Proves: parse is central to this module`
    # in prose and `@TAG PROVES:` inside a string fixture, neither of which the
    # collectors see as tags: rewriting them is the same "changed what was
    # never reported" this function exists to have stopped doing.
    def blind(word):
        return "".join(f"[{ch.lower()}{ch.upper()}]" if ch.isalpha()
                       else re.escape(ch) for ch in word)

    if dialect == "elixir":
        pattern = re.compile(
            rf"@tag\s+proves:\s*:({blind(atom)})(?![\w?!])"
            rf"(?:\s*,\s*rev:\s*[\"'][^\"']*[\"'])?")
        return pattern.subn(
            lambda m: f'@tag proves: :{m.group(1)}, rev: "{fresh}"', text)
    pattern = re.compile(
        rf"#\s*proves:\s*({blind(slug)}|{blind(atom)})(?![\w-])"
        rf"(?:\s*,\s*rev:\s*[\"']?[^\"'\s,]*[\"']?)?")
    return pattern.subn(
        lambda m: f'# proves: {m.group(1)}, rev: "{fresh}"', text)


# ─────────────────────────────────────────────────────────────────────────────
# Entry point
# ─────────────────────────────────────────────────────────────────────────────

def fail(message, code=2):
    print(message, file=sys.stderr)
    raise SystemExit(code)


def agent_names(text):
    """`--agents claude,keel-agent` into the canonical list.

    Unknown names fail here rather than quietly installing for two of three:
    a typo that silently equips fewer agents is the kind of half-done the whole
    tool exists against.
    """
    names = [name.strip() for name in text.split(",") if name.strip()]
    unknown = [name for name in names if name not in AGENT_NAMES]
    if unknown:
        raise argparse.ArgumentTypeError(
            "%s — known: %s" % (", ".join(unknown), ", ".join(AGENT_NAMES)))
    return [name for name in AGENT_NAMES if name in names]


def build_parser():
    parser = argparse.ArgumentParser(
        prog="keel", description="Keel: two kinds of document, and the checks that hold them.")
    parser.add_argument("--version", action="version", version=VERSION)
    parser.add_argument("-C", dest="chdir", metavar="DIR",
                        help="work in this directory")
    sub = parser.add_subparsers(dest="command", required=True)

    new = sub.add_parser("new", help="skeleton of a wave or a contract")
    new.add_argument("kind", choices=("wave", "contract"))
    new.add_argument("slug")

    gaps = sub.add_parser("gaps", help="what is missing from a wave")
    gaps.add_argument("wave", nargs="?", help="a wave; without it, the branch's wave")

    check = sub.add_parser("check", help="every check")
    check.add_argument("--fast", action="store_true",
                       help="only the ones that run nothing, as on pre-commit")
    check.add_argument("--no-tests", action="store_true", help="do not run anything")
    check.add_argument("--branch", metavar="NAME",
                       help="the branch name where git does not know it, as on CI")
    check.add_argument("--json", action="store_true")

    nxt = sub.add_parser("next", help="the package for the next move")
    nxt.add_argument("--json", action="store_true")

    rev = sub.add_parser("rev", help="revisions that have drifted apart")
    rev.add_argument("--write", action="store_true", help="record the new revisions")

    hooks = sub.add_parser("hooks", help="the git hooks: pre-commit and pre-push")
    hooks.add_argument("--install", action="store_true")
    hooks.add_argument("--force", action="store_true",
                       help="overwrite a hook another tool owns")

    init = sub.add_parser("init", help="install Keel into a project")
    init.add_argument("--force", action="store_true",
                      help="overwrite a hook another tool owns")
    init.add_argument("--no-commit", action="store_true",
                      help="do not commit what was written")
    init.add_argument("--docs", choices=LANGS,
                      help="language of the references placed in the project")
    init.add_argument("--lang", choices=LANGS,
                      help="language the agent writes in, the skills catch, "
                           "and this tool speaks")
    init.add_argument("--agents", type=agent_names, metavar="LIST",
                      help="comma-separated, whom to equip with the skills and "
                           "hooks: " + ", ".join(AGENT_NAMES) + ". Empty means "
                           "nobody. Default: " + ", ".join(DEFAULT_AGENTS))
    init.add_argument("--adapter", choices=[name for name in ADAPTER_NAMES if name],
                      help="which language this project is, when the root says "
                           "more than one")
    init.add_argument("--ci", metavar="COMMAND",
                      help="the project's own gate — a build, a linter, its "
                           "suite. Any command; the condition is that it "
                           "succeeds. 'none' says there is deliberately no gate")
    init.add_argument("--mode", choices=MODES,
                      help="how much of itself Keel installs: strict (the agent "
                           "starts the procedures and the hooks watch the "
                           "boundaries), soft (the agent starts them, nothing "
                           "watches), manual (only you start them, by typing "
                           "/keel-plan)")
    init.add_argument("--agent-hooks", dest="agent_hooks", action="store_true",
                      default=None,
                      help="install the hooks that watch what the agent writes, "
                           "whatever the mode says")
    init.add_argument("--no-agent-hooks", dest="agent_hooks", action="store_false",
                      help="leave them out, whatever the mode says")
    init.set_defaults(install=True)

    sub.add_parser("skills", help="regenerate the skills from the methodology")

    show = sub.add_parser("show", help="a wave as a person reads it")
    show.add_argument("wave", nargs="?", help="a wave; without it, the branch's wave")

    update = sub.add_parser("update", help="update the copies in a project")
    update.add_argument("--diff", action="store_true", help="show the difference")
    update.add_argument("--force", action="store_true",
                        help="overwrite hand-edited files too")

    hook = sub.add_parser("hook",
                          help="answer an agent hook; called by a config")
    hook.add_argument("event", choices=("session", "write"))
    hook.add_argument("--agent", choices=AGENT_NAMES, required=True)

    return parser


def main(argv=None):
    args = build_parser().parse_args(argv)
    start = args.chdir or os.getcwd()
    if not os.path.isdir(start):
        fail(t("no such directory: {path}", path=start))
    root = find_root(start)
    # The tool speaks the project's language: the same `lang` that decides what
    # the agent writes and which phrases the skills catch, because one person
    # reads all three. Set before the documents are read, not after: a document
    # composes its own error message as it is parsed, and doing this afterwards
    # left half of a broken-file report in the other language.
    settings = read_config(root)
    global OUTPUT_LANG
    OUTPUT_LANG = settings["lang"]
    project = Project(root, settings)
    project.branch_override = getattr(args, "branch", None)

    if args.command == "hook" and not project.ready:
        # A leftover hook entry after keel/ was removed. Failing here exits 2,
        # and for a PreToolUse hook exit 2 means deny — every write in the
        # repository would be blocked by a tool that is not even installed.
        # Answer nothing and wave aside.
        return 0
    if args.command not in ("new", "init") and not project.ready:
        fail(t("{root} has no keel/ directory — Keel is not installed here", root=root))

    handlers = {"new": cmd_new, "gaps": cmd_gaps, "check": cmd_check,
                "next": cmd_next, "rev": cmd_rev, "hooks": cmd_hooks,
                "init": cmd_init, "skills": cmd_skills, "hook": cmd_hook,
                "update": cmd_update, "show": cmd_show}
    return handlers[args.command](project, args)


if __name__ == "__main__":
    sys.exit(main())

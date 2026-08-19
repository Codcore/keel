#!/usr/bin/env python3
"""keel — the tool behind the Keel method.

One file, standard library only. It knows state; it writes no prose.
Messages printed to the human stay in Ukrainian: they are prose, not code.

    keel new step <slug>       skeleton of a step
    keel new contract <slug>   skeleton of a contract
    keel gaps                  what the step description is missing
    keel next                  package for the next move
    keel check                 the six checks
    keel rev                   revisions that have drifted apart
    keel hooks                 git hooks: pre-commit and pre-push
    keel init                  put Keel into a project
"""

import argparse
import functools
import hashlib
import json
import os
import re
import subprocess
import sys

VERSION = "0.1.0"

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
    "list is not closed by a bracket": "список не закритий дужкою",
    "map is not closed by a bracket": "мапа не закрита дужкою",
    "no colon in the map entry: {entry}": "у мапі немає двокрапки: {entry}",
    "duplicate key {key}": "ключ {key} повторюється",
    "indented with a tab": "відступ табуляцією",
    "unexpected indent": "несподіваний відступ",
    "a header has to be a set of keys, not a list":
        "шапка має бути набором ключів, а не списком",
    "this line is not a list item": "рядок не є елементом списку",
    "a list where a key was expected": "список там, де очікується ключ",
    "no key before the colon: {line}": "немає ключа перед двокрапкою: {line}",
    "two colons on one line: {line}": "дві двокрапки в одному рядку: {line}",
    "empty key": "порожній ключ",
    "no header between --- markers": "немає шапки між рисками ---",
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
    "keel: step {step} declares no transforms, so nothing says which files "
    "belong to this work.":
        "keel: крок {step} не оголошує трансформ, тож ніщо не каже, які файли "
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
    "depends_on points at a step that does not exist: {slug}":
        "depends_on показує на крок, якого немає: {slug}",
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
    "branch {branch} is not named after a step — there is nothing to compare "
    "scope against":
        "гілка {branch} не називається кроком — немає з чим звіряти межі",
    "changed but not declared: {name}": "файл змінено, але не оголошено: {name}",
    "declared but not changed: {name}": "файл оголошено, але не змінено: {name}",
    "nothing to run the tests with: the root has none of {markers}":
        "не знайшов, чим запускати тести: у корені немає жодного з {markers}",
    "scenario {slug} has no test": "сценарій {slug} не має тесту",
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
    'Step {id} · {file}': 'Крок {id} · {file}',
    'Closed: {names}': 'Закрито: {names}',
    'After this one: {names}': 'Після цієї: {names}',
    'Why the step': 'Навіщо крок',
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
    'step {slug}': 'крок {slug}',
    'there is no step file for {branch} yet': 'файла кроку для {branch} ще немає',
    "(not run)": "(не запускалась)",
    "documents do not parse": "документи не читаються",
    "clean": "чисто",
    "problems: {count}": "проблем: {count}",
    "bad slug: {slug}": "поганий слаг: {slug}",
    "already there: {path}": "вже є: {path}",
    "no such step: {step}": "кроку немає: {step}",
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
    "the tests did not finish within {seconds}s ({command}). Nothing was proved, "
    "which is not the same as nothing being wrong.":
        "тести не завершились за {seconds}с ({command}). Нічого не доведено, а це "
        "не те саме, що «нічого не зламано».",
    "{command} did not answer within {seconds}s":
        "{command} не відповів за {seconds}с",
    "{command} was not found": "{command} не знайдено",
    "path/to/file": "шлях/до/файлу",
    "What exactly is promised, and to whom.": "Що саме обіцяно й кому.",
    "Why": "Навіщо",
    "why this step exists and what is missing without it":
        "навіщо цей крок і чого без нього бракує",
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
    "{file} (our hook entries taken out)": "{file} (наші записи хуків вилучено)",
    "Implements: [{slug}](../contracts/{slug}.md)@{rev}":
        "Виконує: [{slug}](../contracts/{slug}.md)@{rev}",
    "Proves: {proves} · revision `{rev}`":
        "Доводить: {proves} · ревізія `{rev}`",
    "Test tag: `proves: :{atom}, rev: \"{rev}\"`":
        "Тег тесту: `proves: :{atom}, rev: \"{rev}\"`",
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
    "branch {branch} is not named after a step. Work happens on a branch named "
    "after the step, planning on plan/<step>.":
        "гілка {branch} не названа за кроком. Робота йде на гілці, названій за "
        "кроком, планування — на plan/<крок>.",
    "every transform of step {step} is closed by a commit. Next: keel check, "
    "then the PR.":
        "кожен трансформ кроку {step} закрито комітом. Далі: keel check, потім PR.",
    "keel: the hook payload carried no file path, so scope was not checked. "
    "Files the step declares: {declared}":
        "keel: у виклику хука не було шляху до файла, тож обсяг не перевірено. "
        "Файли, які оголошує крок: {declared}",
    "keel: {target} is outside the repository, so the step's scope does not "
    "apply to it. Judge for yourself whether it should be written to.":
        "keel: {target} лежить поза репозиторієм, тож обсяг кроку на нього не "
        "поширюється. Чи варто туди писати — вирішуй сам.",
    "step {step} is not on {main} yet: the plan is not approved and there is no work.":
        "кроку {step} ще немає на {main}: план не затверджено, і роботи немає.",
    "this is a plan branch: the step is written here, not code. keel gaps says "
    "what is missing.":
        "це гілка плану: тут пишеться крок, а не код. Чого бракує — каже keel gaps.",
    "{count} Keel files are not in git yet. Commit them separately from the "
    "work:\n  git add {paths}\n  git commit -m \"Keel in the project\"":
        "{count} файлів Keel ще не в git. Закоміть їх окремо від роботи:\n"
        "  git add {paths}\n  git commit -m \"Keel у проєкті\"",
    "{name} is not declared in step {step}. Declared: {declared}. If this file "
    "is the one you need, add it to the transform in {file}: drift is not "
    "forbidden, it has to stay a line in the diff.":
        "{name} не оголошено в кроці {step}. Оголошено: {declared}. Якщо потрібен "
        "саме цей файл, додай його до трансформа в {file}: відхилення не "
        "заборонене, воно має лишитись рядком у diff.",
    "⚠ the step holds {held}, the contract is now {now} — keel rev first":
        "⚠ крок тримає {held}, контракт тепер {now} — спершу keel rev",
    "Take the {name} skill.": "Візьми скіл {name}.",
    "Keel is in manual mode: ask the person to type /{name}.":
        "Keel у ручному режимі: попроси людину набрати /{name}.",
    "no agent hooks: mode is {mode}": "агентських хуків немає: режим {mode}",
    "Keel: plan branch {branch}, {where}. The plan is written here, not code.\n"
    "{take} What is missing is what `python3 {tool} gaps` says.":
        "Keel: гілка плану {branch}, {where}. Тут пишеться план, а не код.\n"
        "{take} Чого бракує — каже `python3 {tool} gaps`.",
    "Keel: branch {branch} is not named after a step, so there is no planned "
    "work here.\nA new step: first `python3 {tool} new step <slug>` — it prints "
    "the file name with its number — and only then the branch "
    "`plan/<that same name>`. {take}":
        "Keel: гілка {branch} не названа за кроком, тож запланованої роботи тут "
        "немає.\nНовий крок: спершу `python3 {tool} new step <слаг>` — він друкує "
        "імʼя файла з номером — і аж тоді гілка `plan/<те саме імʼя>`. {take}",
    "Keel: {file} does not parse: {reason}":
        "Keel: {file} не розбирається: {reason}",
    "Keel: every transform of step {slug} is closed by a commit.\n{take} Then "
    "`python3 {tool} check` and the PR.":
        "Keel: кожен трансформ кроку {slug} закрито комітом.\n{take} Тоді "
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


ESCAPES = {'"': '"', "\\": "\\", "n": "\n", "t": "\t"}


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
    # dict, and Ref(dict) later surfaced as a bogus "step that does not exist"
    # far from the line that caused it.
    if MAP_ITEM.match(stripped) or stripped.startswith("{"):
        raise YamlError(line, t("a map inside a list is not supported: {item}", item=repr(stripped)))
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
                raise YamlError(line, t("duplicate key {key}", key=repr(key)))
            out[key] = _flow(value, line)
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
        # Returning {} here would turn a malformed header into a step with no
        # transforms — which reads as "nothing declared" and switches the write
        # hook off without a word.
        raise YamlError(1, t("a header has to be a set of keys, not a list"))
    return value


def _parse_block(lines, index, indent):
    if index >= len(lines):
        return {}, index
    if lines[index][2].startswith("- "):
        return _parse_list(lines, index, indent)
    return _parse_map(lines, index, indent)


def _parse_list(lines, index, indent):
    items = []
    while index < len(lines):
        number, own, text = lines[index]
        if own < indent:
            break
        if own > indent or not text.startswith("- "):
            raise YamlError(number, t("this line is not a list item"))
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

SECTION_RE = re.compile(r"^##\s+(.+?)\s*$", re.M)
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
            with open(path, encoding="utf-8") as handle:
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
    # write hook and leaves every check green over a step that guards nothing.
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


class Step(Doc):
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
        """Every file the step's transforms declare — the fact check 4 and the
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
        proc = subprocess.run(
            ["git", "-C", self.root, *args],
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
        return self.out("rev-parse", "--abbrev-ref", "HEAD")

    ORIGIN = "refs/remotes/origin/"

    @functools.cached_property
    def main_branch(self):
        """The main branch. On CI it is not local — there it is origin/main."""
        head = self.out("symbolic-ref", "--quiet", "refs/remotes/origin/HEAD")
        # Strip the ref prefix, do not take the last path segment: a default
        # branch named release/2024 would otherwise become "2024", a ref that
        # does not exist, and the scope check would find no baseline.
        short = head[len(self.ORIGIN):] if head.startswith(self.ORIGIN) else ""
        # In a single-branch clone origin/HEAD names the branch under test.
        # Believing it makes a branch its own baseline: the diff covers nothing
        # and the scope check reports green having compared nothing.
        if short and short != self.branch:
            if self.run("rev-parse", "--verify", "--quiet", short)[0] == 0:
                return short
            return f"origin/{short}"
        for name in ("main", "master", "origin/main", "origin/master"):
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
            code, stdout, _ = self.run("diff", "--name-only", base, "HEAD")
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

    def commits_since(self, base):
        """[(sha, message, {files})], oldest first.

        One `git log --name-only`, not one `git show` per commit: the session
        hook and `next` call this, and a spawn per commit made them O(branch).
        """
        if not base:
            return []
        code, stdout, _ = self.run("log", "--format=%x1e%H%x1f%B%x1f",
                                   "--name-only", f"{base}..HEAD")
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

    def file_in_branch(self, branch, path):
        return self.run("cat-file", "-e", f"{branch}:{path}")[0] == 0


# ─────────────────────────────────────────────────────────────────────────────
# Language adapters
#
# Two of the six checks depend on the language: what runs the tests and where
# a module's exports come from. The adapter is chosen by a marker in the root.
# ─────────────────────────────────────────────────────────────────────────────

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
    except FileNotFoundError:
        return Probe(1, "", t("{command} was not found", command=command[0]))


class Adapter:
    name = "?"
    marker = ()
    test_dirs = ()
    test_suffix = ()
    tag_re = None

    @classmethod
    def detect(cls, root):
        return any(os.path.exists(os.path.join(root, item)) for item in cls.marker)

    def test_command(self):
        raise NotImplementedError

    def test_files(self, root):
        found = []
        for directory in self.test_dirs:
            base = os.path.join(root, directory)
            for current, _, names in os.walk(base):
                for name in sorted(names):
                    if name.endswith(tuple(self.test_suffix)):
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


def normalise_slug(text):
    return re.sub(r"[^a-z0-9]+", "-", str(text).strip().lower()).strip("-")


class ElixirAdapter(Adapter):
    name = "elixir"
    marker = ("mix.exs",)
    test_dirs = ("test",)
    test_suffix = ("_test.exs",)
    # rev is captured whatever it looks like, not only hex: rubbish in a
    # revision should turn a check red rather than pass unnoticed.
    tag_re = re.compile(
        r"@tag\s+proves:\s*:([A-Za-z0-9_?!]+)"
        r"(?:\s*,\s*rev:\s*[\"']([^\"']*)[\"'])?"
    )

    def test_command(self):
        return ["mix", "test"]

    def ci_steps(self, root):
        elixir, otp = self.versions()
        return [
            "      - uses: erlef/setup-beam@v1",
            "        with:",
            f"          elixir-version: '{elixir}'",
            f"          otp-version: '{otp}'",
            "      - run: mix deps.get",
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

    def test_files(self, root):
        found = list(super().test_files(root))
        for directory in self.test_dirs:
            base = os.path.join(root, directory)
            for current, _, names in os.walk(base):
                for name in sorted(names):
                    if name.startswith("test_") and name.endswith(".py"):
                        path = os.path.join(current, name)
                        if path not in found:
                            found.append(path)
        return sorted(found)

    def test_command(self):
        return [sys.executable, "-m", "unittest", "discover", "-s", "tests", "-t", "."]

    def ci_steps(self, root):
        return [
            "      - uses: actions/setup-python@v5",
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
        self.git = Git(root)
        self.settings = read_config(root) if settings is None else settings
        self.adapter_candidates = matching_adapters(root)
        self.adapter = detect_adapter(root, self.settings["adapter"],
                                      self.adapter_candidates)
        self.steps = {}
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
        for folder, cls in (("steps", Step), ("contracts", Contract)):
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

    def step_for_branch(self, branch=None):
        branch = branch or self.branch
        if not branch or branch in ("HEAD", self.git.main_short):
            return None
        name = branch.split("/", 1)[1] if branch.startswith("plan/") else branch
        return self.steps.get(name)

    def is_plan_branch(self, branch=None):
        return (branch or self.branch or "").startswith("plan/")

    def transform_state(self, step):
        """{transform -> (commit sha or None, {files of that commit})}."""
        base = self.git.merge_base(self.git.main_branch)
        found = {}
        for sha, message, files in self.git.commits_since(base):
            for slug in step.transforms:
                # First word of the message, not anywhere in it: otherwise a
                # commit for `add-more` also closes `add`, and a passing
                # mention in the body closes whatever it names.
                named = re.match(rf"\s*{re.escape(slug)}(?![\w-])", message)
                if named and slug not in found:
                    found[slug] = (sha, files)
        return {slug: found.get(slug, (None, set())) for slug in step.transforms}


def find_root(start):
    current = os.path.abspath(start)
    while True:
        if os.path.isdir(os.path.join(current, "keel", "steps")):
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
# The six checks
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
KEEL_OWNED_DIRS = (KEEL_DIR_PREFIX, ".claude/skills/", ".cursor/skills/")
KEEL_OWNED_FILES = (".claude/settings.json", ".cursor/hooks.json",
                    ".codex/hooks.json", ".github/workflows/keel.yml",
                    "AGENTS.md")
def keel_owns(name):
    """Ours, by whole path — not by anything that merely starts the same way.

    A bare prefix test claimed AGENTS.mdx and .claude/settings.json.bak, which
    let a plan branch modify somebody's unrelated file and let `init` sweep it
    into its own commit — against the one promise that commit makes.
    """
    return name.startswith(KEEL_OWNED_DIRS) or name in KEEL_OWNED_FILES


def check_structure(project):
    return [Problem(0, doc.error, doc.rel) for doc in project.broken]


def check_refs(project):
    problems = []
    for step in project.steps.values():
        if step.error:
            continue
        for ref in step.depends_on:
            if ref.slug not in project.steps:
                problems.append(Problem(
                    1, t("depends_on points at a step that does not exist: {slug}", slug=ref.slug),
                    step.rel, step.line_of(ref.slug)))
        for slug in step.scenarios:
            for ref in step.proves(slug):
                if ref.slug not in project.contracts:
                    problems.append(Problem(
                        1, t("scenario {scenario} proves a contract that does not exist: {slug}", scenario=slug, slug=ref.slug),
                        step.rel, step.line_of(ref.raw)))
        for slug in step.transforms:
            for name in step.transform_implements(slug):
                if name not in step.scenarios:
                    problems.append(Problem(
                        1, t("transform {transform} implements a scenario that does not exist: {scenario}", transform=slug, scenario=name),
                        step.rel, step.line_of(name)))
            for ref in step.transform_contracts(slug):
                if ref.slug not in project.contracts:
                    problems.append(Problem(
                        1, t("transform {transform} implements a contract that does not exist: {slug}", transform=slug, slug=ref.slug),
                        step.rel, step.line_of(ref.raw)))

    for doc in list(project.steps.values()) + list(project.contracts.values()):
        for match in LINK_RE.finditer(doc.body):
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
                                    project.steps[slug].rel))
            return
        state[slug] = "open"
        step = project.steps.get(slug)
        if step and not step.error:
            for ref in step.depends_on:
                if ref.slug in project.steps:
                    walk(ref.slug, trail + [slug])
        state[slug] = "done"

    for slug in sorted(project.steps):
        walk(slug, [])
    seen = set()
    return [p for p in problems if not (p.message in seen or seen.add(p.message))]


def contract_refs(step):
    """Everything in a step that leans on a contract: (who leans, reference)."""
    for slug in step.scenarios:
        for ref in step.proves(slug):
            yield t("scenario {slug}", slug=slug), ref
    for slug in step.transforms:
        for ref in step.transform_contracts(slug):
            yield t("transform {slug}", slug=slug), ref


def scenario_tags(project):
    """(step, scenario, body, [(file, line, revision)]) — scenarios and their tags."""
    tags = project.adapter.tags(project.root) if project.adapter else {}
    for step in project.steps.values():
        if step.error:
            continue
        for slug in step.scenarios:
            body = step.scenario_body(slug)
            if body is None:
                continue  # check 7 catches this
            yield step, slug, body, tags.get(normalise_slug(slug), [])


def drifted_contract_refs(project):
    """(step, who, ref, contract) for every reference not matching its contract.

    One definition of "drifted", consumed by check 3 to report and by `rev` to
    restamp: two hand-kept copies of this loop could disagree about what needs
    fixing and what got fixed.
    """
    for step in project.steps.values():
        if step.error:
            continue
        for who, ref in contract_refs(step):
            contract = project.contracts.get(ref.slug)
            if contract is None or contract.error:
                continue
            if ref.rev and contract.rev_ok(ref.rev):
                continue
            yield step, who, ref, contract


def drifted_tags(project):
    """(step, slug, body, path, line, rev) for every tag not matching its scenario."""
    for step, slug, body, found in scenario_tags(project):
        for path, line, rev in found:
            if rev and rev_matches(rev, body):
                continue
            yield step, slug, body, path, line, rev


def check_revisions(project):
    problems = []
    for step, who, ref, contract in drifted_contract_refs(project):
        if not ref.rev:
            problems.append(Problem(
                3, t("{who} leans on {slug} without a revision; it is now {now}",
                  who=who, slug=ref.slug, now=contract.revision),
                step.rel, step.line_of(ref.raw)))
        else:
            problems.append(Problem(
                3, t("{who} holds {slug}@{held}, and the contract is now {now}",
                  who=who, slug=ref.slug, held=ref.rev, now=contract.revision),
                step.rel, step.line_of(ref.raw)))
    return problems


def check_scope(project):
    if not project.git.available:
        return [Problem(4, t("this is not a git repository — nothing to check scope against"))]
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
    changed = project.git.changed_files(base)

    if project.is_plan_branch(branch):
        stray = sorted(name for name in changed if not keel_owns(name))
        return [Problem(4, t("a plan branch is touching code: {name}", name=name)) for name in stray]

    # Keel's own furniture is out of scope on a work branch too. `update` may
    # refresh a skill in the middle of the work, and telling the person to
    # declare our own generated file in their transform is the same mine the
    # plan branch was cleared of.
    changed = {name for name in changed if not keel_owns(name)}

    step = project.step_for_branch(branch)
    if step is None:
        return [Problem(4, t("branch {branch} is not named after a step — there is nothing "
                        "to compare scope against", branch=branch))]
    if step.error:
        return []

    declared = step.declared_files()

    problems = []
    for name in sorted(changed - declared):
        problems.append(Problem(4, t("changed but not declared: {name}", name=name), step.rel))
    for name in sorted(declared - changed):
        problems.append(Problem(4, t("declared but not changed: {name}", name=name),
                                step.rel, step.line_of(name)))
    return problems


def check_scenarios(project, run_tests=True):
    steps = [step for step in project.steps.values() if not step.error and step.scenarios]
    if not steps:
        return []
    problems = adapter_problem(project, 5)
    if project.adapter is None:
        return problems + [Problem(
            5, t("nothing to run the tests with: the root has none of {markers}",
                 markers=", ".join(item for cls in ADAPTERS
                                   for item in cls.marker)))]

    for step, slug, body, found in scenario_tags(project):
        if not found:
            problems.append(Problem(
                5, t("scenario {slug} has no test", slug=slug), step.rel,
                step.section_lines.get(f"scenario: {slug}")))
    for _, slug, body, path, line, rev in drifted_tags(project):
        if not rev:
            problems.append(Problem(
                5, t("the test for {slug} carries no revision; it is now {now}",
                 slug=slug, now=revision(body)), path, line))
        else:
            problems.append(Problem(
                5, t("the test holds {slug}@{held}, and the scenario is now {now}",
                 slug=slug, held=rev, now=revision(body)), path, line))

    if run_tests:
        command = project.adapter.test_command()
        try:
            proc = subprocess.run(command, cwd=project.root, capture_output=True,
                                  text=True, stdin=subprocess.DEVNULL,
                                  timeout=TEST_TIMEOUT)
        except subprocess.TimeoutExpired:
            return problems + [Problem(
                5, t("the tests did not finish within {seconds}s ({command}). "
                     "Nothing was proved, which is not the same as nothing "
                     "being wrong.", seconds=TEST_TIMEOUT,
                     command=" ".join(command)))]
        if proc.returncode != 0:
            tail = (proc.stdout or proc.stderr).strip().splitlines()[-12:]
            problems.append(Problem(
                5, t("the tests are red ({command}):", command=" ".join(command))
                + "\n" + "\n".join("      " + line for line in tail)))
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
    for doc in list(project.steps.values()) + list(project.contracts.values()):
        for title in sorted(set(doc.repeated)):
            problems.append(Problem(
                7, t("the heading ## {title} appears twice — the first is read and the "
                     "last is counted", title=title),
                doc.rel, doc.section_lines.get(title)))
    for step in project.steps.values():
        if step.error:
            continue
        for kind, declared in (("scenario", step.scenarios), ("transform", step.transforms)):
            in_body = set(step.named_sections(kind))
            in_head = set(declared)
            for slug in sorted(in_head - in_body):
                problems.append(Problem(
                    7, t("the header has {kind} {slug} and the body has no "
                         "section for it", kind=kind, slug=slug),
                    step.rel, step.line_of(slug)))
            for slug in sorted(in_body - in_head):
                problems.append(Problem(
                    7, t("the body has ## {kind}: {slug} and the header does not",
                         kind=kind, slug=slug),
                    step.rel, step.section_lines.get(f"{kind}: {slug}")))
    return problems


def run_checks(project, only=None, run_tests=True):
    only = set(only or CHECK_NAMES)
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
    for number in sorted(runners):
        results[number] = runners[number]() if number in only else None
    return structural, results


# ─────────────────────────────────────────────────────────────────────────────
# Commands
# ─────────────────────────────────────────────────────────────────────────────

# The skeletons are written into somebody's project, so they follow `lang` like
# every other line the tool produces. The heading is the one structural word in
# them, and the reader accepts either spelling — a project may change language
# without its existing steps becoming unreadable.
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


def step_skeleton(slug):
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


WHY_HINT = "why this step exists and what is missing without it"


def unfilled_why(step):
    """The skeleton's own placeholder, in whatever language it was written in.

    Derived from the catalogue entry the skeleton writes, not restated: reword
    the hint and this recogniser follows, instead of silently going blind.
    """
    text = step.why.strip()
    if not text.startswith(f"{step.slug}:"):
        return False
    tail = text[len(step.slug) + 1:].strip().lower()
    hints = (WHY_HINT, UK.get(WHY_HINT, WHY_HINT))
    return any(tail.startswith(hint.lower()) for hint in hints)


def cmd_new(project, args):
    kind, slug = args.kind, args.slug
    clean = normalise_slug(slug)
    if not clean:
        fail(t("bad slug: {slug}", slug=repr(slug)))

    if kind == "step":
        folder = os.path.join(project.keel, "steps")
        numbers = [int(m.group(1)) for name in os.listdir(folder)
                   if (m := re.match(r"(\d{4})-", name))] if os.path.isdir(folder) else []
        number = max(numbers, default=0) + 1
        name = f"{number:04d}-{clean}.md"
        text = step_skeleton(clean)
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


def cmd_gaps(project, args):
    steps = ([project.steps[args.step]] if args.step and args.step in project.steps
             else [project.step_for_branch()] if not args.step and project.step_for_branch()
             else list(project.steps.values()))
    if args.step and args.step not in project.steps:
        fail(t("no such step: {step}", step=args.step))
    steps = [step for step in steps if step]

    problems = []
    for step in steps:
        if step.error:
            problems.append(Problem(0, step.error, step.rel))
            continue
        if not step.why.strip() or unfilled_why(step):
            problems.append(Problem(0, t("the Why section is empty"), step.rel))
        if not step.scenarios:
            problems.append(Problem(0, t("no scenarios at all"), step.rel))
        if not step.transforms:
            problems.append(Problem(0, t("no transforms at all"), step.rel))

        implemented = set()
        for slug in step.transforms:
            implemented.update(step.transform_implements(slug))
            if not step.transform_files(slug):
                problems.append(Problem(
                    0, t("transform {slug} declared no files", slug=slug), step.rel,
                    step.line_of(slug)))
            if not step.transform_implements(slug):
                problems.append(Problem(
                    0, t("transform {slug} implements no scenario", slug=slug), step.rel,
                    step.line_of(slug)))
            if not (step.transform_body(slug) or "").strip():
                problems.append(Problem(
                    0, t("transform {slug} has no body: what it does and where its edges are", slug=slug), step.rel))
        for slug in step.scenarios:
            if not step.proves(slug):
                problems.append(Problem(
                    0, t("scenario {slug} has no proves", slug=slug), step.rel, step.line_of(slug)))
            if slug not in implemented:
                problems.append(Problem(
                    0, t("no transform implements scenario {slug}", slug=slug), step.rel,
                    step.line_of(slug)))
            if not (step.scenario_body(slug) or "").strip():
                problems.append(Problem(
                    0, t("scenario {slug} has no body: given/when/then", slug=slug), step.rel))

    mine = {step.rel for step in steps}
    problems += [p for p in check_headings(project) if p.where in mine]
    problems += [p for p in check_refs(project) if p.where in mine]

    names = ", ".join(step.slug for step in steps) or t("nothing")
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

    if args.json:
        payload = {
            "ok": not structural and not any(results.get(n) for n in results),
            "structure": [p.as_dict() for p in structural],
            "checks": {
                str(number): {
                    "name": t(CHECK_NAMES[number]),
                    "run": results[number] is not None,
                    "problems": [p.as_dict() for p in (results[number] or [])],
                }
                for number in sorted(results)
            },
        }
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return 0 if payload["ok"] else 1

    total = len(structural)
    if structural:
        print("✗ " + t("documents do not parse"))
        for problem in structural:
            print(problem.render())
        print()

    for number in sorted(results):
        problems = results[number]
        if problems is None:
            print(f"– {number}. " + t(CHECK_NAMES[number]) + " " + t("(not run)"))
            continue
        total += len(problems)
        if not problems:
            print(f"✓ {number}. " + t(CHECK_NAMES[number]))
            continue
        print(f"✗ {number}. " + t(CHECK_NAMES[number]))
        for problem in problems:
            print(problem.render())
    print()
    print(t("clean") if total == 0 else t("problems: {count}", count=total))
    return 0 if total == 0 else 1


def next_transform(project, step):
    state = project.transform_state(step)
    for slug in step.transforms:
        if state[slug][0] is None:
            return slug, state
    return None, state


def cmd_next(project, args):
    step = project.step_for_branch()
    branch = project.branch
    if step is None:
        message = t("branch {branch} is not named after a step. Work happens on "
                    "a branch named after the step, planning on plan/<step>.",
                    branch=branch)
        return emit_next_error(args, message)
    if project.is_plan_branch(branch):
        return emit_next_error(args, t("this is a plan branch: the step is written "
                                       "here, not code. keel gaps says what is "
                                       "missing."))
    if not project.git.file_in_branch(project.git.main_branch, step.rel):
        return emit_next_error(
            args, t("step {step} is not on {main} yet: the plan is not approved "
                    "and there is no work.", step=step.slug,
                    main=project.git.main_branch))
    if step.error:
        return emit_next_error(args, f"{step.rel}: {step.error}")

    slug, state = next_transform(project, step)
    if slug is None:
        message = t("every transform of step {step} is closed by a commit. "
                    "Next: keel check, then the PR.", step=step.slug)
        return emit_next_error(args, message, code=0)

    package = next_package(project, step, slug, state)
    if args.json:
        print(json.dumps(package, ensure_ascii=False, indent=2))
    else:
        print(render_next(package))
    return 0


def next_package(project, step, slug, state):
    """Everything needed for one move, and nothing beyond it."""
    contracts = []
    for ref in step.transform_contracts(slug):
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
    for name in step.transform_implements(slug):
        body = step.scenario_body(name)
        scenarios.append({
            "slug": name,
            "rev": step.scenario_revision(name),
            "proves": [ref.raw for ref in step.proves(name)],
            "body": (body or "").strip(),
        })

    return {
        "step": {"id": step.slug, "file": step.rel, "why": step.why.strip()},
        "transform": {
            "slug": slug,
            "body": (step.transform_body(slug) or "").strip(),
            "files": step.transform_files(slug),
        },
        "scenarios": scenarios,
        "contracts": contracts,
        "done": [name for name, (sha, _) in state.items() if sha],
        "left": [name for name in step.transforms
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
    step, transform = package["step"], package["transform"]
    out = [f"# {transform['slug']}", ""]
    out.append(t("Step {id} · {file}", id=step["id"], file=step["file"]))
    if package["done"]:
        out.append(t("Closed: {names}", names=", ".join(package["done"])))
    if package["left"]:
        out.append(t("After this one: {names}", names=", ".join(package["left"])))
    out.append("")
    if step["why"]:
        out += ["## " + t("Why the step"), "", step["why"], ""]
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
            out.append(t('Test tag: `proves: :{atom}, rev: "{rev}"`',
                         atom=item["slug"].replace("-", "_"), rev=item["rev"]))
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
                out.append(t("⚠ the step holds {held}, the contract is now "
                             "{now} — keel rev first",
                             held=item["rev"], now=item["rev_now"]))
                out.append("")

    out += ["## " + t("The commit"), "", f"    {package['commit']}", ""]
    out.append(t("The transform slug in the message is the only link between "
                 "the work and the plan."))
    return "\n".join(out)


INIT_DIRS = ("keel/steps", "keel/contracts")
AGENTS_START = "<!-- keel:start -->"
AGENTS_END = "<!-- keel:end -->"
VENDORED = "keel/keel.py"
CI_FILE = ".github/workflows/keel.yml"
# References travel as copies: AGENTS.md points at them, and you can only point
# at what sits in the same repository. Methodology, tool, quality cuts.
REFERENCES = ("KEEL.md", "README.md", "QUALITY.md")

# Two settings, and they are deliberately separate: someone may well want the
# reference in English while the agent writes and listens in their own language.
#
#   docs — which translation of the references lands in the project
#   lang — what the agent writes (steps, commits) and what phrases the skills catch
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
            "adapter": "", "agent_hooks": ""}
ALLOWED = {"docs": LANGS, "lang": LANGS, "mode": MODES,
           "adapter": ADAPTER_NAMES, "agent_hooks": ("", True, False)}
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
        for agent, relative in skill_targets(skill):
            out[relative] = render_skill(skill, agent, settings["lang"],
                                         settings["mode"])
    adapter = detect_adapter(root, settings["adapter"])
    out[CI_FILE] = CI_TEMPLATE.format(
        tool=VENDORED,
        setup="".join(line + "\n" for line in (adapter.ci_steps(root) if adapter else [])))
    if agent_hooks_wanted(settings):
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


def read_config(root):
    settings = dict(DEFAULTS)
    path = os.path.join(root, CONFIG_FILE)
    if os.path.exists(path):
        try:
            stored = json.loads(read_text(path))
        except ValueError:
            return settings
        if isinstance(stored, dict):
            for key in DEFAULTS:
                if stored.get(key) in ALLOWED[key]:
                    settings[key] = stored[key]
    return settings


def write_config(root, settings, done, manifest=None):
    path = os.path.join(root, CONFIG_FILE)
    found = {}
    if os.path.exists(path):
        try:
            found = json.loads(read_text(path))
        except ValueError:
            # Overwriting would silently reset docs and lang to the defaults,
            # and regenerate the skills with the wrong trigger language.
            print("  " + t("{file}: does not parse as JSON, leaving it alone", file=CONFIG_FILE))
            return
        if not isinstance(found, dict):
            # Valid JSON that is not an object is just as much somebody's file:
            # replacing it wholesale is the destroy this function refuses.
            print("  " + t("{file}: not an object, leaving it alone", file=CONFIG_FILE))
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
    `update` rather than with `check`: the six checks are about a project's own
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

This project's method: two kinds of document — step and contract — and six
checks. Steps live in `keel/steps/`, contracts in `keel/contracts/`.

{principles}

Two commands:

- `python3 {tool} next` — what to do next: the transform, its files and
  boundaries, the scenarios it brings closer, the contracts it leans on.
- `python3 {tool} check` — what is wrong right now. Before a commit and before a PR.

Three references — open them when something is unclear:

- `keel/KEEL.md` — the method: what goes in a step's header, how revisions work,
  what each of the six checks looks at.
- `keel/README.md` — the tool: every command with its flags, language adapters,
  hooks, skills.
- `keel/QUALITY.md` — forty quality cuts. Walked once per step, where the
  scenarios are written.

This block is generated; edits between the markers are overwritten on the next update.
{end}"""

AGENTS_BLOCK = """{start}
## Keel

Методика цього проєкту: два типи документів — крок і контракт — і шість
перевірок. Кроки лежать у `keel/steps/`, контракти в `keel/contracts/`.

{principles}

Дві команди:

- `python3 {tool} next` — що робити далі: трансформа, її файли й межі,
  сценарії, які вона наближає, тіла контрактів, на які спирається.
- `python3 {tool} check` — що не так зараз. Перед коммітом і перед PR.

Три довідники — відкривай, коли не ясно:

- `keel/KEEL.md` — методика: що йде в шапку кроку, як влаштовані редакції,
  що саме перевіряє кожна з шести перевірок.
- `keel/README.md` — інструмент: усі команди з прапорцями, адаптери мов,
  хуки, скіли.
- `keel/QUALITY.md` — сорок розрізів якості. Проходяться раз на крок, там,
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
SKILL_DIRS = {"claude": ".claude/skills", "cursor": ".cursor/skills"}
DESCRIPTION_CAP = 1536      # Claude truncates the skill listing at this

PLAN_BODY = """\
## Start here

Planning and work are separate on purpose: this branch writes the step and not a
line of code. A person reads the step and lets it through, which is why it is its
own pull request.

Create the branch and the skeleton:

    python3 keel/keel.py new step <slug>

It prints the file it made, and the number in that name is part of the step's
identity. Branch after the file, not after the slug you typed:

    git checkout -b plan/0007-session-loop

Get this the wrong way round and nothing links the branch to the step: the tool
looks the step up by branch name, finds nothing, and the session hook will tell
you the step does not exist while you are looking straight at it.

The slug may arrive as an argument to `/keel-plan`. If it did not, ask in one
sentence which step we are writing rather than inventing it for the person.

**Header fields are English; the prose is the project's own language.** The
fields become code, test tags and file names, so they stay English everywhere.
The prose is read and approved by a person, so it follows whatever language the
existing steps in `keel/steps/` are written in. If there are none yet, ask which
language this project writes in — and write commit messages the same way.

## Order

**The Why section** — the heading `keel new step` wrote, one or two sentences on
what is missing without this step. Not a retelling of what you will do: the
reason it is worth doing.

**Scenarios** — what we promise. Each one becomes a test, so word them so that it
is visible when one fails: **Given** a state, **When** an action, **Then** an
observable consequence. Name them; do not number them. A scenario nobody can
check is a wish — either reword it or admit it is a boundary.

**Transforms** — what does the work. Each becomes exactly one commit. List the
files by name before the work starts. Globs do not exist here: under a glob an
agent creates ten files where one was meant, and nothing objects.

For the exact header shape — which fields exist, how `proves`, `implements` and
`contracts` are written, where a revision comes from — read `keel/KEEL.md`. The
skeleton shows the bones; the reference explains the fields.

## Ask, do not guess

A person will read and approve this step, so a guess written into the plan costs
more than a question. When you reach a fork, ask through the **question tool** if
there is one — in Claude Code that is `AskUserQuestion` — and offer ready options
with the consequence of each. Without such a tool, ask in chat and stop until
there is an answer.

Ask about:

- **the choice of approach**, when several exist and they lead to different work;
- **the edges of the step** — does the neighbouring thing belong here or in the
  next step;
- **a quality cut you were silent on** — do we write a scenario, or say "no" out
  loud;
- **a promise something makes to us** — a library, a service, a binary: does it
  earn a contract of its own;
- **contract names**, when the step draws a new boundary between modules.

Write every question so it stands on its own, as if the person had just walked in
with no context.

Do not ask about anything readable: the shape of the code, whether a file exists,
anything `keel gaps` already answers. Questions are for judgement, not for facts.

An open question that blocks nothing does not stop the work: write everything
that does not depend on the answer, and raise the question where it belongs.

## Quality cuts

Before treating the list of scenarios as complete, walk `keel/QUALITY.md` — forty
questions under nine headings. One pass per step, here, while the scenarios are
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

Check what arrived with the library before claiming something is missing.

## Where the line runs

**Scenario or boundary.** If a test can prove it, it is a scenario. If it is "we
deliberately do not do this", it is the boundaries paragraph inside the
transform — the one the skeleton opens for you. A boundary without a scenario is
honest; a scenario without a test is not.

**A transform is not yet atomic** if you cannot name its files in advance. That
is not a reason to write a glob, it is a reason to cut further. The other tell:
you want to write the commit message with an "and" in it.

**A contract appears** when a promise outlives the step that created it. Ours:
module, exported functions, meaning. Somebody else's — a library, a service, a
binary — is the same thing, and it carries `verify`, a command whose success is
the proof. A promise nothing can check is not a contract; it is a boundary.

**There are no decision files.** What outlives a step and promises something is a
contract; "we deliberately do not do this" is the boundaries paragraph; a rule
about architecture belongs to the linter's config.

## Before handing the plan over

Run this until it comes back clean:

    python3 keel/keel.py gaps

It reports what is missing mechanically: slugs without sections, transforms
without files, scenarios without `proves`. If you lean on a contract, a fresh
revision comes from `python3 keel/keel.py rev --write`.

Then commit on the `plan/<step>` branch and open the PR. **No code goes in this
PR** — a plan branch touches only `keel/`, and the scope check enforces it.

Approval is written nowhere: it is the fact that the step file reached the main
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

If `next` refuses, it says why: the branch is not named after a step, or the plan
has not reached the main branch yet. Neither is a reason to start by hand — both
are a reason to go back to `/keel-plan`.

Do exactly that work, then run

    python3 keel/keel.py check

and commit with the **transform slug as the first word** of the message:

    drive-turns-on-reqllm: keep turning while the model calls tools

The slug is English because it comes from the step header; the rest of the
message follows the project's prose language, like the step files do. The slug is
the only link between the work and the plan — without it the transform stays open
no matter what the code says.

Repeat until `next` reports nothing open. The step is then ready for review —
that is `/keel-review`.

## Boundaries

The files on the list, and only those. If you need a file that is not there, add
it to the transform in the step file. Drift is not forbidden — it is named, and
it shows up as a line in the diff. Leaving the list silently is what is
forbidden, and the write hook will refuse it.

Every scenario the transform brings closer needs a test carrying its name and its
revision in the tag. `next` prints the revision; `keel rev --write` records a new
one once the scenario text has changed — after you have reread it, which is the
entire point of the mechanism.
"""

REVIEW_BODY = """\
## What happens

    python3 keel/keel.py check

The full gate: references, cycles, revisions, scope, scenarios with green tests,
module exports. Red gets fixed, not explained.

Then the part no check can see. Reread the step and ask not "what else should be
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

When it is clean, push and open the PR. The step stands whole: every transform
closed by a commit, every scenario proved by a test, six checks green. Nothing
else needs marking — the statuses are derived.
"""

SKILLS = (
    {
        "name": "keel-plan",
        "description": ("Write a Keel step: why it exists, scenarios drawn through "
                        "the quality cuts, transforms with exact file lists. In a "
                        "project that has a keel/ directory, use this skill "
                        "whenever any new work begins — even when nobody says the "
                        "word step and the request is {triggers}, or just a "
                        "description of what is missing. Use it as well when "
                        "keel/steps/*.md is being edited, when asked how to split "
                        "work into transforms or how a scenario differs from a "
                        "boundary, and when keel gaps reports an incomplete step."),
        "triggers": {
            "uk": "«додай», «зроби це», «реалізуй», «давай спланую»",
            "en": "\"add\", \"build\", \"implement\", \"let's plan this\"",
        },
        "paths": ["keel/steps/*.md"],
        "argument_hint": "[слаг нового кроку]",
        "body": PLAN_BODY,
    },
    {
        "name": "keel-work",
        "description": ("Do the next transform of a Keel step: keel next, work "
                        "strictly inside the named files, keel check, commit "
                        "carrying the transform slug. In a project with a keel/ "
                        "directory, use this skill whenever the request is to write "
                        "code, continue the work, {triggers} — and whenever the "
                        "branch is named after a step, even if Keel was never "
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
        "description": ("Check a Keel step before the pull request: the full keel "
                        "check plus the question of what we stayed silent about. In "
                        "a project with a keel/ directory, use this skill when asked "
                        "to open a PR or merge a branch, and when the question is "
                        "{triggers} — and whenever every transform of the step is "
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
    for raw, escaped in (("\\", "\\\\"), ('"', '\\"'), ("\n", "\\n"), ("\t", "\\t")):
        text = text.replace(raw, escaped)
    return '"' + text + '"'


# ─────────────────────────────────────────────────────────────────────────────
# Agent hooks
#
# Event names match across agents; the replies do not. One command answers in
# whichever dialect the flag names, so the configs stay thin and cannot drift
# apart. Codex is left out until its apply_patch payload is worked out.
# ─────────────────────────────────────────────────────────────────────────────

CLAUDE_SETTINGS = ".claude/settings.json"
CURSOR_HOOKS = ".cursor/hooks.json"
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


def claude_hook_config():
    return {
        "SessionStart": [{
            "matcher": "startup|resume|clear",
            "hooks": [{"type": "command",
                       "command": hook_command("session", "claude"),
                       "timeout": 30}],
        }],
        "PreToolUse": [{
            "matcher": "Write|Edit|NotebookEdit",
            "hooks": [{"type": "command",
                       "command": hook_command("write", "claude"),
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
        step = project.step_for_branch(branch)
        where = (t("step {slug}", slug=step.slug) if step
                 else t("there is no step file for {branch} yet", branch=branch))
        return t("Keel: plan branch {branch}, {where}. The plan is written here, "
                 "not code.\n{take} What is missing is what "
                 "`python3 {tool} gaps` says.",
                 branch=branch, where=where, tool=VENDORED,
                 take=take(project, "keel-plan"))

    step = project.step_for_branch(branch)
    if step is None:
        # Order matters and is easy to get backwards: the number only exists
        # after `new step`, and a branch named before it never links to the step.
        return t("Keel: branch {branch} is not named after a step, so there is no "
                 "planned work here.\nA new step: first `python3 {tool} new step "
                 "<slug>` — it prints the file name with its number — and only then "
                 "the branch `plan/<that same name>`. {take}",
                 branch=branch, tool=VENDORED, take=take(project, "keel-plan"))
    if step.error:
        return t("Keel: {file} does not parse: {reason}",
                 file=step.rel, reason=step.error)

    slug, state = next_transform(project, step)
    if slug is None:
        return t("Keel: every transform of step {slug} is closed by a commit.\n"
                 "{take} Then `python3 {tool} check` and the PR.",
                 slug=step.slug, tool=VENDORED, take=take(project, "keel-review"))

    package = next_package(project, step, slug, state)
    return (t("Keel: {take} Here is the package for the next "
              "move — work from it, nothing around it needs opening.",
              take=take(project, "keel-work")) + "\n\n"
            + render_next(package))


def hook_reply(agent, event, kind, message):
    """The same verdict, in the dialect of one agent."""
    if agent == "claude":
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


def write_verdict(project, payload):
    """(kind, message), or None when there is nothing to say."""
    branch = project.branch
    if (not branch or branch in ("HEAD", project.git.main_short)
            or project.is_plan_branch(branch)):
        return None
    step = project.step_for_branch(branch)
    if step is None:
        return None
    if step.error:
        # Unreadable is not the same as unrestricted. Waving the write through
        # in silence is how a broken header turns the guard off without a word.
        return ("note", t("keel: {file} does not parse, so scope is not being "
                          "checked: {reason}", file=step.rel, reason=step.error))
    if not step.transforms:
        return ("note", t("keel: step {step} declares no transforms, so nothing "
                          "says which files belong to this work.", step=step.slug))

    declared = step.declared_files()

    target = find_path(payload)
    if target is None:
        return ("note", t("keel: the hook payload carried no file path, so scope "
                          "was not checked. Files the step declares: {declared}",
                          declared=", ".join(sorted(declared)) or t("none")))

    # realpath on both sides: on macOS /tmp is a symlink to /private/tmp, and the
    # agent hands over the path the user sees. Comparing the two unresolved turns
    # every write into "outside the repository" — that is, into silence.
    absolute = target if os.path.isabs(target) else os.path.join(project.root, target)
    relative = os.path.relpath(os.path.realpath(absolute),
                               os.path.realpath(project.root))
    relative = relative.replace(os.sep, "/")
    if relative == ".." or relative.startswith("../"):
        return ("note", t("keel: {target} is outside the repository, so the "
                          "step's scope does not apply to it. Judge for yourself "
                          "whether it should be written to.", target=target))
    if keel_owns(relative):
        return None      # the same exemption check 4 applies, so the hook is
                         # never stricter than the gate
    if relative in declared:
        return None

    return ("deny", t("{name} is not declared in step {step}. Declared: "
                      "{declared}. If this file is the one you need, add it to the "
                      "transform in {file}: drift is not forbidden, it has to stay "
                      "a line in the diff.",
                      name=relative, step=step.slug,
                      declared=", ".join(sorted(declared)) or t("none"),
                      file=step.rel))


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
    path = os.path.join(root, CURSOR_HOOKS)
    if not os.path.exists(path):
        return
    if digest(read_text(path)) != read_manifest(root).get(CURSOR_HOOKS):
        print("  " + t("{file}: not what Keel wrote, leaving it in place — the "
                       "hooks in it still run", file=CURSOR_HOOKS))
        return
    os.remove(path)
    done.append(t("{file} removed", file=CURSOR_HOOKS))


def edit_claude_settings(path, change, done, label, create=False):
    """Load, hand the file to `change`, write it back if anything moved.

    Both the adding and the removing pass go through here. Two copies of this
    were how the two ended up validating the file differently, and only one of
    them noticed that a hook event holding something other than a list is not
    ours to rewrite.
    """
    if not os.path.exists(path) and not create:
        return
    try:
        data = json.loads(read_text(path)) if os.path.exists(path) else {}
    except ValueError:
        print("  " + t("{file}: does not parse as JSON, leaving it alone", file=CLAUDE_SETTINGS))
        return
    if not isinstance(data, dict):
        print("  " + t("{file}: not an object, leaving it alone", file=CLAUDE_SETTINGS))
        return

    before = json.dumps(data, ensure_ascii=False, sort_keys=True)
    change(data)
    if json.dumps(data, ensure_ascii=False, sort_keys=True) == before:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(json.dumps(data, ensure_ascii=False, indent=2) + "\n")
    done.append(label)


def ours_only(entries):
    """Our entries out of one hook event, or None when it is not a list of them.

    A value we do not recognise stays exactly as it is: this file belongs to the
    project, and rewriting a shape we did not expect would destroy somebody's
    configuration while reporting that we only took our own out.
    """
    if not isinstance(entries, list):
        return None
    return [item for item in entries if not is_ours(item)]


def strip_claude_settings(path, done):
    """Ours out of a file that is not ours: the rest of it stays untouched."""
    def change(data):
        hooks = data.get("hooks")
        if not isinstance(hooks, dict):
            return
        for event in list(hooks):
            kept = ours_only(hooks[event])
            if kept is None:
                continue
            if kept:
                hooks[event] = kept
            else:
                del hooks[event]
        if not hooks:
            del data["hooks"]

    edit_claude_settings(path, change, done,
                         t("{file} (our hook entries taken out)", file=CLAUDE_SETTINGS))


def merge_claude_settings(path, done):
    """Settings.json belongs to the project; we own only our own entries in it."""
    def change(data):
        hooks = data.setdefault("hooks", {})
        if not isinstance(hooks, dict):
            return
        for event, entries in claude_hook_config().items():
            existing = ours_only(hooks.get(event, []))
            if existing is None:
                continue      # somebody else's shape; adding to it would break it
            hooks[event] = existing + entries

    edit_claude_settings(path, change, done, CLAUDE_SETTINGS, create=True)


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
    if agent == "claude" and skill.get("argument_hint"):
        extra += f"argument-hint: {yaml_string(skill['argument_hint'])}\n"
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


def skill_targets(skill):
    """Where this skill lands, per agent. Both accept /<name> from the operator."""
    return tuple((agent, f"{folder}/{skill['name']}/SKILL.md")
                 for agent, folder in sorted(SKILL_DIRS.items()))


def cmd_skills(project, args=None):
    done = []
    write_skills(project.root, project.settings["lang"], done,
                 mode=project.settings["mode"])
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
    done.append("keel/steps, keel/contracts")
    project.settings = settings

    # One owner of "which files, with what content": the same table update and
    # survey read. Init rendering its own copies by hand is how the CI block
    # once honoured --adapter here and not there.
    for relative, text in generated_files(project.root, settings).items():
        target = os.path.join(project.root, relative)
        if relative == VENDORED and os.path.abspath(target) == os.path.abspath(__file__):
            continue      # init run from the vendored copy itself
        write_if_changed(target, text, done, relative, manifest)

    # The two shared files generated_files cannot express: Keel owns entries
    # inside them, not the files.
    if agent_hooks_wanted(settings):
        merge_claude_settings(os.path.join(project.root, CLAUDE_SETTINGS), done)
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
    return sorted({row[3:] for row in stdout.splitlines() if keel_owns(row[3:])})


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
    if project.git.run("commit", "--no-verify", "-m", message)[0] != 0:
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
            lines.append("\n" + t("{count} Keel files are not in git yet. Commit "
                                  "them separately from the work:\n  git add {paths}"
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
    """Add the block to AGENTS.md without touching the rest of the file."""
    old = read_text(path) if os.path.exists(path) else ""
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
    git_dir = project.git.out("rev-parse", "--absolute-git-dir")
    if not git_dir:
        fail(t("this is not a git repository — there is nowhere to put the hooks"))
    folder = os.path.join(git_dir, "hooks")
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
    """A step as a person reads it: links that resolve, and derived state.

    The header is YAML because a machine reads it, and a preview renders that
    badly. Rather than split the file — which would let the slug and its text
    drift apart in two places — this view is built on the fly and stored nowhere.
    """
    step = project.steps.get(args.step) if args.step else project.step_for_branch()
    if step is None:
        fail(t("no such step: {step}",
               step=args.step or t("branch {branch}", branch=project.branch)))
    if step.error:
        fail(f"{step.rel}: {step.error}")

    state = project.transform_state(step) if project.git.available else {}
    # Every link is relative to the step file, so they resolve wherever the
    # rendered text is read from — including the file's own directory.
    out = [f"# {step.slug}", "",
           f"[{step.rel}]({os.path.basename(step.path)})", ""]
    if step.why.strip():
        out += [step.why.strip(), ""]

    depends = [f"[{ref.slug}](../steps/{ref.slug}.md)" for ref in step.depends_on]
    if depends:
        out += ["**" + t("Depends on:") + "** " + ", ".join(depends), ""]

    out += ["## " + t("Scenarios"), ""]
    for slug in step.scenarios:
        proves = []
        for ref in step.proves(slug):
            contract = project.contracts.get(ref.slug)
            ok = contract and not contract.error and contract.rev_ok(ref.rev)
            proves.append(f"[{ref.slug}](../contracts/{ref.slug}.md)"
                          f"@{ref.rev or '—'} {'✓' if ok else '✗'}")
        out.append(f"### {slug}")
        out.append("")
        out.append(t("Proves: {proves} · revision `{rev}`",
                     proves=", ".join(proves) or "—",
                     rev=step.scenario_revision(slug) or "—"))
        out.append("")
        out.append((step.scenario_body(slug) or "_" + t("no body") + "_").strip())
        out.append("")

    out += ["## " + t("Transforms"), ""]
    for slug in step.transforms:
        sha = state.get(slug, (None, set()))[0]
        out.append(f"### {slug} — " + (t("closed {sha}", sha=sha[:7]) if sha
                                       else t("open")))
        out.append("")
        near = ", ".join(f"[{name}](#{name})"
                         for name in step.transform_implements(slug))
        out.append(t("Brings closer: {names}", names=near or "—"))
        for ref in step.transform_contracts(slug):
            out.append(t("Implements: [{slug}](../contracts/{slug}.md)@{rev}",
                         slug=ref.slug, rev=ref.rev or "—"))
        out.append("")
        for name in step.transform_files(slug):
            here = os.path.exists(os.path.join(project.root, name))
            out.append(f"- [{name}](../../{name})"
                       + ("" if here else " — " + t("not there yet")))
        out.append("")
        out.append((step.transform_body(slug) or "_" + t("no body") + "_").strip())
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
        merge_claude_settings(os.path.join(project.root, CLAUDE_SETTINGS), done)
    else:
        # Refusing to add them back is half the job: a mode narrowed by hand in
        # keel.json would otherwise leave the old entries firing forever, and
        # generated_files no longer lists the cursor file, so survey never
        # mentions it either.
        remove_hook_configs(project.root, done)
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

    for step, who, ref, contract in drifted_contract_refs(project):
        fresh = contract.revision
        report.append((step.rel, who, f"{ref.slug}@{ref.rev or '—'}",
                       f"{ref.slug}@{fresh}"))
        edits.setdefault(step.path, []).append((ref.raw, f"{ref.slug}@{fresh}"))

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
                text = rewrite_tag(text, old[1], new)
            else:
                text = rewrite_ref(text, old, new)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
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


def rewrite_tag(text, slug, fresh):
    """Write a fresh revision into a test tag, in whichever form is already there.

    The slug is bounded on the right, as rewrite_ref's is: without the boundary,
    restamping `parse` also matched the front of `parse_error` — renaming the
    other scenario's tag and splicing the new revision into the middle of it,
    which a second run could not undo.
    """
    atom = slug.replace("-", "_")
    elixir = re.compile(
        rf"@tag\s+proves:\s*:({re.escape(atom)})(?![\w?!])"
        rf"(?:\s*,\s*rev:\s*[\"'][^\"']*[\"'])?"
    )
    text = elixir.sub(lambda m: f'@tag proves: :{m.group(1)}, rev: "{fresh}"', text)
    python = re.compile(
        rf"#\s*proves:\s*({re.escape(slug)}|{re.escape(atom)})(?![\w-])"
        rf"(?:\s*,\s*rev:\s*[\"']?[^\"'\s,]*[\"']?)?"
    )
    return python.sub(lambda m: f'# proves: {m.group(1)}, rev: "{fresh}"', text)


# ─────────────────────────────────────────────────────────────────────────────
# Entry point
# ─────────────────────────────────────────────────────────────────────────────

def fail(message, code=2):
    print(message, file=sys.stderr)
    raise SystemExit(code)


def build_parser():
    parser = argparse.ArgumentParser(
        prog="keel", description="Keel: two kinds of document, six checks.")
    parser.add_argument("--version", action="version", version=VERSION)
    parser.add_argument("-C", dest="chdir", metavar="DIR",
                        help="work in this directory")
    sub = parser.add_subparsers(dest="command", required=True)

    new = sub.add_parser("new", help="skeleton of a step or a contract")
    new.add_argument("kind", choices=("step", "contract"))
    new.add_argument("slug")

    gaps = sub.add_parser("gaps", help="what is missing from a step")
    gaps.add_argument("step", nargs="?", help="a step; without it, the branch's step")

    check = sub.add_parser("check", help="the six checks")
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
    init.add_argument("--adapter", choices=[name for name in ADAPTER_NAMES if name],
                      help="which language this project is, when the root says "
                           "more than one")
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

    show = sub.add_parser("show", help="a step as a person reads it")
    show.add_argument("step", nargs="?", help="a step; without it, the branch's step")

    update = sub.add_parser("update", help="update the copies in a project")
    update.add_argument("--diff", action="store_true", help="show the difference")
    update.add_argument("--force", action="store_true",
                        help="overwrite hand-edited files too")

    hook = sub.add_parser("hook",
                          help="answer an agent hook; called by a config")
    hook.add_argument("event", choices=("session", "write"))
    hook.add_argument("--agent", choices=("claude", "cursor"), required=True)

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
        # Answer nothing and step aside.
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

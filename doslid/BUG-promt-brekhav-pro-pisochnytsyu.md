# Вада: промт казав, що в пісочниці немає нічого, крім `tool`

**Коли:** 23 серпня 2026, перший прогін `code-python`
**Де:** `agent/lib/keel_agent/session/prompt.ex`, `@python_dialect`

## Що стояло

    Your program runs in a sandboxed Python: `import tool` works, and nothing
    else does.

Це неправда. У Pyex є сорок із гаком модулів: `json`, `re`, `pathlib`, `math`,
`datetime`, `collections`, `itertools`, `csv`, `hashlib`, `unittest`, `io`,
`sys`, `os`, `glob`, `random`, `textwrap`, `uuid` та інші.

Немає теж чимало: `subprocess`, `tempfile`, `unittest.mock`.

## Чим це коштувало

Mellum2-12B, перший прогін: вісім ходів, 8644 токени, **порожня тека**.

    the program did not run: import: ImportError: no module named 'subprocess'
    the program did not run: import: ImportError: no module named 'tempfile'
    the program did not run: import: ImportError: no module named 'unittest.mock'

Модель писала звичайний Python — і тричі натикалась на межу, про яку їй сказали
неправду. Прогін записався `finished`, бо після цього вона здалась.

## Чому це особливо прикро

Промт сказав **менше**, ніж є. Модель, яка повірила б буквально, не взяла б
навіть `json` — і мусила б робити все руками.

А модель, яка не повірила (Mellum), стала гадати навмання — і вгадала тричі
неправильно.

**Обидві поведінки погані, і обидві спричинені одним реченням.**

## Лік

Сказати, що є, і чесно назвати, чого немає:

    Much of the standard library is here: json, re, pathlib, math, datetime,
    collections, itertools, functools, string, csv, hashlib, random, textwrap,
    unittest. Some of it is not: there is no subprocess, no tempfile, no
    unittest.mock. An import that is not available says so and lists what is —
    read it and use something else.

Останнє речення важливе: **повний перелік уже є в самій помилці Pyex**, і
переписувати його в промт означало б завести друге джерело правди, яке
розійдеться з першим на наступній версії бібліотеки.

## Це той самий урок, що вже був сьогодні

`BUG-ne-nazyvay-chogo-nemaie.md` — про те, що заперечення називає предмет.
Тут навпаки: **замовчування теж бреше**. Сказати «нічого немає» простіше, ніж
перелічити, — і саме тому це коштувало прогону.


---

## Лік слушний, але причина була не в ньому

Прогін після правки: промт **прямо називає** `subprocess`, `tempfile` і
`unittest.mock` як відсутні (відбиток `62d8c4e0bae5`, перевірено). Модель усе
одно бере всі три:

    ImportError: no module named 'subprocess'
    ImportError: no module named 'tempfile'
    ImportError: cannot import name 'mock' from 'unittest'

Тобто **читає й робить своє** — так само, як Laguna з `shell` після сімнадцяти
прямих вказівок, і Mellum зі `str_replace_editor`
([`CHUZHI-INSTRUMENTY-z-navchannya.md`](CHUZHI-INSTRUMENTY-z-navchannya.md)).

**Що лишається правдою:** промт таки брехав, і казати неправду про середовище не
можна. Правка потрібна.

**Що виявилось хибним:** мій висновок, ніби через неї Mellum і не створила
файлів. Причина глибша — навчений рефлекс, якого текстом не переважити.

Це вже **третій** випадок за добу, коли пряме слово в промті нічого не міняє.
Візерунок: **промт лікує незнання, а не звичку.**
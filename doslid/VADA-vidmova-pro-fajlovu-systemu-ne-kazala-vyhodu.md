# Вада: відмова про файлову систему називала стан, а не вихід

**Коли:** 23 серпня 2026, 21:46 UTC
**Де:** `KeelAgent.Sandbox.Python.broken/1`
**Ціна:** Mellum — 3 ходи, 9.7 секунди, жодного файлу

## Що модель писала

```python
import pathlib
wave_path = pathlib.Path('keel/waves/0001-tally.md')
wave_content = wave_path.read_text()
```

## Що діставала

```
the program did not run: python: OSError: no filesystem configured (line 6)
```

## Що з цього виснувала

> The program did not run due to an operating system error indicating that no
> filesystem is configured. This typically happens when the Python environment
> is not properly set up.

Тобто прочитала правду про пісочницю як звістку про зламане середовище — і
закінчила сесію, не спробувавши інакше.

## Чому промт не врятував

Промт `code-python` каже: «`import tool` gives you the tools below — that is
the only way to touch files or run commands». Сказано прямо. Але модель шукала
причину **в мить помилки**, а не в промті, прочитаному сорок секунд тому.

Це вже третій випадок того самого класу за добу: `said([])`, `instead/1`, і
тепер цей. Правило одне: відмова, що називає лише стан, коштує прогону.

## Лік

```
python: OSError: no filesystem configured (line 5) — files are reached through
the tools, not through open() or pathlib: `text, err = tool.read("path")` to
read, `tool.write("path", text)` to write.
```

Перевірено прямим викликом на живому Pyex і тестом. 467 тестів зелені.

## Побічно

Ця вада вилізла тільки після ліку двох каналів: доти Mellum обходився
натівними викликами `read`/`write`, і до Python-коду справа не доходила. Один
лік відкриває наступний — це не привід не лікувати, а привід міряти після
кожного.

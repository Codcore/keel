# Нова редакція системного промту — на завірення

Підстава — вісім моделей на двох випробуваннях, 23 серпня 2026. Кожна зміна
має за собою зміряний провал, не здогад.

---

## 1. Суперечність про блоки — ВЖЕ ЗМІНЕНО

**Було** (два місця, за двадцять рядків одне від одного):

> Two blocks in one turn is not allowed: it is refused and the turn is wasted.

> …fit: **one turn that does a lot** beats ten that each do one line.

Модель читає друге й кладе в хід усе, що має. Devstral написала **вісімнадцять**
блоків, тоді чотири, тоді три, тоді знову чотири — і згоріла, не створивши
жодного файла. Ми ніде не сказали, **як** зробити багато за хід.

**Стало:**

> To do several things at once, put them in the SAME program, one after
> another — not in several blocks:
>
> ```lua
> tool.write("a.py", code)
> local out = tool.run("pytest", {"a.py"})
> return out.output
> ```
>
> Turns follow one another. Whatever does not fit, do next turn.

і замість «one turn that does a lot»:

> one program that does several things beats ten that each do one line.
> That means a longer program — not more blocks.

---

## 2. Екранування в довгих рядках — ПРОПОНУЮ

**Було:** нічого.

**Ціна:** гемма 8 ходів, Qwen3.6 — 27 ходів і 22 892 токени. Обидві писали
`f.write(line + "\\n")` усередині `[[...]]`, дивувались, чому файл виходить
одним рядком, і шукали ваду в Python.

**Пропоную додати** до опису мови:

> Inside `[[ ]]` nothing is escaped: `\\n` there is a backslash and an `n`, not
> a newline. To put code in a file, write real line breaks inside the brackets.

---

## 3. Lua — мова керування, не продукту — ПРОПОНУЮ

**Було:** «You write a program in Lua 5.3» — і більше нічого про те, якою
мовою писати **замовлену** річ.

**Ціна:** Mellum написала список покупок мовою Lua, кличучи `tool.read`
усередині продукту. Гемма почала так само й схаменулась аж на середині ходу.

**Пропоную додати** одразу за першим рядком — формулювання ваше, воно точніше
за моє перше:

> **Lua is the tool-calling language in this environment.** It is not the
> language of what you build. Unless the person says otherwise, write their
> program in whatever language fits the request, and use Lua only to put it on
> disk and run it.

Чому це краще за «how you talk to this project»: слово **tool-calling** прямо
називає роль, і модель, навчена на native tool calling, впізнає її одразу.
«Talk to this project» лишало б місце для тлумачення «пиши тут усе».

---

## 4. Порожній проєкт — ВІДХИЛЕНО, зроблено інакше

**Пропонував** додати в промт: «if the project is empty, that is the starting
point: create what you need».

**Відхилено, і слушно:** такий рядок стоїть у промті **завжди**. Модель, яка
побачить порожню теку в будь-якій іншій роботі, почне творити, хоч її не
просили.

**Зроблено натомість** — сказано у виводі самого інструмента:

```
(empty)  →  Empty directory
```

Різниця не в довжині. «(empty)» модель читає як службову позначку й питає ще
раз; «Empty directory» — як стан світу, про який їй щойно сказали.

Це той самий випадок, що з `max_tokens` і параметрами вибірки: сказати правду
там, де вона потрібна, дешевше за правило, яке діє всюди.

**Межа названа:** `tool.glob`, який нічого не знайшов, покаже те саме слово.
Це неточно; виправляти треба там, де відомо, хто повернув результат.

## 5. Перелік дозволених команд — ПРОПОНУЮ

**Було:** перелік їде в промт дослівно, з `python3 keel/keel.py next|check|…`
і п'ятьма `mix`.

**Ціна:** Devstral вирішила, що проєкт на Elixir, і написала `mix.exs` із
ExUnit. Laguna й GLM шукали неіснуючий `keel/keel.py`.

**Пропоную** давати кожному випробуванню свій перелік — для `0002-shopping`
це `python3`, `pytest`, `git`. І додати рядок:

> This list says what may run, not what the project contains.

---

## Чого НЕ чіпаю

- **сам поділ на ходи** — правило «один блок» лишається, воно потрібне;
- **вимогу доказу** — вона сьогодні спинила GLM, яка вигадала огляд;
- **опис інструментів** — жодна модель на ньому не спіткнулась.


---

## Підказки Legion знайшлися — і одна влучає точно в нашу ваду

Файл є: `lib/legion/prompts/system_prompt.eex`, ліцензія MIT. Спершу я його не
знайшов, бо шукав у README й на `deflua.com`; він лежить у дереві коду.

### 6. «Не завершуй хід, щоб оголосити намір» — ПРОПОНУЮ

У Legion є рядок, який лікує рівно те, на чому загинула GLM:

> Never finish just to announce what you are about to do; do it now

**Наш випадок:** GLM написала «I will create the shopping list module, its
JSON persistence layer, and the test suite from scratch, all within the single
fenced block below» — і **завершила хід**. Двічі, слово в слово. Блока не було.

Тобто автори Legion зустріли те саме живим і вилікували одним реченням.

**Пропоную додати:**

> Never end your turn just to announce what you are about to do. Do it in this
> turn's program.

### 7. Чи переживають змінні хід — ПРОПОНУЮ

У Legion сказано прямо, залежно від налаштування: «Variables do not persist»
або «Variables persist». Ми не кажемо **нічого**.

**Зміряно 23 серпня** на нашій пісочниці:

```
Sandbox.run("x = 42; return x")  →  {:ok, 42}
Sandbox.run("return x")          →  {:ok, [nil]}
```

Не переживають. Модель, яка припустить протилежне, напише другий хід, що
спирається на змінну з першого, — і дістане `nil` без пояснення.

**Пропоную додати:**

> Each turn starts with a clean slate: variables from a previous turn are gone.
> What must survive, write to a file.

### Чого в них узяли НЕ будемо

Legion віддає моделі **вихідний код інструментів** (`tool_contents`). Рішення
25 це відкинуло, і слушно: наші інструменти несуть усередині ланцюг, і моделі
з тих сотень рядків нема чого взяти.

## Що з інших джерел

| джерело | що там є |
|---|---|
| `legion/lib/legion/prompts/system_prompt.eex` | **є підказки** — дві взяли, див. вище |
| `deflua.com` (Lua.ex) | «agent-ready», взірець `deflua` — порад немає |
| `tv-labs/lua` README | одна згадка про агентів, і все |

Решта — наші власні заміри: вісім моделей, два випробування, чотирнадцять
записаних вад.

# Модель плутає мову інструментів із мовою продукту

**Спіймано:** 22 серпня 2026, випробування `0002-shopping`, Mellum2-12B.

## Що сталося

Людина попросила «зроби мені командну штуку для списку покупок». Модель
почала писати цей список покупок **мовою Lua** — і всередині продукту кликати
`tool.read` і `tool.write`:

```lua
-- shopping_list.lua
local LIST_FILE = "shopping_list.txt"

local function load_list()
  local text, err = tool.read(LIST_FILE)   -- ← агентський інструмент
  ...
local function save_list(list)
  local result, err = tool.write(LIST_FILE, content)   -- ← і тут
```

Це не працюватиме ніколи: `tool.*` живуть лише в нашій пісочниці, у
самостійній програмі їх немає. Та й Lua користувач не просив.

## Чому це наша вада

Системний промт каже:

> You write a program in Lua 5.3. It runs in an empty virtual machine…

Це сказано про **спосіб керувати інструментами** — про те, як модель звертається
до нас. Але з погляду моделі, яка щойно прочитала прохання «зроби мені штуку»,
це читається як «пиши мовою Lua».

Двох різних речей — мова керування й мова замовленого продукту — промт не
розрізняє **зовсім**. Слова «продукт», «проєкт», «те, що ти пишеш для людини»
в ньому немає.

## Чого це коштує

Модель витрачає ходи на код, який не може працювати, і людина дістає
непрацездатну річ. Для випробування, де завдання — написати проєкт з нуля, це
вада першого порядку: вона псує саме те, що ми міряємо.

## Що мало б бути в промті

Приблизно так — окремим абзацом, поруч із описом діалекту:

> Lua is how you talk to this project — how you read, write and run things.
> It is not the language of what you build. Unless the person says otherwise,
> write their program in whatever language fits the request, and use Lua only
> to put it on disk and run it.

**Не полагоджено.** Це зміна системного промту, тобто нова норма в
`keel-agent`, а не вада в межах прогону. Чекає на рішення.

## Гемма впала в ту саму пастку — і вибралась сама

Другі ваги, той самий перший хід. Спершу пише програму **мовою Lua** й кладе
її у файл із розширенням `.py`:

```lua
local function main()
  local file = "shopping_list.txt"
  local f = io.open(file, "r")      -- Lua
  ...
end
main()
]]
tool.write("shopping_list.py", app_code)   -- ← Lua-код у файлі .py
```

А тоді схаменається — просто в коментарях, посеред того самого ходу:

```lua
-- Wait, the user asked for a command-line thing.
-- Python is easier for a quick "runnable" thing that works with standard CLI patterns.
-- However, I am in a Lua environment. I will write the script as a python script
-- because 'tool.run' supports python3…
-- Actually, I'll just write a single python script that acts as the CLI.
```

І переписує на Python — уже правильно.

**Двоє з двох.** Mellum не вибралась зовсім і спалила всі сорок ходів; гемма
вибралась, але ціною половини ходу й зайвого файла зі сміттям.

Це вже не властивість однієї моделі. Промт справді не розрізняє двох мов, і
кожна модель мусить здогадатись про це сама.

## Скільки ще

Прогін іде всіма вісьмома. Кожні наступні ваги — ще одне свідчення; підсумок
буде в записі випробування.

# Вада: curl зберіг сторінку «Entry not found» як файл моделі

**Коли:** 24 серпня 2026, 02:00 UTC
**Де:** моє завантаження Nemotron чотирма потоками
**Ознака:** сервер не піднявся з `Unrecognized processing class`

## Що сталось

`hf download` стояв на нулі, тож 17 ГБ я качав `curl`-ом, перебираючи імена
файлів списком:

```bash
for f in config.json generation_config.json chat_template.jinja tokenizer.json \
         tokenizer_config.json model.safetensors.index.json special_tokens_map.json; do
  curl -sL -o "$D/$f" "$B/$f"
done
```

`special_tokens_map.json` у цьому репозиторії **немає**. HuggingFace віддав
404 із тілом `Entry not found`, а `curl -o` слухняно поклав ці 15 байтів у
файл із потрібним іменем.

Далі `AutoTokenizer` спіткнувся об нього:

```
JSONDecodeError: Expecting value: line 1 column 1 (char 0)
```

а сервер переклав це в `400: Failed to load model: Unrecognized processing class`
— повідомлення, з якого справжньої причини не видно взагалі.

## Лік

Прибрати файл, якого не має бути. Токенізатор завантажився одразу:
`TokenizersBackend`.

## Як не повторити

`curl -f` (або `--fail`) вертає помилку замість того, щоб зберігати тіло
404-ї. Із ним цикл просто не створив би файлу.

Перелік файлів треба брати з `/api/models/<id>` — він у нас уже був
надрукований, і `special_tokens_map.json` у ньому **не значився**. Я взяв
список із голови, а не з відповіді, яку сам же отримав.

## Побічно

`mlxsrv_lenient.py` (VLM-сервер) для цієї моделі не годиться й без того: вона
текстова, а він шукає процесор зображень. Тип у `MODELS` виправлено на `lm`.

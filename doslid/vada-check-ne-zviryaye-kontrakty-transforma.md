# `check` не звіряє контракти трансформа з тим, що доводять його сценарії

**Знайшла модель, не ми.** 25 серпня 2026 Laguna XS 2.1 писала хвилю
`0001-parse-readings` і сама помітила:

> I notice an issue — the scenarios prove contracts, but the transform needs to
> declare those contracts in its `contracts` field.

**Що в документі.** Вісім сценаріїв, усі доводять один контракт:

    scenarios:
      parse_raw_readings_line: {proves: parse-readings-readings@0e06aa}
      parse_readings_with_comma_delimiter: {proves: parse-readings-readings@0e06aa}
      … ще шість такого ж

    transforms:
      parse-readings-encoding:
        implements:
          - parse_raw_readings_line
          - … ті самі вісім
        contracts: []          ← ось воно
        files:
          - lib/voda/readings_parser.ex
          - test/voda/readings_parser_test.exs

Трансформ реалізує вісім сценаріїв, кожен із яких доводить контракт, — а сам
каже, що контрактів не має.

**Що кажуть наші перевірки.**

    $ keel gaps
    the plan is complete: 0001-parse-readings

    $ keel check
    ✓ 1. references lead somewhere
    ✓ 2. depends_on without cycles
    ✓ 3. contract revisions match
    ✓ 4. changed files match those declared
    ✓ 7. names in the header match the headings
    clean

Жодна не зачепила. №1 дивиться, чи посилання ведуть кудись, — порожній
перелік не веде нікуди й тому бездоганний. №3 звіряє редакції тих контрактів,
які **названі**; неназваних вона не бачить.

**Чого бракує.** Перевірки, яка питає: чи кожен контракт, що його доводять
сценарії трансформа, оголошений у `contracts` того самого трансформа. І,
можливо, зворотної: чи не оголошено контракт, якого не доводить жоден
сценарій.

**Чому це важить.** `contracts` трансформа — те, за чим §6 звіряє, чи виконано
обіцяне, коли з'явиться код. Порожній перелік означає, що звіряти буде нічого:
хвиля пройде `check` на гілці плану, а на гілці роботи мовчки не перевірить
контракт, який сама ж і назвала в сценаріях.

**Де лагодити.** У самому `keel`, не в стенді: це перевірка методики, а не
властивість досліду. З тестами.

# Вада скарги: «has no proves» там, де `proves` порожній

## Що в файлі

    scenarios:
      parse_correct_line: {proves: []}
      parse_line_with_extra_spaces: {proves: []}

## Що каже `gaps`

    keel/waves/0001-data-ingestion.md:5  scenario parse_correct_line has no proves

## Чому це вада

Поле `proves` **є**. Порожній список — не те саме, що відсутнє поле.
Модель бачить у файлі `proves`, читає «has no proves» і не може узгодити
одне з одним.

Granite 4.2 (повна думка, 26 серпня 2026) на цьому кружляла: чотири рази
поспіль «create a contract» в одному ході, хоч контракт уже лежав на диску.
Її власна здогадка вловила причину дослівно:

    being interpreted as "no proves" rather than an explicit empty list

Тобто скарга описує стан, якого у файлі немає, і не називає дії.

## Як мало б бути

Сказати, що список порожній, і чого від нього хочуть:

    scenario parse_correct_line proves nothing — name the contract it
    leans on, as {proves: reader@<rev>}

Тоді дія очевидна з самої скарги, без здогадів.

## Не зроблено

Правка заслону — зміна кодексу. Рішення за Андрієм.

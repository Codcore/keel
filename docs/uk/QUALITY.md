# Quality model

Сорок розрізів під девʼятьма заголовками, якими проходить кожен крок, і файл,
на який Keel показує, коли каже «пройди розрізи».

Це характеристики якості продукту з **ISO/IEC 25010:2023**, у порядку самого
стандарту, а підхарактеристики — питання. Тут нічого не вигадано: цінність
стандартного списку в тому, що його писали люди, які вже щось забували, і сенс —
перестати залежати від того, що саме спало на думку конкретному агентові.

## Коли це читається

**Один прохід на крок** — там, де пишуться сценарії. Кожен розріз питається про
те, що планується, і те, що він виявляє, стає сценарієм або рішенням. Не на
кожному рівні й не до збіжності: рівнів у Keel немає, є крок.

Перед pull request список **не проходять удруге**. Там питання вужче й без
переліку: не «що ще має бути правдою», а «про що ми промовчали». Знайдене
закривається до PR, як і будь-що інше.

Два повні проходи на той самий крок дають майже ті самі відповіді за подвійну
ціну, і саме так списки перестають читати.

## Як відповідати на розріз

Одна з трьох відповідей, і тільки одна:

- **не стосується** — з реченням, чому. Розріз про людину за інтерфейсом не
  стосується файла збірки;
- **відповіли** — назвавши сценарій, який відповідає. Сценарій, що доводить
  вужче, ніж питає розріз, — це не відповідь, це наступний випадок;
- **промовчали** — розріз доречний, ніщо його не закриває, і жодне рішення від
  нього не відмовляється. Скажи, що конкретно може піти не так на цьому проєкті,
  і напиши сценарій, який це закриє.

**Розріз, який доречний, а відповідь на нього свідомо «ні», — це рішення.**
«Резервні копії не в цьому кроці» — це відновлюваність, названа вголос. Мовчання
є тим, що цей список припиняє; відмова — ні.

**Перевір, що прийшло з бібліотекою, перш ніж казати, що чогось бракує.** Три
агенти підряд повідомили, що в рушія синхронізації немає перевірки живості; усі
троє читали файл проєкту, і жоден не заглянув в образ, де вона була вбудована.

**І те саме в другий бік: межа перевіряється проти залежності так само, як
розріз.** «Ми свідомо цього не робимо» — це теж твердження про чужий код, і воно
буває неправдивим із моменту, коли його написали. Крок обіцяв межею «повторів
немає», а бібліотека мала власний лічильник на три спроби: один хід ішов чотирма
запитами. Ніхто не брехав — повтори приїхали усталеним значенням. Перш ніж
казати, що чогось **немає**, подивіться, чи його справді немає.

## Сорок

Сорок питань під девʼятьма заголовками — увесь ISO/IEC 25010:2023. Написано
списком, бо його проходять, а не читають: розріз, проминутий не глянувши, — це
те, проти чого файл існує.

### 1. Functional suitability

- **completeness** — is everything that was asked for here
- **correctness** — is the result right
- **appropriateness** — does this make the actual task easier, rather than a neighbouring one

### 2. Performance efficiency

- **time behaviour** — how long does it take
- **capacity** — how much does it hold before it stops working
- **resource utilisation** — what does it consume while it works

### 3. Compatibility

- **co-existence** — what does this take from whatever else runs on the same machine
- **interoperability** — what does it have to agree with to work at all

### 4. Interaction capability

- **appropriateness recognisability** — can a person tell what it is for
- **learnability** — can they learn it
- **operability** — can they drive it
- **user error protection** — does it stop them making a mistake
- **user engagement** — does it hold their attention
- **inclusivity** — does it work for people who read, see or move differently
- **user assistance** — does it help when they are stuck
- **self-descriptiveness** — does it explain itself

### 5. Reliability

- **faultlessness** — is it wrong in ordinary use
- **fault tolerance** — does it survive its dependencies failing
- **availability** — is it there when it is needed
- **recoverability** — what does it take to get it back after it stops

### 6. Security

- **confidentiality** — who else can see this
- **integrity** — who else can change it
- **non-repudiation** — can an act be denied afterwards
- **accountability** — is it visible who did what
- **authenticity** — is the caller who they say they are
- **resistance** — what does it do to somebody trying

### 7. Maintainability

- **modularity** — is it in one piece or several
- **reusability** — is any of it usable elsewhere
- **analysability** — can a person find out what broke
- **modifiability** — how much has to move to change it
- **testability** — can it be tested at all

### 8. Flexibility

- **adaptability** — does it survive a change of environment
- **scalability** — does it survive more of everything
- **installability** — what does installing it take
- **replaceability** — what does replacing it take

### 9. Safety

- **operational constraints** — what must never happen while it runs
- **risk identification** — which of those are known
- **fail safe** — what does it do when it fails
- **hazard warning** — does it say so before harm
- **safe integration** — what does adding it to a running system risk

**Не кожен розріз стосується кожного кроку, і вдавати протилежне — найкоротший
шлях до того, що список перестануть читати.** Розріз, якому нема чого сказати,
не лишає по собі нічого: ні сценарію, ні рядка, ні нотатки. Ціна списку — один
прохід; купує він те, що розріз, якого ніхто не придумав, тепер не можна
проминути мовчки.

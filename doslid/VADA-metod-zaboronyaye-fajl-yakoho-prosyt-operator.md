# Метод забороняє файл, якого просить оператор

**Знайдено** 28 серпня 2026 — моделлю, не мною. Ornith-1.5-9B на
`0005-voda-plan` назвала це «a genuine tension» і витратила на розв'язання
частину ходу.

## Дві цитати з одного випробування

`fixture/keel/METHODOLOGY.md`, Appendix B «What deliberately does not exist»:

> No requirements, **no questions**, no journal, no statuses, no tags, no
> numbers inside a wave, no decision files. A promise is written once — as a
> scenario; a question lives for hours in the discussion of a pull request
> rather than for years in the graph.

`prompt.md`, слово оператора:

> **The operator is away.** The skill says to ask wherever there is a fork —
> there is nobody to ask, so write every question into `keel/QUESTIONS.md`

Метод оголошує файли питань тим, чого свідомо не існує. Оператор просить
завести саме такий файл.

## Кого це вже зачепило

**Qwen3.8-27B**, прогін того ж дня. Дійшла до правильного висновку сама, але
не безкоштовно:

> this is part of the session framework ("The operator is away"), not part of
> the plan itself. So I'll create `keel/QUESTIONS.md` in that form

**Ornith-1.5-9B**, цей прогін:

> The system prompt says "write every question into keel/QUESTIONS.md." But
> the method (which is normative, §1.1) says "no questions" files.
> Hmm, this is a genuine tension.

Обидві впорались, обидві заплатили ходами за суперечність, яку створили ми.

## Чому це наша вада, а не їхня

Модель, що читає METHODOLOGY як норму, мусить спинитись на цьому місці — і
це правильна поведінка. Ми ж вимагаємо від неї порушити писане правило, не
сказавши, що це виняток.

## Лік уже є поруч — для іншого файлу

Про `PLAN.md` у тому ж `prompt.md` сказано прямо: **«This file is not part of
the method»**. І цей рядок працює: Ornith процитувала його й пішла далі без
жодних вагань:

> It's explicitly outside the method ("This file is not part of the method")

Про `QUESTIONS.md` такого речення немає — тому саме на ньому обидві моделі й
спинились. Тобто лік не треба вигадувати: досить дописати те саме речення,
яке вже стоїть двома абзацами вище й уже доведено діє.

Правити треба `prompt.md` самого випробування, тож не під час прогону: текст
задачі не можна міняти на ходу.

Пов'язане: [[VADA-skarhy-nazyvayut-vadu-ne-diyu]]

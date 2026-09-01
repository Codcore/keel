# Чи законне «N/A» у розрізах якості — метод уже відповів

## Що побачено

Granite 4.2 у режимі low-effort, плануючи хвилю `parse-readings`, відповіла
на заголовки `QUALITY.md` так:

    - user error protection: invalid lines are logged and skipped, not raised
    - user engagement: N/A (no UI)
    - inclusivity: N/A (no UI)
    - user assistance: error messages specify which line failed and why

## Відповідь методу

`METHODOLOGY.md` §10.3 і `QUALITY.md` це вже покривають:

> Every cut gets exactly one of three answers: does not apply — with the
> reason; answered — naming the scenario that answers it; silent.

> **does not apply** — with a sentence saying why. A cut about the person at
> the interface does not apply to a build file.

Отже відмова законна, але **з причиною**, і приклад у `QUALITY.md` — саме
такий: розріз про людину біля інтерфейсу не стосується файлу збірки.
Парсер тут того самого роду.

## Що лишається відкритим (дрібне)

`N/A (no UI)` формою відповідає правилу: відмова названа, причина є. Питання
лише в тому, чи має `gaps` перевіряти, що причина **написана**, а не лише
що рядок непорожній — бо голе `N/A` без дужок метод порушує, а перевірка
цього не побачить.

Це вже не про метод, а про суворість заслону. Нічого не міняю без дозволу.

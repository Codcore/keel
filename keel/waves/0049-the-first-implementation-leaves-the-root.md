---
depends_on: [0046-the-fifth-tongue]

transforms:
  the-first-implementation-leaves-the-root:
    chore: "keel.py and the Python tests of the first implementation leave the root; one tool in the repository, and the README says which"
    files:
      - keel.py
      - tests/__init__.py
      - tests/support.py
      - tests/test_adapters.py
      - tests/test_agent_hooks.py
      - tests/test_agents.py
      - tests/test_ci.py
      - tests/test_commands.py
      - tests/test_documents.py
      - tests/test_git_hooks.py
      - tests/test_install.py
      - tests/test_language.py
      - tests/test_modes.py
      - tests/test_mutation.py
      - tests/test_scope.py
      - tests/test_setup.py
      - tests/test_skills.py
      - tests/test_yamlish.py
      - README.md
      - BACKLOG.md
      - keel/reviews/0049-the-first-implementation-leaves-the-root.md

decisions:
  functional.completeness: "не застосовується"
  functional.correctness: "не застосовується"
  functional.appropriateness: "не застосовується"
  performance.time-behaviour: "не застосовується"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "не застосовується"
  compatibility.interoperability: "не застосовується"
  interaction.appropriateness-recognisability: "не застосовується"
  interaction.learnability: "не застосовується"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "не застосовується"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується"
  interaction.user-assistance: "не застосовується"
  interaction.self-descriptiveness: "свідомо без тесту: README і BACKLOG кажуть, що перша реалізація вийшла з кореня і де її шукати — у git до цієї хвилі"
  reliability.faultlessness: "свідомо без тесту: батарея крейта не читає tests/ кореня — 177 зелених до і після, і це міряє сама батарея"
  reliability.fault-tolerance: "не застосовується"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується"
  security.confidentiality: "не застосовується"
  security.integrity: "не застосовується"
  security.non-repudiation: "свідомо без тесту: видалення — один коміт із назвою трансформи, і його diff — сама подія; історія v1 не переписується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "не застосовується"
  maintainability.modularity: "не застосовується"
  maintainability.reusability: "не застосовується"
  maintainability.analysability: "свідомо без тесту, і це вся хвиля: два інструменти в одному репозиторії — keel.py першої реалізації (312 КБ) і крейт tool/ — і жоден суд не казав, який чинний; після хвилі в корені один інструмент, і README каже це"
  maintainability.modifiability: "свідомо без тесту: видалення сирців, не документів — §4.12 забороняє видаляти документи методики, а keel.py і tests/*.py — код v1, чия історія лишається в git"
  maintainability.testability: "не застосовується"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується"
  safety.risk-identification: "не застосовується"
  safety.fail-safe: "не застосовується"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "свідомо без тесту: жоден workflow, hook чи скрипт не кличе keel.py (grep по .yml/.sh/.toml/.rs — нуль згадок поза документами); docs/uk/README.md і NOTES-ROZBIR згадують його як історію, і лишаються"
---

## Why

Хвиля 0037 і оновлення README назвали одне й те саме: у корені лежать
два інструменти — `keel.py` першої реалізації (312 КБ) разом із її
Python-тестами у `tests/` (17 файлів), і крейт `tool/`, яким живе все
з хвилі 0001. Жоден суд не каже, який чинний; README описує лише
крейт. Черга каже: «це видалення, тож окремою хвилею» — це вона.

**Зміряно:** `grep` по `.yml`, `.sh`, `.toml`, `.rs` — жодної згадки
`keel.py` поза документами; батарея крейта `tests/` кореня не читає
(адаптер cargo читає `tool/tests/`); `pyproject.toml`/`pytest.ini` нема,
тож ці тести ніхто не жене вже понад сорок хвиль. Згадки в
`docs/uk/README.md`, `NEW-CONCEPT.md`, `NOTES-ROZBIR.md` — історія
(«поки його не сховали»), і документи не видаляються (§4.12).

Легка хвиля (§6.8): одна chore-трансформа, контрактів не чіпає, нічого
не знімає — одна гілка, один merge. Видалення — код, не документи.

## transform: the-first-implementation-leaves-the-root

`git rm keel.py tests/` — 18 файлів; README дістає рядок про те, що
перша реалізація вийшла з кореня і живе в історії до цієї хвилі;
BACKLOG знімає два рядки про два інструменти в одному репозиторії.

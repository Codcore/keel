---
depends_on: [0014-field-frame]

transforms:
  one-home:
    chore: "the debts of two refusals aloud come due: the §1.2 slug predicate (three byte-identical copies, 0013 R-5) and the 0013 write school (two copies, 0014 review) each move into one home; behaviour changes nowhere"
    files:
      - tool/src/docs.rs
      - tool/src/gate.rs
      - tool/src/plan.rs
      - tool/src/init.rs
      - docs/uk/V2-PROCESS.md

decisions:
  functional.completeness: "не застосовується: жодної нової поведінки — зведення копій в один дім"
  functional.correctness: "свідомо без нового тесту: поведінка не міняється ніде — тримає наявна батарея 67 тестів усіх попередніх хвиль"
  functional.appropriateness: "не застосовується: обвʼязка, оплата двох відмов уголос"
  performance.time-behaviour: "не застосовується: ті самі виклики, менше копій"
  performance.capacity: "не застосовується"
  performance.resource-utilisation: "не застосовується"
  compatibility.co-existence: "свідомо без нового тесту: тексти контрактів не міняються — редакції стоять як стояли"
  compatibility.interoperability: "не застосовується: git і cargo не чіпаються"
  interaction.appropriateness-recognisability: "не застосовується: жодної нової команди чи слова"
  interaction.learnability: "не застосовується"
  interaction.operability: "не застосовується"
  interaction.user-error-protection: "свідомо без нового тесту: ті самі відмови тими самими словами — ключі i18n не міняються"
  interaction.user-engagement: "не застосовується"
  interaction.inclusivity: "не застосовується: текстів не додано"
  interaction.user-assistance: "не застосовується"
  interaction.self-descriptiveness: "не застосовується"
  reliability.faultlessness: "свідомо без нового тесту: один дім замість трьох копій — менше місць для розбіжности; батарея тримає"
  reliability.fault-tolerance: "не застосовується: шляхи відмов ті самі"
  reliability.availability: "не застосовується"
  reliability.recoverability: "не застосовується: школа запису та сама, лише в одному домі"
  security.confidentiality: "не застосовується"
  security.integrity: "не застосовується: жодного нового запису"
  security.non-repudiation: "не застосовується"
  security.accountability: "не застосовується"
  security.authenticity: "не застосовується"
  security.resistance: "свідомо без нового тесту: предикат §1.2 той самий байт-у-байт, лише один"
  maintainability.modularity: "свідомо без нового тесту: сенс хвилі — питати, не переказувати (дух §1.6, конституція п.6); тримають борги-відмови 0013 R-5 і 0014"
  maintainability.reusability: "не застосовується: pub(crate), не публічна поверхня"
  maintainability.analysability: "свідомо без нового тесту: одне правило — один дім"
  maintainability.modifiability: "свідомо без нового тесту: зміна предиката чи школи запису тепер в одному місці"
  maintainability.testability: "не застосовується: тести не міняються"
  flexibility.adaptability: "не застосовується"
  flexibility.scalability: "не застосовується"
  flexibility.installability: "не застосовується"
  flexibility.replaceability: "не застосовується"
  safety.operational-constraints: "не застосовується: жодного бігу"
  safety.risk-identification: "не застосовується: загрозу (тихе розходження копій) хвиля і знімає"
  safety.fail-safe: "не застосовується"
  safety.hazard-warning: "не застосовується"
  safety.safe-integration: "свідомо без нового тесту: чотири файли, жодного нового; контракти-власники тримають свої exports незмінними — суд форми §7.6 стоїть на варті"
---

## Why

Перша легка хвиля власного проєкту (§6.8): одна chore-трансформа,
жодного контракту, жодного сценарію — один PR, закриття фактом
merge. Вона платить борги двох відмов уголос: рецензія 0013 R-5
назвала три байт-у-байт копії предиката §1.2 (docs, gate, plan) і
дістала відмову «зведення — окремою хвилею»; рецензія 0014 назвала
другу копію write_new (plan, init) — та сама відмова. Це та хвиля:
предикат слага їде в один дім docs (двері документів і власник
§1.2), школа запису лишається в домі народження plan — gate, plan
та init віднині питають, не переказують. Поведінка не міняється
ніде — батарея 67 тестів тримає це словами всіх попередніх хвиль.

Відступи bootstrap, названі вголос: хвиля їде робочою гілкою сесії;
план затверджено словом оператора наперед (§8.6, стояче слово в
журналі 2026-09-02); журнальний запис їде цією ж chore.

## transform: one-home

docs::slug_ok відкривається pub(crate) — gate і plan викидають
власні копії і питають docs; plan::write_new відкривається
pub(crate) — init викидає власну копію і питає plan (відмова
запису перекладається в рядок звіту словами причини). Тексти
контрактів не міняються — редакції tool-docs, tool-gate,
tool-plan, tool-init стоять як стояли.

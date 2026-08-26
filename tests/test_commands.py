#!/usr/bin/env python3
"""new, gaps, next, rev, check — the commands a person runs."""

import json
import os
import re
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args, CONTRACT, ProjectCase, WAVE  # noqa: E402




class TestNext(ProjectCase):
    def run_next(self, **kwargs):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_next(self.project, Args(json=False, **kwargs))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def test_on_main_it_names_the_wave_that_is_ready(self):
        """Було «гілка не названа за кроком» — правда, але не відповідь."""
        code, out = self.run_next()
        self.assertEqual(code, 1)
        self.assertIn("git checkout -b 0001-session-loop", out)

    def test_a_branch_that_is_neither_main_nor_a_wave_still_says_the_rule(self):
        self.fixture.branch("spike/whatever")
        code, out = self.run_next()
        self.assertEqual(code, 1)
        self.assertIn("is not named after a wave", out)

    def test_on_a_plan_branch_it_answers_about_the_plan(self):
        """Роботи тут немає — але «немає роботи» не те саме, що «нема що робити».

        Досі це була відмова з відсиланням до `gaps`. 25 серпня 2026 виявилось,
        що на ПОВНОМУ плані обидва порадники кивали один на одного: `gaps` казав
        «the plan is complete», `next` — «keel gaps says what is missing». Тепер
        повний план дістає наступний крок, а неповний — лік того, чого бракує
        (див. TestNextAnswersOnAFinishedPlan).
        """
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.run_next()
        self.assertEqual(code, 0)
        self.assertIn("pull request", out)

    def test_refuses_while_plan_is_not_in_main(self):
        self.fixture.branch("plan/0002-later")
        self.fixture.write("keel/waves/0002-later.md", WAVE.format(rev=self.fixture.contract_rev))
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план кроку 2")
        self.fixture.git("checkout", "-b", "0002-later")
        code, out = self.run_next()
        self.assertEqual(code, 1)
        self.assertIn("is not on main yet", out)

    def test_package_has_files_scenario_and_contract(self):
        self.fixture.branch("0001-session-loop")
        code, out = self.run_next()
        self.assertEqual(code, 0)
        self.assertIn("# drive-turns", out)
        self.assertIn("lib/session.ex", out)
        self.assertIn("**Then** розмова завершується.", out)
        self.assertIn("Одна розмова з однією моделлю.", out)
        self.assertIn("Demo.Session", out)
        self.assertIn("drive-turns: <what was done>", out)

    def test_package_names_the_tag_to_write(self):
        self.fixture.branch("0001-session-loop")
        _, out = self.run_next()
        self.assertIn(f'proves: :finishes_when_no_tool_called, rev: "{self.fixture.scenario_rev()}"',
                      out)

    def test_a_longer_slug_does_not_close_a_shorter_one(self):
        """`add-more:` не має закривати трансформу `add`."""
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns-later: чуже")
        code, out = self.run_next()
        self.assertEqual(code, 0)
        self.assertIn("# drive-turns", out)

    def test_a_slug_mentioned_in_the_body_does_not_close_it(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "щось інше\n\nпоки не чіпаю drive-turns")
        code, out = self.run_next()
        self.assertEqual(code, 0)
        self.assertIn("# drive-turns", out)

    def test_closed_transform_is_not_handed_out_again(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns: перший хід")
        code, out = self.run_next()
        self.assertEqual(code, 0)
        self.assertIn("every transform", out)

    def test_json_package(self):
        import json as jsonlib
        from io import StringIO
        self.fixture.branch("0001-session-loop")
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_next(self.project, Args(json=True))
        finally:
            sys.stdout = saved
        package = jsonlib.loads(stream.getvalue())
        self.assertEqual(package["transform"]["slug"], "drive-turns")
        self.assertEqual(package["transform"]["files"], ["lib/session.ex"])
        self.assertEqual(package["contracts"][0]["module"], "Demo.Session")
        self.assertTrue(package["contracts"][0]["rev_ok"])


# ─────────────────────────────────────────────────────────────────────────────
# keel rev
# ─────────────────────────────────────────────────────────────────────────────




# ─────────────────────────────────────────────────────────────────────────────
# keel rev
# ─────────────────────────────────────────────────────────────────────────────

class TestRev(ProjectCase):
    def run_rev(self, write=False):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_rev(self.project, Args(write=write))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def test_nothing_to_do(self):
        code, out = self.run_rev()
        self.assertEqual(code, 0)
        self.assertIn("every revision matches", out)

    def test_reports_without_writing(self):
        self.fixture.write("keel/contracts/session-run.md", CONTRACT + "\nІ ще речення.\n")
        code, out = self.run_rev()
        self.assertEqual(code, 1)
        self.assertIn("→", out)
        self.assertIn(self.fixture.contract_rev, self.fixture.read("keel/waves/0001-session-loop.md"))

    def test_writes_fresh_contract_revision(self):
        self.fixture.write("keel/contracts/session-run.md", CONTRACT + "\nІ ще речення.\n")
        code, _ = self.run_rev(write=True)
        self.assertEqual(code, 0)
        self.assertEqual(keel.check_revisions(self.project), [])

    def test_writes_fresh_tag_revision(self):
        self.fixture.write(
            "test/session_test.exs",
            'defmodule Demo.SessionTest do\n'
            '  @tag proves: :finishes_when_no_tool_called, rev: "deadbe"\n'
            '  test "x", do: assert true\n'
            'end\n')
        self.run_rev(write=True)
        self.assertIn(f'rev: "{self.fixture.scenario_rev()}"',
                      self.fixture.read("test/session_test.exs"))
        self.assertEqual(keel.check_scenarios(self.project, run_tests=False), [])

    def test_replaces_nonsense_revision_instead_of_doubling_it(self):
        self.fixture.write(
            "test/session_test.exs",
            'defmodule Demo.SessionTest do\n'
            '  @tag proves: :finishes_when_no_tool_called, rev: "старий"\n'
            '  test "x", do: assert true\n'
            'end\n')
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertIn("the test holds", problems[0].message)
        self.run_rev(write=True)
        text = self.fixture.read("test/session_test.exs")
        self.assertEqual(text.count("rev:"), 1)
        self.assertNotIn("старий", text)
        self.assertEqual(keel.check_scenarios(self.project, run_tests=False), [])

    def test_write_touches_the_header_and_nothing_else(self):
        """Гола заміна підрядком нівечила імена трансформ, файли й прозу."""
        wave = "keel/waves/0001-session-loop.md"
        text = self.fixture.read(wave).replace(
            f"session-run@{self.fixture.contract_rev}", "session-run")
        text = text.replace("Крутити ходи", "Крутити ходи session-run")
        self.fixture.write(wave, text)
        self.run_rev(write=True)
        after = self.fixture.read(wave)
        self.assertNotIn("@@", after)
        self.assertIn("Крутити ходи session-run", after)
        self.assertIn("files:      [lib/session.ex]", after)
        self.assertEqual(keel.check_revisions(self.project), [])

    def test_adds_missing_tag_revision(self):
        self.fixture.write(
            "test/session_test.exs",
            'defmodule Demo.SessionTest do\n'
            '  @tag proves: :finishes_when_no_tool_called\n'
            '  test "x", do: assert true\n'
            'end\n')
        self.run_rev(write=True)
        self.assertEqual(keel.check_scenarios(self.project, run_tests=False), [])

    LATER_WAVE = """---
depends_on: []

scenarios:
  later-finishes: {{proves: session-run@{rev}}}

transforms:
  later-turns:
    implements: [later-finishes]
    contracts:  [session-run@{rev}]
    files:      [lib/later.ex]
---

## Навіщо

Друга хвиля, якої ще ніхто не закривав.

## scenario: later-finishes

**Given** порожній набір інструментів,
**When** модель відповідає текстом,
**Then** розмова завершується.

## transform: later-turns

Крутити ходи ще раз.

Межі: лічильника спроб немає.
"""

    def close_the_wave(self):
        """Хвиля закрита так, як її закриває робота: коммітом на main."""
        self.fixture.git("commit", "--allow-empty", "-m", "drive-turns: ходи закрито")

    def test_a_closed_wave_keeps_the_revision_it_proved(self):
        """Переписати її означало б сказати, що доводили проти тексту, якого не було."""
        self.close_the_wave()
        held = self.fixture.contract_rev
        self.fixture.write("keel/contracts/session-run.md", CONTRACT + "\nІ ще речення.\n")
        code, out = self.run_rev(write=True)
        self.assertEqual(code, 0)
        self.assertIn("nothing open drifted", out)
        self.assertIn(f"session-run@{held}",
                      self.fixture.read("keel/waves/0001-session-loop.md"))

    def test_a_closed_wave_is_not_reported_as_drifted_either(self):
        """Інакше гейт червонів би вічно там, де рев відмовляється писати."""
        self.close_the_wave()
        self.fixture.write("keel/contracts/session-run.md", CONTRACT + "\nІ ще речення.\n")
        self.assertEqual(keel.check_revisions(self.project), [])

    def test_an_open_wave_beside_it_is_still_restamped(self):
        """Тиша про закрите не є тишею про те, що в роботі."""
        held = self.fixture.contract_rev
        self.fixture.write("keel/waves/0002-later.md",
                           self.LATER_WAVE.format(rev=held))
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план другої хвилі")
        self.close_the_wave()
        body = CONTRACT + "\nІ ще речення.\n"
        self.fixture.write("keel/contracts/session-run.md", body)
        fresh = keel.revision(body)
        self.assertNotEqual(fresh, held)
        self.run_rev(write=True)
        self.assertIn(f"session-run@{held}",
                      self.fixture.read("keel/waves/0001-session-loop.md"))
        self.assertIn(f"session-run@{fresh}",
                      self.fixture.read("keel/waves/0002-later.md"))

    def test_the_closed_wave_is_named_out_loud_with_the_reason(self):
        """Заслон, що мовчки не робить, не відрізнити від зламаного."""
        self.close_the_wave()
        self.fixture.write("keel/contracts/session-run.md", CONTRACT + "\nІ ще речення.\n")
        code, out = self.run_rev()
        self.assertEqual(code, 0)
        self.assertIn("0001-session-loop", out)
        self.assertIn("session-run", out)
        self.assertIn("closed wave", out)
        self.assertIn("proven against", out)
        self.assertNotIn("every revision matches", out)

    def test_the_reason_is_said_while_writing_too(self):
        """Той, чию правку не проштампували, дізнається це саме тоді, коли штампують."""
        held = self.fixture.contract_rev
        self.fixture.write("keel/waves/0002-later.md",
                           self.LATER_WAVE.format(rev=held))
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план другої хвилі")
        self.close_the_wave()
        self.fixture.write("keel/contracts/session-run.md", CONTRACT + "\nІ ще речення.\n")
        code, out = self.run_rev(write=True)
        self.assertEqual(code, 0)
        self.assertIn("closed wave", out)
        self.assertIn("0001-session-loop", out)
        self.assertIn("0002-later", out)
        self.assertIn("recorded", out)

    # ── розрізи якості ──

    def wave_without(self, heading):
        """Хвиля, у якій один заголовок лишився без відповіді."""
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        lines = [l for l in text.split("\n") if not l.startswith(f"- {heading}:")]
        self.fixture.write("keel/waves/0001-session-loop.md", "\n".join(lines))

    def test_a_why_written_one_level_up_is_named_as_such(self):
        """ЗНАЙДЕНО ПРОГОНОМ 26 серпня 2026, Devstral.

        Вона написала `# Why` і абзац під ним. Секцією вважається лише `## `,
        тож засіб казав «секція «Навіщо» порожня» — стоячи над непорожнім
        текстом. Скарга, яка суперечить тому, що модель бачить у файлі, гірша
        за мовчання: рухатись від неї нікуди.
        """
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        self.fixture.write("keel/waves/0001-session-loop.md",
                           text.replace("## Навіщо", "# Навіщо", 1))
        problems = keel.gaps_problems(self.project, [self.project.waves["0001-session-loop"]])
        skarhy = [p.message for p in problems]
        self.assertIn("one level too high", " ".join(skarhy))
        self.assertIn("`# Навіщо`", " ".join(skarhy))
        self.assertNotIn("the Why section is empty", skarhy)

    def test_a_why_that_is_simply_absent_still_says_so(self):
        """Заголовка немає зовсім — тоді скарга та сама, що й була."""
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        head, _, tail = text.partition("## Навіщо")
        self.fixture.write("keel/waves/0001-session-loop.md",
                           head + tail.partition("\n## ")[1] + tail.partition("\n## ")[2])
        skarhy = [p.message for p in keel.gaps_problems(
            self.project, [self.project.waves["0001-session-loop"]])]
        self.assertIn("the Why section is empty", skarhy)

    def test_a_wave_without_the_cuts_section_is_incomplete(self):
        """Прохід не перевіряється; перевіряється слід, який він лишає."""
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        self.fixture.write("keel/waves/0001-session-loop.md",
                           text.split("## Розрізи якості")[0])
        problems = keel.gaps_problems(self.project, [self.project.waves["0001-session-loop"]])
        self.assertEqual(len(problems), 1)
        self.assertIn("quality cuts", problems[0].message)

    def test_every_heading_must_carry_an_answer(self):
        for heading in ("Security", "Safety", "Flexibility"):
            with self.subTest(heading=heading):
                self.setUp()
                self.wave_without(heading)
                problems = keel.gaps_problems(
                    self.project, [self.project.waves["0001-session-loop"]])
                self.assertEqual([p.message for p in problems],
                                 [f"the cut {heading} has no answer"])

    def test_a_heading_named_with_nothing_after_it_is_no_answer(self):
        """Порожній рядок — не відповідь: інакше девʼять двокрапок закривали б список."""
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        self.fixture.write("keel/waves/0001-session-loop.md",
                           text.replace("- Security: не стосується — інструментів ще немає.",
                                        "- Security:"))
        problems = keel.gaps_problems(self.project, [self.project.waves["0001-session-loop"]])
        self.assertEqual([p.message for p in problems], ["the cut Security has no answer"])

    def test_the_list_as_quality_md_writes_it_is_accepted(self):
        """Скопіювати заголовки з номерами — найдешевше чесне; воно мусить проходити."""
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        head, _ = text.split("## Розрізи якості")
        numbered = "\n".join(
            f"**{n}. {h}** — не стосується, бо хвиля мала."
            for n, h in enumerate(keel.QUALITY_HEADINGS, 1))
        self.fixture.write("keel/waves/0001-session-loop.md",
                           head + "## Розрізи якості\n\n" + numbered + "\n")
        self.assertEqual(
            keel.gaps_problems(self.project, [self.project.waves["0001-session-loop"]]), [])

    def test_a_closed_wave_is_not_asked_for_a_section_that_did_not_exist(self):
        """Інакше правило зробило б червоними всі хвилі, закриті до нього."""
        self.close_the_wave()
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        self.fixture.write("keel/waves/0001-session-loop.md",
                           text.split("## Розрізи якості")[0])
        self.assertEqual(
            keel.gaps_problems(self.project, [self.project.waves["0001-session-loop"]]), [])

    def test_a_wave_without_transforms_is_not_closed_by_having_none(self):
        """Скелет, якого ніхто не заповнив, не є завершеною роботою."""
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        self.fixture.write("keel/waves/0001-session-loop.md",
                           text.replace("transforms:", "transforms: {}\n_transforms:", 1))
        self.assertFalse(keel.closed_wave(
            self.project, self.project.waves["0001-session-loop"]))


# ─────────────────────────────────────────────────────────────────────────────
# keel new and keel gaps
# ─────────────────────────────────────────────────────────────────────────────




# ─────────────────────────────────────────────────────────────────────────────
# keel new and keel gaps
# ─────────────────────────────────────────────────────────────────────────────

class TestNewAndPlan(ProjectCase):
    def capture(self, function, *args):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = function(*args)
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def test_new_wave_takes_the_next_number(self):
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="wave", slug="Tool Calls"))
        self.assertEqual(code, 0)
        self.assertIn("0002-tool-calls.md", out)
        self.assertTrue(os.path.exists(self.fixture.path("keel/waves/0002-tool-calls.md")))

    def test_a_number_parked_on_another_branch_is_taken(self):
        """§8.5 makes the number a unique prefix, and the working tree is one
        branch's worth of it. A plan written and parked lives on its own branch
        and nowhere else; without looking there the next wave takes 0002 twice
        and the collision surfaces at the merge."""
        self.fixture.branch("plan/0002-parked")
        self.fixture.write("keel/waves/0002-parked.md", "---\nscenarios: {}\n---\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план, який відклали")
        self.fixture.git("checkout", "main")

        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="wave", slug="tool-calls"))
        self.assertEqual(code, 0)
        self.assertIn("0003-tool-calls.md", out)
        self.assertFalse(os.path.exists(self.fixture.path("keel/waves/0002-tool-calls.md")))

    def test_a_number_binned_after_a_commit_comes_back(self):
        """Branch tips, not the whole history — and the two differ.

        A plan committed and then binned lives in the history for ever, and a
        rule reading `rev-list --all` would hold its number hostage. What makes
        a number taken is a plan somebody can still merge, which means a branch
        that still carries the file. The distinction is the whole point of
        looking at refs rather than commits, so the case that separates them is
        the one worth a test: committed, then removed, on the same branch.
        """
        self.fixture.write("keel/waves/0002-binned.md", "---\nscenarios: {}\n---\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план, який потім передумали")
        os.remove(self.fixture.path("keel/waves/0002-binned.md"))
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "передумали")

        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="wave", slug="tool-calls"))
        self.assertEqual(code, 0)
        self.assertIn("0002-tool-calls.md", out)

    def test_a_slug_that_starts_with_a_number_loses_it(self):
        """ЗНАЙДЕНО ПРОГОНОМ 26 серпня 2026, Devstral.

        Вона взяла слаг із власного `PLAN.md`, де хвилі звуться «Wave 1», і
        подала `1-parse-meter-readings`. Засіб додав свій номер — вийшло
        `0001-1-parse-meter-readings.md`, і подвоєння потягнулось далі в імʼя
        гілки. Модель нумерує, не знаючи, що нумерує засіб.
        """
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="wave", slug="1-parse-meter-readings"))
        self.assertEqual(code, 0)
        self.assertIn("0002-parse-meter-readings.md", out)
        self.assertNotIn("0002-1-parse", out)

    def test_the_number_it_dropped_is_said_out_loud(self):
        """Зрізане мовчки — це зрізане навмання: `2024-migration` теж
        починається з цифр, і там номер може бути частиною назви."""
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="wave", slug="0001-parse"))
        self.assertEqual(code, 0)
        self.assertIn("dropped the leading `0001-`", out)
        self.assertIn("0002-parse.md", out)

    def test_a_number_inside_a_word_stays(self):
        """`2fa-login` і `v2-parser` — номер тут частина імені, не префікс."""
        for slug, expected in (("2fa-login", "0002-2fa-login.md"),
                               ("v2-parser", "0002-v2-parser.md")):
            with self.subTest(slug=slug):
                self.setUp()
                code, out = self.capture(keel.cmd_new, self.project,
                                         Args(kind="wave", slug=slug))
                self.assertEqual(code, 0)
                self.assertIn(expected, out)

    def test_a_slug_of_digits_alone_survives(self):
        """Зрізати все — значить лишити хвилю без імені."""
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="wave", slug="42"))
        self.assertEqual(code, 0)
        self.assertIn("0002-42.md", out)

    def test_a_contract_keeps_its_leading_number(self):
        """Контракт засіб не нумерує, тож зрізати нема від чого."""
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="contract", slug="2fa-tokens"))
        self.assertEqual(code, 0)
        self.assertIn("2fa-tokens.md", out)

    def test_new_wave_names_the_branch_to_stand_on(self):
        """ЗНАЙДЕНО ПРОГОНОМ 26 серпня 2026, третя модель поспіль.

        Гілка мусить зватись `plan/0002-tool-calls` — із номером, який щойно
        додав сам засіб. Модель знає лише свій слаг `tool-calls`, номера не
        бачить, і називає гілку `plan/tool-calls`. Далі `check` вимагає
        перейменувати: Laguna, гема 26B і Devstral пройшли цей гак кожна.
        """
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="wave", slug="tool-calls"))
        self.assertEqual(code, 0)
        self.assertIn("git checkout -b plan/0002-tool-calls", out)

    def test_the_branch_line_is_absent_when_you_already_stand_there(self):
        """Порада, яку вже виконано, — шум: вона каже зробити зроблене."""
        self.fixture.branch("plan/0002-tool-calls")
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="wave", slug="tool-calls"))
        self.assertEqual(code, 0)
        self.assertNotIn("git checkout -b", out)

    def test_a_contract_says_nothing_about_branches(self):
        """Контракт не має власної гілки: він живе на гілці хвилі."""
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="contract", slug="queue"))
        self.assertEqual(code, 0)
        self.assertNotIn("git checkout -b", out)

    def test_new_wave_shows_a_filled_example(self):
        """Скелет каже форму, але не показує вигляду.

        25 серпня 2026 чотири моделі поспіль спинялись на місці, де треба
        заповнити `scenarios:` і `transforms:`, і йшли шукати зразок: кликали
        `--help` десятками, читали сам засіб байтами, шукали неіснуючий
        README. Від 19 до 50 відсотків дій до першого запису — на це.
        """
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="wave", slug="tool-calls"))
        self.assertEqual(code, 0)

        # Шлях лишається першим рядком: його читають скриптами.
        self.assertEqual(out.splitlines()[0], "keel/waves/0002-tool-calls.md")

        # Приклад показує ВСІ три місця, на яких вони спинялись.
        self.assertIn("{proves: queue@7f21ac}", out)
        self.assertIn("implements: [", out)
        self.assertIn("contracts:  [queue@7f21ac]", out)
        self.assertIn("files:      [", out)
        self.assertIn("**Given**", out)

        # І звідки береться редакція — це питали найчастіше.
        self.assertIn("keel rev --write", out)

    def test_the_example_is_not_written_into_the_file(self):
        """У файлі приклад став би сміттям, яке треба стерти, а `gaps` лаявся б
        на нього як на справжній вміст."""
        self.capture(keel.cmd_new, self.project, Args(kind="wave", slug="tool-calls"))
        text = self.fixture.read("keel/waves/0002-tool-calls.md")
        self.assertNotIn("queue@7f21ac", text)
        self.assertNotIn("retry-policy", text)

    def test_a_contract_gets_no_example(self):
        """Спинялись на хвилі, не на контракті: його скелет самодостатній."""
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="contract", slug="tool-registry"))
        self.assertEqual(code, 0)
        self.assertNotIn("queue@7f21ac", out)

    def test_rev_says_what_is_missing_instead_of_all_is_well(self):
        """Засіб відповідав правду, з якої нічого не випливає.

        25 серпня 2026 Laguna написала контракт, одразу покликала
        `rev --write` — і дістала «every revision matches». Звіряти справді
        було нічого: жодна хвиля контракту ще не називала. Модель прочитала це
        як «я не розумію інструмента» і стерла все, що зробила:
        `rm -rf keel/contracts keel/waves`.

        Редакція живе в ПОСИЛАННІ, не в контракті. Поки посилання немає,
        ставити нікуди — і сказати треба саме це.
        """
        # Прибираємо всі посилання: лишаємо контракти, знімаємо хвилі.
        for name in os.listdir(self.fixture.path("keel/waves")):
            os.remove(self.fixture.path(f"keel/waves/{name}"))
        project = keel.Project(self.fixture.root)

        code, out = self.capture(keel.cmd_rev, project, Args(write=True))
        self.assertEqual(code, 0)
        self.assertIn("nothing refers to these contracts yet", out)
        self.assertIn("proves: <contract>", out)
        self.assertNotIn("every revision matches", out)

    def test_rev_tells_how_to_stamp_a_bare_reference(self):
        """Питання «а звідки береться @rev» мусить мати відповідь там, де воно
        виникає.

        25 серпня 2026 Laguna дійшла до готової хвилі з `queue@—`, спитала себе
        «how to properly set up a contract with a revision» і пішла шукати
        `keel/README.md`, якого у фікстурі немає. Тричі поспіль, аж доки цикл не
        спинив її за повтор. Приклад із `new wave` це пояснює — але його
        показано на початку, за тисячі токенів до питання.
        """
        # У фікстурі редакції вже проставлені — знімаємо їх, бо саме голе
        # посилання й породжує питання.
        shlyah = "keel/waves/0001-session-loop.md"
        text = self.fixture.read(shlyah)
        self.fixture.write(shlyah, re.sub(r"@[0-9a-f]{6}", "", text))
        project = keel.Project(self.fixture.root)

        # Код виходу тут ненульовий навмисно: редакції розійшлись, і без
        # `--write` це стан, а не успіх. Питання тесту — що саме сказано.
        code, out = self.capture(keel.cmd_rev, project, Args(write=False))
        self.assertEqual(code, 1)
        self.assertIn("a reference without a revision is not an error", out)
        self.assertIn("keel rev --write", out)

    def test_the_advice_is_silent_when_there_is_nothing_bare(self):
        """Коли редакції вже стоять, поради не буває: вона про порожнє поле."""
        _, out = self.capture(keel.cmd_rev, self.project, Args(write=False))
        self.assertNotIn("a reference without a revision is not an error", out)

    def test_rev_still_records_when_there_is_a_reference(self):
        """Зворотне: коли посилання є, він робить свою роботу мовчки й точно."""
        code, out = self.capture(keel.cmd_rev, self.project, Args(write=True))
        self.assertEqual(code, 0)
        self.assertNotIn("nothing refers to these contracts", out)

    def test_the_example_points_at_the_reference_not_the_contract(self):
        """Приклад вів у ту саму пастку: «спершу контракт, тоді rev --write»."""
        _, out = self.capture(keel.cmd_new, self.project,
                              Args(kind="wave", slug="tool-calls"))
        self.assertIn("lives in the reference", out)
        self.assertIn("keel rev --write", out)

    def test_new_wave_skeleton_parses(self):
        self.capture(keel.cmd_new, self.project, Args(kind="wave", slug="tool-calls"))
        wave = self.project.waves["0002-tool-calls"]
        self.assertIsNone(wave.error)

    def test_new_contract(self):
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="contract", slug="tool-registry"))
        self.assertEqual(code, 0)
        self.assertIn("keel/contracts/tool-registry.md", out)

    def test_new_refuses_to_overwrite(self):
        with self.assertRaises(SystemExit):
            self.capture(keel.cmd_new, self.project, Args(kind="contract", slug="session-run"))

    def test_plan_is_complete(self):
        code, out = self.capture(keel.cmd_gaps, self.project, Args(wave="0001-session-loop"))
        self.assertEqual(code, 0)
        self.assertIn("the plan is complete", out)

    def test_an_empty_plan_is_not_called_complete(self):
        """«the plan is complete: nothing» читалось як похвала.

        25 серпня 2026 гема 26B написала хвилю руками — у `keel/WAVE-1.md`,
        поруч із методикою, а не в `keel/waves/`. Засіб її не побачив: немає
        хвиль — немає й прогалин, — і відповів «план повний». Модель завершила
        роботу з повною певністю, що впоралась.
        """
        for name in os.listdir(self.fixture.path("keel/waves")):
            os.remove(self.fixture.path(f"keel/waves/{name}"))
        project = keel.Project(self.fixture.root)

        code, out = self.capture(keel.cmd_gaps, project, Args(wave=None))
        self.assertEqual(code, 0)
        self.assertIn("there are no waves yet", out)
        self.assertIn("keel new wave", out)
        self.assertNotIn("the plan is complete", out)

        # ЗНАЙДЕНО ПРОГОНАМИ 26 серпня 2026, Laguna двічі поспіль: не знаючи
        # форми, вона створювала хвилю-зразок, дивилась на скелет і потім її
        # прибирала. У python зразок забрав номер 0001, і справжня хвиля стала
        # другою. Засіб уміє показати скелет без створення файлу — і сказано
        # про це було лише в `new --help`, куди модель заглядає рідко.
        self.assertIn("without creating one", out)
        self.assertIn("`keel new wave` with no name", out)

    def test_a_wave_shaped_file_outside_the_folder_is_named(self):
        """Файл, що виглядає як хвиля, але лежить не там, більше не мовчазний."""
        for name in os.listdir(self.fixture.path("keel/waves")):
            os.remove(self.fixture.path(f"keel/waves/{name}"))
        self.fixture.write("keel/WAVE-1.md", "# Wave 1\n\n## Purpose\n\nЩось.\n")
        project = keel.Project(self.fixture.root)

        _, out = self.capture(keel.cmd_gaps, project, Args(wave=None))
        self.assertIn("keel/WAVE-1.md", out)
        self.assertIn("not in keel/waves/", out)

    def test_a_plan_with_waves_still_reports_completeness(self):
        """Зворотне: там, де хвилі є, відповідь лишається тією, що була."""
        code, out = self.capture(keel.cmd_gaps, self.project, Args(wave=None))
        self.assertEqual(code, 0)
        self.assertIn("the plan is complete", out)
        self.assertNotIn("there are no waves yet", out)

    def test_gaps_without_an_argument_names_only_its_own_wave(self):
        """Заголовок казав один крок, а список — інший."""
        self.fixture.write("keel/waves/0009-other.md",
                           "---\ndepends_on: []\nscenarios:\n  zzz: {proves: session-run}\n"
                           "transforms: {}\n---\n\n## Навіщо\n\nx\n")
        self.fixture.branch("0001-session-loop")
        code, out = self.capture(keel.cmd_gaps, self.project, Args(wave=None))
        self.assertNotIn("0009-other", out)

    def test_plan_finds_a_transform_without_files(self):
        self.fixture.write(
            "keel/waves/0001-session-loop.md",
            self.fixture.read("keel/waves/0001-session-loop.md").replace(
                "    files:      [lib/session.ex]\n", ""))
        code, out = self.capture(keel.cmd_gaps, self.project, Args(wave="0001-session-loop"))
        self.assertEqual(code, 1)
        self.assertIn("declared no files", out)

    def test_plan_finds_a_scenario_nobody_implements(self):
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        text = text.replace(
            "  finishes-when-no-tool-called: ",
            "  only-handed-tools-are-callable: {proves: session-run@%s}\n  finishes-when-no-tool-called: "
            % self.fixture.contract_rev)
        text += "\n## scenario: only-handed-tools-are-callable\n\n**Then** інших немає.\n"
        self.fixture.write("keel/waves/0001-session-loop.md", text)
        code, out = self.capture(keel.cmd_gaps, self.project, Args(wave="0001-session-loop"))
        self.assertEqual(code, 1)
        self.assertIn("no transform implements scenario", out)

    def test_new_skeleton_is_not_a_complete_plan(self):
        self.capture(keel.cmd_new, self.project, Args(kind="wave", slug="tool-calls"))
        code, out = self.capture(keel.cmd_gaps, self.project, Args(wave="0002-tool-calls"))
        self.assertEqual(code, 1)
        self.assertIn("no scenarios at all", out)


# ─────────────────────────────────────────────────────────────────────────────
# keel check
# ─────────────────────────────────────────────────────────────────────────────




# ─────────────────────────────────────────────────────────────────────────────
# keel check
# ─────────────────────────────────────────────────────────────────────────────

class TestShow(ProjectCase):
    """Читальний вигляд: посилання ведуть кудись, стан виводиться."""

    def show(self, wave=None):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_show(self.project, Args(wave=wave))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def test_links_resolve_from_the_wave_file(self):
        code, out = self.show("0001-session-loop")
        self.assertEqual(code, 0)
        base = os.path.dirname(self.fixture.path("keel/waves/0001-session-loop.md"))
        for target in re.findall(r"\]\(([^)#]+)\)", out):
            self.assertTrue(os.path.exists(os.path.normpath(
                os.path.join(base, target))), target)

    def test_shows_the_scenario_text_and_its_revision(self):
        _, out = self.show("0001-session-loop")
        self.assertIn("**Then** розмова завершується.", out)
        self.assertIn(self.fixture.scenario_rev(), out)

    def test_a_matching_contract_revision_is_ticked(self):
        _, out = self.show("0001-session-loop")
        self.assertIn("✓", out)
        self.assertNotIn("✗", out)

    def test_a_stale_contract_revision_is_crossed(self):
        self.fixture.write("keel/contracts/session-run.md", CONTRACT + "\nЩе речення.\n")
        _, out = self.show("0001-session-loop")
        self.assertIn("✗", out)

    def test_transform_state_is_derived_not_written(self):
        self.fixture.branch("0001-session-loop")
        _, out = self.show()
        self.assertIn("drive-turns — open", out)
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns: хід")
        _, out = self.show()
        self.assertIn("closed", out)

    def test_an_existing_file_is_not_marked_as_missing(self):
        _, out = self.show("0001-session-loop")
        self.assertIn("[lib/session.ex]", out)
        self.assertNotIn("not there yet", out)

    def test_a_file_that_does_not_exist_yet_says_so(self):
        """Видно, що з оголошеного вже лежить, а чого ще нема."""
        wave = "keel/waves/0001-session-loop.md"
        self.fixture.write(wave, self.fixture.read(wave).replace(
            "files:      [lib/session.ex]",
            "files:      [lib/session.ex, lib/поки_немає.ex]"))
        _, out = self.show("0001-session-loop")
        self.assertIn("not there yet", out)
        self.assertIn("lib/поки_немає.ex", out)

    def test_an_unknown_wave_refuses(self):
        with self.assertRaises(SystemExit):
            self.show("0009-nope")


class TestGapsAsksAboutAForgottenEdge(ProjectCase):
    """Порожнє `depends_on` виглядає однаково: залежності немає чи її забули.

    Перевірка 2 шукає цикли, а в порожньому графі їх не буває, тож забуте ребро
    не бачив ніхто. Питання, не вирок: два кроки можуть законно правити один
    файл, не спираючись один на одного.
    """

    def second_wave(self, depends="[]", files="[lib/session.ex]",
                    contracts="[session-run@%s]"):
        rev = self.fixture.contract_rev
        self.fixture.write("keel/waves/0002-later.md", f"""---
depends_on: {depends}

scenarios:
  later-holds: {{proves: session-run@{rev}}}

transforms:
  later-turns:
    implements: [later-holds]
    contracts:  {contracts % rev}
    files:      {files}
---

## Навіщо

Другий крок, щоб було на чому показати ребро.

## scenario: later-holds

**Given** одне, **When** друге, **Then** третє.

## transform: later-turns

Робить пізніше.

Межі: не робить раніше.
""")

    def messages(self, slug="0002-later"):
        return [p.message for p in
                keel.missing_edges(self.project, self.project.waves[slug])]

    def test_a_shared_file_without_an_edge_is_asked_about(self):
        self.second_wave()
        self.assertTrue(any("lib/session.ex" in m for m in self.messages()),
                        self.messages())

    def test_the_edge_silences_it(self):
        self.second_wave(depends="[0001-session-loop]")
        self.assertEqual(self.messages(), [])

    def test_the_wave_that_is_leaned_on_is_not_asked(self):
        """Напрямок читається з графа, а не з номерів у назвах."""
        self.second_wave(depends="[0001-session-loop]")
        self.assertEqual(self.messages("0001-session-loop"), [])

    def test_a_shared_contract_alone_is_not_asked_about(self):
        """Спиратись на спільну обіцянку — не те саме, що залежати від кроку,
        який її написав. Питання про це було майже завжди хибним і давало
        N×(N−1) рядків на один поширений контракт."""
        self.second_wave(files="[lib/nothing_shared.ex]")
        self.assertEqual(self.messages(), [])

    def test_a_dependency_two_waves_away_still_counts_as_named(self):
        self.second_wave(depends="[0001-session-loop]")
        self.fixture.write("keel/waves/0003-last.md",
                           self.fixture.read("keel/waves/0002-later.md")
                           .replace("depends_on: [0001-session-loop]",
                                    "depends_on: [0002-later]")
                           .replace("later-", "last-"))
        self.assertEqual(self.messages("0003-last"), [])


class TestGapsAsksAboutAContractNobodyLeansOn(ProjectCase):
    """Перевірка 1 питає один бік — чи має слаг свій файл.

    Зворотного не питав ніхто, тож контракт міг лежати, на нього ніхто не
    спирався, файл його `verify` не був оголошений — і `gaps` казав «план
    повний». Так і сталось у кроці 0003; агент помітив сам, але це не заслон.
    """

    def new_contract(self, slug="brand-new"):
        self.fixture.write(f"keel/contracts/{slug}.md",
                           "---\nmodule: Demo.New\nexports: [run/1]\n---\n\nНове.\n")

    def messages(self):
        wave = self.project.waves["0001-session-loop"]
        return [p.message for p in keel.unclaimed_contracts(self.project, wave)]

    def test_a_contract_this_wave_brings_and_nobody_names_is_asked_about(self):
        self.new_contract()
        self.assertTrue(any("brand-new" in m for m in self.messages()),
                        self.messages())

    def test_a_contract_a_transform_leans_on_is_not_asked_about(self):
        self.assertEqual(self.messages(), [])

    def test_a_contract_already_on_the_main_branch_belongs_to_somebody_else(self):
        self.new_contract()
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "чужий контракт")
        self.assertEqual(self.messages(), [])


class TestNextOnTheMainBranch(ProjectCase):
    """Інструмент знає стан — і мусить його сказати, а не переказати правило.

    Раніше на головній гілці `next` відповідав «гілка не названа за кроком», хоч
    із документів і git йому відомо, який крок готовий до роботи.
    """

    def answer(self):
        return keel.main_branch_answer(self.project)

    def close_the_only_transform(self):
        self.fixture.git("commit", "--allow-empty", "-m", "drive-turns: зроблено")

    def add_wave(self, slug, body):
        self.fixture.write(f"keel/waves/{slug}.md", body)
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", f"план {slug}")

    def test_the_ready_wave_is_named_with_its_branch(self):
        answer = self.answer()
        self.assertIn("0001-session-loop", answer)
        self.assertIn("git checkout -b 0001-session-loop", answer)
        self.assertIn("1 of 1", answer)

    def test_when_everything_is_closed_it_says_plan_the_next(self):
        self.close_the_only_transform()
        self.assertIn("every wave is finished", self.answer())

    def test_a_skeleton_is_not_a_finished_project(self):
        """Крок без трансформ закривати нічим — і він читався як завершений."""
        self.close_the_only_transform()
        self.add_wave("0002-empty", keel.wave_skeleton("0002-empty"))
        answer = self.answer()
        self.assertIn("0002-empty", answer)
        self.assertIn("the plan is not written yet", answer)

    def test_a_wave_whose_ground_is_not_laid_is_not_offered(self):
        self.close_the_only_transform()
        # Своя назва трансформи: інакше комміт, що закрив `drive-turns`
        # у першому кроці, закриває однойменну і в другому.
        self.add_wave("0002-later", self.fixture.read(
            "keel/waves/0001-session-loop.md")
            .replace("depends_on: []", "depends_on: [0009-never-planned]")
            .replace("drive-turns", "later-turns"))
        answer = self.answer()
        self.assertIn("waiting on another", answer)
        self.assertIn("0002-later", answer)
        self.assertNotIn("git checkout", answer)


class TestCheck(ProjectCase):
    def capture(self, args):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_check(self.project, args)
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def test_fast_runs_only_five(self):
        code, out = self.capture(Args(fast=True, no_tests=True, json=False))
        self.assertIn("5. every scenario has a green test (not run)", out)
        self.assertIn("✓ 1.", out)
        self.assertEqual(code, 0)

    def test_full_check_wants_a_test(self):
        code, out = self.capture(Args(fast=False, no_tests=True, json=False))
        self.assertEqual(code, 1)
        self.assertIn("has no test", out)

    def stray_skeleton(self):
        """An unfilled skeleton of somebody else's wave, left behind untracked."""
        self.fixture.write("keel/waves/0002-left-behind.md",
                           keel.wave_skeleton("0002-left-behind"))

    def test_a_stray_wave_is_named_so_nobody_moves_it(self):
        self.stray_skeleton()
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.capture(Args(fast=True, no_tests=True, json=False))
        self.assertEqual(code, 1)
        self.assertIn("0002-left-behind", out)
        self.assertIn("not moved, renamed or deleted", out)

    def test_the_branch_own_wave_earns_no_such_note(self):
        self.fixture.branch("plan/0001-session-loop")
        self.fixture.write("keel/waves/0001-session-loop.md",
                           self.fixture.read("keel/waves/0001-session-loop.md")
                           + "\n## transform: never-declared\n\nWhat it does.\n")
        code, out = self.capture(Args(fast=True, no_tests=True, json=False))
        self.assertEqual(code, 1)
        self.assertIn("never-declared", out)
        self.assertNotIn("not moved, renamed or deleted", out)

    def test_on_the_main_branch_the_note_stays_quiet(self):
        self.stray_skeleton()
        code, out = self.capture(Args(fast=True, no_tests=True, json=False))
        self.assertEqual(code, 1)
        self.assertNotIn("not moved, renamed or deleted", out)

    def fileless_transform(self):
        """A plan that says nothing about which files the work will touch."""
        wave = "keel/waves/0001-session-loop.md"
        self.fixture.write(wave, self.fixture.read(wave).replace(
            "    files:      [lib/session.ex]", "    files:      []"))

    def test_a_plan_branch_is_not_asked_for_code_it_has_none_of(self):
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.capture(Args(fast=False, no_tests=False, json=False))
        self.assertIn("5. every scenario has a green test (not run: a plan branch", out)
        self.assertIn("6. contracts hold (not run: a plan branch", out)
        self.assertEqual(code, 0, out)

    def test_an_incomplete_plan_does_not_get_pushed(self):
        self.fileless_transform()
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.capture(Args(fast=False, no_tests=True, json=False))
        self.assertIn("the plan is missing things", out)
        self.assertIn("declared no files", out)
        self.assertEqual(code, 1)

    def test_a_half_written_plan_still_commits(self):
        """Коміт на гілці плану може бути недописаним; пуш і мерж — ні."""
        self.fileless_transform()
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.capture(Args(fast=True, no_tests=True, json=False))
        self.assertNotIn("the plan is missing things", out)
        self.assertEqual(code, 0, out)

    def test_the_plan_gate_shows_up_in_json(self):
        import json as jsonlib
        self.fileless_transform()
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.capture(Args(fast=False, no_tests=True, json=True))
        payload = jsonlib.loads(out)
        self.assertFalse(payload["ok"])
        self.assertTrue(any("declared no files" in p["message"] for p in payload["plan"]))
        self.assertFalse(payload["checks"]["5"]["run"])
        self.assertEqual(code, 1)

    def test_a_work_branch_is_still_asked_for_everything(self):
        self.fixture.branch("0001-session-loop")
        code, out = self.capture(Args(fast=False, no_tests=True, json=False))
        self.assertIn("5. every scenario has a green test", out)
        self.assertNotIn("a plan branch has no code", out)
        self.assertEqual(code, 1)

    def test_json_shape(self):
        import json as jsonlib
        code, out = self.capture(Args(fast=True, no_tests=True, json=True))
        payload = jsonlib.loads(out)
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["checks"]["1"]["name"], "references lead somewhere")
        self.assertFalse(payload["checks"]["5"]["run"])
        self.assertEqual(code, 0)


# ─────────────────────────────────────────────────────────────────────────────
# keel hooks
# ─────────────────────────────────────────────────────────────────────────────


class TestNextDictatesNothingPoisoned(ProjectCase):
    """A scenario with no body has no revision — so no tag is dictated."""

    def test_a_bodyless_scenario_gets_no_tag_line(self):
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        head, _, _ = text.partition("## scenario:")
        self.fixture.write("keel/waves/0001-session-loop.md", head)
        self.fixture.branch("0001-session-loop")
        wave = self.project.waves["0001-session-loop"]
        slug, state = keel.next_transform(self.project, wave)
        package = keel.next_package(self.project, wave, slug, state)
        for item in package["scenarios"]:
            self.assertIsNone(item["rev"])
            self.assertIsNone(item["tag"])
        self.assertNotIn('rev: "None"', keel.render_next(package))


class TestCheckOrderIsHonest(unittest.TestCase):
    """Check 4 reads git after 5 and 6 ran: their side effects are in the verdict."""

    def test_files_dropped_by_the_test_run_are_seen_by_check_4(self):
        """Два послідовні прогони мають описувати дерево однаково."""
        import tempfile, shutil, subprocess
        root = tempfile.mkdtemp(prefix="keel-order-")
        self.addCleanup(shutil.rmtree, root, True)
        for folder in ("keel/waves", "keel/contracts", "tests"):
            os.makedirs(os.path.join(root, folder))
        with open(os.path.join(root, "pyproject.toml"), "w") as handle:
            handle.write("[project]\nname='d'\n")
        subprocess.run(["git", "init", "-b", "main", "-q", root], check=True)
        for key, value in (("user.email", "t@e"), ("user.name", "t")):
            subprocess.run(["git", "-C", root, "config", key, value], check=True)
        with open(os.path.join(root, "keel/waves/0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\nscenarios:\n  does-a: {}\n"
                         "transforms:\n  do:\n    implements: [does-a]\n"
                         "    files: [lib/a.py]\n---\n\n## Why\n\nх.\n\n"
                         "## scenario: does-a\n\n**Given** щось.\n\n"
                         "## transform: do\n\nЩось.\n")
        subprocess.run(["git", "-C", root, "add", "-A"], check=True)
        subprocess.run(["git", "-C", root, "commit", "-q", "-m", "план"], check=True)
        subprocess.run(["git", "-C", root, "checkout", "-q", "-b", "0001-a"],
                       check=True)
        os.makedirs(os.path.join(root, "lib"))
        with open(os.path.join(root, "lib/a.py"), "w") as handle:
            handle.write("x = 1\n")
        body = keel.Project(root).waves["0001-a"].scenario_body("does-a")
        with open(os.path.join(root, "tests/__init__.py"), "w") as handle:
            handle.write("")
        # тест, який при прогоні лишає в дереві неоголошений файл — як _build/
        with open(os.path.join(root, "tests/side_test.py"), "w",
                  encoding="utf-8") as handle:
            handle.write("import unittest, pathlib\n"
                         f"# proves: does-a, rev: \"{keel.revision(body)}\"\n"
                         "class T(unittest.TestCase):\n"
                         "    def test_x(self):\n"
                         "        pathlib.Path('side-effect.txt').write_text('x')\n")
        first = keel.run_checks(keel.Project(root))[1][4]
        second = keel.run_checks(keel.Project(root))[1][4]
        self.assertEqual([x.message for x in first],
                         [x.message for x in second])
        self.assertTrue(any("side-effect.txt" in x.message for x in first),
                        [x.message for x in first])


class TestRevWritesHonestly(unittest.TestCase):
    """rev --write edits what it reported, and reports what it edited."""

    def setUp(self):
        import tempfile, shutil, subprocess
        self.root = tempfile.mkdtemp(prefix="keel-revh-")
        self.addCleanup(shutil.rmtree, self.root, True)
        for folder in ("keel/waves", "keel/contracts", "tests"):
            os.makedirs(os.path.join(self.root, folder))
        with open(os.path.join(self.root, "pyproject.toml"), "w") as handle:
            handle.write("[project]\nname='d'\n")
        with open(os.path.join(self.root, "keel/waves/0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\nscenarios:\n  parse: {}\n---\n\n"
                         "## scenario: parse\n\n**Given** щось.\n")

    def rev_write(self):
        from io import StringIO
        from tests.support import Args
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_rev(keel.Project(self.root), Args(write=True))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def test_the_other_dialect_inside_a_fixture_is_left_alone(self):
        """Elixir-тег у рядковій фікстурі python-тесту не перештамповується."""
        with open(os.path.join(self.root, "tests/parse_test.py"), "w",
                  encoding="utf-8") as handle:
            handle.write('FIXTURE = \'@tag proves: :parse, rev: "keepme99"\'\n'
                         '# proves: parse, rev: "00000000"\n')
        code, _ = self.rev_write()
        with open(os.path.join(self.root, "tests/parse_test.py"),
                  encoding="utf-8") as handle:
            after = handle.read()
        self.assertIn('keepme99', after)
        self.assertNotIn('"00000000"', after)
        self.assertEqual(code, 0)

    def test_a_capitalised_tag_is_restamped_not_claimed(self):
        """«recorded: 1» над файлом, що не змінився, — заборонений клас."""
        with open(os.path.join(self.root, "tests/parse_test.py"), "w",
                  encoding="utf-8") as handle:
            handle.write('# proves: Parse, rev: "00000000"\n')
        code, out = self.rev_write()
        with open(os.path.join(self.root, "tests/parse_test.py"),
                  encoding="utf-8") as handle:
            after = handle.read()
        self.assertNotIn('"00000000"', after)
        self.assertEqual(code, 0, out)


class TestAgentsMarkers(unittest.TestCase):
    """Markers out of balance are named and left, not compounded."""

    def setUp(self):
        import tempfile, shutil
        self.root = tempfile.mkdtemp(prefix="keel-mark-")
        self.addCleanup(shutil.rmtree, self.root, True)
        self.path = os.path.join(self.root, "AGENTS.md")

    def run_update(self, text):
        from io import StringIO
        with open(self.path, "w", encoding="utf-8") as handle:
            handle.write(text)
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            changed = keel.update_agents(self.path, "БЛОК\n")
        finally:
            sys.stdout = saved
        with open(self.path, encoding="utf-8") as handle:
            return changed, handle.read(), stream.getvalue()

    def test_an_orphaned_start_is_left_alone(self):
        """Другий прогін зʼїдав усе між маркером-сиротою і новим блоком."""
        text = (keel.AGENTS_START + "\nстаре\n\n## Правила дому\n\n"
                "Не чіпати прод.\n")
        changed, after, out = self.run_update(text)
        self.assertFalse(changed)
        self.assertEqual(after, text)
        self.assertIn("out of balance", out)

    def test_balanced_markers_still_update(self):
        text = ("своє\n" + keel.AGENTS_START + "\nстаре\n" + keel.AGENTS_END
                + "\nхвіст\n")
        changed, after, _ = self.run_update(text)
        self.assertTrue(changed)
        self.assertIn("БЛОК", after)
        self.assertIn("хвіст", after)


if __name__ == "__main__":
    unittest.main()


class TestAnEmptyProjectIsNotAFinishedOne(ProjectCase):
    """«Усі хвилі завершені» над проєктом, у якому немає жодної, — неправда,
    і це перший рядок, який читає агент нового проєкту."""

    def test_no_waves_says_so(self):
        os.remove(self.fixture.path("keel/waves/0001-session-loop.md"))
        answer = keel.main_branch_answer(self.project)
        self.assertIn("no waves yet", answer)
        self.assertNotIn("every wave is finished", answer)

    def test_all_done_still_says_finished(self):
        self.fixture.git("commit", "--allow-empty", "-m", "drive-turns: зроблено")
        self.assertIn("every wave is finished", keel.main_branch_answer(self.project))


class TestAWaveNobodyHasBegunIsNotJudged(ProjectCase):
    """Знайдено живим, двічі, на двох різних хвилях.

    Схваленням плану є те, що файл хвилі дійшов головної гілки. Між тим
    моментом і приїздом роботи головна гілка тримає обіцянки без коду —
    і перевірки 5 та 6 судять її за тестами, яких ще нема, і модулями, яких
    ще не написали. `plan_wave` вимкнула це для гілки плану з тією самою
    причиною; злиття плану причину не скасовує, а переносить.

    Ціна була не лише в червоному CI: поки головна червона, у неї не
    запушиш нічого — зокрема й README з беклогом, яким 0.7.3 навмисно дав
    дорогу.
    """

    def close_the_only_transform(self):
        self.fixture.git("commit", "--allow-empty", "-m", "drive-turns: зроблено")

    def plan_a_second_wave(self, closed=False):
        body = WAVE.format(rev=self.fixture.contract_rev).replace(
            "drive-turns:", "carry-tools:").replace(
            "finishes-when-no-tool-called", "asks-before-it-writes")
        self.fixture.write("keel/waves/0002-tools.md", body)
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план 0002")
        if closed:
            self.fixture.git("commit", "--allow-empty", "-m", "carry-tools: зроблено")

    def scenario_problems(self):
        return [p for p in keel.check_scenarios(self.project, run_tests=False)
                if "asks-before-it-writes" in p.message]

    def test_its_scenarios_are_not_asked_for_tests_on_main(self):
        self.close_the_only_transform()
        self.plan_a_second_wave()
        self.assertEqual(self.scenario_problems(), [])

    def test_the_same_wave_is_judged_on_its_own_branch(self):
        """Робота йде під носом у перевірки, і історія головної про це мовчить."""
        self.close_the_only_transform()
        self.plan_a_second_wave()
        self.fixture.git("checkout", "-b", "0002-tools")
        self.assertNotEqual(self.scenario_problems(), [])

    def test_a_wave_with_one_transform_closed_is_under_way(self):
        """Почата — значить судять, і решту її сценаріїв теж."""
        self.close_the_only_transform()
        self.plan_a_second_wave(closed=True)
        self.assertNotEqual(self.scenario_problems(), [])

    def test_a_project_that_never_closed_a_transform_gets_no_exemption(self):
        """Усі хвилі непочаті — значить це не проміжок, а інший спосіб роботи.

        Осліпити перевірки над цілим проєктом було б рівно тією тишею, проти
        якої вони й стоять.
        """
        self.plan_a_second_wave()
        self.assertNotEqual(self.scenario_problems(), [])
        self.assertEqual(keel.unbegun_waves(self.project), set())

    def test_the_contract_of_an_unbegun_wave_alone_is_not_asked_for_its_module(self):
        self.close_the_only_transform()
        self.fixture.write("keel/contracts/tool-call.md",
                           "---\nmodule: Demo.Missing\nexports: [run/1]\n---\n\nОбіцянка.\n")
        rev = keel.revision(self.fixture.read("keel/contracts/tool-call.md"))
        body = WAVE.format(rev=self.fixture.contract_rev).replace(
            "drive-turns:", "carry-tools:").replace(
            "finishes-when-no-tool-called", "asks-before-it-writes").replace(
            f"session-run@{self.fixture.contract_rev}", f"tool-call@{rev}")
        self.fixture.write("keel/waves/0002-tools.md", body)
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план 0002")
        self.assertIn("tool-call", keel.unbegun_contracts(self.project))
        problems = keel.check_exports(self.project)
        self.assertEqual([p for p in problems if "Demo.Missing" in p.message], [])

    def test_a_contract_two_waves_lean_on_is_still_asked(self):
        """Одна почата, одна ні — код першої мусить його тримати."""
        self.close_the_only_transform()
        self.plan_a_second_wave()
        self.assertNotIn("session-run", keel.unbegun_contracts(self.project))

    def rewrite_the_contract(self):
        """Друга хвиля дописує контрактові обіцянку, якої код ще не тримає."""
        body = CONTRACT.replace("exports: [run/3]",
                                        "exports: [run/3, looked?/1]")
        self.fixture.write("keel/contracts/session-run.md", body)
        return keel.revision(body)

    def test_a_contract_the_unbegun_wave_rewrote_is_not_asked(self):
        """Знайдено живим на хвилі 0018.

        Почата хвиля лежить на **старій** редакції контракту й нічого не
        каже про нову. Друга хвиля переписала контракт, дописавши обіцянку,
        якої код ще не має, — і поки її роботи на головній немає, питати
        цю обіцянку нема в кого. Раніше сам факт, що на слаг посилається
        ще й закрита хвиля, будив контракт, і перевірка 6 судила старий
        код за новою обіцянкою: головна червоніла, а з нею глухнув і
        виняток §4.11 — беклог у неї не запушиш.
        """
        self.close_the_only_transform()
        rev = self.rewrite_the_contract()
        body = WAVE.format(rev=rev).replace(
            "drive-turns:", "carry-tools:").replace(
            "finishes-when-no-tool-called", "asks-before-it-writes")
        self.fixture.write("keel/waves/0002-tools.md", body)
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план 0002")
        self.assertIn("session-run", keel.unbegun_contracts(self.project))

    def test_the_rewritten_contract_wakes_once_the_work_lands(self):
        """Одна закрита трансформа — і питати вже є в кого."""
        self.close_the_only_transform()
        rev = self.rewrite_the_contract()
        body = WAVE.format(rev=rev).replace(
            "drive-turns:", "carry-tools:").replace(
            "finishes-when-no-tool-called", "asks-before-it-writes")
        self.fixture.write("keel/waves/0002-tools.md", body)
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план 0002")
        self.fixture.git("commit", "--allow-empty", "-m", "carry-tools: зроблено")
        self.assertNotIn("session-run", keel.unbegun_contracts(self.project))


class TestAMergedBranchHandsOutNoWork(ProjectCase):
    """Знайдено живим: агент узяв пакет і пішов шукати, що там переробити.

    Закриття рахується по діапазону `merge_base..HEAD`. Щойно гілку злито,
    той діапазон порожній — точкою розходження стає сам HEAD, — і кожна
    трансформа читається як незакрита. `next` видавав першу заново.

    Відновити діапазон після злиття не можна й не треба: робота не береться
    з гілки, яка вже всередині.
    """

    def run_next(self):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_next(self.project, Args(json=False))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def work_branch(self):
        self.fixture.git("checkout", "-b", "0001-session-loop")

    def close_the_transform(self):
        self.fixture.git("commit", "--allow-empty", "-m", "drive-turns: зроблено")

    def merge_into_main(self):
        self.fixture.git("checkout", "main")
        self.fixture.git("merge", "--no-ff", "-m", "злиття", "0001-session-loop")
        self.fixture.git("checkout", "0001-session-loop")

    def test_a_merged_branch_says_the_wave_is_finished(self):
        self.work_branch()
        self.close_the_transform()
        self.merge_into_main()
        code, out = self.run_next()
        self.assertNotEqual(code, 0, out)
        self.assertIn("already merged", out)

    def test_a_fresh_branch_still_hands_out_the_first_transform(self):
        """Межа, об яку легко спіткнутись: свіжа гілка теж міститься в main.

        Комітів на ній ще немає, тож git ці два стани не розрізняє. Розрізняє
        історія main — вона закриває трансформи цієї хвилі лише після злиття.
        """
        self.work_branch()
        code, out = self.run_next()
        self.assertEqual(code, 0, out)
        self.assertIn("drive-turns", out)

    def test_a_branch_with_work_on_it_is_untouched(self):
        self.work_branch()
        self.fixture.git("commit", "--allow-empty", "-m", "щось інше")
        code, out = self.run_next()
        self.assertEqual(code, 0, out)
        self.assertIn("drive-turns", out)


class TestAFabricatedRevisionIsNamedAsOne(ProjectCase):
    """`@000000` — не застаріле посилання, а вигадане.

    ЗНАЙДЕНО ПРОГОНОМ 25 серпня 2026: моделі ставили заповнювач замість гаша,
    а засіб відповідав «and the contract is now …» — так, ніби контракт
    змінився під ними. Модель ішла шукати ту зміну й правила текст контракту.
    """

    def hold(self, rev):
        wave = "keel/waves/0001-session-loop.md"
        text = self.fixture.read(wave).replace(
            f"session-run@{self.fixture.contract_rev}", f"session-run@{rev}")
        self.fixture.write(wave, text)

    def test_a_revision_that_never_was_says_so(self):
        self.hold("000000")
        problems = keel.check_revisions(self.project)
        self.assertTrue(problems)
        self.assertIn("no version of that contract ever had this revision",
                      problems[0].message)
        self.assertIn("keel rev", problems[0].message)

    def test_a_revision_that_once_matched_is_plain_drift(self):
        """Той самий вигляд посилання, інша історія — інша порада."""
        staryj = self.fixture.contract_rev
        self.fixture.write("keel/contracts/session-run.md",
                           self.fixture.read("keel/contracts/session-run.md")
                           + "\nДодали абзац, і редакція змінилась.\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "контракт зріс")

        self.hold(staryj)                       # редакція, яка справді була
        problems = keel.check_revisions(self.project)
        self.assertTrue(problems)
        self.assertIn("and the contract is now", problems[0].message)
        self.assertNotIn("ever had this revision", problems[0].message)

    def test_rev_stamps_the_fabricated_one_away(self):
        """Порада має справджуватись: `keel rev` і справді це лагодить."""
        self.hold("000000")
        keel.cmd_rev(self.project, Args(write=True, wave=None))
        self.assertEqual(keel.check_revisions(self.project), [])


class TestADocumentInTheWrongFolderSaysSo(ProjectCase):
    """Контракт із полями хвилі — не контракт, а хвиля не в тій теці.

    ЗНАЙДЕНО ПРОГОНОМ 25 серпня 2026: гема 26B поклала `proves:` і `scenarios:`
    у файл контракту. Заголовок контракту читає лише свої поля, чужі минав
    мовчки — тож файл проходив перевірку, а плану не існувало.
    """

    def test_wave_fields_in_a_contract_are_reported(self):
        self.fixture.write("keel/contracts/session-run.md",
                           "---\nmodule: Demo.Session\n"
                           "scenarios:\n  finishes: {proves: session-run}\n---\n\nТекст.\n")
        contract = self.project.contracts["session-run"]
        self.assertIsNotNone(contract.error)
        self.assertIn("scenarios", contract.error)
        self.assertIn("keel/waves/", contract.error)

    def test_contract_fields_in_a_wave_are_reported(self):
        self.fixture.write("keel/waves/0002-stray.md",
                           "---\nmodule: Demo.Stray\nexports: [run/1]\n---\n\nТекст.\n")
        wave = self.project.waves["0002-stray"]
        self.assertIsNotNone(wave.error)
        self.assertIn("module", wave.error)
        self.assertIn("keel/contracts/", wave.error)

    def test_a_plain_contract_stays_clean(self):
        """Межа: своє поле чужим не стає."""
        self.assertIsNone(self.project.contracts["session-run"].error)
        self.assertIsNone(self.project.waves["0001-session-loop"].error)


class TestAPlanBranchThatNamesNoWave(ProjectCase):
    """Гілка `plan/X`, коли хвилі `X` немає, — скарга, а не інша порода гілки.

    ЗНАЙДЕНО ПРОГОНОМ 25 серпня 2026. Laguna назвала гілку `plan/meter-readings`
    при файлі `keel/waves/0001-meter-readings.md` — зрізала номер, проти чого
    §8.2 застерігає дослівно. Далі сталося те, чого ніхто не казав: засіб не
    знайшов хвилі, перестав вважати гілку плановою, і перевірки 5, 6 та CI
    встали — тобто зажадали тестів і модуля на гілці, де коду свідомо немає.

    Та сама пісочниця на правильно названій гілці давала `clean`, а тут —
    `problems: 8`. Завдання при цьому забороняло писати код, і модель витратила
    десятки ходів, зʼясовуючи, чому засіб суперечить власній методиці.
    """

    def capture(self, args):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_check(self.project, args)
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def zlamana(self):
        """Та сама помилка, що й у прогоні: номер зрізано."""
        self.fixture.branch("plan/session-loop")

    def test_the_branch_is_named_and_so_is_the_candidate(self):
        self.zlamana()
        code, out = self.capture(Args(fast=False, no_tests=True, json=False))

        self.assertEqual(code, 1)
        self.assertIn("plan/session-loop", out)
        self.assertIn("names no wave", out)
        # Кандидат — цілим імʼям файлу, разом із номером: саме його бракувало.
        self.assertIn("plan/0001-session-loop", out)

    def test_checks_five_and_six_stand_down_all_the_same(self):
        """Головне: гілка планова за іменем, отже коду на ній немає."""
        self.zlamana()
        _code, out = self.capture(Args(fast=False, no_tests=False, json=False))

        self.assertIn("– 5. ", out)
        self.assertIn("– 6. ", out)
        self.assertNotIn("has no test", out)
        self.assertNotIn("did not build", out)

    def test_the_fast_run_refuses_too(self):
        """`pre-commit` кличе саме його: коміт на такій гілці не проходить."""
        self.zlamana()
        code, out = self.capture(Args(fast=True, no_tests=True, json=False))

        self.assertEqual(code, 1)
        self.assertIn("names no wave", out)

    def test_a_correctly_named_branch_is_untouched(self):
        """Межа: правильна гілка нічого не втратила."""
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.capture(Args(fast=False, no_tests=True, json=False))

        self.assertEqual(code, 0)
        self.assertNotIn("names no wave", out)
        self.assertIn("– 5. ", out)

    def test_a_branch_named_nothing_like_a_wave_gets_the_name(self):
        """Перевірка 4 каже дію, а не самий наслідок.

        ЗНАЙДЕНО ПРОГОНОМ 26 серпня 2026: гема назвала гілку `wave_1_parsing`
        при файлі `0001-wave-1-parsing.md` — ані `plan/`, ані номера, ані рисок.
        Засіб казав, що звіряти обсяг нема з чим, і на цьому все.
        """
        self.fixture.branch("session_loop")
        _code, out = self.capture(Args(fast=True, no_tests=True, json=False))

        self.assertIn("Rename it to", out)
        self.assertIn("0001-session-loop", out)

    def test_a_branch_resembling_nothing_says_only_what_is_true(self):
        """Межа: схожої хвилі немає — імені не вигадуємо."""
        self.fixture.branch("зовсім-стороннє")
        _code, out = self.capture(Args(fast=True, no_tests=True, json=False))

        self.assertIn("there is nothing to compare scope against", out)
        self.assertNotIn("Rename it to", out)

    def test_a_work_branch_still_runs_the_code_checks(self):
        """Межа: не планова гілка — не стає нічого."""
        self.fixture.branch("0001-session-loop")
        _code, out = self.capture(Args(fast=False, no_tests=True, json=False))

        self.assertNotIn("names no wave", out)
        self.assertIn("5. ", out)
        self.assertNotIn("– 5. ", out)

    def test_without_a_candidate_the_waves_that_exist_are_named(self):
        """Схожої хвилі немає — вгадувати нічого, але назвати наявні можна.

        ЗНАЙДЕНО ПРОГОНОМ 26 серпня 2026: тут стояло правило замість дії — «the
        branch is named after the wave file, number and all». Гема 26B зробила
        з нього зворотний висновок і покликала `new wave wave-1`, щоб хвиля
        збіглася з гілкою. Вибір із переліку такого не допускає.
        """
        self.fixture.branch("plan/зовсім-інше")
        _code, out = self.capture(Args(fast=False, no_tests=True, json=False))

        self.assertIn("names no wave", out)
        self.assertIn("0001-session-loop", out)
        self.assertIn("rename the branch", out)
        self.assertNotIn("Rename it to plan/зовсім", out)

    def test_with_no_waves_at_all_the_wave_comes_first(self):
        """Перейменовувати гілку нема під що — спершу хвиля."""
        os.remove(self.fixture.path("keel/waves/0001-session-loop.md"))
        self.fixture.branch("plan/wave-1")
        _code, out = self.capture(Args(fast=False, no_tests=True, json=False))

        self.assertIn("keel/waves/ is empty", out)
        self.assertIn("keel new wave", out)
        # Головне: наказ, а не опис норми. Саме опис і збив гему.
        self.assertNotIn("The branch is named after the wave file", out)

    def test_json_says_it_too(self):
        """Скрипти читають цей вантаж, а не прозу під ним."""
        self.zlamana()
        code, out = self.capture(Args(fast=False, no_tests=True, json=True))
        payload = json.loads(out)

        self.assertEqual(code, 1)
        self.assertFalse(payload["ok"])
        self.assertTrue(payload["branch"])
        self.assertIn("plan/0001-session-loop", payload["branch"][0])

    def test_next_names_the_candidate_instead_of_reciting_the_rule(self):
        """`next` — поводир, і саме він першим натрапляє на цю гілку."""
        self.zlamana()
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_next(self.project, Args(json=False, wave=None))
        finally:
            sys.stdout = saved

        said = stream.getvalue()
        self.assertIn("plan/0001-session-loop", said)
        self.assertNotIn("planning on plan/<wave>", said)


class TestTheExampleCostsNothing(ProjectCase):
    """Зразок хвилі показують, не створюючи хвилі.

    ЗНАЙДЕНО ПРОГОНОМ 26 серпня 2026: заповнений приклад друкувався ЛИШЕ після
    створення файлу. Гема 26B, якій він був потрібен, покликала
    `new wave test-wave` — і в теці лишилась хвиля-заготовка, через яку `gaps`
    рахував зайві прогалини, а заслон вимагав її прибрати.
    """

    def said(self, **kwargs):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_new(self.project, Args(**kwargs))
        except SystemExit as stop:
            code = stop.code
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def test_a_wave_without_a_slug_shows_the_example(self):
        bulo = sorted(os.listdir(self.fixture.path("keel/waves")))
        code, out = self.said(kind="wave", slug=None)

        self.assertEqual(code, 0)
        self.assertIn("filled in", out)
        self.assertIn("proves:", out)
        # Головне: нічого не створено.
        self.assertEqual(sorted(os.listdir(self.fixture.path("keel/waves"))), bulo)

    def test_the_example_is_the_same_one_new_prints_after_creating(self):
        """Одне джерело: копія в довідці розійшлася б першої ж правки форми."""
        _code, bez_slaga = self.said(kind="wave", slug=None)
        _code, zi_slagom = self.said(kind="wave", slug="proba-zrazka")

        self.assertIn(bez_slaga.strip(), zi_slagom)

    def test_the_general_help_leads_to_the_example(self):
        """Підказка мусить бути ЗНАЙДЕНОЮ, а не просто наявною.

        ЗНАЙДЕНО ПРОГОНОМ 26 серпня 2026: `keel new wave` без імені показує
        приклад, і це записано в довідці самої команди. Але моделі читають
        `keel --help`, де стояло тільки «skeleton of a wave or a contract».
        Devstral шість заходів вгадувала форму шапки, аж поки не завела зайву
        хвилю, щоб приклад побачити; гема зробила так само.
        """
        parser = keel.build_parser()
        for action in parser._actions:
            if isinstance(action, keel.argparse._SubParsersAction):
                pro_new = action.choices["new"]
                break
        else:
            self.fail("підкоманд не знайдено")

        # Довідка самої команди — там опис аргументу.
        assert pro_new.format_help()
        # І головне: загальний перелік теж веде до прикладу.
        # Перенос рядка ставить argparse — звіряємо по слову, що не ламається.
        vzahali = " ".join(parser.format_help().split())
        self.assertIn("without a name, an example", vzahali)

    def test_a_contract_without_a_slug_says_it_has_no_example(self):
        """Межа: зразок є лише в хвилі, і вигадувати його для контракту не з чого."""
        from io import StringIO
        stream, saved = StringIO(), sys.stderr
        sys.stderr = stream
        try:
            keel.cmd_new(self.project, Args(kind="contract", slug=None))
        except SystemExit:
            pass
        finally:
            sys.stderr = saved

        self.assertIn("keel new contract", stream.getvalue())


class TestACommandNameIsNotAWave(ProjectCase):
    """`keel show next` — той, хто це пише, уже знає потрібне слово.

    ЗНАЙДЕНО ПРОГОНОМ 25 серпня 2026: Laguna шукала поводиря й майже намацала
    його — `next` існує рівно під цим імʼям. Засіб відповів «no such wave: next»
    і про власну команду змовчав.
    """

    def refusal(self, call, **kwargs):
        """`fail()` каже в stderr — там і слухаємо."""
        from io import StringIO
        stream, saved = StringIO(), sys.stderr
        sys.stderr = stream
        try:
            call(self.project, Args(**kwargs))
        except SystemExit:
            pass
        finally:
            sys.stderr = saved
        return stream.getvalue()

    def test_show_names_the_command(self):
        said = self.refusal(keel.cmd_show, wave="next", json=False)
        self.assertIn("no such wave: next", said)
        self.assertIn("keel next", said)

    def test_gaps_names_it_too(self):
        said = self.refusal(keel.cmd_gaps, wave="next", json=False)
        self.assertIn("keel next", said)

    def test_a_plain_typo_stays_a_plain_refusal(self):
        """Межа: не команда — не вигадуємо поради."""
        said = self.refusal(keel.cmd_show, wave="сесія-якої-немає", json=False)
        self.assertIn("no such wave", said)
        self.assertNotIn("command with that name", said)

    def test_the_names_come_from_the_parser(self):
        """Перелік руками розійшовся б із розбирачем, і розійшовся б тихо."""
        names = keel.command_names()
        for known in ("next", "gaps", "check", "show", "rev"):
            self.assertIn(known, names)


class TestNextAnswersOnAFinishedPlan(ProjectCase):
    """Поводир мовчав саме там, куди довів.

    ЗНАЙДЕНО ПРОГОНОМ 25 серпня 2026: Laguna довела план до кінця — `gaps` казав
    «the plan is complete», — а `next` і далі відсилав до `gaps`. Двоє вбудованих
    порадників кивали один на одного, і жоден не казав, що робити.
    """

    def said(self):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_next(self.project, Args(json=False, wave=None))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def test_a_complete_plan_gets_the_next_move(self):
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.said()

        self.assertEqual(code, 0)
        self.assertIn("complete", out)
        self.assertIn("pull request", out)
        self.assertNotIn("keel gaps names", out)

    def test_an_unfinished_plan_still_points_at_gaps(self):
        """Межа: доки бракує — каже, скільки саме, і хто це назве."""
        self.fixture.write("keel/waves/0001-session-loop.md",
                           self.fixture.read("keel/waves/0001-session-loop.md")
                           .replace("## Навіщо\n\nОдна розмова з моделлю проти "
                                    "набору інструментів, який дали ззовні.",
                                    "## Навіщо"))
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.said()

        self.assertEqual(code, 1)
        self.assertIn("keel gaps names", out)


class TestAStrayBesideARealWave(ProjectCase):
    """Файл, схожий на хвилю, помічають і тоді, коли справжня хвиля вже є.

    ЗНАЙДЕНО ПРОГОНОМ 26 серпня 2026. Перевірку додали напередодні, і вона
    казала своє лише в гілці «хвиль немає зовсім». Кодер поклав `keel/wave1.md`
    поруч із справжньою хвилею — і засіб змовчав, бо хвиля вже була. Тобто
    перевірка не працювала в найзвичайнішому випадку.
    """

    def skarhy(self):
        waves = list(self.project.waves.values())
        return "\n".join(p.message for p in keel.gaps_problems(self.project, waves))

    def test_a_stray_is_named_even_when_a_wave_exists(self):
        self.fixture.write("keel/wave1.md", "# Wave 1\n\nчернетка\n")
        said = self.skarhy()

        self.assertIn("keel/wave1.md", said)
        self.assertIn("is not in keel/waves/", said)

    def test_without_strays_nothing_is_said(self):
        """Межа: у чистому проєкті про заблуди мовчимо."""
        self.assertNotIn("looks like a wave", self.skarhy())

    def test_the_methodology_itself_is_not_a_stray(self):
        """Межа: `keel/METHODOLOGY.md` теж `.md` у `keel/`, і він не хвиля."""
        self.fixture.write("keel/METHODOLOGY.md", "# Метод\n")
        self.assertNotIn("METHODOLOGY", self.skarhy())


class TestTheHeaderSaysEveryFault(ProjectCase):
    """Шапка, що не розібралась, називає все, що видно, — а не першу ваду.

    ЗНАЙДЕНО ПРОГОНОМ 26 серпня 2026. Devstral писала хвилю руками й шість
    разів вгадувала форму: словник → список → словник → список. Помилок у неї
    було пʼять, а `parse_yaml` спинявся на першій, тож відповідь щоразу
    приходила одна. Сорок два ходи, пів мільйона токенів контексту, робота не
    зрушила: кожна правка відкривала наступну ваду.
    """

    def zlamana(self, header):
        self.fixture.write("keel/waves/0001-session-loop.md",
                           "---\n" + header + "---\n\n## Навіщо\n\nтекст\n")
        return keel.gaps_problems(self.project, [self.project.waves["0001-session-loop"]])

    def test_contracts_in_a_wave_are_told_where_they_belong(self):
        """ЗНАЙДЕНО ПРОГОНАМИ 26 серпня 2026 — Devstral і Laguna, обидві.

        Обидві оголосили контракт УСЕРЕДИНІ хвилі, з полями й експортами, хоча
        файл `keel/contracts/…` уже створили. Скарга називала, чого не можна
        («contracts is not a field of a wave»), і мовчала про те, де контракти
        живуть. Laguna на цьому вигоріла 47 ходів і спинилась заслоном кола.

        Поле, назване чужим без указання місця, — це половина відповіді.
        """
        problems = self.zlamana(
            "depends_on: []\n"
            "scenarios:\n"
            "  parse: {proves: meter-reader}\n"
            "contracts:\n"
            "  meter_reader:\n"
            "    exports: [parse-reading]\n")
        said = "\n".join(problem.message for problem in problems)

        self.assertIn("contracts is not a field of a wave", said)
        # Куди подіти — своїм файлом і своїм рядком у трансформі.
        self.assertIn("keel/contracts/", said)
        self.assertIn("contracts: [", said)

    def test_a_field_that_is_merely_foreign_says_nothing_extra(self):
        """Порада про місце — лише там, де місце є. `title` нікуди не
        переносять, і вигадувати йому дім було б новою неправдою."""
        problems = self.zlamana(
            "depends_on: []\n"
            "title: Parse Meter Readings\n"
            "scenarios:\n"
            "  parse: {proves: meter-reader}\n")
        said = "\n".join(problem.message for problem in problems)

        self.assertIn("title is not a field of a wave", said)
        self.assertNotIn("keel/contracts/", said)

    def test_the_real_header_from_the_run_gets_every_fault(self):
        """Та сама шапка, на якій Devstral крутилась сорок два ходи."""
        problems = self.zlamana(
            "slug: 1-parse\n"
            "title: Parse Meter Readings\n"
            "depends_on: []\n"
            "scenarios:\n"
            "  - parse-various-formats\n"
            "contracts:\n"
            "  - slug: meter-reading-format\n")
        said = "\n".join(problem.message for problem in problems)

        # Вигадані поля — усі три, поіменно.
        for field in ("slug", "title", "contracts"):
            self.assertIn(f"{field} is not a field of a wave", said)
        # Карта в списку — з самим рядком, а не самим номером.
        self.assertIn("- slug: meter-reading-format", said)
        # І готова форма з її ж іменем сценарію.
        self.assertIn("parse-various-formats: {proves: contract@rev}", said)

    def test_the_known_fields_are_named(self):
        """Відмова каже не лише «не те», а й що буває."""
        said = "\n".join(p.message for p in self.zlamana("titel: щось\n"))
        self.assertIn("depends_on, scenarios, transforms", said)

    def test_a_good_header_says_nothing(self):
        """Межа: жодної скарги на здоровій шапці."""
        problems = keel.gaps_problems(
            self.project, [self.project.waves["0001-session-loop"]])
        said = "\n".join(problem.message for problem in problems)

        self.assertNotIn("is not a field", said)
        self.assertNotIn("map inside a list", said)

    def test_a_list_of_plain_names_is_left_alone(self):
        """Межа: `depends_on: [a, b]` — законний список, не злам."""
        said = "\n".join(p.message for p in self.zlamana(
            "depends_on:\n  - 0002-other\n  - 0003-third\n"
            "scenarios:\n  парсер: {proves: session-run}\n"
            "transforms:\n  крутити: {files: [lib/a.ex]}\n"))
        self.assertNotIn("map inside a list", said)
        self.assertNotIn("is not a field", said)

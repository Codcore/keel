#!/usr/bin/env python3
"""new, gaps, next, rev, check — the commands a person runs."""

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

    def test_on_main_it_names_the_step_that_is_ready(self):
        """Було «гілка не названа за кроком» — правда, але не відповідь."""
        code, out = self.run_next()
        self.assertEqual(code, 1)
        self.assertIn("git checkout -b 0001-session-loop", out)

    def test_a_branch_that_is_neither_main_nor_a_step_still_says_the_rule(self):
        self.fixture.branch("spike/whatever")
        code, out = self.run_next()
        self.assertEqual(code, 1)
        self.assertIn("is not named after a wave", out)

    def test_refuses_on_plan_branch(self):
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.run_next()
        self.assertEqual(code, 1)
        self.assertIn("this is a plan branch", out)

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

    def test_new_step_takes_the_next_number(self):
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="wave", slug="Tool Calls"))
        self.assertEqual(code, 0)
        self.assertIn("0002-tool-calls.md", out)
        self.assertTrue(os.path.exists(self.fixture.path("keel/waves/0002-tool-calls.md")))

    def test_new_step_skeleton_parses(self):
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

    def test_gaps_without_an_argument_names_only_its_own_step(self):
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

    def test_links_resolve_from_the_step_file(self):
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

    def test_an_unknown_step_refuses(self):
        with self.assertRaises(SystemExit):
            self.show("0009-nope")


class TestGapsAsksAboutAForgottenEdge(ProjectCase):
    """Порожнє `depends_on` виглядає однаково: залежності немає чи її забули.

    Перевірка 2 шукає цикли, а в порожньому графі їх не буває, тож забуте ребро
    не бачив ніхто. Питання, не вирок: два кроки можуть законно правити один
    файл, не спираючись один на одного.
    """

    def second_step(self, depends="[]", files="[lib/session.ex]",
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
        self.second_step()
        self.assertTrue(any("lib/session.ex" in m for m in self.messages()),
                        self.messages())

    def test_the_edge_silences_it(self):
        self.second_step(depends="[0001-session-loop]")
        self.assertEqual(self.messages(), [])

    def test_the_step_that_is_leaned_on_is_not_asked(self):
        """Напрямок читається з графа, а не з номерів у назвах."""
        self.second_step(depends="[0001-session-loop]")
        self.assertEqual(self.messages("0001-session-loop"), [])

    def test_a_shared_contract_alone_is_not_asked_about(self):
        """Спиратись на спільну обіцянку — не те саме, що залежати від кроку,
        який її написав. Питання про це було майже завжди хибним і давало
        N×(N−1) рядків на один поширений контракт."""
        self.second_step(files="[lib/nothing_shared.ex]")
        self.assertEqual(self.messages(), [])

    def test_a_dependency_two_steps_away_still_counts_as_named(self):
        self.second_step(depends="[0001-session-loop]")
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

    def test_a_contract_this_step_brings_and_nobody_names_is_asked_about(self):
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

    def add_step(self, slug, body):
        self.fixture.write(f"keel/waves/{slug}.md", body)
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", f"план {slug}")

    def test_the_ready_step_is_named_with_its_branch(self):
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
        self.add_step("0002-empty", keel.step_skeleton("0002-empty"))
        answer = self.answer()
        self.assertIn("0002-empty", answer)
        self.assertIn("the plan is not written yet", answer)

    def test_a_step_whose_ground_is_not_laid_is_not_offered(self):
        self.close_the_only_transform()
        # Своя назва трансформи: інакше комміт, що закрив `drive-turns`
        # у першому кроці, закриває однойменну і в другому.
        self.add_step("0002-later", self.fixture.read(
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
                           keel.step_skeleton("0002-left-behind"))

    def test_a_stray_step_is_named_so_nobody_moves_it(self):
        self.stray_skeleton()
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.capture(Args(fast=True, no_tests=True, json=False))
        self.assertEqual(code, 1)
        self.assertIn("0002-left-behind", out)
        self.assertIn("not moved, renamed or deleted", out)

    def test_the_branch_own_step_earns_no_such_note(self):
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

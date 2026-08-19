#!/usr/bin/env python3
"""new, gaps, next, rev, check — the commands a person runs."""

import os
import re
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args, CONTRACT, ProjectCase, STEP  # noqa: E402




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

    def test_refuses_on_main(self):
        code, out = self.run_next()
        self.assertEqual(code, 1)
        self.assertIn("is not named after a step", out)

    def test_refuses_on_plan_branch(self):
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.run_next()
        self.assertEqual(code, 1)
        self.assertIn("this is a plan branch", out)

    def test_refuses_while_plan_is_not_in_main(self):
        self.fixture.branch("plan/0002-later")
        self.fixture.write("keel/steps/0002-later.md", STEP.format(rev=self.fixture.contract_rev))
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
        self.assertIn(self.fixture.contract_rev, self.fixture.read("keel/steps/0001-session-loop.md"))

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
        step = "keel/steps/0001-session-loop.md"
        text = self.fixture.read(step).replace(
            f"session-run@{self.fixture.contract_rev}", "session-run")
        text = text.replace("Крутити ходи", "Крутити ходи session-run")
        self.fixture.write(step, text)
        self.run_rev(write=True)
        after = self.fixture.read(step)
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
                                 Args(kind="step", slug="Tool Calls"))
        self.assertEqual(code, 0)
        self.assertIn("0002-tool-calls.md", out)
        self.assertTrue(os.path.exists(self.fixture.path("keel/steps/0002-tool-calls.md")))

    def test_new_step_skeleton_parses(self):
        self.capture(keel.cmd_new, self.project, Args(kind="step", slug="tool-calls"))
        step = self.project.steps["0002-tool-calls"]
        self.assertIsNone(step.error)

    def test_new_contract(self):
        code, out = self.capture(keel.cmd_new, self.project,
                                 Args(kind="contract", slug="tool-registry"))
        self.assertEqual(code, 0)
        self.assertIn("keel/contracts/tool-registry.md", out)

    def test_new_refuses_to_overwrite(self):
        with self.assertRaises(SystemExit):
            self.capture(keel.cmd_new, self.project, Args(kind="contract", slug="session-run"))

    def test_plan_is_complete(self):
        code, out = self.capture(keel.cmd_gaps, self.project, Args(step="0001-session-loop"))
        self.assertEqual(code, 0)
        self.assertIn("the plan is complete", out)

    def test_gaps_without_an_argument_names_only_its_own_step(self):
        """Заголовок казав один крок, а список — інший."""
        self.fixture.write("keel/steps/0009-other.md",
                           "---\ndepends_on: []\nscenarios:\n  zzz: {proves: session-run}\n"
                           "transforms: {}\n---\n\n## Навіщо\n\nx\n")
        self.fixture.branch("0001-session-loop")
        code, out = self.capture(keel.cmd_gaps, self.project, Args(step=None))
        self.assertNotIn("0009-other", out)

    def test_plan_finds_a_transform_without_files(self):
        self.fixture.write(
            "keel/steps/0001-session-loop.md",
            self.fixture.read("keel/steps/0001-session-loop.md").replace(
                "    files:      [lib/session.ex]\n", ""))
        code, out = self.capture(keel.cmd_gaps, self.project, Args(step="0001-session-loop"))
        self.assertEqual(code, 1)
        self.assertIn("declared no files", out)

    def test_plan_finds_a_scenario_nobody_implements(self):
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        text = text.replace(
            "  finishes-when-no-tool-called: ",
            "  only-handed-tools-are-callable: {proves: session-run@%s}\n  finishes-when-no-tool-called: "
            % self.fixture.contract_rev)
        text += "\n## scenario: only-handed-tools-are-callable\n\n**Then** інших немає.\n"
        self.fixture.write("keel/steps/0001-session-loop.md", text)
        code, out = self.capture(keel.cmd_gaps, self.project, Args(step="0001-session-loop"))
        self.assertEqual(code, 1)
        self.assertIn("no transform implements scenario", out)

    def test_new_skeleton_is_not_a_complete_plan(self):
        self.capture(keel.cmd_new, self.project, Args(kind="step", slug="tool-calls"))
        code, out = self.capture(keel.cmd_gaps, self.project, Args(step="0002-tool-calls"))
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

    def show(self, step=None):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_show(self.project, Args(step=step))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def test_links_resolve_from_the_step_file(self):
        code, out = self.show("0001-session-loop")
        self.assertEqual(code, 0)
        base = os.path.dirname(self.fixture.path("keel/steps/0001-session-loop.md"))
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
        step = "keel/steps/0001-session-loop.md"
        self.fixture.write(step, self.fixture.read(step).replace(
            "files:      [lib/session.ex]",
            "files:      [lib/session.ex, lib/поки_немає.ex]"))
        _, out = self.show("0001-session-loop")
        self.assertIn("not there yet", out)
        self.assertIn("lib/поки_немає.ex", out)

    def test_an_unknown_step_refuses(self):
        with self.assertRaises(SystemExit):
            self.show("0009-nope")


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
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        head, _, _ = text.partition("## scenario:")
        self.fixture.write("keel/steps/0001-session-loop.md", head)
        self.fixture.branch("0001-session-loop")
        step = self.project.steps["0001-session-loop"]
        slug, state = keel.next_transform(self.project, step)
        package = keel.next_package(self.project, step, slug, state)
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
        for folder in ("keel/steps", "keel/contracts", "tests"):
            os.makedirs(os.path.join(root, folder))
        with open(os.path.join(root, "pyproject.toml"), "w") as handle:
            handle.write("[project]\nname='d'\n")
        subprocess.run(["git", "init", "-b", "main", "-q", root], check=True)
        for key, value in (("user.email", "t@e"), ("user.name", "t")):
            subprocess.run(["git", "-C", root, "config", key, value], check=True)
        with open(os.path.join(root, "keel/steps/0001-a.md"), "w",
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
        body = keel.Project(root).steps["0001-a"].scenario_body("does-a")
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
        for folder in ("keel/steps", "keel/contracts", "tests"):
            os.makedirs(os.path.join(self.root, folder))
        with open(os.path.join(self.root, "pyproject.toml"), "w") as handle:
            handle.write("[project]\nname='d'\n")
        with open(os.path.join(self.root, "keel/steps/0001-a.md"), "w",
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

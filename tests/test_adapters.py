#!/usr/bin/env python3
"""Checks 5 and 6: tests and exports, through the language adapters."""

import os
import shutil
import tempfile
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import ProjectCase  # noqa: E402




# ─────────────────────────────────────────────────────────────────────────────
# Check 5: scenarios and tags
# ─────────────────────────────────────────────────────────────────────────────

class TestScenarios(ProjectCase):
    def tag(self, rev, slug="finishes_when_no_tool_called"):
        self.fixture.write(
            "test/session_test.exs",
            f'defmodule Demo.SessionTest do\n'
            f'  @tag proves: :{slug}, rev: "{rev}"\n'
            f'  test "розмова завершується" do\n'
            f'    assert true\n'
            f'  end\n'
            f'end\n')

    def test_adapter_is_elixir(self):
        self.assertEqual(self.project.adapter.name, "elixir")

    def test_missing_test_is_a_problem(self):
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertEqual(len(problems), 1)
        self.assertIn("не має тесту", problems[0].message)

    def test_tag_with_right_revision_is_clean(self):
        self.tag(self.fixture.scenario_rev())
        self.assertEqual(keel.check_scenarios(self.project, run_tests=False), [])

    def test_tag_with_stale_revision(self):
        self.tag("deadbe")
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertEqual(len(problems), 1)
        self.assertIn("тримає редакцію", problems[0].message)
        self.assertEqual(problems[0].where, "test/session_test.exs")

    def test_tag_without_revision(self):
        self.fixture.write(
            "test/session_test.exs",
            'defmodule Demo.SessionTest do\n'
            '  @tag proves: :finishes_when_no_tool_called\n'
            '  test "x", do: assert true\n'
            'end\n')
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertIn("без редакції", problems[0].message)

    def test_scenario_text_edited_makes_the_tag_stale(self):
        self.tag(self.fixture.scenario_rev())
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        self.fixture.write("keel/steps/0001-session-loop.md",
                           text.replace("розмова завершується", "розмова завершується сама"))
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertEqual(len(problems), 1)
        self.assertIn("тримає редакцію", problems[0].message)

    def test_slug_dashes_match_atom_underscores(self):
        self.assertEqual(keel.normalise_slug("finishes_when_no_tool_called"),
                         keel.normalise_slug("finishes-when-no-tool-called"))


# ─────────────────────────────────────────────────────────────────────────────
# Check 6: exports (the Python adapter — it runs without mix)
# ─────────────────────────────────────────────────────────────────────────────




# ─────────────────────────────────────────────────────────────────────────────
# Check 6: exports (the Python adapter — it runs without mix)
# ─────────────────────────────────────────────────────────────────────────────

class TestExports(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-py-")
        self.addCleanup(shutil.rmtree, self.root, True)
        os.makedirs(os.path.join(self.root, "keel/contracts"))
        os.makedirs(os.path.join(self.root, "keel/steps"))
        self.write("pyproject.toml", "[project]\nname = 'demo'\n")
        self.write("demo.py", "def run(a, b, c):\n    return a\n")

    def write(self, name, text):
        with open(os.path.join(self.root, name), "w", encoding="utf-8") as handle:
            handle.write(text)

    def contract(self, exports):
        self.write("keel/contracts/demo.md",
                   f"---\nmodule: demo\nexports: {exports}\n---\n\nЩо обіцяє demo.\n")

    def test_adapter_is_python(self):
        self.contract("[run/3]")
        self.assertEqual(keel.Project(self.root).adapter.name, "python")

    def test_promised_export_exists(self):
        self.contract("[run/3]")
        self.assertEqual(keel.check_exports(keel.Project(self.root)), [])

    def test_promised_export_missing(self):
        self.contract("[halt/1]")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("не експортує обіцяне: halt/1", problems[0].message)

    def test_wrong_arity_is_missing(self):
        self.contract("[run/2]")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertIn("run/2", problems[0].message)

    def test_a_promise_that_is_not_a_module_is_proved_by_a_command(self):
        """Ollama на порту, бінарник на PATH, ліба потрібної версії."""
        self.write("keel/contracts/runtime.md",
                   "---\nverify: \"true\"\n---\n\nЩось, що має працювати.\n")
        self.assertEqual(keel.check_exports(keel.Project(self.root)), [])

    def test_a_command_that_fails_is_a_broken_contract(self):
        self.write("keel/contracts/runtime.md",
                   "---\nverify: \"echo нема така служба >&2; exit 1\"\n---\n\nСлужба.\n")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("не підтвердився", problems[0].message)
        self.assertIn("нема така служба", problems[0].message)

    def test_a_contract_may_carry_both_a_module_and_a_command(self):
        self.contract("[run/3]")
        self.write("keel/contracts/runtime.md",
                   "---\nverify: \"false\"\n---\n\nСлужба.\n")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("runtime.md", problems[0].where)

    def test_module_absent(self):
        self.write("keel/contracts/demo.md",
                   "---\nmodule: nosuchmodule\nexports: [run/1]\n---\n\nТекст.\n")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertIn("не зібрався", problems[0].message)


# ─────────────────────────────────────────────────────────────────────────────
# keel next
# ─────────────────────────────────────────────────────────────────────────────


if __name__ == "__main__":
    unittest.main()

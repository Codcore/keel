#!/usr/bin/env python3
"""Tests for keel.py. Standard library only, like the tool itself.

    python3 -m unittest discover -s tests -t .
"""

import os
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402

try:                                 # only to check what we generate, never to run
    import yaml                      # noqa: F401
    HAS_PYYAML = True
except ImportError:
    HAS_PYYAML = False


# ─────────────────────────────────────────────────────────────────────────────
# YAML
# ─────────────────────────────────────────────────────────────────────────────

class TestYaml(unittest.TestCase):
    def test_scalars_and_flow_list(self):
        parsed = keel.parse_yaml('module: Foo.Bar\nexports: [run/3, halt/1]\n')
        self.assertEqual(parsed["module"], "Foo.Bar")
        self.assertEqual(parsed["exports"], ["run/3", "halt/1"])

    def test_empty_flow_list(self):
        self.assertEqual(keel.parse_yaml("depends_on: []")["depends_on"], [])

    def test_flow_map_inside_block_map(self):
        parsed = keel.parse_yaml(
            "scenarios:\n"
            "  finishes-when-no-tool-called:   {proves: session-run@7c40de}\n"
            "  only-handed-tools-are-callable: {proves: session-run@7c40de}\n"
        )
        self.assertEqual(
            parsed["scenarios"]["finishes-when-no-tool-called"],
            {"proves": "session-run@7c40de"},
        )
        self.assertEqual(len(parsed["scenarios"]), 2)

    def test_three_levels_with_block_list(self):
        parsed = keel.parse_yaml(
            "transforms:\n"
            "  drive-turns:\n"
            "    implements: [a, b]\n"
            "    files:\n"
            "      - lib/session.ex\n"
            "      - test/session_test.exs\n"
            "    commit: (open)\n"
        )
        transform = parsed["transforms"]["drive-turns"]
        self.assertEqual(transform["files"], ["lib/session.ex", "test/session_test.exs"])
        self.assertEqual(transform["commit"], "(open)")

    def test_top_level_block_list(self):
        self.assertEqual(keel.parse_yaml("depends_on:\n  - one\n  - two\n")["depends_on"],
                         ["one", "two"])

    def test_comments_and_blank_lines_ignored(self):
        parsed = keel.parse_yaml(
            "# заголовок\n"
            "module: Foo   # праворуч теж\n"
            "\n"
            "exports: []\n"
        )
        self.assertEqual(parsed, {"module": "Foo", "exports": []})

    def test_hash_inside_quotes_is_not_a_comment(self):
        self.assertEqual(keel.parse_yaml('note: "a # b"')["note"], "a # b")

    def test_empty_value_is_none(self):
        self.assertIsNone(keel.parse_yaml("scenarios:\n")["scenarios"])

    def test_duplicate_key_is_an_error(self):
        with self.assertRaises(keel.YamlError):
            keel.parse_yaml("module: A\nmodule: B\n")

    def test_tab_indent_is_an_error(self):
        with self.assertRaises(keel.YamlError):
            keel.parse_yaml("transforms:\n\tone: 1\n")

    def test_unclosed_bracket_is_an_error(self):
        with self.assertRaises(keel.YamlError):
            keel.parse_yaml("exports: [a, b\n")

    def test_missing_colon_is_an_error(self):
        with self.assertRaises(keel.YamlError) as caught:
            keel.parse_yaml("module Foo\n")
        self.assertIn("рядок 1", str(caught.exception))


# ─────────────────────────────────────────────────────────────────────────────
# Revisions
# ─────────────────────────────────────────────────────────────────────────────

class TestRevision(unittest.TestCase):
    def test_repeated_whitespace_collapses(self):
        self.assertEqual(keel.revision("а  б\n\nв"), keel.revision("а б в"))

    def test_comma_changes_the_revision(self):
        self.assertNotEqual(keel.revision("а б в"), keel.revision("а, б в"))

    def test_case_changes_the_revision(self):
        self.assertNotEqual(keel.revision("Given ready"), keel.revision("given ready"))

    def test_length_is_six(self):
        self.assertEqual(len(keel.revision("будь-що")), 6)

    def test_short_recorded_revision_matches_as_prefix(self):
        full = keel.full_revision("текст")
        self.assertTrue(keel.rev_matches(full[:4], "текст"))
        self.assertTrue(keel.rev_matches(full[:6], "текст"))

    def test_too_short_revision_does_not_match(self):
        full = keel.full_revision("текст")
        self.assertFalse(keel.rev_matches(full[:3], "текст"))

    def test_wrong_revision_does_not_match(self):
        self.assertFalse(keel.rev_matches("deadbe", "текст"))

    def test_ref_splits_slug_and_revision(self):
        ref = keel.Ref("session-run@7c40de")
        self.assertEqual((ref.slug, ref.rev), ("session-run", "7c40de"))
        self.assertIsNone(keel.Ref("session-run").rev)


# ─────────────────────────────────────────────────────────────────────────────
# Fixture: a real project in a temporary directory
# ─────────────────────────────────────────────────────────────────────────────

STEP = """---
depends_on: []

scenarios:
  finishes-when-no-tool-called: {{proves: session-run@{rev}}}

transforms:
  drive-turns:
    implements: [finishes-when-no-tool-called]
    contracts:  [session-run@{rev}]
    files:      [lib/session.ex]
---

## Навіщо

Одна розмова з моделлю проти набору інструментів, який дали ззовні.

## scenario: finishes-when-no-tool-called

**Given** порожній набір інструментів,
**When** модель відповідає текстом,
**Then** розмова завершується.

## transform: drive-turns

Крутити ходи, доки модель кличе інструменти.

Межі: лічильника спроб немає.
"""

CONTRACT = """---
module: Demo.Session
exports: [run/3]
---

Одна розмова з однією моделлю.
"""


class Fixture:
    """A project with git, a step, a contract and the Elixir adapter."""

    def __init__(self):
        self.root = tempfile.mkdtemp(prefix="keel-test-")
        for folder in ("keel/steps", "keel/contracts", "keel/decisions", "lib", "test"):
            os.makedirs(os.path.join(self.root, folder), exist_ok=True)
        self.write("mix.exs", "defmodule Demo.MixProject do\nend\n")
        self.write("keel/contracts/session-run.md", CONTRACT)
        self.contract_rev = keel.revision(CONTRACT)
        self.write("keel/steps/0001-session-loop.md", STEP.format(rev=self.contract_rev))
        self.write("lib/session.ex", "defmodule Demo.Session do\nend\n")
        self.git("init", "-b", "main")
        self.git("config", "user.email", "test@example.com")
        self.git("config", "user.name", "test")
        self.git("add", "-A")
        self.git("commit", "-m", "план")

    # — files —

    def path(self, name):
        return os.path.join(self.root, name)

    def write(self, name, text):
        target = self.path(name)
        os.makedirs(os.path.dirname(target), exist_ok=True)
        with open(target, "w", encoding="utf-8") as handle:
            handle.write(text)

    def read(self, name):
        with open(self.path(name), encoding="utf-8") as handle:
            return handle.read()

    # — git —

    def git(self, *args):
        return subprocess.run(["git", "-C", self.root, *args],
                              capture_output=True, text=True, check=False)

    def branch(self, name):
        self.git("checkout", "-b", name)

    # — the tool —

    def project(self):
        return keel.Project(self.root)

    def scenario_rev(self):
        return self.project().steps["0001-session-loop"].scenario_revision(
            "finishes-when-no-tool-called")

    def close(self):
        shutil.rmtree(self.root, ignore_errors=True)


class ProjectCase(unittest.TestCase):
    def setUp(self):
        self.fixture = Fixture()
        self.addCleanup(self.fixture.close)

    @property
    def project(self):
        return self.fixture.project()


# ─────────────────────────────────────────────────────────────────────────────
# Documents
# ─────────────────────────────────────────────────────────────────────────────

class TestDocuments(ProjectCase):
    def test_step_is_read(self):
        step = self.project.steps["0001-session-loop"]
        self.assertIsNone(step.error)
        self.assertEqual(list(step.scenarios), ["finishes-when-no-tool-called"])
        self.assertEqual(step.transform_files("drive-turns"), ["lib/session.ex"])
        self.assertEqual(step.transform_implements("drive-turns"),
                         ["finishes-when-no-tool-called"])
        self.assertIn("Одна розмова", step.why)

    def test_contract_is_read(self):
        contract = self.project.contracts["session-run"]
        self.assertEqual(contract.module, "Demo.Session")
        self.assertEqual(contract.exports, ["run/3"])
        self.assertTrue(contract.rev_ok(self.fixture.contract_rev))

    def test_scenario_body_and_revision(self):
        step = self.project.steps["0001-session-loop"]
        body = step.scenario_body("finishes-when-no-tool-called")
        self.assertIn("**Then** розмова завершується.", body)
        self.assertEqual(step.scenario_revision("finishes-when-no-tool-called"),
                         keel.revision(body))

    def test_document_without_front_matter_is_broken(self):
        self.fixture.write("keel/steps/0002-nohead.md", "просто текст\n")
        project = self.project
        self.assertTrue(any(doc.slug == "0002-nohead" for doc in project.broken))
        self.assertTrue(keel.check_structure(project))


# ─────────────────────────────────────────────────────────────────────────────
# Checks 1, 2, 3, 7
# ─────────────────────────────────────────────────────────────────────────────

class TestRefChecks(ProjectCase):
    def test_clean_project_has_no_problems(self):
        for check in (keel.check_refs, keel.check_cycles,
                      keel.check_revisions, keel.check_headings):
            self.assertEqual(check(self.project), [], check.__name__)

    def test_dangling_contract_reference(self):
        self.fixture.write(
            "keel/steps/0001-session-loop.md",
            self.fixture.read("keel/steps/0001-session-loop.md").replace(
                "session-run@", "no-such@"))
        problems = keel.check_refs(self.project)
        self.assertEqual(len(problems), 2)
        self.assertTrue(all("no-such" in p.message for p in problems))

    def test_dangling_depends_on(self):
        self.fixture.write(
            "keel/steps/0001-session-loop.md",
            self.fixture.read("keel/steps/0001-session-loop.md").replace(
                "depends_on: []", "depends_on: [0000-nowhere]"))
        problems = keel.check_refs(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("0000-nowhere", problems[0].message)

    def test_transform_implements_unknown_scenario(self):
        self.fixture.write(
            "keel/steps/0001-session-loop.md",
            self.fixture.read("keel/steps/0001-session-loop.md").replace(
                "implements: [finishes-when-no-tool-called]", "implements: [ghost]"))
        problems = keel.check_refs(self.project)
        self.assertTrue(any("ghost" in p.message for p in problems))

    def test_broken_markdown_link_in_body(self):
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        self.fixture.write("keel/steps/0001-session-loop.md",
                           text + "\nДив. [рішення](../decisions/none.md).\n")
        problems = keel.check_refs(self.project)
        self.assertTrue(any("none.md" in p.message for p in problems))

    def test_existing_link_to_decision_is_fine(self):
        self.fixture.write("keel/decisions/no-retry.md", "Повторів немає.\n")
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        self.fixture.write("keel/steps/0001-session-loop.md",
                           text + "\nДив. [рішення](../decisions/no-retry.md).\n")
        self.assertEqual(keel.check_refs(self.project), [])

    def test_cycle_is_found(self):
        first = self.fixture.read("keel/steps/0001-session-loop.md")
        self.fixture.write("keel/steps/0001-session-loop.md",
                           first.replace("depends_on: []", "depends_on: [0002-other]"))
        self.fixture.write("keel/steps/0002-other.md",
                           "---\ndepends_on: [0001-session-loop]\n---\n\n## Навіщо\n\nЦикл.\n")
        problems = keel.check_cycles(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("цикл", problems[0].message)

    def test_stale_contract_revision(self):
        self.fixture.write("keel/contracts/session-run.md", CONTRACT + "\nІ ще речення.\n")
        problems = keel.check_revisions(self.project)
        self.assertEqual(len(problems), 2)  # the scenario and the transform
        self.assertTrue(all("тримає редакцію" in p.message for p in problems))

    def test_reference_without_revision(self):
        self.fixture.write(
            "keel/steps/0001-session-loop.md",
            self.fixture.read("keel/steps/0001-session-loop.md").replace(
                f"session-run@{self.fixture.contract_rev}", "session-run"))
        problems = keel.check_revisions(self.project)
        self.assertEqual(len(problems), 2)
        self.assertTrue(all("без редакції" in p.message for p in problems))

    def test_heading_without_header_entry(self):
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        self.fixture.write("keel/steps/0001-session-loop.md",
                           text + "\n## scenario: orphan\n\nБез шапки.\n")
        problems = keel.check_headings(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("orphan", problems[0].message)

    def test_header_entry_without_heading(self):
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        self.fixture.write(
            "keel/steps/0001-session-loop.md",
            text.replace("transforms:\n", "transforms:\n  ghost:\n    files: [a.ex]\n"))
        problems = keel.check_headings(self.project)
        self.assertTrue(any("ghost" in p.message for p in problems))


# ─────────────────────────────────────────────────────────────────────────────
# Check 4: scope
# ─────────────────────────────────────────────────────────────────────────────

class TestScope(ProjectCase):
    def test_main_branch_is_not_checked(self):
        self.assertEqual(keel.check_scope(self.project), [])

    def test_declared_and_touched_is_clean(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "defmodule Demo.Session do\n  def run(_,_,_), do: :ok\nend\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_touched_beyond_scope(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("lib/extra.ex", "не оголошено\n")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("lib/extra.ex", problems[0].message)
        self.assertIn("не оголошено", problems[0].message)

    def test_declared_but_untouched(self):
        self.fixture.branch("0001-session-loop")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("оголошено, але не змінено", problems[0].message)

    def test_committed_change_counts(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns: перший хід")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_keel_documents_are_outside_scope(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("keel/decisions/no-retry.md", "Повторів немає.\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_plan_branch_must_not_touch_code(self):
        self.fixture.branch("plan/0001-session-loop")
        self.fixture.write("lib/session.ex", "код у гілці плану\n")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("гілка плану чіпає код", problems[0].message)

    def test_branch_that_is_not_a_step(self):
        self.fixture.branch("random-branch")
        problems = keel.check_scope(self.project)
        self.assertIn("не називається кроком", problems[0].message)


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

    def test_module_absent(self):
        self.write("keel/contracts/demo.md",
                   "---\nmodule: nosuchmodule\nexports: [run/1]\n---\n\nТекст.\n")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertIn("не зібрався", problems[0].message)


# ─────────────────────────────────────────────────────────────────────────────
# keel next
# ─────────────────────────────────────────────────────────────────────────────

class Args:
    def __init__(self, **kwargs):
        self.__dict__.update(kwargs)


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
        self.assertIn("не називається кроком", out)

    def test_refuses_on_plan_branch(self):
        self.fixture.branch("plan/0001-session-loop")
        code, out = self.run_next()
        self.assertEqual(code, 1)
        self.assertIn("гілка плану", out)

    def test_refuses_while_plan_is_not_in_main(self):
        self.fixture.branch("plan/0002-later")
        self.fixture.write("keel/steps/0002-later.md", STEP.format(rev=self.fixture.contract_rev))
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план кроку 2")
        self.fixture.git("checkout", "-b", "0002-later")
        code, out = self.run_next()
        self.assertEqual(code, 1)
        self.assertIn("не в гілці main", out)

    def test_package_has_files_scenario_and_contract(self):
        self.fixture.branch("0001-session-loop")
        code, out = self.run_next()
        self.assertEqual(code, 0)
        self.assertIn("# drive-turns", out)
        self.assertIn("lib/session.ex", out)
        self.assertIn("**Then** розмова завершується.", out)
        self.assertIn("Одна розмова з однією моделлю.", out)
        self.assertIn("Demo.Session", out)
        self.assertIn("drive-turns: <що зроблено>", out)

    def test_package_names_the_tag_to_write(self):
        self.fixture.branch("0001-session-loop")
        _, out = self.run_next()
        self.assertIn(f'proves: :finishes_when_no_tool_called, rev: "{self.fixture.scenario_rev()}"',
                      out)

    def test_closed_transform_is_not_handed_out_again(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns: перший хід")
        code, out = self.run_next()
        self.assertEqual(code, 0)
        self.assertIn("усі трансформи", out)

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
        self.assertIn("збігаються", out)

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
        self.assertIn("тримає редакцію", problems[0].message)
        self.run_rev(write=True)
        text = self.fixture.read("test/session_test.exs")
        self.assertEqual(text.count("rev:"), 1)
        self.assertNotIn("старий", text)
        self.assertEqual(keel.check_scenarios(self.project, run_tests=False), [])

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
# keel new and keel plan
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
        code, out = self.capture(keel.cmd_plan, self.project, Args(step="0001-session-loop"))
        self.assertEqual(code, 0)
        self.assertIn("план повний", out)

    def test_plan_finds_a_transform_without_files(self):
        self.fixture.write(
            "keel/steps/0001-session-loop.md",
            self.fixture.read("keel/steps/0001-session-loop.md").replace(
                "    files:      [lib/session.ex]\n", ""))
        code, out = self.capture(keel.cmd_plan, self.project, Args(step="0001-session-loop"))
        self.assertEqual(code, 1)
        self.assertIn("не оголосила файлів", out)

    def test_plan_finds_a_scenario_nobody_implements(self):
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        text = text.replace(
            "  finishes-when-no-tool-called: ",
            "  only-handed-tools-are-callable: {proves: session-run@%s}\n  finishes-when-no-tool-called: "
            % self.fixture.contract_rev)
        text += "\n## scenario: only-handed-tools-are-callable\n\n**Then** інших немає.\n"
        self.fixture.write("keel/steps/0001-session-loop.md", text)
        code, out = self.capture(keel.cmd_plan, self.project, Args(step="0001-session-loop"))
        self.assertEqual(code, 1)
        self.assertIn("не наближає жодна трансформа", out)

    def test_new_skeleton_is_not_a_complete_plan(self):
        self.capture(keel.cmd_new, self.project, Args(kind="step", slug="tool-calls"))
        code, out = self.capture(keel.cmd_plan, self.project, Args(step="0002-tool-calls"))
        self.assertEqual(code, 1)
        self.assertIn("жодного сценарію", out)


# ─────────────────────────────────────────────────────────────────────────────
# keel check
# ─────────────────────────────────────────────────────────────────────────────

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
        self.assertIn("5. у кожного сценарію зелений тест (не запускалась)", out)
        self.assertIn("✓ 1.", out)
        self.assertEqual(code, 0)

    def test_full_check_wants_a_test(self):
        code, out = self.capture(Args(fast=False, no_tests=True, json=False))
        self.assertEqual(code, 1)
        self.assertIn("не має тесту", out)

    def test_json_shape(self):
        import json as jsonlib
        code, out = self.capture(Args(fast=True, no_tests=True, json=True))
        payload = jsonlib.loads(out)
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["checks"]["1"]["name"], "посилання ведуть кудись")
        self.assertFalse(payload["checks"]["5"]["run"])
        self.assertEqual(code, 0)


# ─────────────────────────────────────────────────────────────────────────────
# keel hooks
# ─────────────────────────────────────────────────────────────────────────────

class TestHooks(ProjectCase):
    def capture(self, **kwargs):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_hooks(self.project, Args(
                **{"install": False, "force": False, **kwargs}))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def hook(self, name):
        return os.path.join(self.fixture.root, ".git", "hooks", name)

    def test_status_before_install(self):
        code, out = self.capture()
        self.assertEqual(code, 0)
        self.assertIn("pre-commit: немає", out)
        self.assertIn("pre-push: немає", out)

    def test_install_writes_both_hooks(self):
        code, _ = self.capture(install=True)
        self.assertEqual(code, 0)
        for name in ("pre-commit", "pre-push"):
            self.assertTrue(os.access(self.hook(name), os.X_OK), name)
            self.assertIn(keel.HOOK_MARK, self.fixture.read(f".git/hooks/{name}"))
        self.assertIn("check --fast", self.fixture.read(".git/hooks/pre-commit"))
        self.assertNotIn("--fast", self.fixture.read(".git/hooks/pre-push"))

    def test_status_after_install(self):
        self.capture(install=True)
        code, out = self.capture()
        self.assertEqual(code, 0)
        self.assertIn("стоять обидва", out)

    def test_install_is_idempotent(self):
        self.capture(install=True)
        first = self.fixture.read(".git/hooks/pre-commit")
        self.capture(install=True)
        self.assertEqual(first, self.fixture.read(".git/hooks/pre-commit"))

    def test_foreign_hook_is_left_alone(self):
        self.fixture.write(".git/hooks/pre-commit", "#!/bin/sh\necho чуже\n")
        code, out = self.capture(install=True)
        self.assertEqual(code, 1)
        self.assertIn("чужий хук", out)
        self.assertIn("чуже", self.fixture.read(".git/hooks/pre-commit"))

    def test_force_overwrites_foreign_hook(self):
        self.fixture.write(".git/hooks/pre-commit", "#!/bin/sh\necho чуже\n")
        code, _ = self.capture(install=True, force=True)
        self.assertEqual(code, 0)
        self.assertIn(keel.HOOK_MARK, self.fixture.read(".git/hooks/pre-commit"))

    def test_hook_blocks_a_commit_beyond_scope(self):
        self.capture(install=True)
        env = dict(os.environ, KEEL=os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(keel.__file__))), "keel.py"))
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("lib/extra.ex", "не оголошено\n")
        self.fixture.git("add", "-A")
        done = subprocess.run(
            ["git", "-C", self.fixture.root, "-c", "user.email=t@e.com",
             "-c", "user.name=t", "commit", "-m", "drive-turns: спроба"],
            capture_output=True, text=True, env=env)
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("lib/extra.ex", done.stdout + done.stderr)

    def test_hook_lets_a_clean_commit_through(self):
        self.capture(install=True)
        env = dict(os.environ, KEEL=os.path.abspath(keel.__file__))
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("add", "-A")
        done = subprocess.run(
            ["git", "-C", self.fixture.root, "-c", "user.email=t@e.com",
             "-c", "user.name=t", "commit", "-m", "drive-turns: перший хід"],
            capture_output=True, text=True, env=env)
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)


# ─────────────────────────────────────────────────────────────────────────────
# keel init
# ─────────────────────────────────────────────────────────────────────────────

class TestInit(unittest.TestCase):
    """An empty mix project where Keel has not been installed yet."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-init-")
        self.addCleanup(shutil.rmtree, self.root, True)
        with open(os.path.join(self.root, "mix.exs"), "w", encoding="utf-8") as handle:
            handle.write("defmodule Demo.MixProject do\nend\n")
        subprocess.run(["git", "init", "-b", "main", "-q", self.root], check=True)

    def read(self, name):
        with open(os.path.join(self.root, name), encoding="utf-8") as handle:
            return handle.read()

    def write(self, name, text):
        with open(os.path.join(self.root, name), "w", encoding="utf-8") as handle:
            handle.write(text)

    def init(self):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_init(keel.Project(self.root), Args(install=True, force=False))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def test_creates_the_three_folders(self):
        self.init()
        for folder in keel.INIT_DIRS:
            self.assertTrue(os.path.isdir(os.path.join(self.root, folder)), folder)

    def test_vendors_the_tool_verbatim(self):
        self.init()
        with open(keel.__file__, encoding="utf-8") as handle:
            self.assertEqual(self.read(keel.VENDORED), handle.read())

    def test_vendored_tool_runs(self):
        self.init()
        done = subprocess.run(
            [sys.executable, os.path.join(self.root, keel.VENDORED), "check", "--fast"],
            cwd=self.root, capture_output=True, text=True)
        self.assertIn("посилання ведуть кудись", done.stdout)

    def test_copies_both_references(self):
        self.init()
        self.assertIn("ISO/IEC 25010", self.read("keel/QUALITY.md"))
        self.assertIn("шість перевірок", self.read("keel/KEEL.md"))

    def test_agents_block_points_at_both_references(self):
        self.init()
        block = self.read("AGENTS.md")
        for name in keel.REFERENCES:
            self.assertIn(f"keel/{name}", block)

    def test_agents_block_holds_seven_principles(self):
        self.init()
        block = self.read("AGENTS.md")
        self.assertIn(keel.AGENTS_START, block)
        self.assertIn(keel.AGENTS_END, block)
        self.assertIn("7. Не заводь сутність", block)
        self.assertIn("1. Обіцянка перевіряється", block)

    def test_existing_agents_file_is_kept(self):
        self.write("AGENTS.md", "# Проєкт\n\nСвоє, написане руками.\n")
        self.init()
        text = self.read("AGENTS.md")
        self.assertIn("Своє, написане руками.", text)
        self.assertIn(keel.AGENTS_START, text)

    def test_second_init_replaces_only_the_block(self):
        self.write("AGENTS.md", "# Проєкт\n\nСвоє.\n")
        self.init()
        self.write("AGENTS.md", self.read("AGENTS.md") + "\nДописане після.\n")
        self.init()
        text = self.read("AGENTS.md")
        self.assertEqual(text.count(keel.AGENTS_START), 1)
        self.assertIn("Своє.", text)
        self.assertIn("Дописане після.", text)

    def test_ci_names_the_vendored_tool_and_the_language(self):
        self.init()
        workflow = self.read(keel.CI_FILE)
        self.assertIn("python3 keel/keel.py check", workflow)
        self.assertIn("erlef/setup-beam", workflow)
        self.assertIn("fetch-depth: 0", workflow)

    def test_ci_passes_the_branch_name(self):
        self.init()
        self.assertIn("--branch \"${{ github.head_ref || github.ref_name }}\"",
                      self.read(keel.CI_FILE))

    def test_installs_hooks(self):
        self.init()
        self.assertTrue(os.access(os.path.join(self.root, ".git/hooks/pre-commit"), os.X_OK))

    def test_second_init_changes_nothing(self):
        self.init()
        _, out = self.init()
        self.assertNotIn("AGENTS.md", out)
        self.assertNotIn(keel.CI_FILE, out)


# ─────────────────────────────────────────────────────────────────────────────
# keel skills
# ─────────────────────────────────────────────────────────────────────────────

class TestSkills(ProjectCase):
    def generate(self):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_skills(self.project, Args())
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def head(self, text):
        front, _, _ = keel.split_front_matter(text)
        self.assertIsNotNone(front, "у породженого немає шапки")
        return keel.parse_yaml(front)

    def test_writes_both_sets(self):
        self.generate()
        for skill in keel.SKILLS:
            for _, relative in keel.skill_targets(skill):
                self.assertTrue(os.path.exists(self.fixture.path(relative)), relative)

    def test_claude_frontmatter(self):
        self.generate()
        for skill in keel.SKILLS:
            head = self.head(self.fixture.read(
                f".claude/skills/{skill['name']}/SKILL.md"))
            self.assertEqual(head["name"], skill["name"])
            self.assertTrue(head["description"])

    def test_cursor_frontmatter(self):
        self.generate()
        for skill in keel.SKILLS:
            head = self.head(self.fixture.read(f".cursor/rules/{skill['name']}.mdc"))
            self.assertEqual(head["alwaysApply"], "false")
            self.assertTrue(head["description"])
            self.assertNotIn("name", head)

    def test_cursor_uses_mdc_because_md_is_ignored(self):
        self.generate()
        for name in os.listdir(self.fixture.path(".cursor/rules")):
            self.assertTrue(name.endswith(".mdc"), name)

    def test_description_fits_the_listing_cap(self):
        for skill in keel.SKILLS:
            self.assertLessEqual(len(skill["description"]), keel.DESCRIPTION_CAP,
                                 skill["name"])

    def test_description_is_quoted_because_it_carries_a_colon(self):
        self.generate()
        for skill in keel.SKILLS:
            for _, relative in keel.skill_targets(skill):
                front, _, _ = keel.split_front_matter(self.fixture.read(relative))
                line = next(row for row in front.splitlines()
                            if row.startswith("description:"))
                self.assertTrue(line.startswith('description: "'), line)
                self.assertTrue(line.endswith('"'), line)

    def test_description_survives_the_round_trip(self):
        self.generate()
        for skill in keel.SKILLS:
            wanted = " ".join(skill["description"].split())
            for _, relative in keel.skill_targets(skill):
                head = self.head(self.fixture.read(relative))
                self.assertEqual(head["description"], wanted, relative)

    def test_description_is_one_line(self):
        self.generate()
        for skill in keel.SKILLS:
            for _, relative in keel.skill_targets(skill):
                front, _, _ = keel.split_front_matter(self.fixture.read(relative))
                line = [row for row in front.splitlines()
                        if row.startswith("description:")]
                self.assertEqual(len(line), 1, relative)

    def test_only_the_planning_skill_is_glob_scoped(self):
        self.generate()
        planning = self.head(self.fixture.read(".cursor/rules/keel-plan.mdc"))
        self.assertEqual(planning["globs"], "keel/steps/*.md")
        self.assertNotIn("globs", self.head(
            self.fixture.read(".cursor/rules/keel-work.mdc")))

    def test_body_is_the_same_in_both_dialects(self):
        self.generate()
        for skill in keel.SKILLS:
            bodies = []
            for _, relative in keel.skill_targets(skill):
                _, body, _ = keel.split_front_matter(self.fixture.read(relative))
                bodies.append(body)
            self.assertEqual(bodies[0], bodies[1], skill["name"])

    def test_generated_files_say_so(self):
        self.generate()
        for skill in keel.SKILLS:
            for _, relative in keel.skill_targets(skill):
                self.assertIn("Породжено", self.fixture.read(relative))

    def test_planning_skill_sends_you_to_quality(self):
        self.generate()
        body = self.fixture.read(".claude/skills/keel-plan/SKILL.md")
        self.assertIn("keel/QUALITY.md", body)
        for answer in ("не стосується", "відповіли", "промовчали"):
            self.assertIn(answer, body)

    def test_thin_skills_name_the_commands(self):
        self.generate()
        self.assertIn("keel.py next", self.fixture.read(
            ".claude/skills/keel-work/SKILL.md"))
        self.assertIn("keel.py check", self.fixture.read(
            ".claude/skills/keel-review/SKILL.md"))

    @unittest.skipUnless(HAS_PYYAML, "PyYAML не встановлений")
    def test_a_real_yaml_parser_reads_the_frontmatter(self):
        """Свій читач поблажливий; шапку читатимуть Claude і Cursor, не він."""
        import yaml
        self.generate()
        for skill in keel.SKILLS:
            for _, relative in keel.skill_targets(skill):
                front, _, _ = keel.split_front_matter(self.fixture.read(relative))
                head = yaml.safe_load(front)
                self.assertEqual(head["description"],
                                 " ".join(skill["description"].split()), relative)
                self.assertIs(head.get("alwaysApply", False), False, relative)

    def test_second_run_changes_nothing(self):
        self.generate()
        _, out = self.generate()
        self.assertIn("не змінились", out)

    def test_hand_edit_is_restored(self):
        self.generate()
        self.fixture.write(".cursor/rules/keel-work.mdc", "правлено руками\n")
        _, out = self.generate()
        self.assertIn("keel-work.mdc", out)
        self.assertIn("Породжено", self.fixture.read(".cursor/rules/keel-work.mdc"))


class TestBranchOverride(ProjectCase):
    def test_detached_head_with_branch_flag(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns: хід")
        self.fixture.git("checkout", "-q", "--detach")
        project = self.project
        self.assertIn("HEAD відчеплений", keel.check_scope(project)[0].message)
        project.branch_override = "0001-session-loop"
        self.assertEqual(keel.check_scope(project), [])


if __name__ == "__main__":
    unittest.main()

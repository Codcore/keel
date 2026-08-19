#!/usr/bin/env python3
"""Check 4: what the branch touched against what it declared."""

import os
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import ProjectCase  # noqa: E402




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
        self.assertIn("not declared", problems[0].message)

    def test_declared_but_untouched(self):
        self.fixture.branch("0001-session-loop")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("declared but not changed", problems[0].message)

    def test_committed_change_counts(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns: перший хід")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_keel_documents_are_outside_scope(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("keel/contracts/no-retry.md", "---\nverify: \"true\"\n---\n\nх\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_a_plan_branch_may_carry_keels_own_files(self):
        """Інакше перший же комміт плану впирається в те, що поклав init."""
        self.fixture.branch("plan/0001-session-loop")
        for name in ("AGENTS.md", ".claude/skills/keel-plan/SKILL.md",
                     ".cursor/hooks.json", ".github/workflows/keel.yml",
                     ".claude/settings.json"):
            self.fixture.write(name, "породжене\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_a_work_branch_may_carry_keels_own_files_too(self):
        """`update` серед роботи не має вимагати оголосити наш власний скіл."""
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        for name in ("AGENTS.md", ".claude/skills/keel-plan/SKILL.md",
                     ".cursor/hooks.json", ".github/workflows/keel.yml",
                     ".claude/settings.json"):
            self.fixture.write(name, "породжене\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_a_declared_keel_file_earns_no_false_report(self):
        """Оголошений AGENTS.md давав «declared but not changed» над diff-ом,
        який його явно змінив."""
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        self.fixture.write("keel/steps/0001-session-loop.md",
                           text.replace("files:      [lib/session.ex]",
                                        "files:      [lib/session.ex, AGENTS.md]"))
        self.fixture.write("AGENTS.md", "змінений блок\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_a_work_branch_still_catches_an_undeclared_project_file(self):
        """Звільнення стосується нашої обстановки, не будь-чого поруч."""
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("lib/stray.ex", "не оголошено\n")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("lib/stray.ex", problems[0].message)

    def test_plan_branch_must_not_touch_code(self):
        self.fixture.branch("plan/0001-session-loop")
        self.fixture.write("lib/session.ex", "код у гілці плану\n")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("a plan branch is touching code", problems[0].message)

    def test_a_missing_merge_base_is_red_not_silently_green(self):
        """Без бази diff бачить лише незакомічене — і все закомічене проходить."""
        self.fixture.git("branch", "-m", "main", "trunk")
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("lib/undeclared.ex", "не оголошено\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "drive-turns: хід")
        problems = keel.check_scope(self.project)
        self.assertTrue(problems)
        self.assertIn("cannot tell where this branch left from", problems[0].message)

    def test_origin_head_pointing_at_the_work_branch_is_not_trusted(self):
        """Одногілковий клон робив гілку власною базою — і все проходило."""
        self.fixture.branch("0001-session-loop")
        self.fixture.git("update-ref", "refs/remotes/origin/HEAD",
                         "refs/heads/0001-session-loop")
        self.fixture.git("symbolic-ref", "refs/remotes/origin/HEAD",
                         "refs/remotes/origin/0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("lib/stray.ex", "не оголошено\n")
        problems = keel.check_scope(self.project)
        self.assertTrue(problems, "перевірка мовчала на неоголошеному файлі")
        self.assertTrue(any("lib/stray.ex" in p.message
                            or "не знайшов, від чого" in p.message for p in problems))

    def test_branch_that_is_not_a_step(self):
        self.fixture.branch("random-branch")
        problems = keel.check_scope(self.project)
        self.assertIn("is not named after a step", problems[0].message)


# ─────────────────────────────────────────────────────────────────────────────
# Check 5: scenarios and tags
# ─────────────────────────────────────────────────────────────────────────────




class TestBranchOverride(ProjectCase):
    def test_detached_head_with_branch_flag(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns: хід")
        self.fixture.git("checkout", "-q", "--detach")
        project = self.project
        self.assertIn("the head is detached", keel.check_scope(project)[0].message)
        project.branch_override = "0001-session-loop"
        self.assertEqual(keel.check_scope(project), [])




class TestNestedKeelRoot(unittest.TestCase):
    """A keel root inside a bigger repository — a layout find_root supports."""

    def setUp(self):
        self.top = tempfile.mkdtemp(prefix="keel-nested-")
        self.addCleanup(shutil.rmtree, self.top, True)
        self.root = os.path.join(self.top, "sub")
        for folder in ("keel/steps", "keel/contracts", "lib"):
            os.makedirs(os.path.join(self.root, folder))
        os.makedirs(os.path.join(self.top, "other"))
        with open(os.path.join(self.root, "keel/steps/0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\ntransforms:\n  do:\n    files: [lib/foo.txt]\n"
                         "---\n\n## Why\n\nх.\n\n## transform: do\n\nЩось.\n")
        with open(os.path.join(self.top, "other/x.txt"), "w") as handle:
            handle.write("чуже\n")
        self.git("init", "-q", "-b", "main", ".")
        self.git("config", "user.email", "t@e")
        self.git("config", "user.name", "t")
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "base")
        self.git("checkout", "-q", "-b", "0001-a")

    def git(self, *args):
        subprocess.run(["git", "-C", self.top, *args], check=False,
                       capture_output=True)

    def test_a_declared_file_matches_despite_the_prefix(self):
        """git каже sub/lib/foo.txt, крок оголошує lib/foo.txt."""
        with open(os.path.join(self.root, "lib/foo.txt"), "w") as handle:
            handle.write("змінено\n")
        self.assertEqual(keel.check_scope(keel.Project(self.root)), [])

    def test_a_sibling_directory_is_not_this_steps_business(self):
        with open(os.path.join(self.root, "lib/foo.txt"), "w") as handle:
            handle.write("змінено\n")
        with open(os.path.join(self.top, "other/x.txt"), "a") as handle:
            handle.write("чужа зміна\n")
        self.assertEqual(keel.check_scope(keel.Project(self.root)), [])

    def test_an_undeclared_file_inside_the_keel_root_is_still_caught(self):
        with open(os.path.join(self.root, "lib/foo.txt"), "w") as handle:
            handle.write("змінено\n")
        with open(os.path.join(self.root, "lib/stray.txt"), "w") as handle:
            handle.write("не оголошено\n")
        problems = keel.check_scope(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("lib/stray.txt", problems[0].message)


if __name__ == "__main__":
    unittest.main()

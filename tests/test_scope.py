#!/usr/bin/env python3
"""Check 4: what the branch touched against what it declared."""

import os
import sys
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

    def test_a_plan_branch_may_carry_keels_own_files(self):
        """Інакше перший же комміт плану впирається в те, що поклав init."""
        self.fixture.branch("plan/0001-session-loop")
        for name in ("AGENTS.md", ".claude/skills/keel-plan/SKILL.md",
                     ".cursor/hooks.json", ".github/workflows/keel.yml",
                     ".claude/settings.json"):
            self.fixture.write(name, "породжене\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_plan_branch_must_not_touch_code(self):
        self.fixture.branch("plan/0001-session-loop")
        self.fixture.write("lib/session.ex", "код у гілці плану\n")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("гілка плану чіпає код", problems[0].message)

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
        self.assertIn("не знайшов, від чого відійшла гілка", problems[0].message)

    def test_branch_that_is_not_a_step(self):
        self.fixture.branch("random-branch")
        problems = keel.check_scope(self.project)
        self.assertIn("не називається кроком", problems[0].message)


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
        self.assertIn("HEAD відчеплений", keel.check_scope(project)[0].message)
        project.branch_override = "0001-session-loop"
        self.assertEqual(keel.check_scope(project), [])


if __name__ == "__main__":
    unittest.main()


if __name__ == "__main__":
    unittest.main()

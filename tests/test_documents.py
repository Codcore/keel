#!/usr/bin/env python3
"""Reading steps and contracts; the checks that only read text."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import CONTRACT, ProjectCase  # noqa: E402




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
                           text + "\nДив. [контракт](../contracts/none.md).\n")
        problems = keel.check_refs(self.project)
        self.assertTrue(any("none.md" in p.message for p in problems))

    def test_existing_link_to_decision_is_fine(self):
        self.fixture.write("keel/contracts/no-retry.md",
                           "---\nverify: \"true\"\n---\n\nПовторів немає.\n")
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        self.fixture.write("keel/steps/0001-session-loop.md",
                           text + "\nДив. [контракт](../contracts/no-retry.md).\n")
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


if __name__ == "__main__":
    unittest.main()

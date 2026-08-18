#!/usr/bin/env python3
"""Reading steps and contracts; the checks that only read text."""

import os
import shutil
import sys
import tempfile
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
        self.assertIn("cycle in depends_on", problems[0].message)

    def test_stale_contract_revision(self):
        self.fixture.write("keel/contracts/session-run.md", CONTRACT + "\nІ ще речення.\n")
        problems = keel.check_revisions(self.project)
        self.assertEqual(len(problems), 2)  # the scenario and the transform
        self.assertTrue(all("holds" in p.message for p in problems))

    def test_reference_without_revision(self):
        self.fixture.write(
            "keel/steps/0001-session-loop.md",
            self.fixture.read("keel/steps/0001-session-loop.md").replace(
                f"session-run@{self.fixture.contract_rev}", "session-run"))
        problems = keel.check_revisions(self.project)
        self.assertEqual(len(problems), 2)
        self.assertTrue(all("without a revision" in p.message for p in problems))

    def test_a_repeated_heading_is_reported(self):
        """Читають перший, а редакція рахується з останнього."""
        step = "keel/steps/0001-session-loop.md"
        self.fixture.write(step, self.fixture.read(step)
                           + "\n## scenario: finishes-when-no-tool-called\n\n"
                             "**Then** зовсім інше.\n")
        problems = keel.check_headings(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("appears twice", problems[0].message)

    def test_a_repeated_transform_heading_too(self):
        step = "keel/steps/0001-session-loop.md"
        self.fixture.write(step, self.fixture.read(step)
                           + "\n## transform: drive-turns\n\nІнше тіло.\n")
        self.assertTrue(any("appears twice" in p.message
                            for p in keel.check_headings(self.project)))

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


class TestHeaderShape(unittest.TestCase):
    """A field of the wrong shape is an error, not an empty default.

    An empty default reads as "nothing declared", and nothing declared is what
    switches the write hook off while every check stays green.
    """

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-shape-")
        self.addCleanup(shutil.rmtree, self.root, True)
        for folder in ("keel/steps", "keel/contracts"):
            os.makedirs(os.path.join(self.root, folder))

    def doc(self, kind, text):
        name = "0001-a.md" if kind == "step" else "a.md"
        path = os.path.join(self.root, "keel", kind + "s", name)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        cls = keel.Step if kind == "step" else keel.Contract
        return cls(path, self.root)

    def test_transforms_as_a_list_is_named(self):
        doc = self.doc("step", "---\ntransforms:\n  - do-it\n---\n\n## Why\n\nх.\n")
        self.assertIn("transforms has to be", doc.error)
        self.assertIn("list", doc.error)

    def test_scenarios_as_a_list_is_named(self):
        doc = self.doc("step", "---\nscenarios:\n  - one\n---\n\n## Why\n\nх.\n")
        self.assertIn("scenarios has to be", doc.error)

    def test_depends_on_as_a_map_is_named(self):
        doc = self.doc("step", "---\ndepends_on:\n  a: b\n---\n\n## Why\n\nх.\n")
        self.assertIn("depends_on has to be", doc.error)

    def test_exports_as_a_string_is_named(self):
        doc = self.doc("contract", '---\nmodule: Demo\nexports: "run/3"\n---\n\nх.\n')
        self.assertIn("exports has to be", doc.error)

    def test_module_as_a_list_is_named(self):
        doc = self.doc("contract", "---\nmodule: [Demo]\n---\n\nх.\n")
        self.assertIn("module has to be", doc.error)

    def test_a_missing_field_is_not_a_wrong_shape(self):
        doc = self.doc("step", "---\ndepends_on: []\n---\n\n## Why\n\nх.\n")
        self.assertIsNone(doc.error)

    def test_an_empty_field_is_not_a_wrong_shape(self):
        """`transforms:` без нічого під ним — ще не написано, а не зламано."""
        doc = self.doc("step", "---\ntransforms:\n---\n\n## Why\n\nх.\n")
        self.assertIsNone(doc.error)

    def test_verify_is_left_to_check_six(self):
        """Там уже є повідомлення з типом і значенням — дублювати не треба."""
        doc = self.doc("contract", '---\nverify: ["curl", "x"]\n---\n\nх.\n')
        self.assertIsNone(doc.error)

    def test_a_file_that_cannot_be_read_is_named(self):
        path = os.path.join(self.root, "keel/steps/0002-gone.md")
        os.symlink("/немає/такого", path)
        doc = keel.Step(path, self.root)
        self.assertIn("cannot be read", doc.error)


class TestLinksLeadSomewhere(unittest.TestCase):
    """Check 1 is named that, and a broken link leads nowhere wherever it points."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-links-")
        self.addCleanup(shutil.rmtree, self.root, True)
        for folder in ("keel/steps", "keel/contracts", "docs"):
            os.makedirs(os.path.join(self.root, folder))

    def step(self, body):
        with open(os.path.join(self.root, "keel/steps/0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\ndepends_on: []\n---\n\n## Why\n\n" + body + "\n")
        return keel.check_refs(keel.Project(self.root))

    def test_a_broken_link_inside_keel(self):
        problems = self.step("Див. [сусід](../contracts/nope.md).")
        self.assertEqual(len(problems), 1)
        self.assertIn("nope.md", problems[0].message)

    def test_a_broken_link_outside_keel_is_caught_too(self):
        """Раніше все, що виходило за keel/, не перевірялось узагалі."""
        problems = self.step("Див. [задум](../../docs/design.md).")
        self.assertEqual(len(problems), 1)
        self.assertIn("design.md", problems[0].message)

    def test_a_link_that_resolves_outside_keel_is_clean(self):
        with open(os.path.join(self.root, "docs/design.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("задум\n")
        self.assertEqual(self.step("Див. [задум](../../docs/design.md)."), [])

    def test_a_link_beyond_the_repository_is_named_differently(self):
        problems = self.step("Див. [чуже](../../../elsewhere/x.md).")
        self.assertEqual(len(problems), 1)
        self.assertIn("leaves the repository", problems[0].message)

    def test_an_http_link_is_left_alone(self):
        self.assertEqual(self.step("Див. [доки](https://example.com/x.md)."), [])


if __name__ == "__main__":
    unittest.main()

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

    def test_a_transform_entry_that_is_a_string_is_named(self):
        """Раніше всі читачі віддавали [], і gaps казав «не оголосила файлів»."""
        doc = self.doc("step", "---\ntransforms:\n  do-it: щось\n---\n\n"
                               "## Why\n\nх.\n")
        self.assertIn("transform do-it has to be a set of named fields", doc.error)

    def test_a_scenario_entry_that_is_a_string_is_named(self):
        doc = self.doc("step", "---\nscenarios:\n  does-a: текст\n---\n\n"
                               "## Why\n\nх.\n")
        self.assertIn("scenario does-a has to be a set of named fields", doc.error)

    def test_files_as_a_map_is_named(self):
        doc = self.doc("step", "---\ntransforms:\n  do-it:\n    files: {a: b}\n"
                               "---\n\n## Why\n\nх.\n")
        self.assertIn("files of transform do-it has to be a list", doc.error)

    def test_implements_and_contracts_too(self):
        for field in ("implements", "contracts"):
            doc = self.doc("step", f"---\ntransforms:\n  do-it:\n    {field}: "
                                   "{a: b}\n---\n\n## Why\n\nх.\n")
            self.assertIn(f"{field} of transform do-it", doc.error)

    def test_proves_as_a_map_is_named(self):
        doc = self.doc("step", "---\nscenarios:\n  does-a:\n    proves: {a: b}\n"
                               "---\n\n## Why\n\nх.\n")
        self.assertIn("proves of scenario does-a has to be a list", doc.error)

    def test_the_string_shorthand_still_stands(self):
        """files: один-файл і proves: один-контракт — дозволена коротка форма."""
        doc = self.doc("step", "---\nscenarios:\n  does-a: {proves: c@abcd}\n"
                               "transforms:\n  do-it:\n    files: lib/a.ex\n"
                               "    implements: does-a\n---\n\n## Why\n\nх.\n")
        self.assertIsNone(doc.error)
        self.assertEqual(doc.transform_files("do-it"), ["lib/a.ex"])

    def test_an_empty_transform_is_still_a_skeleton_not_an_error(self):
        doc = self.doc("step", "---\ntransforms:\n  do-it:\n---\n\n"
                               "## Why\n\nх.\n")
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




class TestSectionSplitting(unittest.TestCase):
    """A heading is a section; a heading inside a fence is text."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-sec-")
        self.addCleanup(shutil.rmtree, self.root, True)
        os.makedirs(os.path.join(self.root, "keel", "steps"))

    def step(self, body):
        path = os.path.join(self.root, "keel/steps/0001-a.md")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(body)
        return keel.Step(path, self.root)

    def test_a_heading_in_a_fence_is_not_a_section(self):
        doc = self.step("---\ntransforms:\n  do-it: {files: [a.ex]}\n---\n\n"
                        "## transform: do-it\n\nтіло\n```\n## scenario: fake\n```\nхвіст\n")
        self.assertEqual(list(doc.sections), ["transform: do-it"])
        self.assertIn("## scenario: fake", doc.transform_body("do-it"))
        self.assertIn("хвіст", doc.transform_body("do-it"))

    def test_a_case_variant_duplicate_is_flagged(self):
        """`## scenario: s` і `## Scenario:  s` — той самий сценарій для читача."""
        doc = self.step("---\nscenarios:\n  s: {}\n---\n\n## scenario: s\n\n"
                        "ПЕРШЕ.\n\n## Scenario:  s\n\nДРУГЕ.\n")
        self.assertTrue(doc.repeated)

    def test_a_true_distinct_heading_is_not_flagged(self):
        doc = self.step("---\nscenarios:\n  a: {}\n  b: {}\n---\n\n"
                        "## scenario: a\n\nх.\n\n## scenario: b\n\nх.\n")
        self.assertEqual(doc.repeated, [])

    def test_the_line_number_points_at_the_heading(self):
        doc = self.step("---\nscenarios:\n  s: {}\n---\n\n## scenario: s\n\nтіло\n")
        self.assertEqual(doc.section_lines["scenario: s"], 6)


class TestRewriteTag(unittest.TestCase):
    """Restamping one scenario's tag leaves its longer-named sibling alone."""

    def test_a_superstring_slug_is_not_a_match(self):
        text = ('@tag proves: :parse, rev: "aaaaaa"\n'
                '@tag proves: :parse_error, rev: "bbbbbb"\n')
        out = keel.rewrite_tag(text, "parse", "ffffff")
        self.assertIn(':parse, rev: "ffffff"', out)
        self.assertIn(':parse_error, rev: "bbbbbb"', out)

    def test_the_python_form_too(self):
        text = ('# proves: parse, rev: "aaaaaa"\n'
                '# proves: parse-error, rev: "bbbbbb"\n')
        out = keel.rewrite_tag(text, "parse", "ffffff")
        self.assertIn('parse, rev: "ffffff"', out)
        self.assertIn('parse-error, rev: "bbbbbb"', out)


class TestRewriteRef(unittest.TestCase):
    """rev --write restamps a reference, and nothing that merely shares its name."""

    def test_a_scenario_named_like_the_contract_is_untouched(self):
        text = ("---\nscenarios:\n  parser: {proves: parser@aaaa}\n"
                "transforms:\n  parser:\n    implements: [parser]\n"
                "    contracts: [parser@aaaa]\n    files: [lib/parser.ex]\n---\n\n"
                "тіло про parser.\n")
        out = keel.rewrite_ref(text, "parser@aaaa", "parser@ffff")
        self.assertIn("proves: parser@ffff", out)
        self.assertIn("contracts: [parser@ffff]", out)
        self.assertIn("implements: [parser]", out)          # scenario ref, untouched
        self.assertIn("files: [lib/parser.ex]", out)
        self.assertRegex(out, r"\n  parser:")               # the mapping key stays

    def test_a_block_list_item_is_restamped(self):
        text = ("---\ntransforms:\n  do:\n    contracts:\n      - parser@aaaa\n"
                "      - other@bbbb\n---\nтіло\n")
        out = keel.rewrite_ref(text, "parser@aaaa", "parser@ffff")
        self.assertIn("- parser@ffff", out)
        self.assertIn("- other@bbbb", out)

    def test_a_revision_is_added_where_there_was_none(self):
        text = "---\nscenarios:\n  s: {proves: parser}\n---\nтіло\n"
        self.assertIn("proves: parser@ffff",
                      keel.rewrite_ref(text, "parser", "parser@ffff"))

    def test_the_trailing_newline_is_kept(self):
        text = "---\nscenarios:\n  s: {proves: c@aaaa}\n---\n\nтіло\n"
        self.assertTrue(keel.rewrite_ref(text, "c@aaaa", "c@ffff").endswith("тіло\n"))


if __name__ == "__main__":
    unittest.main()

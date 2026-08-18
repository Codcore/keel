#!/usr/bin/env python3
"""The narrow YAML subset and the revision hashes."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402




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

    def test_escaped_quotes_are_undone(self):
        self.assertEqual(keel.parse_yaml(r'note: "he said \"go\""')["note"],
                         'he said "go"')
        self.assertEqual(keel.parse_yaml(r'note: "back\\slash"')["note"],
                         "back\\slash")
        self.assertEqual(keel.parse_yaml("note: 'it''s'")["note"], "it's")

    def test_quoting_survives_the_round_trip(self):
        for value in ('a: b', 'he said "go"', "it's", "back\\slash", "colon: here"):
            written = keel.yaml_string(value)
            self.assertEqual(keel.parse_yaml(f"note: {written}")["note"], value)

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

    def test_two_colons_in_a_line_are_an_error(self):
        with self.assertRaises(keel.YamlError):
            keel.parse_yaml("a: b: c")

    def test_a_key_without_a_space_after_the_colon_is_an_error(self):
        with self.assertRaises(keel.YamlError):
            keel.parse_yaml("a:b")

    def test_a_url_is_still_a_plain_value(self):
        self.assertEqual(keel.parse_yaml("x: http://e.com/p")["x"], "http://e.com/p")

    def test_a_top_level_list_is_an_error_not_an_empty_header(self):
        """Порожня шапка читалась би як крок без трансформ — і вимикала хук."""
        with self.assertRaises(keel.YamlError):
            keel.parse_yaml("- a\n- b\n")

    def test_a_map_inside_a_list_is_an_error(self):
        """Валідний YAML, який ми читали б рядком — краще впасти, ніж збрехати."""
        for src in ("depends_on:\n  - name: a", "x: [a: b, c]"):
            with self.assertRaises(keel.YamlError, msg=src):
                keel.parse_yaml(src)

    def test_an_apostrophe_does_not_swallow_the_comment(self):
        self.assertEqual(keel.parse_yaml("a: файл ім'я # коментар")["a"], "файл ім'я")

    def test_an_apostrophe_inside_a_flow_list_item(self):
        self.assertEqual(keel.parse_yaml("files: [a.py, src/ім'я.py]  # нотатка")["files"],
                         ["a.py", "src/ім'я.py"])

    def test_quoting_still_works_where_a_value_starts(self):
        self.assertEqual(keel.parse_yaml('x: ["a, b", c]')["x"], ["a, b", "c"])
        self.assertEqual(keel.parse_yaml("n: ['a', 'b']")["n"], ["a", "b"])

    def test_missing_colon_is_an_error(self):
        with self.assertRaises(keel.YamlError) as caught:
            keel.parse_yaml("module Foo\n")
        self.assertIn("line 1", str(caught.exception))


# ─────────────────────────────────────────────────────────────────────────────
# Revisions
# ─────────────────────────────────────────────────────────────────────────────




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


if __name__ == "__main__":
    unittest.main()

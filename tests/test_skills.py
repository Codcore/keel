#!/usr/bin/env python3
"""Generated skills: shape, frontmatter, and what the text must say."""

import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args, HAS_PYYAML, ProjectCase  # noqa: E402




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
            head = self.head(self.fixture.read(
                f".cursor/skills/{skill['name']}/SKILL.md"))
            self.assertEqual(head["name"], skill["name"])
            self.assertTrue(head["description"])

    def test_both_agents_get_the_same_file_name(self):
        """Скіл кличеться /<name> в обох — імʼя дає тека, тож теки однакові."""
        self.generate()
        for skill in keel.SKILLS:
            paths = [relative for _, relative in keel.skill_targets(skill)]
            self.assertTrue(all(p.endswith(f"/{skill['name']}/SKILL.md")
                                for p in paths), paths)

    def test_argument_hint_only_where_the_field_is_known(self):
        self.generate()
        claude = self.fixture.read(".claude/skills/keel-plan/SKILL.md")
        cursor = self.fixture.read(".cursor/skills/keel-plan/SKILL.md")
        self.assertIn("argument-hint:", claude)
        self.assertNotIn("argument-hint:", cursor)

    def test_when_to_take_it_lives_in_the_description(self):
        """Рекомендація Anthropic: тригери в описі, тіло — про роботу."""
        for skill in keel.SKILLS:
            self.assertIn("use this skill", skill["description"].lower(), skill["name"])
            self.assertIn("{triggers}", skill["description"], skill["name"])
            self.assertIn("keel/", skill["description"], skill["name"])

    def test_description_fits_the_listing_cap(self):
        for skill in keel.SKILLS:
            for lang in keel.LANGS:
                self.assertLessEqual(len(keel.skill_description(skill, lang)),
                                     keel.DESCRIPTION_CAP, skill["name"])

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
            wanted = keel.skill_description(skill, self.project.settings["lang"])
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

    def test_only_the_planning_skill_is_path_scoped(self):
        self.generate()
        for _, relative in keel.skill_targets(keel.SKILLS[0]):
            self.assertEqual(self.head(self.fixture.read(relative))["paths"],
                             ["keel/steps/*.md"], relative)
        for _, relative in keel.skill_targets(keel.SKILLS[1]):
            self.assertNotIn("paths", self.head(self.fixture.read(relative)))

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
                self.assertIn("Generated by", self.fixture.read(relative))

    def test_planning_skill_sends_you_to_quality(self):
        self.generate()
        body = self.fixture.read(".claude/skills/keel-plan/SKILL.md")
        self.assertIn("keel/QUALITY.md", body)
        for answer in ("does not apply", "answered", "silent"):
            self.assertIn(answer, body)

    def test_planning_skill_asks_instead_of_guessing(self):
        self.generate()
        for _, relative in keel.skill_targets(keel.SKILLS[0]):
            body = self.fixture.read(relative)
            self.assertIn("AskUserQuestion", body, relative)
            self.assertIn("Ask, do not guess", body, relative)

    def test_thin_skills_name_the_commands(self):
        self.generate()
        self.assertIn("keel.py next", self.fixture.read(
            ".claude/skills/keel-work/SKILL.md"))
        self.assertIn("keel.py check", self.fixture.read(
            ".claude/skills/keel-review/SKILL.md"))

    def test_each_skill_hands_over_to_the_next_stage(self):
        """Цикл не має обриватись: скіл каже, куди йти, коли етап скінчився."""
        self.generate()
        plan = self.fixture.read(".claude/skills/keel-plan/SKILL.md")
        work = self.fixture.read(".claude/skills/keel-work/SKILL.md")
        review = self.fixture.read(".claude/skills/keel-review/SKILL.md")
        self.assertIn("PR", plan)
        self.assertIn("/keel-review", work)
        self.assertIn("PR", review)

    def test_planning_skill_points_at_the_format_reference(self):
        self.generate()
        self.assertIn("keel/KEEL.md",
                      self.fixture.read(".claude/skills/keel-plan/SKILL.md"))

    @unittest.skipUnless(HAS_PYYAML, "PyYAML не встановлений")
    def test_a_real_yaml_parser_reads_the_frontmatter(self):
        """Свій читач поблажливий; шапку читатимуть Claude і Cursor, не він."""
        import yaml
        self.generate()
        for skill in keel.SKILLS:
            for _, relative in keel.skill_targets(skill):
                front, _, _ = keel.split_front_matter(self.fixture.read(relative))
                head = yaml.safe_load(front)
                self.assertEqual(
                    head["description"],
                    keel.skill_description(skill, self.project.settings["lang"]),
                    relative)
                self.assertIs(head.get("alwaysApply", False), False, relative)

    def test_second_run_changes_nothing(self):
        self.generate()
        _, out = self.generate()
        self.assertIn("не змінились", out)

    def test_hand_edit_is_restored(self):
        self.generate()
        target = ".cursor/skills/keel-work/SKILL.md"
        self.fixture.write(target, "правлено руками\n")
        _, out = self.generate()
        self.assertIn(target, out)
        self.assertIn("Generated by", self.fixture.read(target))


# ─────────────────────────────────────────────────────────────────────────────
# Agent hooks
# ─────────────────────────────────────────────────────────────────────────────




class TestSkillQuality(ProjectCase):
    """Механічне з рекомендацій Anthropic, плюс те, чого жодна ліба не знає."""

    # Поля специфікації Agent Skills плюс розширення, які знає Claude.
    KNOWN_FIELDS = {"name", "description", "paths", "argument-hint", "when_to_use",
                    "allowed-tools", "license", "compatibility", "metadata"}
    SHOUTY = ("ЗАВЖДИ", "НІКОЛИ", "ALWAYS", "NEVER", "MUST", "ОБОВ")

    def setUp(self):
        super().setUp()
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_skills(self.project, Args())
        finally:
            sys.stdout = saved

    def files(self):
        for skill in keel.SKILLS:
            for _, relative in keel.skill_targets(skill):
                yield relative, self.fixture.read(relative)

    def test_every_command_named_in_a_skill_exists(self):
        """Найцінніше: перейменували команду — скіл каже неправду, і тест червоніє."""
        import argparse
        import re
        parser = keel.build_parser()
        action = next(a for a in parser._actions
                      if isinstance(a, argparse._SubParsersAction))
        known = set(action.choices)
        for relative, text in self.files():
            for named in set(re.findall(r"keel\.py ([a-z-]+)", text)):
                self.assertIn(named, known, f"{relative}: команди {named} немає")

    def test_body_fits_the_recommended_budget(self):
        for relative, text in self.files():
            self.assertLess(len(text.splitlines()), 500, relative)

    def test_frontmatter_has_no_unknown_fields(self):
        for relative, text in self.files():
            front, _, _ = keel.split_front_matter(text)
            unknown = set(keel.parse_yaml(front)) - self.KNOWN_FIELDS
            self.assertFalse(unknown, f"{relative}: {unknown}")

    def test_no_shouting(self):
        """Рекомендація: пояснюй чому, замість капслочних наказів."""
        for relative, text in self.files():
            _, body, _ = keel.split_front_matter(text)
            for word in self.SHOUTY:
                self.assertNotIn(word, body, f"{relative}: {word}")

    def test_description_says_what_and_when(self):
        for skill in keel.SKILLS:
            description = keel.skill_description(skill)
            self.assertRegex(description, r"^[A-Z]\w+", skill["name"])
            self.assertNotIn("This skill", description, skill["name"])
            self.assertIn("use this skill", description.lower(), skill["name"])

    def test_description_names_concrete_trigger_phrases(self):
        """Проти недоспрацювання: опис має ловити те, як людина каже насправді."""
        import re
        for skill in keel.SKILLS:
            for lang in keel.LANGS:
                text = keel.skill_description(skill, lang)
                quoted = re.findall(r"«[^»]+»|\"[^\"]+\"", text)
                self.assertGreaterEqual(len(quoted), 2,
                                        f"{skill['name']}/{lang}: {quoted}")


if __name__ == "__main__":
    unittest.main()

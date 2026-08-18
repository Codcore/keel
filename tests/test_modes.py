#!/usr/bin/env python3
"""Three modes: how much of itself Keel installs, and who starts a procedure."""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from io import StringIO

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args  # noqa: E402

FLAG = "disable-model-invocation: true"


class ModeCase(unittest.TestCase):
    """A fresh mix project, installed once per test with the mode under test."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-mode-")
        self.addCleanup(shutil.rmtree, self.root, True)
        with open(os.path.join(self.root, "mix.exs"), "w", encoding="utf-8") as handle:
            handle.write("defmodule Demo.MixProject do\nend\n")
        subprocess.run(["git", "init", "-b", "main", "-q", self.root], check=True)

    def init(self, **kwargs):
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_init(keel.Project(self.root), Args(
                **{"install": True, "force": False, "no_commit": True, **kwargs}))
        finally:
            sys.stdout = saved
        return stream.getvalue()

    def path(self, name):
        return os.path.join(self.root, name)

    def read(self, name):
        with open(self.path(name), encoding="utf-8") as handle:
            return handle.read()

    def skills(self):
        return [self.read(relative)
                for skill in keel.SKILLS
                for _, relative in keel.skill_targets(skill)]

    def has_agent_hooks(self):
        """Both dialects or neither — a half-installed guard is worse than none."""
        cursor = os.path.exists(self.path(keel.CURSOR_HOOKS))
        settings = self.path(keel.CLAUDE_SETTINGS)
        claude = os.path.exists(settings) and "hooks" in json.loads(self.read(
            keel.CLAUDE_SETTINGS))
        self.assertEqual(cursor, claude, "хуки поставлені лише для одного агента")
        return cursor


class TestStrict(ModeCase):
    """The default: the agent starts the procedures, the hooks watch the edges."""

    def test_strict_is_what_you_get_without_saying_anything(self):
        self.init()
        self.assertEqual(json.loads(self.read(keel.CONFIG_FILE))["mode"], "strict")

    def test_the_agent_may_start_a_procedure(self):
        self.init()
        for text in self.skills():
            self.assertNotIn(FLAG, text)

    def test_the_hooks_are_installed(self):
        self.init()
        self.assertTrue(self.has_agent_hooks())


class TestSoft(ModeCase):
    """The agent still starts them; nothing watches while it works."""

    def test_the_agent_may_still_start_a_procedure(self):
        self.init(mode="soft")
        for text in self.skills():
            self.assertNotIn(FLAG, text)

    def test_no_agent_hooks(self):
        self.init(mode="soft")
        self.assertFalse(self.has_agent_hooks())

    def test_init_says_it_left_them_out(self):
        """Мовчазна відсутність охорони — та сама тиха зелена перевірка."""
        self.assertIn("soft", self.init(mode="soft"))


class TestManual(ModeCase):
    """Only the operator starts a procedure, by typing it."""

    def test_the_skills_become_commands(self):
        self.init(mode="manual")
        for text in self.skills():
            self.assertIn(FLAG, text)

    def test_both_agents_get_the_same_line(self):
        """Claude і Cursor читають одне й те саме поле — обидва мають його мати."""
        self.init(mode="manual")
        for skill in keel.SKILLS:
            for _, relative in keel.skill_targets(skill):
                self.assertIn(FLAG, self.read(relative), relative)

    def test_no_agent_hooks_either(self):
        self.init(mode="manual")
        self.assertFalse(self.has_agent_hooks())

    def test_the_session_hook_addresses_the_person(self):
        """Під manual виклик скіла моделлю блокується — текст має це знати."""
        self.init(mode="manual")
        project = keel.Project(self.root)
        self.assertIn("/keel-plan", keel.session_context(project))
        self.assertNotIn("Take the keel-plan skill", keel.session_context(project))


class TestTheFlagsOverruleTheMode(ModeCase):
    """The fourth combination the three words alone would lose."""

    def test_manual_with_hooks(self):
        self.init(mode="manual", agent_hooks=True)
        self.assertTrue(self.has_agent_hooks())
        for text in self.skills():
            self.assertIn(FLAG, text)

    def test_strict_without_hooks(self):
        self.init(mode="strict", agent_hooks=False)
        self.assertFalse(self.has_agent_hooks())


class TestTheModeIsRemembered(ModeCase):
    def test_it_is_written_down(self):
        self.init(mode="manual")
        self.assertEqual(json.loads(self.read(keel.CONFIG_FILE))["mode"], "manual")

    def test_regenerating_the_skills_keeps_it(self):
        """keel skills не має тихо повернути процедури моделі."""
        self.init(mode="manual")
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_skills(keel.Project(self.root))
        finally:
            sys.stdout = saved
        for text in self.skills():
            self.assertIn(FLAG, text)

    def test_update_does_not_drift_off_the_mode(self):
        """generated_files описує те, що методика поклала б зараз."""
        self.init(mode="manual")
        settings = keel.read_config(self.root)
        wanted = keel.generated_files(self.root, settings)
        for relative, text in wanted.items():
            if relative.endswith("SKILL.md"):
                self.assertIn(FLAG, text, relative)

    def test_rubbish_in_the_config_falls_back_instead_of_crashing(self):
        self.init()
        with open(self.path(keel.CONFIG_FILE), encoding="utf-8") as handle:
            stored = json.load(handle)
        stored["mode"] = "надзвичайний"
        with open(self.path(keel.CONFIG_FILE), "w", encoding="utf-8") as handle:
            json.dump(stored, handle)
        self.assertEqual(keel.read_config(self.root)["mode"],
                         keel.DEFAULTS["mode"])


class TestNarrowingTheModeTakesTheHooksBack(ModeCase):
    """Installing is half the job; a mode that excludes them has to remove them."""

    def test_the_cursor_file_goes(self):
        self.init()
        self.assertTrue(os.path.exists(self.path(keel.CURSOR_HOOKS)))
        self.init(mode="manual")
        self.assertFalse(os.path.exists(self.path(keel.CURSOR_HOOKS)))

    def test_our_entries_leave_the_claude_settings(self):
        self.init()
        self.init(mode="soft")
        self.assertFalse(self.has_agent_hooks())

    def test_somebody_elses_settings_survive_it(self):
        """Файл наш лише почасти — решта в ньому чужа й лишається."""
        self.init()
        path = self.path(keel.CLAUDE_SETTINGS)
        data = json.loads(self.read(keel.CLAUDE_SETTINGS))
        data["model"] = "opus"
        data["hooks"].setdefault("SessionStart", []).append(
            {"hooks": [{"type": "command", "command": "echo чуже"}]})
        with open(path, "w", encoding="utf-8") as handle:
            json.dump(data, handle)
        self.init(mode="manual")
        after = json.loads(self.read(keel.CLAUDE_SETTINGS))
        self.assertEqual(after["model"], "opus")
        self.assertIn("echo чуже", json.dumps(after, ensure_ascii=False))
        self.assertNotIn(keel.HOOK_TAG, json.dumps(after, ensure_ascii=False))

    def test_a_hand_edited_file_is_named_and_left(self):
        """Файл, якого ми не писали, не наш, щоб його видаляти."""
        self.init()
        with open(self.path(keel.CURSOR_HOOKS), "w", encoding="utf-8") as handle:
            handle.write('{"version": 1, "hooks": {}}\n')
        out = self.init(mode="manual")
        self.assertTrue(os.path.exists(self.path(keel.CURSOR_HOOKS)))
        self.assertIn("not what Keel wrote", out)

    def test_the_line_it_prints_is_true(self):
        """Раніше воно писало «хуків немає», поки хуки далі перехоплювали."""
        self.init()
        out = self.init(mode="manual")
        self.assertIn("no agent hooks", out)
        self.assertFalse(self.has_agent_hooks())


class TestUpdateKeepsTheMode(ModeCase):
    """The refresh that quietly put the guard back."""

    def update(self):
        done = subprocess.run(
            [sys.executable, os.path.join(keel.home(), "keel.py"),
             "-C", self.root, "update"], capture_output=True, text=True)
        return done.stdout + done.stderr

    def test_update_does_not_reinstall_what_the_mode_excluded(self):
        self.init(mode="manual")
        self.update()
        self.assertFalse(self.has_agent_hooks())

    def test_update_still_installs_them_under_strict(self):
        self.init(mode="strict")
        os.remove(self.path(keel.CURSOR_HOOKS))
        self.update()
        self.assertTrue(os.path.exists(self.path(keel.CURSOR_HOOKS)))


class TestGitHooksAreNotTheAgentHooks(ModeCase):
    """Two different animals under one word: these guard the repository."""

    def assert_git_hooks(self, mode):
        self.init(mode=mode)
        folder = os.path.join(self.root, ".git", "hooks")
        for name in keel.HOOKS:
            self.assertTrue(os.path.exists(os.path.join(folder, name)),
                            f"{mode}: {name}")

    def test_under_strict(self):
        self.assert_git_hooks("strict")

    def test_under_soft(self):
        self.assert_git_hooks("soft")

    def test_under_manual(self):
        self.assert_git_hooks("manual")


if __name__ == "__main__":
    unittest.main()

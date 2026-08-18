#!/usr/bin/env python3
"""The agent hooks: payloads in, verdicts out, one dialect each."""

import os
import json
import shutil
import subprocess
import tempfile
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args, ProjectCase  # noqa: E402




# ─────────────────────────────────────────────────────────────────────────────
# Agent hooks
# ─────────────────────────────────────────────────────────────────────────────

class TestFindPath(unittest.TestCase):
    """Only Claude documents the shape. The rest is defence, and it is tested."""

    def test_claude_write(self):
        self.assertEqual(keel.find_path({
            "tool_name": "Write",
            "tool_input": {"file_path": "/repo/lib/a.ex", "file_text": "x"},
        }), "/repo/lib/a.ex")

    def test_claude_edit(self):
        self.assertEqual(keel.find_path({
            "tool_name": "Edit",
            "tool_input": {"file_path": "/repo/lib/b.ex",
                           "old_str": "a", "new_str": "b"},
        }), "/repo/lib/b.ex")

    def test_cursor_tool_input_arrives_as_a_json_string(self):
        self.assertEqual(keel.find_path({
            "tool_name": "Write",
            "tool_input": '{"file_path": "/repo/lib/c.ex", "content": "x"}',
        }), "/repo/lib/c.ex")

    def test_camel_case_spelling(self):
        self.assertEqual(keel.find_path({"tool_input": {"filePath": "/repo/d.ex"}}),
                         "/repo/d.ex")

    def test_shell_payload_has_no_path(self):
        self.assertIsNone(keel.find_path({
            "tool_name": "Shell",
            "tool_input": {"command": "npm install", "working_directory": "/repo"},
        }))

    def test_empty_and_broken_payloads(self):
        self.assertIsNone(keel.find_path({}))
        self.assertIsNone(keel.find_path(None))
        self.assertIsNone(keel.find_path({"tool_input": "не json"}))

    def test_blank_path_does_not_count(self):
        self.assertIsNone(keel.find_path({"tool_input": {"file_path": "   "}}))




class TestWriteVerdict(ProjectCase):
    def payload(self, path):
        return {"tool_name": "Write", "tool_input": {"file_path": path}}

    def test_main_branch_says_nothing(self):
        self.assertIsNone(keel.write_verdict(self.project, self.payload("lib/x.ex")))

    def test_plan_branch_says_nothing(self):
        self.fixture.branch("plan/0001-session-loop")
        self.assertIsNone(keel.write_verdict(self.project, self.payload("lib/x.ex")))

    def test_declared_file_passes(self):
        self.fixture.branch("0001-session-loop")
        self.assertIsNone(keel.write_verdict(self.project,
                                             self.payload("lib/session.ex")))

    def test_absolute_path_is_resolved(self):
        self.fixture.branch("0001-session-loop")
        self.assertIsNone(keel.write_verdict(
            self.project, self.payload(self.fixture.path("lib/session.ex"))))

    def test_undeclared_file_is_denied(self):
        self.fixture.branch("0001-session-loop")
        kind, message = keel.write_verdict(self.project, self.payload("lib/extra.ex"))
        self.assertEqual(kind, "deny")
        self.assertIn("lib/extra.ex", message)
        self.assertIn("lib/session.ex", message)

    def test_keel_documents_pass(self):
        self.fixture.branch("0001-session-loop")
        self.assertIsNone(keel.write_verdict(
            self.project, self.payload("keel/steps/0001-session-loop.md")))

    def test_file_outside_the_repository_is_named_not_ignored(self):
        self.fixture.branch("0001-session-loop")
        kind, message = keel.write_verdict(self.project, self.payload("/etc/hosts"))
        self.assertEqual(kind, "note")
        self.assertIn("поза репозиторієм", message)

    def test_a_symlinked_path_still_resolves_to_the_project(self):
        """На macOS /tmp — симлінк; без realpath хук мовчав би геть завжди."""
        self.fixture.branch("0001-session-loop")
        unresolved = self.fixture.root.replace("/private/var", "/var")
        unresolved = unresolved.replace("/private/tmp", "/tmp")
        verdict = keel.write_verdict(
            self.project, self.payload(os.path.join(unresolved, "lib/extra.ex")))
        self.assertIsNotNone(verdict, unresolved)
        self.assertEqual(verdict[0], "deny")

    def test_notebook_path_is_recognised(self):
        self.fixture.branch("0001-session-loop")
        kind, message = keel.write_verdict(self.project, {
            "tool_name": "NotebookEdit",
            "tool_input": {"notebook_path": "lib/extra.ipynb"}})
        self.assertEqual(kind, "deny")
        self.assertIn("lib/extra.ipynb", message)

    def test_a_path_nested_in_a_list_is_found(self):
        self.assertEqual(keel.find_path(
            {"tool_input": {"edits": [{"file_path": "/repo/a.ex"}]}}), "/repo/a.ex")

    def test_unknown_payload_speaks_up_instead_of_passing_silently(self):
        self.fixture.branch("0001-session-loop")
        kind, message = keel.write_verdict(self.project, {"tool_input": {"nope": 1}})
        self.assertEqual(kind, "note")
        self.assertIn("не знайшлося шляху", message)
        self.assertIn("lib/session.ex", message)




class TestHookReply(unittest.TestCase):
    def test_claude_deny_shape(self):
        reply = keel.hook_reply("claude", "write", "deny", "причина")
        self.assertEqual(reply["hookSpecificOutput"]["permissionDecision"], "deny")
        self.assertEqual(reply["hookSpecificOutput"]["hookEventName"], "PreToolUse")
        self.assertEqual(reply["hookSpecificOutput"]["permissionDecisionReason"],
                         "причина")

    def test_cursor_deny_shape(self):
        reply = keel.hook_reply("cursor", "write", "deny", "причина")
        self.assertEqual(reply["permission"], "deny")
        self.assertEqual(reply["agent_message"], "причина")
        self.assertNotIn("hookSpecificOutput", reply)

    def test_claude_session_shape(self):
        reply = keel.hook_reply("claude", "session", "context", "текст")
        self.assertEqual(reply["hookSpecificOutput"]["hookEventName"], "SessionStart")
        self.assertEqual(reply["hookSpecificOutput"]["additionalContext"], "текст")

    def test_cursor_session_shape(self):
        self.assertEqual(keel.hook_reply("cursor", "session", "context", "текст"),
                         {"additional_context": "текст"})

    def test_note_never_carries_a_permission_decision(self):
        self.assertNotIn("permission", keel.hook_reply("cursor", "write", "note", "х"))
        self.assertNotIn("hookSpecificOutput",
                         keel.hook_reply("claude", "write", "note", "х"))




class TestSessionContext(ProjectCase):
    def test_main_branch_points_at_planning(self):
        text = keel.session_context(self.project)
        self.assertIn("keel-plan", text)

    def test_plan_branch_points_at_planning(self):
        self.fixture.branch("plan/0001-session-loop")
        text = keel.session_context(self.project)
        self.assertIn("keel-plan", text)
        self.assertIn("0001-session-loop", text)

    def test_work_branch_points_at_work_and_carries_the_package(self):
        self.fixture.branch("0001-session-loop")
        text = keel.session_context(self.project)
        self.assertIn("keel-work", text)
        self.assertIn("drive-turns", text)
        self.assertIn("lib/session.ex", text)

    def test_closed_step_points_at_review(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns: хід")
        text = keel.session_context(self.project)
        self.assertIn("keel-review", text)

    def test_every_state_names_exactly_one_skill(self):
        for branch in (None, "plan/0001-session-loop", "0001-session-loop"):
            if branch:
                self.fixture.git("checkout", "-q", "-b", branch)
            text = keel.session_context(self.project)
            named = [skill["name"] for skill in keel.SKILLS if skill["name"] in text]
            self.assertEqual(len(named), 1, f"{branch}: {named}")




class TestHookConfigs(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-hooks-")
        self.addCleanup(shutil.rmtree, self.root, True)

    def read(self, name):
        with open(os.path.join(self.root, name), encoding="utf-8") as handle:
            return json.load(handle)

    def write(self, name, data):
        path = os.path.join(self.root, name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(data if isinstance(data, str) else json.dumps(data))

    def generate(self):
        done = []
        keel.write_hook_configs(self.root, done)
        return done

    def test_cursor_config_shape(self):
        self.generate()
        config = self.read(keel.CURSOR_HOOKS)
        self.assertEqual(config["version"], 1)
        self.assertIn("sessionStart", config["hooks"])
        self.assertIn("preToolUse", config["hooks"])
        self.assertIn("--agent cursor", config["hooks"]["preToolUse"][0]["command"])

    def test_claude_config_shape(self):
        self.generate()
        hooks = self.read(keel.CLAUDE_SETTINGS)["hooks"]
        self.assertEqual(hooks["PreToolUse"][0]["matcher"], "Write|Edit|NotebookEdit")
        inner = hooks["PreToolUse"][0]["hooks"][0]
        self.assertEqual(inner["type"], "command")
        self.assertIn("--agent claude", inner["command"])

    def test_commands_do_not_depend_on_the_working_directory(self):
        """Хук може запуститись не з кореня — шлях має витримати це."""
        self.generate()
        claude = self.read(keel.CLAUDE_SETTINGS)["hooks"]["SessionStart"][0]["hooks"][0]
        self.assertIn("${CLAUDE_PROJECT_DIR}/", claude["command"])
        cursor = self.read(keel.CURSOR_HOOKS)["hooks"]["sessionStart"][0]["command"]
        self.assertIn("./" + keel.VENDORED, cursor)

    def test_existing_settings_are_kept(self):
        self.write(keel.CLAUDE_SETTINGS, {"model": "opus", "hooks": {
            "PreToolUse": [{"matcher": "Bash",
                            "hooks": [{"type": "command", "command": "./mine.sh"}]}]}})
        self.generate()
        data = self.read(keel.CLAUDE_SETTINGS)
        self.assertEqual(data["model"], "opus")
        commands = [item["command"] for entry in data["hooks"]["PreToolUse"]
                    for item in entry["hooks"]]
        self.assertIn("./mine.sh", commands)
        self.assertEqual(len([c for c in commands if keel.HOOK_TAG in c]), 1)

    def test_second_run_does_not_duplicate_our_entry(self):
        self.generate()
        self.generate()
        entries = self.read(keel.CLAUDE_SETTINGS)["hooks"]["PreToolUse"]
        self.assertEqual(len(entries), 1)

    def test_second_run_reports_nothing(self):
        self.generate()
        self.assertEqual(self.generate(), [])

    def test_unreadable_settings_are_left_alone(self):
        self.write(keel.CLAUDE_SETTINGS, "{ це не json")
        self.generate()
        with open(os.path.join(self.root, keel.CLAUDE_SETTINGS), encoding="utf-8") as h:
            self.assertEqual(h.read(), "{ це не json")




class TestHookCommand(ProjectCase):
    """Наскрізь: конфіг кличе саме те, що працює."""

    def run_hook(self, event, agent, payload):
        command = keel.hook_command(event, agent).split()
        self.assertTrue(command[1].endswith(keel.VENDORED), command[1])
        command[1] = os.path.abspath(keel.__file__)
        done = subprocess.run(command, cwd=self.fixture.root, input=json.dumps(payload),
                              capture_output=True, text=True)
        self.assertEqual(done.returncode, 0, done.stderr)
        return json.loads(done.stdout) if done.stdout.strip() else None

    def test_session_hook_hands_over_the_package(self):
        self.fixture.branch("0001-session-loop")
        reply = self.run_hook("session", "claude", {"hook_event_name": "SessionStart"})
        context = reply["hookSpecificOutput"]["additionalContext"]
        self.assertIn("keel-work", context)
        self.assertIn("lib/session.ex", context)

    def test_write_hook_denies_in_cursor_dialect(self):
        self.fixture.branch("0001-session-loop")
        reply = self.run_hook("write", "cursor", {
            "tool_name": "Write",
            "tool_input": '{"file_path": "lib/extra.ex", "content": "x"}'})
        self.assertEqual(reply["permission"], "deny")
        self.assertIn("lib/extra.ex", reply["user_message"])

    def test_write_hook_stays_quiet_on_a_declared_file(self):
        self.fixture.branch("0001-session-loop")
        self.assertIsNone(self.run_hook("write", "cursor", {
            "tool_name": "Write",
            "tool_input": {"file_path": "lib/session.ex"}}))


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3
"""The agent hooks: payloads in, verdicts out, one dialect each."""

import os
import json
import shutil
import shlex
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




class TestScopeWidenedAfterApproval(ProjectCase):
    """Розширювати список дозволено — але не мовчки.

    Раніше послідовність «допиши файл у крок, тоді пиши що завгодно» проходила
    обидва заслони без жодного слова, і сказати про це могла хіба що перевірка,
    три ходи по тому й в іншому місці.
    """

    def verdict(self, path):
        return keel.write_verdict(
            self.project, {"tool_input": {"file_path": self.fixture.path(path)}})

    def widen(self, name):
        wave = "keel/waves/0001-session-loop.md"
        self.fixture.write(wave, self.fixture.read(wave).replace(
            "files:      [lib/session.ex]", f"files:      [lib/session.ex, {name}]"))

    def test_a_file_from_the_approved_plan_passes_in_silence(self):
        self.fixture.branch("0001-session-loop")
        self.assertIsNone(self.verdict("lib/session.ex"))

    def test_a_file_added_here_is_allowed_and_said(self):
        self.fixture.branch("0001-session-loop")
        self.widen("lib/extra.ex")
        kind, message = self.verdict("lib/extra.ex")
        self.assertEqual(kind, "note")
        self.assertIn("lib/extra.ex", message)
        self.assertIn("approved", message)

    def test_a_file_in_no_list_at_all_is_still_refused(self):
        self.fixture.branch("0001-session-loop")
        kind, _ = self.verdict("lib/nobody.ex")
        self.assertEqual(kind, "deny")

    def test_a_wave_that_never_reached_main_has_nothing_to_compare_with(self):
        """Крок, якого на головній гілці немає, ще ніхто не схвалював."""
        self.fixture.branch("0002-fresh")
        self.fixture.write("keel/waves/0002-fresh.md", self.fixture.read(
            "keel/waves/0001-session-loop.md"))
        self.assertIsNone(keel.approved_files(
            self.project, self.project.waves["0002-fresh"]))


class TestTheMainBranchIsNotAWorkbench(ProjectCase):
    """Гілка, де за планом нічого не роблять, була єдиною, де можна було все.

    Перевірка 4 порівнює гілку з `main`, тож, стоячи на самому `main`, їй нема
    з чим порівнювати; хук мовчав. Знайдено прогулянкою циклом, не тестом.
    """

    def verdict(self, path):
        return keel.write_verdict(
            self.project, {"tool_input": {"file_path": self.fixture.path(path)}})

    def test_code_on_the_main_branch_is_refused(self):
        verdict = self.verdict("lib/whatever.ex")
        self.assertIsNotNone(verdict)
        kind, message = verdict
        self.assertEqual(kind, "deny")
        self.assertIn("lib/whatever.ex", message)

    def test_keels_own_furniture_stays_free(self):
        for name in ("keel/waves/0001-session-loop.md", "AGENTS.md",
                     "keel/keel.json"):
            self.assertIsNone(self.verdict(name), name)

    def test_a_project_with_no_waves_is_not_walled_in(self):
        """Проєкт, який щойно взяв Keel, ще не має плану, з якого працювати."""
        os.remove(self.fixture.path("keel/waves/0001-session-loop.md"))
        self.assertIsNone(self.verdict("lib/whatever.ex"))

    def test_a_wave_branch_is_judged_by_its_scope_as_before(self):
        self.fixture.branch("0001-session-loop")
        self.assertIsNone(self.verdict("lib/session.ex"))
        kind, _ = self.verdict("lib/nobody_declared.ex")
        self.assertEqual(kind, "deny")


class TestWriteVerdict(ProjectCase):
    def payload(self, path):
        return {"tool_name": "Write", "tool_input": {"file_path": path}}

    def test_main_branch_refuses_code(self):
        """Було «мовчить»; мовчання й виявилось дірою — див. TestTheMainBranchIsNotAWorkbench."""
        kind, _ = keel.write_verdict(self.project, self.payload("lib/x.ex"))
        self.assertEqual(kind, "deny")

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
            self.project, self.payload("keel/waves/0001-session-loop.md")))

    def test_file_outside_the_repository_is_named_not_ignored(self):
        self.fixture.branch("0001-session-loop")
        kind, message = keel.write_verdict(self.project, self.payload("/etc/hosts"))
        self.assertEqual(kind, "note")
        self.assertIn("is outside the repository", message)

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
        self.assertIn("carried no file path", message)
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




class TestSessionContextRespectsTheGate(ProjectCase):
    """The hook may not dictate work the method's own gate refuses."""

    def test_an_unapproved_wave_hands_out_no_package(self):
        """`next` відмовляє, поки крок не на main — хук казав протилежне."""
        self.fixture.git("checkout", "-q", "-b", "plan/0002-later")
        self.fixture.write("keel/waves/0002-later.md",
                           "---\ntransforms:\n  do:\n    files: [lib/b.ex]\n"
                           "---\n\n## Why\n\nх.\n\n## transform: do\n\nЩось.\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-q", "-m", "план")
        self.fixture.git("checkout", "-q", "-b", "0002-later")
        text = keel.session_context(self.project)
        self.assertIn("is not on", text)
        self.assertNotIn("keel-work", text)

    def test_an_approved_wave_still_hands_out_the_package(self):
        self.fixture.branch("0001-session-loop")
        self.assertIn("keel-work", keel.session_context(self.project))


class TestSessionContext(ProjectCase):
    def test_the_hook_does_not_contradict_the_skill_on_order(self):
        """Номер зʼявляється після new wave, тож гілку заводять після нього."""
        self.fixture.git("commit", "--allow-empty", "-m", "drive-turns: зроблено")
        text = keel.session_context(self.project)
        self.assertLess(text.index("new wave"), text.index("plan/"), text)

    def test_on_main_the_hook_says_what_next_says(self):
        """Хук читають першим, і він казав «роботи тут немає» над схваленим
        кроком, кожна трансформа якого стояла незакритою."""
        text = keel.session_context(self.project)
        self.assertIn("0001-session-loop", text)
        self.assertIn("git checkout -b 0001-session-loop", text)

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

    def test_closed_wave_points_at_review(self):
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
        """The same two writes init does: the cursor file from the table, the
        Claude entries merged — write_hook_configs went away with the dedup."""
        done = []
        keel.write_if_changed(
            os.path.join(self.root, keel.CURSOR_HOOKS),
            json.dumps(keel.cursor_hook_config(), ensure_ascii=False, indent=2) + "\n",
            done, keel.CURSOR_HOOKS)
        keel.merge_claude_settings(os.path.join(self.root, keel.CLAUDE_SETTINGS), done)
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




class TestHookCommandQuoting(unittest.TestCase):
    """The command is one word to the shell even when the path has a space."""

    def test_the_path_is_quoted(self):
        command = keel.hook_command("session", "claude")
        self.assertIn('"${CLAUDE_PROJECT_DIR}/keel/keel.py"', command)
        parts = shlex.split(command)
        self.assertEqual(parts[1], "${CLAUDE_PROJECT_DIR}/keel/keel.py")
        self.assertEqual(parts[2:], ["hook", "session", "--agent", "claude"])

    def test_our_entries_are_still_recognised_as_ours(self):
        """Лапка розірвала «keel.py hook»; мітка мусить це пережити."""
        for agent in ("claude", "cursor"):
            for event in ("session", "write"):
                self.assertIn(keel.HOOK_TAG, keel.hook_command(event, agent))


class TestHookCommand(ProjectCase):
    """Наскрізь: конфіг кличе саме те, що працює."""

    def run_hook(self, event, agent, payload):
        command = shlex.split(keel.hook_command(event, agent))
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


class TestTheHookSpeaksWhenItCannotJudge(unittest.TestCase):
    """Unreadable is not the same as unrestricted."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-mute-")
        self.addCleanup(shutil.rmtree, self.root, True)
        for folder in ("keel/waves", "keel/contracts"):
            os.makedirs(os.path.join(self.root, folder))
        subprocess.run(["git", "init", "-b", "main", "-q", self.root], check=True)
        for name, value in (("user.email", "t@e.com"), ("user.name", "t")):
            subprocess.run(["git", "-C", self.root, "config", name, value], check=True)
        subprocess.run(["git", "-C", self.root, "commit", "-q", "--allow-empty",
                        "-m", "base"], check=True)
        subprocess.run(["git", "-C", self.root, "checkout", "-q", "-b",
                        "0001-a"], check=True)

    def wave(self, text):
        with open(os.path.join(self.root, "keel/waves/0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write(text)

    def verdict(self):
        return keel.write_verdict(keel.Project(self.root),
                                  {"tool_input": {"file_path": "lib/anything.ex"}})

    def test_a_wave_that_does_not_parse_is_said_out_loud(self):
        """Раніше зламана шапка вимикала охорону без жодного слова."""
        self.wave("---\ntransforms:\n  - do-it\n---\n\n## Why\n\nх.\n")
        kind, message = self.verdict()
        self.assertEqual(kind, "note")
        self.assertIn("scope is not being checked", message)

    def test_a_wave_with_no_transforms_is_said_too(self):
        self.wave("---\ndepends_on: []\n---\n\n## Why\n\nх.\n")
        kind, message = self.verdict()
        self.assertEqual(kind, "note")
        self.assertIn("declares no transforms", message)

    def test_keels_own_furniture_is_not_denied(self):
        """Хук навмисно не суворіший за гейт: що звільняє перевірка 4, те й він."""
        self.wave("---\ntransforms:\n  do-it:\n    files: [lib/a.ex]\n---\n\n"
                  "## Why\n\nх.\n\n## transform: do-it\n\nЩось.\n")
        for name in ("AGENTS.md", ".claude/skills/keel-plan/SKILL.md",
                     ".github/workflows/keel.yml"):
            verdict = keel.write_verdict(
                keel.Project(self.root), {"tool_input": {"file_path": name}})
            self.assertIsNone(verdict, name)

    def test_a_readable_wave_still_denies_an_undeclared_file(self):
        self.wave("---\ntransforms:\n  do-it:\n    files: [lib/a.ex]\n---\n\n"
                  "## Why\n\nх.\n\n## transform: do-it\n\nЩось.\n")
        kind, message = self.verdict()
        self.assertEqual(kind, "deny")
        self.assertIn("lib/anything.ex", message)


if __name__ == "__main__":
    unittest.main()


class TestTheHookSaysWhenItCannotJudgeOnMain(ProjectCase):
    """Той самий невідомий був гучним на гілці кроку й тихим на головній —
    саме там, де коду взагалі не місце."""

    def test_a_payload_without_a_path_is_named_on_main(self):
        kind, message = keel.write_verdict(self.project, {"tool_name": "Write"})
        self.assertEqual(kind, "note")
        self.assertIn("no file path", message)

    def test_a_payload_without_a_path_is_still_named_on_a_wave_branch(self):
        self.fixture.branch("0001-session-loop")
        kind, message = keel.write_verdict(self.project, {"tool_name": "Write"})
        self.assertEqual(kind, "note")
        self.assertIn("no file path", message)


class TestApprovedFilesComeFromTheBranchPoint(ProjectCase):
    """Крок могли поправити на main уже після того, як гілку відрізали."""

    def test_an_amendment_made_on_main_is_not_this_branch_widening(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.git("checkout", "main")
        wave = "keel/waves/0001-session-loop.md"
        self.fixture.write(wave, self.fixture.read(wave).replace(
            "files:      [lib/session.ex]", "files:      [lib/elsewhere.ex]"))
        self.fixture.git("commit", "-am", "хтось інший переписав список")
        self.fixture.git("checkout", "0001-session-loop")
        self.assertFalse(keel.widened_here(
            self.project, self.project.waves["0001-session-loop"], "lib/session.ex"))


class TestTheHookSpeaksOnADetachedHead(ProjectCase):
    """Перервана перебазова, bisect, checkout за хешем — і заслон вимикався
    без жодного слова, хоч перевірка 4 про той самий стан каже вголос."""

    def test_a_detached_head_is_named(self):
        self.fixture.git("checkout", "--detach", "HEAD")
        verdict = keel.write_verdict(
            self.project, {"tool_input": {"file_path": self.fixture.path("lib/x.ex")}})
        self.assertIsNotNone(verdict)
        kind, message = verdict
        self.assertEqual(kind, "note")
        self.assertIn("detached", message)

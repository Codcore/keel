#!/usr/bin/env python3
"""Whom Keel equips, and whom it leaves alone.

Two agents exist in the world; the third is ours and nobody else's yet. Writing
`.keel-agent/` into a stranger's repository would leave a folder for a tool they
do not have, so the third is asked for and never assumed.
"""

import contextlib
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args, ProjectCase  # noqa: E402


class TestTheStoredListIsReadCarefully(ProjectCase):
    def stored(self, value):
        self.fixture.write(keel.CONFIG_FILE, json.dumps({"agents": value}) + "\n")
        return keel.read_config(self.fixture.root)["agents"]

    def test_the_default_is_the_two_that_exist(self):
        self.assertEqual(keel.DEFAULTS["agents"], list(keel.DEFAULT_AGENTS))

    def test_order_is_ours_and_not_the_order_it_was_typed_in(self):
        """Otherwise one choice generates two different sets."""
        self.assertEqual(self.stored(["keel-agent", "claude"]),
                         ["claude", "keel-agent"])

    def test_a_name_nobody_knows_leaves_the_default_standing(self):
        self.assertEqual(self.stored(["claude", "mystery"]),
                         list(keel.DEFAULT_AGENTS))

    def test_empty_is_an_answer_and_not_a_missing_one(self):
        self.assertEqual(self.stored([]), [])

    def test_something_that_is_not_a_list_is_refused(self):
        self.assertEqual(self.stored("claude"), list(keel.DEFAULT_AGENTS))


class TestWhatLandsInTheProject(ProjectCase):
    def files(self, agents):
        return keel.generated_files(self.fixture.root,
                                    dict(keel.DEFAULTS, agents=agents))

    def under(self, agents, folder):
        return sorted(name for name in self.files(agents)
                      if name.startswith(folder))

    def test_by_default_the_third_folder_is_never_touched(self):
        """The whole point: a stranger's repository stays as it was."""
        self.assertEqual(self.under(list(keel.DEFAULT_AGENTS), ".keel-agent/"), [])

    def test_asking_for_it_puts_every_skill_there(self):
        landed = self.under(["keel-agent"], ".keel-agent/skills/")
        self.assertEqual(len(landed), len(keel.SKILLS))
        for skill in keel.SKILLS:
            self.assertIn(".keel-agent/skills/%s/SKILL.md" % skill["name"], landed)

    def test_the_others_are_left_out_when_they_are_not_asked_for(self):
        """Half of the check: present when asked is worthless without absent when not."""
        self.assertEqual(self.under(["keel-agent"], ".claude/"), [])
        self.assertEqual(self.under(["keel-agent"], ".cursor/"), [])

    def test_cursors_generated_hook_file_follows_cursor(self):
        self.assertIn(keel.CURSOR_HOOKS, self.files(["claude", "cursor"]))
        self.assertNotIn(keel.CURSOR_HOOKS, self.files(["claude", "keel-agent"]))

    def test_nobody_asked_for_means_no_agent_files_at_all(self):
        for name in self.files([]):
            for folder in (".claude/", ".cursor/", ".keel-agent/"):
                self.assertFalse(name.startswith(folder), name)


class TestTheThirdSpeaksClaudesContract(unittest.TestCase):
    """Decision 10 of the project being built: its hooks are Claude's contract,
    with no events of its own. So the config has Claude's shape and its own name."""

    def test_the_events_are_the_same_as_claudes(self):
        self.assertEqual(sorted(keel.claude_hook_config("keel-agent")),
                         sorted(keel.claude_hook_config()))

    def test_every_command_calls_the_hook_under_its_own_name(self):
        for entries in keel.claude_hook_config("keel-agent").values():
            for entry in entries:
                for hook in entry["hooks"]:
                    self.assertIn("--agent keel-agent", hook["command"])

    def test_the_hook_command_accepts_the_name(self):
        parsed = keel.build_parser().parse_args(
            ["hook", "write", "--agent", "keel-agent"])
        self.assertEqual(parsed.agent, "keel-agent")

    def test_its_skills_are_rendered_the_way_claudes_are(self):
        for skill in keel.SKILLS:
            self.assertEqual(keel.render_skill(skill, "keel-agent", "uk", "strict"),
                             keel.render_skill(skill, "claude", "uk", "strict"))


class TestTheHooksGoIntoItsOwnFile(ProjectCase):
    def merge(self, agents):
        done = []
        keel.merge_agent_settings(self.fixture.root,
                                  dict(keel.DEFAULTS, agents=agents), done)
        return done

    def test_asked_for_it_gets_its_own_settings_file(self):
        self.merge(["keel-agent"])
        written = json.loads(self.fixture.read(keel.KEEL_AGENT_SETTINGS))
        self.assertIn("PreToolUse", written["hooks"])
        self.assertFalse(os.path.exists(self.fixture.path(keel.CLAUDE_SETTINGS)))

    def test_not_asked_for_it_gets_nothing(self):
        self.merge(["claude"])
        self.assertFalse(os.path.exists(self.fixture.path(keel.KEEL_AGENT_SETTINGS)))

    def test_what_is_already_in_that_file_survives(self):
        self.fixture.write(keel.KEEL_AGENT_SETTINGS,
                           json.dumps({"model": "theirs"}) + "\n")
        self.merge(["keel-agent"])
        written = json.loads(self.fixture.read(keel.KEEL_AGENT_SETTINGS))
        self.assertEqual(written["model"], "theirs")

    def test_removing_takes_our_entries_out_of_that_file_too(self):
        self.merge(["keel-agent"])
        keel.remove_hook_configs(self.fixture.root, [])
        path = self.fixture.path(keel.KEEL_AGENT_SETTINGS)
        if os.path.exists(path):
            self.assertNotIn("hooks", json.loads(self.fixture.read(
                keel.KEEL_AGENT_SETTINGS)))
        else:
            # Holding nothing but ours, it left with them — and the folder that
            # existed only for it goes too, which is the point of asking first.
            self.assertFalse(os.path.exists(self.fixture.path(".keel-agent")))


class TestTheFlagSaysWhoRatherThanGuessing(unittest.TestCase):
    def parse(self, text):
        return keel.build_parser().parse_args(["init", "--agents", text]).agents

    def test_it_reads_a_list(self):
        self.assertEqual(self.parse("keel-agent,claude"), ["claude", "keel-agent"])

    def test_a_typo_stops_the_run_rather_than_equipping_fewer(self):
        with self.assertRaises(SystemExit):
            self.parse("claude,cursur")

    def test_empty_reaches_the_command_as_a_choice(self):
        self.assertEqual(self.parse(""), [])


class TestDroppingAnAgentTakesItsHooksAway(ProjectCase):
    """Adding for whoever is asked for is half the job.

    An agent taken off the list keeps its entries otherwise, and entries left in
    a settings file keep firing — the operator reads keel.json, believes that
    agent is not equipped, and it is.
    """

    def merge(self, agents):
        keel.merge_agent_settings(self.fixture.root,
                                  dict(keel.DEFAULTS, agents=agents), [])

    def ours_in(self, relative):
        path = self.fixture.path(relative)
        if not os.path.exists(path):
            return 0
        hooks = json.loads(self.fixture.read(relative)).get("hooks", {})
        return sum(1 for entries in hooks.values()
                   for entry in entries if keel.is_ours(entry))

    def test_the_one_dropped_loses_its_entries(self):
        self.merge(["claude"])
        self.assertEqual(self.ours_in(keel.CLAUDE_SETTINGS), 2)
        self.merge(["keel-agent"])
        self.assertEqual(self.ours_in(keel.CLAUDE_SETTINGS), 0)
        self.assertEqual(self.ours_in(keel.KEEL_AGENT_SETTINGS), 2)

    def test_what_is_not_ours_in_that_file_survives_the_stripping(self):
        self.merge(["claude"])
        data = json.loads(self.fixture.read(keel.CLAUDE_SETTINGS))
        data["model"] = "theirs"
        self.fixture.write(keel.CLAUDE_SETTINGS, json.dumps(data) + "\n")
        self.merge(["keel-agent"])
        self.assertEqual(json.loads(self.fixture.read(keel.CLAUDE_SETTINGS))["model"],
                         "theirs")

    def test_a_file_that_was_never_written_is_not_created_to_be_stripped(self):
        self.merge(["claude"])
        self.assertFalse(os.path.exists(self.fixture.path(keel.KEEL_AGENT_SETTINGS)))


class TestOwnershipIsAnsweredByRecord(ProjectCase):
    """A settings file belongs to the project and holds its keys beside ours.

    Ours is the record of what Keel left there, not the presence of our entries
    in it: taking our entries out is itself something Keel does, and a rule
    reading the file after that edit calls the edit somebody else's.
    """

    def test_a_file_we_never_wrote_in_is_not_ours(self):
        self.fixture.write(keel.CLAUDE_SETTINGS,
                           json.dumps({"model": "theirs"}) + "\n")
        self.assertFalse(keel.keel_owns(keel.CLAUDE_SETTINGS, self.fixture.root))

    def test_installing_makes_it_ours(self):
        self.fixture.install_hooks()
        self.assertTrue(keel.keel_owns(keel.CLAUDE_SETTINGS, self.fixture.root))

    def test_it_stays_ours_after_keel_takes_its_own_entries_out(self):
        """The regression this rule exists for. `update` narrowing a mode on a
        plan branch edited the file, and check 4 then condemned that edit with
        nothing that could satisfy it — the file had stopped being ours at the
        moment we changed it."""
        self.fixture.install_hooks()
        wrote = []
        keel.remove_hook_configs(self.fixture.root, [], wrote)
        keel.write_config(self.fixture.root, keel.read_config(self.fixture.root),
                          [], None, keel.merged_record(self.fixture.root, wrote))
        # Holding nothing but ours, it goes with them — and it stays ours while
        # it does, or check 4 would condemn our own removal.
        self.assertFalse(os.path.exists(self.fixture.path(keel.CLAUDE_SETTINGS)))
        self.assertTrue(keel.keel_owns(keel.CLAUDE_SETTINGS, self.fixture.root))

    def test_a_hand_edit_gives_it_back_to_the_project(self):
        self.fixture.install_hooks()
        data = json.loads(self.fixture.read(keel.CLAUDE_SETTINGS))
        data["model"] = "theirs"
        self.fixture.write(keel.CLAUDE_SETTINGS, json.dumps(data) + "\n")
        self.assertFalse(keel.keel_owns(keel.CLAUDE_SETTINGS, self.fixture.root))

    def test_a_file_somebody_else_deleted_is_not_ours(self):
        """Gone at our hand is recorded as gone; gone at theirs is not."""
        self.fixture.install_hooks()
        os.remove(self.fixture.path(keel.CLAUDE_SETTINGS))
        self.assertFalse(keel.keel_owns(keel.CLAUDE_SETTINGS, self.fixture.root))

    def test_the_third_agents_file_follows_the_same_rule(self):
        self.fixture.install_hooks(agents=["keel-agent"])
        self.assertTrue(keel.keel_owns(keel.KEEL_AGENT_SETTINGS, self.fixture.root))
        self.assertFalse(keel.keel_owns(keel.CLAUDE_SETTINGS, self.fixture.root))

    def test_without_a_root_there_is_nothing_to_read_and_the_answer_is_no(self):
        self.assertFalse(keel.keel_owns(keel.CLAUDE_SETTINGS))

    def test_the_skills_folder_is_still_ours_by_structure(self):
        """Only we write there, so no record is needed and none is read."""
        self.assertTrue(keel.keel_owns(".keel-agent/skills/keel-plan/SKILL.md"))

    def test_its_sessions_are_never_ours(self):
        self.assertFalse(keel.keel_owns(".keel-agent/sessions/x.jsonl",
                                        self.fixture.root))


class TestTheDefaultListIsNotSharedBetweenReaders(ProjectCase):
    def test_two_reads_do_not_hand_back_the_same_object(self):
        first = keel.read_config(self.fixture.root)["agents"]
        second = keel.read_config(self.fixture.root)["agents"]
        self.assertEqual(first, second)
        self.assertIsNot(first, second)

    def test_editing_what_was_read_leaves_the_default_alone(self):
        """The natural way to write such a change, and it used to poison every
        later read in the same process."""
        keel.read_config(self.fixture.root)["agents"].remove("cursor")
        self.assertEqual(keel.read_config(self.fixture.root)["agents"],
                         list(keel.DEFAULT_AGENTS))


class TestARefusalNamesAFileSomebodyCanOpen(ProjectCase):
    def test_broken_json_is_reported_under_its_own_name(self):
        """The note that goes into the report is a sentence; the file is not."""
        self.fixture.write(keel.CLAUDE_SETTINGS, "{ this is not json\n")
        printed = io.StringIO()
        with contextlib.redirect_stdout(printed):
            keel.strip_claude_settings(self.fixture.path(keel.CLAUDE_SETTINGS), [])
        said = printed.getvalue()
        self.assertIn(keel.CLAUDE_SETTINGS, said)
        self.assertNotIn("our hook entries taken out", said)


class TestInitTakesBackWhatItStopsGenerating(unittest.TestCase):
    """Reachable only because `--agents` can narrow.

    init run a second time with a shorter list used to leave the dropped agent's
    skills and hook file on disk, working, while keel.json said that agent was
    not equipped. update already took them back; the two commands disagreed
    about the same setting.
    """

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-agents-init-")
        self.addCleanup(shutil.rmtree, self.root, True)
        with open(os.path.join(self.root, "mix.exs"), "w", encoding="utf-8") as handle:
            handle.write("defmodule Demo.MixProject do\nend\n")
        subprocess.run(["git", "init", "-b", "main", "-q", self.root], check=True)

    def init(self, **kwargs):
        stream, saved = io.StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_init(keel.Project(self.root),
                          Args(**{"install": True, "force": False,
                                  "no_commit": True, **kwargs}))
        finally:
            sys.stdout = saved
        return stream.getvalue()

    def there(self, relative):
        return os.path.exists(os.path.join(self.root, relative))

    def test_a_narrowed_list_leaves_nothing_of_the_agent_it_dropped(self):
        self.init()
        self.assertTrue(self.there(".cursor/skills/keel-plan/SKILL.md"))
        self.assertTrue(self.there(keel.CURSOR_HOOKS))

        self.init(agents=["claude"])
        self.assertFalse(self.there(".cursor/skills/keel-plan/SKILL.md"))
        self.assertFalse(self.there(keel.CURSOR_HOOKS))
        self.assertTrue(self.there(".claude/skills/keel-plan/SKILL.md"))

    def test_a_widened_list_brings_the_third_in(self):
        self.init()
        self.assertFalse(self.there(".keel-agent/skills/keel-plan/SKILL.md"))
        self.init(agents=["claude", "cursor", "keel-agent"])
        self.assertTrue(self.there(".keel-agent/skills/keel-plan/SKILL.md"))


class TestTheCommandsAgreeAboutTheSetting(unittest.TestCase):
    """`init` and `skills` install the same things; they must ask the same
    question about whom for. They did not: `skills` installed for the default
    two whatever the project had asked."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-agree-")
        self.addCleanup(shutil.rmtree, self.root, True)
        with open(os.path.join(self.root, "mix.exs"), "w", encoding="utf-8") as h:
            h.write("defmodule D.MixProject do\nend\n")
        subprocess.run(["git", "init", "-b", "main", "-q", self.root], check=True)

    def run_quiet(self, command, **kwargs):
        stream, saved = io.StringIO(), sys.stdout
        sys.stdout = stream
        try:
            return command(keel.Project(self.root),
                           Args(**{"install": True, "force": False,
                                   "no_commit": True, **kwargs}))
        finally:
            sys.stdout = saved

    def there(self, relative):
        return os.path.exists(os.path.join(self.root, relative))

    def test_skills_installs_for_whom_the_project_asked(self):
        self.run_quiet(keel.cmd_init, agents=["claude"])
        self.run_quiet(keel.cmd_skills)
        self.assertTrue(self.there(".claude/skills/keel-plan/SKILL.md"))
        self.assertFalse(self.there(".cursor/skills/keel-plan/SKILL.md"))

    def test_skills_reaches_the_third_when_it_was_asked_for(self):
        self.run_quiet(keel.cmd_init, agents=["keel-agent"])
        os.remove(os.path.join(self.root, ".keel-agent/skills/keel-plan/SKILL.md"))
        self.run_quiet(keel.cmd_skills)
        self.assertTrue(self.there(".keel-agent/skills/keel-plan/SKILL.md"))

    def test_narrowing_leaves_no_empty_folders_behind(self):
        self.run_quiet(keel.cmd_init, agents=["claude", "cursor", "keel-agent"])
        self.run_quiet(keel.cmd_init, agents=["claude"])
        self.assertFalse(self.there(".cursor/skills"))
        self.assertFalse(self.there(".keel-agent"))
        self.assertTrue(self.there(".cursor"))      # theirs, not ours to sweep
        self.assertTrue(self.there(".claude/skills"))

    def test_a_typo_in_the_file_refuses_as_loudly_as_one_in_the_flag(self):
        """read_config falls back to the default, which for a command that
        installs means equipping two agents nobody named and skipping the one
        they did."""
        self.run_quiet(keel.cmd_init, agents=["keel-agent"])
        path = os.path.join(self.root, keel.CONFIG_FILE)
        stored = json.loads(keel.read_text(path))
        stored["agents"] = ["keel-agent", "cursur"]
        with open(path, "w", encoding="utf-8") as h:
            h.write(json.dumps(stored, indent=2) + "\n")
        with self.assertRaises(SystemExit):
            self.run_quiet(keel.cmd_skills)
        self.assertIsNotNone(keel.config_list_problem(self.root))


if __name__ == "__main__":
    unittest.main()

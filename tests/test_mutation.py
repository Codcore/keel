#!/usr/bin/env python3
"""The mutation gate: asked once, at the close of a wave, and never earlier."""

import contextlib
import io
import json
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args, ProjectCase  # noqa: E402


class MutationCase(ProjectCase):
    """Shared ground: a project that can name a mutation command, and a wave
    that can be brought to its close."""

    def settings(self, command):
        path = self.fixture.path(keel.CONFIG_FILE)
        stored = json.loads(keel.read_text(path)) if os.path.exists(path) else {}
        stored["mutation"] = command
        self.fixture.write(keel.CONFIG_FILE, json.dumps(stored) + "\n")

    def verdict(self, command, run=True):
        self.settings(command)
        return keel.mutation_verdict(self.project, run=run)

    def close_the_wave(self):
        """Every transform of 0001-session-loop closed by a commit.

        The wave has one — `drive-turns` — and §8.4 says a commit names its
        transform by the slug at the start of its message. The plan is already
        on main from the fixture's first commit.
        """
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex",
                           "defmodule Demo.Session do\n  def run(_, _, _), do: :ok\nend\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "drive-turns: ходи покручено")


class TestTheCommandItself(MutationCase):
    """Three states, and the middle one is not CI's middle one.

    A command is run. `none` is a decision said out loud, and it is silent. An
    empty setting is nobody having decided — and at the close of a wave that is
    red, because a wave closes once.
    """

    def test_a_command_that_passes_says_nothing(self):
        problems, note, ran = self.verdict("true")
        self.assertEqual(problems, [])
        self.assertIsNone(note)
        self.assertTrue(ran)

    def test_a_command_that_fails_is_a_problem(self):
        problems, _, ran = self.verdict("false")
        self.assertTrue(ran)
        self.assertEqual(len(problems), 1)
        self.assertIn("false", problems[0].message)

    def test_survivors_are_named_as_survivors(self):
        """Not "the command failed": the reader is told what a red run means."""
        problems, _, _ = self.verdict("false")
        self.assertIn(keel.t("mutations"), problems[0].message.lower())

    def test_a_command_that_is_not_there_says_so(self):
        """127 is the shell saying the thing is absent, which is its own state."""
        problems, _, ran = self.verdict("keel-no-such-mutation-tool-anywhere")
        self.assertTrue(ran)
        self.assertEqual(len(problems), 1)
        self.assertNotIn(keel.t("mutations survived: {command} — the suite passed "
                                "while the code was broken, so what it proves is "
                                "smaller than it looks.")[:20],
                         problems[0].message)

    def test_none_is_silent(self):
        problems, note, ran = self.verdict("none")
        self.assertEqual(problems, [])
        self.assertIsNone(note)
        self.assertFalse(ran)

    def test_empty_at_the_close_is_red(self):
        problems, _, ran = self.verdict("", run=True)
        self.assertEqual(len(problems), 1)
        self.assertFalse(ran)
        self.assertIn("mutation", problems[0].message)

    def test_empty_away_from_the_close_says_nothing(self):
        """The question belongs to the close. Asking it on every commit would
        be noise about something that cannot be answered yet."""
        problems, note, ran = self.verdict("", run=False)
        self.assertEqual(problems, [])
        self.assertIsNone(note)
        self.assertFalse(ran)

    def test_a_named_command_does_not_run_before_the_close(self):
        """Proved by side effect, not by the return value: a command that ran
        would have left the file behind."""
        witness = self.fixture.path("mutants-ran")
        problems, _, ran = self.verdict("touch %s" % witness, run=False)
        self.assertEqual(problems, [])
        self.assertFalse(ran)
        self.assertFalse(os.path.exists(witness))


class TestWhenTheWaveIsFinished(MutationCase):
    """`wave_is_finished` is the whole of "at the close", so it is what gets
    tested — the run itself is the same subprocess call CI already makes."""

    def test_a_wave_with_work_left_is_not_finished(self):
        self.fixture.branch("0001-session-loop")
        self.assertFalse(keel.wave_is_finished(self.project))

    def test_a_wave_whose_transforms_are_all_closed_is_finished(self):
        self.close_the_wave()
        self.assertTrue(keel.wave_is_finished(self.project))

    def test_a_plan_branch_is_never_finished(self):
        """It carries documents and no code; there is nothing to break."""
        self.fixture.branch("plan/0001-session-loop")
        self.assertFalse(keel.wave_is_finished(self.project))

    def test_main_is_never_finished(self):
        """No wave answers for main, so no wave closes there."""
        self.assertFalse(keel.wave_is_finished(self.project))

    def test_a_branch_named_after_no_wave_is_never_finished(self):
        self.fixture.branch("just-a-branch")
        self.assertFalse(keel.wave_is_finished(self.project))


class TestTheGateInCheck(MutationCase):
    """What `keel check` does with all of it."""

    def test_the_command_runs_at_the_close(self):
        """End to end, by side effect: the wave is closed, and the command the
        project named has left its mark."""
        witness = self.fixture.path("mutants-ran")
        self.close_the_wave()
        self.settings("touch %s" % witness)
        problems, _, ran = keel.mutation_verdict(
            self.project, run=keel.wave_is_finished(self.project))
        self.assertTrue(ran)
        self.assertEqual(problems, [])
        self.assertTrue(os.path.exists(witness))

    def run_check(self, **flags):
        """`check` itself, its output swallowed. Going through the real command
        is the point: a test that recomputes the condition proves the copy."""
        args = Args(fast=False, no_tests=False, json=False, branch=None)
        args.__dict__.update(flags)
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            keel.cmd_check(self.project, args)
        return buffer.getvalue()

    def test_check_runs_it_at_the_close(self):
        witness = self.fixture.path("mutants-ran")
        self.close_the_wave()
        self.settings("touch %s" % witness)
        self.run_check()
        self.assertTrue(os.path.exists(witness))

    def test_fast_and_no_tests_leave_it_alone(self):
        """Both flags are documented as skipping the run, and a flag that is
        documented and not tested is a promise nobody keeps."""
        witness = self.fixture.path("mutants-ran")
        self.close_the_wave()
        self.settings("touch %s" % witness)
        for flags in ({"fast": True}, {"no_tests": True}):
            self.run_check(**flags)
            self.assertFalse(os.path.exists(witness), flags)

    def test_nothing_runs_while_work_is_left(self):
        witness = self.fixture.path("mutants-ran")
        self.fixture.branch("0001-session-loop")
        self.settings("touch %s" % witness)
        problems, _, ran = keel.mutation_verdict(
            self.project, run=keel.wave_is_finished(self.project))
        self.assertFalse(ran)
        self.assertEqual(problems, [])
        self.assertFalse(os.path.exists(witness))


if __name__ == "__main__":
    unittest.main()

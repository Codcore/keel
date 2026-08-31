#!/usr/bin/env python3
"""The mutation reminder: said once, at the close of a wave, and never enforced."""

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
    """A project that can name a mutation command, and a wave that can be
    brought to its close."""

    def settings(self, command):
        path = self.fixture.path(keel.CONFIG_FILE)
        stored = json.loads(keel.read_text(path)) if os.path.exists(path) else {}
        stored["mutation"] = command
        self.fixture.write(keel.CONFIG_FILE, json.dumps(stored) + "\n")

    def close_the_wave(self):
        """Every transform of 0001-session-loop closed by a commit.

        The wave has one — `drive-turns` — and §8.4 says a commit names its
        transform by the slug at the start of its message. The plan is already
        on main from the fixture's first commit.
        """
        self.fixture.branch("0001-session-loop")
        self.fixture.write(
            "lib/session.ex",
            "defmodule Demo.Session do\n  def run(_, _, _), do: :ok\nend\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "drive-turns: ходи покручено")

    def run_check(self, **flags):
        """`check` itself, its output captured."""
        args = Args(fast=False, no_tests=True, json=False, branch=None)
        args.__dict__.update(flags)
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code = keel.cmd_check(self.project, args)
        return code, buffer.getvalue()


class TestWhatItSays(MutationCase):
    """Three states, and none of them is a verdict.

    A command named is a command offered. `none` is a decision said out loud,
    and it is silent. Nothing named is the reminder in full — the question
    §7.14 asks has an answer whether or not a tool exists to ask it.
    """

    def test_nothing_is_said_before_the_close(self):
        self.settings("mix muex")
        self.assertIsNone(keel.mutation_reminder(self.project, at_close=False))

    def test_none_is_silent_even_at_the_close(self):
        self.settings("none")
        self.assertIsNone(keel.mutation_reminder(self.project, at_close=True))

    def test_without_a_command_it_points_at_the_breaks(self):
        self.settings("")
        said = keel.mutation_reminder(self.project, at_close=True)
        self.assertIsNotNone(said)
        self.assertIn("7.14", said)

    def test_with_a_command_it_offers_both(self):
        self.settings("mix muex")
        said = keel.mutation_reminder(self.project, at_close=True)
        self.assertIn("7.14", said)
        self.assertIn("mix muex", said)


class TestItNeverRuns(MutationCase):
    """The whole difference from a gate. A mutation run exits zero in states
    that prove nothing, so its exit code is not evidence and Keel does not
    collect it."""

    def test_the_command_is_not_executed(self):
        witness = self.fixture.path("mutants-ran")
        self.settings("touch %s" % witness)
        keel.mutation_reminder(self.project, at_close=True)
        self.assertFalse(os.path.exists(witness))

    def test_check_does_not_execute_it_either(self):
        witness = self.fixture.path("mutants-ran")
        self.close_the_wave()
        self.settings("touch %s" % witness)
        self.run_check()
        self.assertFalse(os.path.exists(witness))

    def test_a_command_that_would_fail_adds_nothing_to_the_verdict(self):
        """`false` exits 1. Were this still a gate, it would add a problem.

        The fixture's own wave is red for reasons of its own, so the assertion
        is a comparison rather than a zero: the same check, once with the
        command refused and once with a command that fails, has to reach the
        same verdict and the same count.
        """
        self.close_the_wave()
        self.settings("none")
        silent_code, silent_out = self.run_check()
        self.settings("false")
        failing_code, failing_out = self.run_check()
        self.assertEqual(silent_code, failing_code)
        problems = lambda text: [l for l in text.splitlines() if l.startswith("\u2717")]
        self.assertEqual(problems(silent_out), problems(failing_out))


class TestWhenTheWaveIsFinished(MutationCase):
    """`wave_is_finished` is the whole of "at the close", so it is what gets
    tested."""

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


class TestWhereItAppears(MutationCase):
    """In the output of `check`, and only where it belongs."""

    def test_it_is_printed_at_the_close(self):
        self.close_the_wave()
        self.settings("mix muex")
        _, out = self.run_check()
        self.assertIn("7.14", out)

    def test_it_is_absent_while_work_is_left(self):
        self.fixture.branch("0001-session-loop")
        self.settings("mix muex")
        _, out = self.run_check()
        self.assertNotIn("7.14", out)

    def test_fast_leaves_it_out(self):
        """--fast keeps to what costs nothing, and reading git to find whether
        a wave is finished is not nothing."""
        self.close_the_wave()
        self.settings("mix muex")
        _, out = self.run_check(fast=True)
        self.assertNotIn("7.14", out)

    def test_the_json_payload_carries_it(self):
        self.close_the_wave()
        self.settings("mix muex")
        _, out = self.run_check(json=True)
        payload = json.loads(out)
        self.assertIn("7.14", payload["mutation"]["note"])

    def test_the_payload_has_no_verdict_to_give(self):
        """Nothing about mutations can make `ok` false any more."""
        self.close_the_wave()
        self.settings("false")
        _, out = self.run_check(json=True)
        self.assertNotIn("problems", json.loads(out)["mutation"])


if __name__ == "__main__":
    unittest.main()

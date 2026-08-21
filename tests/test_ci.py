#!/usr/bin/env python3
"""The project's own gate: a command the operator names, and three states."""

import json
import os
import re
import tempfile
import sys
import unittest
from io import StringIO

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args, ProjectCase  # noqa: E402


class TestCiCommand(ProjectCase):
    """Три стани, і середній — увесь сенс.

    Команда виконується. `none` — свідома відмова, і вона мовчить. Порожньо —
    ніхто нічого не вирішив, і про це кажуть щоразу: злиття, яке пройшло без
    жодної перевірки, і ніхто про це не знає, — та сама тиша, проти якої все
    це й будувалось.
    """

    def settings(self, command):
        path = self.fixture.path(keel.CONFIG_FILE)
        stored = json.loads(keel.read_text(path)) if os.path.exists(path) else {}
        stored["ci"] = command
        self.fixture.write(keel.CONFIG_FILE, json.dumps(stored) + "\n")

    def verdict(self, command, run=True):
        self.settings(command)
        return keel.ci_verdict(self.project, run=run)

    def test_a_command_that_passes_says_nothing(self):
        problems, note, ran = self.verdict("true")
        self.assertEqual(problems, [])
        self.assertIsNone(note)
        self.assertTrue(ran)

    def test_a_command_that_fails_is_red_with_its_own_output(self):
        problems, _, _ = self.verdict("echo щось-зламалось >&2; false")
        self.assertEqual(len(problems), 1)
        self.assertIn("щось-зламалось", problems[0].message)

    def test_a_command_that_does_not_exist_says_so(self):
        problems, _, _ = self.verdict("keel-no-such-command-anywhere")
        self.assertEqual(len(problems), 1)
        self.assertIn("no such command", problems[0].message)

    def test_nothing_decided_is_said_on_every_run(self):
        problems, note, _ = self.verdict("")
        self.assertEqual(problems, [])
        self.assertIsNotNone(note)
        self.assertIn("no CI command", note)

    def test_the_hint_offers_the_language_its_own_command(self):
        """Сказати Python-проєктові «mix ci» — маленька неправда там, де
        повідомлення тільки й існує, щоб за ним щось зробили."""
        _, note, _ = self.verdict("")
        self.assertIn("mix ci", note)

    def test_a_refusal_out_loud_is_silent(self):
        problems, note, _ = self.verdict("none")
        self.assertEqual(problems, [])
        self.assertIsNone(note)

    def test_the_note_is_not_red(self):
        """Проєкт без CI лишається зеленим — але тільки якщо це сказано вголос."""
        problems, _, _ = self.verdict("")
        self.assertEqual(problems, [])

    def test_it_does_not_run_when_asked_not_to(self):
        problems, note, ran = self.verdict("false", run=False)
        self.assertEqual(problems, [])
        self.assertIsNone(note)
        self.assertFalse(ran)


class TestCiInTheGate(ProjectCase):
    def capture(self, args):
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_check(self.project, args)
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def settings(self, command):
        path = self.fixture.path(keel.CONFIG_FILE)
        stored = json.loads(keel.read_text(path)) if os.path.exists(path) else {}
        stored["ci"] = command
        self.fixture.write(keel.CONFIG_FILE, json.dumps(stored) + "\n")

    def test_the_full_run_carries_it_and_fast_does_not(self):
        """Коміт може бути проміжним; пуш і мерж — ні."""
        self.settings("false")
        _, fast = self.capture(Args(fast=True, no_tests=True, json=False))
        self.assertNotIn("CI", fast)
        _, full = self.capture(Args(fast=False, no_tests=False, json=False))
        self.assertIn("CI", full)

    def test_a_red_gate_makes_the_whole_check_red(self):
        self.settings("false")
        code, out = self.capture(Args(fast=False, no_tests=False, json=False))
        self.assertEqual(code, 1)
        self.assertIn("CI", out)

    def test_the_undecided_note_shows_without_turning_it_red(self):
        self.settings("")
        _, out = self.capture(Args(fast=False, no_tests=True, json=False))
        self.assertIn("no CI command", out)

    def test_json_carries_the_command_and_the_verdict(self):
        self.settings("false")
        _, out = self.capture(Args(fast=False, no_tests=False, json=True))
        payload = json.loads(out)
        self.assertEqual(payload["ci"]["command"], "false")
        self.assertTrue(payload["ci"]["problems"])
        self.assertFalse(payload["ok"])


class TestAdaptersProposeOnlyWhereThereIsAConvention(unittest.TestCase):
    def test_elixir_proposes_and_the_base_does_not(self):
        self.assertEqual(keel.ElixirAdapter().ci_command, "mix ci")
        self.assertEqual(keel.Adapter().ci_command, "")

    def test_every_proposal_is_a_plain_command(self):
        for cls in keel.ADAPTERS:
            proposed = cls().ci_command
            self.assertIsInstance(proposed, str, cls.__name__)
            self.assertNotIn("\n", proposed, cls.__name__)


if __name__ == "__main__":
    unittest.main()


class TestTheTickMeansItRan(ProjectCase):
    """Галочку друкували з «немає проблем», а пропущений заслон їх теж не має.

    `check --no-tests` рапортував зелений CI над командою, якої не запускав.
    """

    def capture(self, args):
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_check(self.project, args)
        finally:
            sys.stdout = saved
        return stream.getvalue()

    def settings(self, command):
        path = self.fixture.path(keel.CONFIG_FILE)
        stored = json.loads(keel.read_text(path)) if os.path.exists(path) else {}
        stored["ci"] = command
        self.fixture.write(keel.CONFIG_FILE, json.dumps(stored) + "\n")

    def test_no_tests_does_not_claim_a_gate_that_never_ran(self):
        self.settings("false")
        out = self.capture(Args(fast=False, no_tests=True, json=False))
        self.assertNotIn("✓ CI", out)

    def test_the_tick_appears_only_after_a_real_run(self):
        self.settings("true")
        out = self.capture(Args(fast=False, no_tests=False, json=False))
        self.assertIn("✓ CI: true", out)

    def test_json_says_whether_it_ran(self):
        self.settings("true")
        skipped = json.loads(self.capture(Args(fast=False, no_tests=True, json=True)))
        self.assertFalse(skipped["ci"]["ran"])
        done = json.loads(self.capture(Args(fast=False, no_tests=False, json=True)))
        self.assertTrue(done["ci"]["ran"])


class TestDriftReachesTheMachineReadableOutput(ProjectCase):
    """Сенс називати дрейф у тому, щоб про нього дізнались, а скрипти читають
    payload, а не прозу під ним."""

    def capture(self, args):
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_check(self.project, args)
        finally:
            sys.stdout = saved
        return stream.getvalue()

    def test_json_carries_the_drift(self):
        self.fixture.branch("0001-session-loop")
        wave = "keel/waves/0001-session-loop.md"
        self.fixture.write(wave, self.fixture.read(wave) + "\nПравка тут.\n")
        payload = json.loads(self.capture(Args(fast=True, no_tests=True, json=True)))
        self.assertEqual([row["file"] for row in payload["drift"]], [wave])

    def test_json_drift_is_empty_when_nothing_moved(self):
        self.fixture.branch("0001-session-loop")
        payload = json.loads(self.capture(Args(fast=True, no_tests=True, json=True)))
        self.assertEqual(payload["drift"], [])


class TestTheGeneratedWorkflowSpeaksGitHub(unittest.TestCase):
    """Ключі породженого workflow належать GitHub, а не методиці.

    `steps:` свого часу став `waves:` — глобальне перейменування кроків на
    хвилі зачепило чужий словник. Actions відхиляє такий файл цілком, тобто
    кожен проєкт після `keel init` діставав непрацездатний заслон. Тест на
    шаблон при цьому був: він питав, чи текст англійський, і мовчав про будову.
    """

    # Те, що GitHub розуміє всередині job. Список навмисно повний, а не
    # «steps є»: наступне перейменування зайде іншим словом, і перевірка на
    # одне імʼя його пропустить.
    JOB_KEYS = {
        "runs-on", "steps", "needs", "if", "env", "strategy", "container",
        "services", "outputs", "permissions", "timeout-minutes",
        "continue-on-error", "defaults", "concurrency", "environment",
    }

    def workflow(self, setup=""):
        return keel.CI_TEMPLATE.format(tool=keel.VENDORED, setup=setup)

    def job_keys(self, text):
        job = text.split("\n  check:\n", 1)[1]
        return {line.split(":")[0].strip() for line in job.splitlines()
                if re.match(r"^ {4}\S", line)}

    def test_the_job_carries_steps(self):
        self.assertIn("\n    steps:\n", self.workflow())

    def test_no_key_of_the_job_is_a_word_of_the_method(self):
        self.assertLessEqual(self.job_keys(self.workflow()), self.JOB_KEYS)

    def test_it_holds_with_every_language_block_too(self):
        """Блок мови вставляється всередину job; ключів job він додавати не сміє."""
        with tempfile.TemporaryDirectory() as root:
            for adapter in (keel.ElixirAdapter(), keel.PythonAdapter()):
                setup = "".join(line + "\n" for line in adapter.ci_steps(root))
                self.assertLessEqual(self.job_keys(self.workflow(setup)),
                                     self.JOB_KEYS, adapter.name)


class TestTheProjectGateStandsDownOnAPlanBranch(ProjectCase):
    """Друга половина `PLAN_BLIND`, знайдена живою на хвилі 0010.

    Перевірки 5 і 6 на гілці плану не працюють: гілка везе документи й не
    везе коду. Команда проєкту ганяє тести того самого коду — і лишалась
    стояти. Тобто гейт казав «тут немає чого судити» й тут-таки судив.

    Ціна була не теоретична: один тест, зіпсований попередньою хвилею,
    блокував пуш **будь-якої** гілки плану — зокрема й тієї малої хвилі,
    що писалась цей тест полагодити. Коло без виходу.
    """

    def capture(self, args):
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_check(self.project, args)
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def settings(self, command):
        path = self.fixture.path(keel.CONFIG_FILE)
        stored = json.loads(keel.read_text(path)) if os.path.exists(path) else {}
        stored["ci"] = command
        self.fixture.write(keel.CONFIG_FILE, json.dumps(stored) + "\n")

    def plan_branch(self):
        self.fixture.branch("plan/0001-session-loop")

    def test_a_red_command_does_not_turn_a_plan_branch_red(self):
        self.settings("false")
        self.plan_branch()
        code, out = self.capture(Args(fast=False, no_tests=False, json=False))
        self.assertNotIn("CI червоний", out)
        self.assertNotIn("CI is red", out)
        self.assertEqual(code, 0, out)

    def test_the_same_command_still_turns_a_work_branch_red(self):
        """Сліпота живе на гілці плану й ніде більше."""
        self.settings("false")
        self.fixture.branch("0001-session-loop")
        code, out = self.capture(Args(fast=False, no_tests=False, json=False))
        self.assertNotEqual(code, 0, out)

    def test_standing_down_is_said_out_loud_and_not_by_silence(self):
        """Мовчання читалось би як «команди CI немає» — а це інший факт."""
        self.settings("false")
        self.plan_branch()
        _, out = self.capture(Args(fast=False, no_tests=False, json=False))
        self.assertIn("false", out)
        self.assertIn("a plan branch has no code", out)

    def test_a_project_without_a_ci_command_says_nothing_extra(self):
        self.settings("none")
        self.plan_branch()
        _, out = self.capture(Args(fast=False, no_tests=False, json=False))
        # Той самий рядок друкують і перевірки 5 і 6, тож шукати його в усьому
        # виводі — шукати не те. Питання тут одне: чи є рядок про CI взагалі.
        self.assertEqual(
            [line for line in out.splitlines() if line.startswith("– CI")], [])

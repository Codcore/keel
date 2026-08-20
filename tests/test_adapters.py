#!/usr/bin/env python3
"""Checks 5 and 6: tests and exports, through the language adapters."""

import json
import os
import shutil
import subprocess
import tempfile
import sys
import unittest
import unittest.mock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402


class TestCiStepsWaitForTheirMarker(unittest.TestCase):
    """Мову можна назвати раніше, ніж зʼявиться її маркер.

    Тоді крок установки біжить на гілці, де маркера немає, — а це кожна гілка
    плану за задумом, — і CI червоний з причини, що не стосується гілки.
    """

    def test_every_adapter_guards_its_install_on_the_marker(self):
        for cls in keel.ADAPTERS:
            adapter = cls()
            steps = adapter.ci_steps(".")
            if not steps:
                continue
            guard = adapter.ci_guard()
            for name in adapter.marker:
                self.assertIn(f"'{name}'", guard, cls.__name__)
            installs = [line for line in steps
                        if line.strip().startswith(("- uses:", "- run:"))]
            self.assertTrue(installs, cls.__name__)
            for line in installs:
                position = steps.index(line)
                self.assertEqual(steps[position + 1], guard,
                                 f"{cls.__name__}: {line.strip()} без умови")

    def test_the_condition_is_a_yaml_key_at_the_step_indent(self):
        guard = keel.ElixirAdapter().ci_guard()
        self.assertTrue(guard.startswith("        if: "), guard)
        self.assertNotIn("\n", guard)
from tests.support import ProjectCase  # noqa: E402




# ─────────────────────────────────────────────────────────────────────────────
# Check 5: scenarios and tags
# ─────────────────────────────────────────────────────────────────────────────

class TestScenarios(ProjectCase):
    def tag(self, rev, slug="finishes_when_no_tool_called"):
        self.fixture.write(
            "test/session_test.exs",
            f'defmodule Demo.SessionTest do\n'
            f'  @tag proves: :{slug}, rev: "{rev}"\n'
            f'  test "розмова завершується" do\n'
            f'    assert true\n'
            f'  end\n'
            f'end\n')

    def test_adapter_is_elixir(self):
        self.assertEqual(self.project.adapter.name, "elixir")

    def test_missing_test_is_a_problem(self):
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertEqual(len(problems), 1)
        self.assertIn("has no test", problems[0].message)

    def test_tag_with_right_revision_is_clean(self):
        self.tag(self.fixture.scenario_rev())
        self.assertEqual(keel.check_scenarios(self.project, run_tests=False), [])

    def test_tag_with_stale_revision(self):
        self.tag("deadbe")
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertEqual(len(problems), 1)
        self.assertIn("the test holds", problems[0].message)
        self.assertEqual(problems[0].where, "test/session_test.exs")

    def test_tag_without_revision(self):
        self.fixture.write(
            "test/session_test.exs",
            'defmodule Demo.SessionTest do\n'
            '  @tag proves: :finishes_when_no_tool_called\n'
            '  test "x", do: assert true\n'
            'end\n')
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertIn("carries no revision", problems[0].message)

    def test_scenario_text_edited_makes_the_tag_stale(self):
        self.tag(self.fixture.scenario_rev())
        text = self.fixture.read("keel/waves/0001-session-loop.md")
        self.fixture.write("keel/waves/0001-session-loop.md",
                           text.replace("розмова завершується", "розмова завершується сама"))
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertEqual(len(problems), 1)
        self.assertIn("the test holds", problems[0].message)

    def test_umbrella_tests_are_collected(self):
        """apps/*/test — mix test їх ганяє, а збирач не бачив: вічне хибне
        «has no test» над зеленим тегованим тестом."""
        self.fixture.write(
            "apps/demo/test/demo_test.exs",
            f'defmodule DemoTest do\n'
            f'  @tag proves: :finishes_when_no_tool_called, '
            f'rev: "{self.fixture.scenario_rev()}"\n'
            f'  test "x", do: assert true\nend\n')
        self.assertEqual(keel.check_scenarios(self.project, run_tests=False), [])

    def test_the_top_level_is_looked_up_once_per_invocation(self):
        """check_scope питає на кожен файл — це був git-процес на файл."""
        calls = []
        original = keel.Git.run

        def counted(inner, *args):
            if "--show-toplevel" in args:
                calls.append(args)
            return original(inner, *args)

        git = keel.Git(self.fixture.root)
        with unittest.mock.patch.object(keel.Git, "run", counted):
            for name in ("a.ex", "b.ex", "c.ex", "d.ex"):
                git.relative_to_root(name, self.fixture.root)
        self.assertEqual(len(calls), 1, calls)

    def test_two_waves_sharing_a_scenario_slug_are_named(self):
        """Тег несе лише слаг — за двох власників він принципово неоднозначний."""
        self.fixture.write(
            "keel/waves/0002-other.md",
            "---\nscenarios:\n  finishes-when-no-tool-called: {}\n---\n\n"
            "## scenario: finishes-when-no-tool-called\n\n**Given** інше.\n")
        self.tag(self.fixture.scenario_rev())
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertTrue(any("more than one wave" in x.message for x in problems),
                        [x.message for x in problems])
        # і жодного хибного присвоєння тега тому чи тому кроку
        self.assertFalse(any("has no test" in x.message for x in problems),
                         [x.message for x in problems])
        self.assertFalse(any("the test holds" in x.message for x in problems),
                         [x.message for x in problems])

    def test_the_collision_is_named_without_a_language_adapter(self):
        """Це факт про документи, а не про мову: без маркера він мовчав."""
        import tempfile as tf, shutil as sh
        root = tf.mkdtemp(prefix="keel-slug-")
        self.addCleanup(sh.rmtree, root, True)
        os.makedirs(os.path.join(root, "keel", "waves"))
        os.makedirs(os.path.join(root, "keel", "contracts"))
        for name in ("0001-a", "0002-b"):
            with open(os.path.join(root, "keel/waves", name + ".md"), "w",
                      encoding="utf-8") as handle:
                handle.write("---\nscenarios:\n  same-slug: {}\n---\n\n"
                             "## scenario: same-slug\n\n**Given** х.\n")
        problems = keel.check_scenarios(keel.Project(root), run_tests=False)
        self.assertIsNone(keel.Project(root).adapter)
        self.assertTrue(any("more than one wave" in x.message for x in problems),
                        [x.message for x in problems])

    def test_slug_dashes_match_atom_underscores(self):
        self.assertEqual(keel.normalise_slug("finishes_when_no_tool_called"),
                         keel.normalise_slug("finishes-when-no-tool-called"))


# ─────────────────────────────────────────────────────────────────────────────
# Check 6: exports (the Python adapter — it runs without mix)
# ─────────────────────────────────────────────────────────────────────────────

class TestExports(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-py-")
        self.addCleanup(shutil.rmtree, self.root, True)
        os.makedirs(os.path.join(self.root, "keel/contracts"))
        os.makedirs(os.path.join(self.root, "keel/waves"))
        self.write("pyproject.toml", "[project]\nname = 'demo'\n")
        self.write("demo.py", "def run(a, b, c):\n    return a\n")

    def write(self, name, text):
        with open(os.path.join(self.root, name), "w", encoding="utf-8") as handle:
            handle.write(text)

    def contract(self, exports):
        self.write("keel/contracts/demo.md",
                   f"---\nmodule: demo\nexports: {exports}\n---\n\nЩо обіцяє demo.\n")

    def test_adapter_is_python(self):
        self.contract("[run/3]")
        self.assertEqual(keel.Project(self.root).adapter.name, "python")

    def test_promised_export_exists(self):
        self.contract("[run/3]")
        self.assertEqual(keel.check_exports(keel.Project(self.root)), [])

    def test_promised_export_missing(self):
        self.contract("[halt/1]")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("does not export what was promised: halt/1", problems[0].message)

    def test_wrong_arity_is_missing(self):
        self.contract("[run/2]")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertIn("run/2", problems[0].message)

    def test_a_promise_that_is_not_a_module_is_proved_by_a_command(self):
        """Ollama на порту, бінарник на PATH, ліба потрібної версії."""
        self.write("keel/contracts/runtime.md",
                   "---\nverify: \"true\"\n---\n\nЩось, що має працювати.\n")
        self.assertEqual(keel.check_exports(keel.Project(self.root)), [])

    def test_a_command_that_fails_is_a_broken_contract(self):
        self.write("keel/contracts/runtime.md",
                   "---\nverify: \"echo нема така служба >&2; exit 1\"\n---\n\nСлужба.\n")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("was not confirmed", problems[0].message)
        self.assertIn("нема така служба", problems[0].message)

    def test_a_contract_may_carry_both_a_module_and_a_command(self):
        self.contract("[run/3]")
        self.write("keel/contracts/runtime.md",
                   "---\nverify: \"false\"\n---\n\nСлужба.\n")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("runtime.md", problems[0].where)

    def test_a_command_that_hangs_is_cut_off_and_reported(self):
        """Без межі зависла команда тримала б pre-push і CI скільки завгодно."""
        self.write("keel/contracts/slow.md",
                   '---\nverify: "sleep 30"\n---\n\nПовільна.\n')
        with unittest.mock.patch.object(keel, "VERIFY_TIMEOUT", 1):
            problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("did not answer within 1s", problems[0].message)

    def test_a_verify_that_is_not_a_command_is_named_not_skipped(self):
        """Список замість рядка не має давати зеленої перевірки."""
        self.write("keel/contracts/listy.md",
                   '---\nverify: ["curl", "-sf", "x"]\n---\n\nСписком.\n')
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("has to be a command as a string", problems[0].message)
        self.assertIn("list", problems[0].message)

    def test_an_empty_verify_is_named_too(self):
        self.write("keel/contracts/empty.md",
                   '---\nverify: ""\n---\n\nПорожня.\n')
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("has to be a command as a string", problems[0].message)

    def test_a_contract_that_promises_nothing_checkable_is_named(self):
        """Було «лишаємо в спокої» — і воно збирало зелену шосту ні над чим.

        `METHODOLOGY.md` §2.10: обіцянка, якої ніщо не перевіряє, — не контракт, а межа,
        і живе вона абзацом у трансформі."""
        self.write("keel/contracts/prose.md", "---\nmodule: demo\n---\n\nСама проза.\n")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertTrue(any("prose" in p.message for p in problems),
                        [p.message for p in problems])

    def test_no_tests_does_not_run_verify(self):
        """Прапорець обіцяє нічого не запускати — команда контракту теж запуск."""
        self.write("keel/contracts/boom.md",
                   '---\nverify: "exit 1"\n---\n\nВпаде.\n')
        project = keel.Project(self.root)
        self.assertTrue(keel.check_exports(project))
        self.assertEqual(keel.check_exports(project, run_tests=False), [])

    def test_verify_does_not_inherit_stdin(self):
        """Команда з підказкою мусить упасти одразу, а не з'їсти таймаут."""
        self.write("keel/contracts/asks.md",
                   '---\nverify: "read x"\n---\n\nПитає.\n')
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("was not confirmed", problems[0].message)

    def test_exports_without_a_module_are_named_not_dropped(self):
        """Обіцянка конкретна й перевірна — а не звірялась ні з чим."""
        self.write("keel/contracts/orphan.md",
                   "---\nexports: [run/2, launch/1]\n---\n\nх.\n")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("names no module", problems[0].message)

    def test_no_tests_never_loads_the_projects_code(self):
        """Прапорець обіцяє нічого не запускати — імпорт модуля теж запуск."""
        sentinel = os.path.join(self.root, "ran")
        self.write("demo.py",
                   f"open({sentinel!r}, 'w').write('x')\n"
                   "def run(a, b, c):\n    return a\n")
        self.contract("[run/3]")
        project = keel.Project(self.root)
        self.assertEqual(keel.check_exports(project, run_tests=False), [])
        self.assertFalse(os.path.exists(sentinel), "пробник виконав код проєкту")
        keel.check_exports(project)
        self.assertTrue(os.path.exists(sentinel), "а з тестами мав виконати")

    def test_module_absent(self):
        self.write("keel/contracts/demo.md",
                   "---\nmodule: nosuchmodule\nexports: [run/1]\n---\n\nТекст.\n")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertIn("did not build", problems[0].message)


# ─────────────────────────────────────────────────────────────────────────────
# Which language, when the root says more than one
# ─────────────────────────────────────────────────────────────────────────────

class TestAdapterChoice(unittest.TestCase):
    """A polyglot root is a question, not a silent first-in-the-list."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-poly-")
        self.addCleanup(shutil.rmtree, self.root, True)
        for folder in ("keel/waves", "keel/contracts"):
            os.makedirs(os.path.join(self.root, folder))
        with open(os.path.join(self.root, "keel/waves/0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\nscenarios:\n  does-a: {}\n---\n\n## Why\n\nх.\n\n"
                         "## scenario: does-a\n\n**Given** щось.\n")

    def mark(self, *names):
        for name in names:
            with open(os.path.join(self.root, name), "w", encoding="utf-8") as handle:
                handle.write("x\n")

    def settle(self, name):
        with open(os.path.join(self.root, keel.CONFIG_FILE), "w",
                  encoding="utf-8") as handle:
            json.dump({"adapter": name}, handle)

    def test_one_marker_needs_no_saying(self):
        self.mark("mix.exs")
        project = keel.Project(self.root)
        self.assertEqual(project.adapter.name, "elixir")
        self.assertEqual(keel.adapter_problem(project, 5), [])

    def test_check_six_stays_quiet_when_it_needs_no_adapter(self):
        """Без контрактів шоста мови не питає — і не має про неї говорити."""
        self.mark("mix.exs", "pyproject.toml")
        problems = keel.check_exports(keel.Project(self.root), run_tests=False)
        self.assertEqual(problems, [])

    def test_two_markers_are_named_not_guessed(self):
        self.mark("mix.exs", "pyproject.toml")
        problems = keel.adapter_problem(keel.Project(self.root), 5)
        self.assertEqual(len(problems), 1)
        self.assertIn("elixir, python", problems[0].message)
        self.assertIn("adapter", problems[0].message)

    def test_the_ambiguity_reaches_both_checks_that_depend_on_it(self):
        self.mark("mix.exs", "pyproject.toml")
        with open(os.path.join(self.root, "keel/contracts/a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\nmodule: Demo\nexports: [run/1]\n---\n\nх.\n")
        project = keel.Project(self.root)
        for check, run in ((5, keel.check_scenarios), (6, keel.check_exports)):
            problems = run(project, run_tests=False)
            self.assertTrue(any("matches 2 languages" in p.message for p in problems),
                            f"перевірка {check} мовчить")

    def test_saying_which_settles_it(self):
        self.mark("mix.exs", "pyproject.toml")
        self.settle("python")
        project = keel.Project(self.root)
        self.assertEqual(project.adapter.name, "python")
        self.assertEqual(keel.adapter_problem(project, 5), [])

    def test_a_name_we_do_not_know_falls_back_to_the_markers(self):
        """Сміття в налаштуваннях не має лишати проєкт зовсім без адаптера."""
        self.mark("mix.exs")
        self.settle("cobol")
        self.assertEqual(keel.Project(self.root).adapter.name, "elixir")

    def test_the_ci_file_follows_the_settled_choice(self):
        """Інакше проєкт домовився про мову, а CI ставив би іншу."""
        self.mark("mix.exs", "pyproject.toml")
        self.settle("python")
        wanted = keel.generated_files(self.root, keel.read_config(self.root))
        self.assertIn("setup-python", wanted[keel.CI_FILE])
        self.assertNotIn("setup-beam", wanted[keel.CI_FILE])

    def test_init_writes_the_ci_of_the_flag_not_of_the_first_marker(self):
        """--adapter python у поліглотному корені: CI теж python."""
        import subprocess as sp
        from io import StringIO
        self.mark("mix.exs", "pyproject.toml")
        sp.run(["git", "init", "-b", "main", "-q", self.root], check=True)
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_init(keel.Project(self.root), type("Args", (), {
                "install": True, "force": False, "no_commit": True,
                "adapter": "python", "docs": None, "lang": None, "mode": None,
                "agent_hooks": None})())
        finally:
            sys.stdout = saved
        with open(os.path.join(self.root, keel.CI_FILE), encoding="utf-8") as handle:
            ci = handle.read()
        self.assertIn("setup-python", ci)
        self.assertNotIn("setup-beam", ci)

    def test_no_marker_at_all_is_still_no_adapter(self):
        self.assertIsNone(keel.Project(self.root).adapter)


# ─────────────────────────────────────────────────────────────────────────────
# Nothing that runs the project's own code may run without a bound
# ─────────────────────────────────────────────────────────────────────────────

class SleepAdapter(keel.Adapter):
    """An adapter whose test command never finishes."""

    name = "sleepy"

    def test_command(self, root):
        return ["sleep", "30"]

    def test_files(self, root):
        return []

    def exports(self, root, modules):
        return {}


class TestNothingHangsForever(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-hang-")
        self.addCleanup(shutil.rmtree, self.root, True)
        os.makedirs(os.path.join(self.root, "keel/waves"))
        os.makedirs(os.path.join(self.root, "keel/contracts"))
        with open(os.path.join(self.root, "keel/waves/0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\nscenarios:\n  does-a: {}\n---\n\n"
                         "## scenario: does-a\n\n**Given** щось.\n")

    def test_a_test_run_that_hangs_is_cut_off_and_named(self):
        """Без межі зависла збірка тримала б pre-push і CI скільки завгодно."""
        project = keel.Project(self.root)
        project.adapter = SleepAdapter()
        with unittest.mock.patch.object(keel, "TEST_TIMEOUT", 1):
            problems = keel.check_scenarios(project)
        self.assertTrue(any("did not finish within 1s" in p.message for p in problems),
                        [p.message for p in problems])

    def test_a_probe_that_hangs_comes_back_as_an_error(self):
        with unittest.mock.patch.object(keel, "PROBE_TIMEOUT", 1):
            probe = keel.run_probe(["sleep", "30"], self.root)
        self.assertNotEqual(probe.returncode, 0)
        self.assertIn("did not answer within 1s", probe.stderr)

    def test_a_probe_never_inherits_stdin(self):
        """Команда з підказкою має впасти одразу, а не зʼїсти таймаут."""
        probe = keel.run_probe(
            [sys.executable, "-c", "input()"], self.root)
        self.assertNotEqual(probe.returncode, 0)

    def test_a_missing_interpreter_is_named_not_a_traceback(self):
        probe = keel.run_probe(["немає-такої-команди"], self.root)
        self.assertNotEqual(probe.returncode, 0)
        self.assertIn("could not run", probe.stderr)

    def test_a_non_executable_runner_is_named_too(self):
        """Битий shim на PATH — PermissionError, і теж не трейсбек."""
        shim = os.path.join(self.root, "shim")
        with open(shim, "w", encoding="utf-8") as handle:
            handle.write("не виконуваний\n")
        probe = keel.run_probe([shim], self.root)
        self.assertNotEqual(probe.returncode, 0)
        self.assertIn("could not run", probe.stderr)

    def test_a_missing_test_runner_is_a_check_problem_not_a_crash(self):
        with open(os.path.join(self.root, "keel/waves/0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\nscenarios:\n  does-a: {}\n---\n\n"
                         "## scenario: does-a\n\n**Given** щось.\n")
        project = keel.Project(self.root)

        class GoneAdapter(keel.Adapter):
            name = "gone"
            def test_command(self, root):
                return ["немає-такого-запускача"]
            def test_files(self, root):
                return []
            def tags(self, root):
                return {"does-a": [("t.py", 1, "abcd")]}
        project.adapter = GoneAdapter()
        problems = keel.check_scenarios(project)
        self.assertTrue(any("could not run" in x.message for x in problems),
                        [x.message for x in problems])

    def test_a_timed_out_probe_reads_as_a_failed_build(self):
        """parse_export_output має розібрати відповідь заглушки як помилку."""
        with unittest.mock.patch.object(keel, "PROBE_TIMEOUT", 1):
            probe = keel.run_probe(["sleep", "30"], self.root)
        parsed = keel.parse_export_output(probe, ["Demo"])
        self.assertIn("__error__", parsed)
        self.assertIsNone(parsed["Demo"])


# ─────────────────────────────────────────────────────────────────────────────
# Check 6: exports written as whole signatures
# ─────────────────────────────────────────────────────────────────────────────

class TestModuleNameIsGuarded(unittest.TestCase):
    """The name is interpolated into a probe script, so it must not carry code."""

    def test_a_plain_module_reference_is_accepted(self):
        self.assertTrue(keel.MODULE_NAME.match("KeelAgent.Session"))
        self.assertTrue(keel.MODULE_NAME.match("Demo"))

    def test_a_quote_or_interpolation_is_rejected(self):
        for name in ('Foo"bar', "Foo#{:os.cmd(~c'id')}", "Foo bar", "Foo\n"):
            self.assertIsNone(keel.MODULE_NAME.match(name), name)

    def test_the_adapter_drops_an_unsafe_name_rather_than_run_it(self):
        out = keel.ElixirAdapter().exports(".", ['Foo"; System.halt(0)'])
        self.assertIsNone(out.get('Foo"; System.halt(0)'))


class TestPromisedSignature(unittest.TestCase):
    """`run/3` and `run(a, b, c) :: t` name the same function; both are read."""

    def test_name_and_arity(self):
        self.assertEqual(keel.promised_signature("run/3"), ("run", 3))

    def test_a_whole_spec_counts_its_arguments(self):
        self.assertEqual(
            keel.promised_signature("run(binary(), keyword()) :: {:ok, term()}"),
            ("run", 2))

    def test_no_arguments_is_arity_zero(self):
        self.assertEqual(keel.promised_signature("start() :: :ok"), ("start", 0))

    def test_arrows_written_tight_are_not_a_difference(self):
        """Компілятор ставить пробіли довкола => і ->, людина — не завжди."""
        self.assertEqual(keel.flatten_spec("run(%{atom()=>integer()}) :: :ok"),
                         keel.flatten_spec("run(%{atom() => integer()}) :: :ok"))
        self.assertEqual(keel.flatten_spec("each((integer()->:ok)) :: :ok"),
                         keel.flatten_spec("each((integer() -> :ok)) :: :ok"))

    def test_commas_inside_a_type_do_not_add_arguments(self):
        """{:ok, term()} — один аргумент, а не два."""
        self.assertEqual(
            keel.promised_signature("halt({:ok, term()}, [a: b]) :: :ok"),
            ("halt", 2))

    def test_a_question_mark_belongs_to_the_name(self):
        self.assertEqual(keel.promised_signature("valid?(t()) :: boolean()"),
                         ("valid?", 1))

    def test_a_comma_inside_a_bitstring_is_not_a_separator(self):
        """<<_::binary, _::8>> — один аргумент, а не два."""
        self.assertEqual(
            keel.promised_signature("parse(<<_::binary, _::8>>) :: :ok"),
            ("parse", 1))
        self.assertEqual(
            keel.promised_signature("mix(<<_::8>>, atom()) :: :ok"), ("mix", 2))

    def test_a_parenless_zero_arity_spec_is_legal_elixir(self):
        """`@spec run :: :ok` пишуть без дужок; компілятор рендерить із ними."""
        self.assertEqual(keel.promised_signature("run :: :ok"), ("run", 0))
        self.assertEqual(keel.promised_signature("ready? :: boolean()"),
                         ("ready?", 0))
        self.assertEqual(keel.flatten_spec("run :: :ok"),
                         keel.flatten_spec("run() :: :ok"))

    def test_nonsense_is_nonsense(self):
        for entry in ("сміття", "run/x", "run/", ":: t()", "Мод.run :: t()",
                      "run(a)(b) :: t()", "run(a :: t()", "run/²"):
            self.assertIsNone(keel.promised_signature(entry), entry)

    def test_the_empty_argument_list_is_counted_by_the_helper_itself(self):
        """Нуль аргументів має бути в самій функції, а не на місці виклику."""
        self.assertEqual(keel.count_arguments(""), 0)
        self.assertEqual(keel.count_arguments("   "), 0)
        self.assertEqual(keel.count_arguments("a, b"), 2)


class TestFlattenSpec(unittest.TestCase):
    """The compiler and the person write the same spec differently."""

    def test_the_two_renderings_meet(self):
        compiler = "run( binary(), keyword() ) :: {:ok, term()} | {:error, term()}"
        person = "run(binary(), keyword()) :: {:ok, term()} | {:error, term()}"
        self.assertEqual(keel.flatten_spec(compiler), keel.flatten_spec(person))

    def test_line_breaks_are_not_a_difference(self):
        self.assertEqual(keel.flatten_spec("run(a) ::\n  {:ok, b}"),
                         "run(a) :: {:ok, b}")

    def test_spacing_around_a_named_argument_is_not_a_difference(self):
        """Компілятор пише `text :: binary()`, людина пише `text::binary()`."""
        self.assertEqual(keel.flatten_spec("run(text::binary()) :: :ok"),
                         keel.flatten_spec("run(text :: binary()) :: :ok"))

    def test_a_real_difference_survives(self):
        self.assertNotEqual(keel.flatten_spec("run(binary()) :: :ok"),
                            keel.flatten_spec("run(integer()) :: :ok"))


class StubAdapter(keel.Adapter):
    """An adapter that answers from a dictionary — no compiler, no waiting."""

    name = "stub"
    supports_specs = True

    def __init__(self, answer):
        self.answer = answer

    def exports(self, root, modules):
        return self.answer


class TestSpecContracts(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-spec-")
        self.addCleanup(shutil.rmtree, self.root, True)
        os.makedirs(os.path.join(self.root, "keel/contracts"))
        os.makedirs(os.path.join(self.root, "keel/waves"))

    def contract(self, exports):
        path = os.path.join(self.root, "keel/contracts/demo.md")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(f"---\nmodule: Demo\nexports:\n  - \"{exports}\"\n"
                         "---\n\nЩо обіцяє Demo.\n")

    def check(self, exports, answer):
        self.contract(exports)
        project = keel.Project(self.root)
        project.adapter = StubAdapter(answer)
        return keel.check_exports(project)

    def declared(self, *specs):
        """The module exports exactly what the specs describe — no hand-kept pair."""
        named = ["{0}/{1}".format(*keel.promised_signature(one)) for one in specs]
        self.assertEqual(len(set(named)), 1, "клаузи мають бути про одну функцію")
        return {"Demo": set(named), ("specs", "Demo"): {named[0]: list(specs)}}

    def test_a_matching_spec_passes(self):
        self.assertEqual(
            self.check("run(binary(), keyword()) :: {:ok, term()}",
                       self.declared("run(binary(), keyword()) :: {:ok, term()}")),
            [])

    def test_the_compilers_spacing_is_not_a_difference(self):
        self.assertEqual(
            self.check("run(binary(), keyword()) :: {:ok, term()}",
                       self.declared("run( binary(), keyword() ) :: {:ok, term()}")),
            [])

    def test_a_different_type_is_reported_with_what_the_module_declares(self):
        problems = self.check("run(binary(), keyword()) :: :ok",
                              self.declared("run(binary(), keyword()) :: {:ok, term()}"))
        self.assertEqual(len(problems), 1)
        self.assertIn("is not what the module declares", problems[0].message)
        self.assertIn("{:ok, term()}", problems[0].message)

    def test_the_reported_spec_is_written_the_way_a_person_would(self):
        """Повідомлення існує, щоб його скопіювати в контракт."""
        problems = self.check("run(binary(), keyword()) :: :ok",
                              self.declared("run( binary(), keyword() ) :: :error"))
        self.assertIn("run(binary(), keyword()) :: :error", problems[0].message)

    def test_a_function_may_declare_more_than_one_spec(self):
        """Кілька @spec на одну функцію — кожна з них чесно обіцяна."""
        two = self.declared("run(integer()) :: :small", "run(binary()) :: :big")
        self.assertEqual(self.check("run(binary()) :: :big", two), [])
        self.assertEqual(self.check("run(integer()) :: :small", two), [])

    def test_a_mismatch_shows_every_clause(self):
        problems = self.check("run(atom()) :: :other",
                              self.declared("run(integer()) :: :small",
                                            "run(binary()) :: :big"))
        self.assertEqual(len(problems), 1)
        self.assertIn(":small", problems[0].message)
        self.assertIn(":big", problems[0].message)

    def test_named_arguments_are_read_not_rejected(self):
        """Компілятор сам віддає імена аргументів — їх і копіюють у контракт."""
        spec = "run(text :: binary(), opts :: keyword()) :: {:ok, term()}"
        self.assertEqual(self.check(spec, self.declared(spec)), [])

    def test_spacing_around_commas_and_bars_is_not_a_difference(self):
        self.assertEqual(
            self.check("run(binary()) :: {:ok,term()}|{:error,term()}",
                       self.declared("run(binary()) :: {:ok, term()} | "
                                     "{:error, term()}")),
            [])

    def test_a_module_without_the_spec_is_named_not_passed(self):
        """Функція є, @spec немає — обіцянка є, підтвердження немає."""
        problems = self.check("run(binary(), keyword()) :: :ok",
                              {"Demo": {"run/2"}})
        self.assertEqual(len(problems), 1)
        self.assertIn("declares no @spec", problems[0].message)

    def test_a_missing_function_is_reported_before_its_shape(self):
        problems = self.check("halt(binary()) :: :ok", {"Demo": {"run/2"}})
        self.assertEqual(len(problems), 1)
        self.assertIn("does not export what was promised: halt/1",
                      problems[0].message)

    def test_the_short_form_still_ignores_specs(self):
        """run/2 обіцяє імʼя й арність — специфікацію ніхто не обіцяв."""
        self.assertEqual(
            self.check("run/1", self.declared("run(integer()) :: :error")), [])

    def test_an_entry_that_is_neither_is_named(self):
        problems = self.check("сміття", {"Demo": {"run/2"}})
        self.assertEqual(len(problems), 1)
        self.assertIn("neither name/arity nor a spec", problems[0].message)


class TestSpecsWhereTheLanguageCannot(unittest.TestCase):
    """Python has no @spec. A shape promised there must be said to be unchecked."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-nospec-")
        self.addCleanup(shutil.rmtree, self.root, True)
        os.makedirs(os.path.join(self.root, "keel/contracts"))
        os.makedirs(os.path.join(self.root, "keel/waves"))
        for name, text in (("pyproject.toml", "[project]\nname = 'demo'\n"),
                           ("demo.py", "def run(a, b, c):\n    return a\n")):
            with open(os.path.join(self.root, name), "w", encoding="utf-8") as handle:
                handle.write(text)

    def contract(self, exports):
        with open(os.path.join(self.root, "keel/contracts/demo.md"), "w",
                  encoding="utf-8") as handle:
            handle.write(f"---\nmodule: demo\nexports:\n  - \"{exports}\"\n"
                         "---\n\nПроза.\n")

    def test_the_python_adapter_admits_it_cannot_check_a_shape(self):
        self.assertFalse(keel.PythonAdapter.supports_specs)
        self.contract("run(a, b, c) :: str")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("cannot be asked for types", problems[0].message)
        self.assertIn("python", problems[0].message)

    def test_the_arity_is_still_checked_there(self):
        self.contract("halt(a) :: str")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("does not export what was promised: halt/1",
                      problems[0].message)


@unittest.skipUnless(shutil.which("mix"), "mix недоступний")
class TestSpecsAgainstARealMixProject(unittest.TestCase):
    """The stubs agree with each other; this one agrees with the compiler."""

    @classmethod
    def setUpClass(cls):
        cls.root = tempfile.mkdtemp(prefix="keel-mix-")
        done = subprocess.run(["mix", "new", "specs"], cwd=cls.root,
                              capture_output=True, text=True)
        if done.returncode != 0:
            shutil.rmtree(cls.root, True)
            raise unittest.SkipTest("mix new не спрацював: " + done.stderr[:200])
        cls.root = os.path.join(cls.root, "specs")
        with open(os.path.join(cls.root, "lib", "specs.ex"), "w",
                  encoding="utf-8") as handle:
            handle.write(
                "defmodule Specs do\n"
                "  @spec run(binary(), keyword()) :: {:ok, term()} | {:error, term()}\n"
                "  def run(text, opts), do: {:ok, {text, opts}}\n\n"
                "  @spec named(text :: binary()) :: :ok\n"
                "  def named(text), do: {:ok, text}\n\n"
                "  @spec pick(integer()) :: :small\n"
                "  @spec pick(binary()) :: :big\n"
                "  def pick(x) when is_integer(x), do: :small\n"
                "  def pick(x) when is_binary(x), do: :big\n\n"
                "  def undeclared(x), do: x\n"
                "end\n")
        os.makedirs(os.path.join(cls.root, "keel", "contracts"))
        os.makedirs(os.path.join(cls.root, "keel", "waves"))

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(os.path.dirname(cls.root), True)

    def contract(self, exports):
        with open(os.path.join(self.root, "keel/contracts/specs.md"), "w",
                  encoding="utf-8") as handle:
            handle.write(f"---\nmodule: Specs\nexports:\n  - \"{exports}\"\n"
                         "---\n\nЩо обіцяє Specs.\n")

    def test_the_compiler_hands_back_the_spec_we_wrote(self):
        answer = keel.ElixirAdapter().exports(self.root, ["Specs"])
        self.assertIn("run/2", answer["Specs"])
        self.assertEqual(answer[("specs", "Specs")]["run/2"],
                         ["run( binary(), keyword() ) :: {:ok, term()} | "
                          "{:error, term()}"])

    def test_a_spec_copied_into_the_contract_passes(self):
        self.contract("run(binary(), keyword()) :: {:ok, term()} | {:error, term()}")
        self.assertEqual(keel.check_exports(keel.Project(self.root)), [])

    def test_a_spec_that_drifted_is_caught(self):
        self.contract("run(binary(), keyword()) :: :ok")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("{:error, term()}", problems[0].message)

    def test_a_named_argument_spec_survives_the_round_trip(self):
        """Те, що віддав компілятор, скопійоване в контракт, має проходити."""
        answer = keel.ElixirAdapter().exports(self.root, ["Specs"])
        declared = answer[("specs", "Specs")]["named/1"][0]
        self.assertIn("::", declared.split("::", 1)[1],
                      "у фікстурі мав бути іменований аргумент")
        self.contract(keel.flatten_spec(declared))
        self.assertEqual(keel.check_exports(keel.Project(self.root)), [])

    def test_a_named_argument_spec_copied_from_the_source_passes(self):
        """Людина копіює не наш вивід, а рядок @spec зі свого ж модуля."""
        self.contract("named(text :: binary()) :: :ok")
        self.assertEqual(keel.check_exports(keel.Project(self.root)), [])

    def test_either_clause_of_a_two_spec_function_passes(self):
        for promised in ("pick(integer()) :: :small", "pick(binary()) :: :big"):
            self.contract(promised)
            self.assertEqual(keel.check_exports(keel.Project(self.root)), [],
                             promised)

    def test_a_third_shape_is_refused_and_both_clauses_shown(self):
        self.contract("pick(atom()) :: :other")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn(":small", problems[0].message)
        self.assertIn(":big", problems[0].message)

    def test_a_function_without_a_spec_is_named(self):
        self.contract("undeclared(term()) :: term()")
        problems = keel.check_exports(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("declares no @spec", problems[0].message)


class TestEverySubprocessIsBounded(unittest.TestCase):
    """The rule "everything that runs project code is bounded and stdinless"
    is held by each call site; this reads them all so a new one cannot forget."""

    ALLOWED_BARE = {"Git.run"}      # git is local and fast, and not project code

    def test_every_run_carries_a_bound_or_is_git(self):
        import ast
        source = os.path.join(os.path.dirname(os.path.dirname(
            os.path.abspath(__file__))), "keel.py")
        with open(source, encoding="utf-8") as handle:
            tree = ast.parse(handle.read())
        offenders = []

        def walk(node, owner):
            for child in ast.iter_child_nodes(node):
                name = owner
                if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef,
                                      ast.ClassDef)):
                    name = f"{owner}.{child.name}".strip(".")
                if (isinstance(child, ast.Call)
                        and isinstance(child.func, ast.Attribute)
                        and child.func.attr == "run"
                        and getattr(child.func.value, "id", "") == "subprocess"):
                    keywords = {kw.arg for kw in child.keywords}
                    if not {"timeout", "stdin"} <= keywords and                             not any(name.endswith(ok) for ok in self.ALLOWED_BARE):
                        offenders.append((name, child.lineno))
                walk(child, name)
        walk(tree, "")
        self.assertEqual(offenders, [],
                         f"subprocess.run без timeout+stdin: {offenders}")


class TestRestampedTagsStayRecognisable(unittest.TestCase):
    """The rewriter and the recogniser are separate regexes; this keeps them
    from drifting apart — a restamped tag the adapter cannot read again would
    be a revision written into a tag that stopped being one."""

    def test_elixir(self):
        out, _ = keel.rewrite_tag('@tag proves: :does_a, rev: "aaaaaa"\n',
                                  "does-a", "ffffff", "elixir")
        found = keel.ElixirAdapter.tag_re.search(out)
        self.assertIsNotNone(found)
        self.assertEqual(found.group(2), "ffffff")

    def test_python(self):
        out, _ = keel.rewrite_tag('# proves: does-a, rev: "aaaaaa"\n',
                                  "does-a", "ffffff", "python")
        found = keel.PythonAdapter.tag_re.search(out)
        self.assertIsNotNone(found)
        self.assertEqual(found.group(2), "ffffff")

    def test_a_tag_written_without_a_revision_gains_one_recognisably(self):
        out, _ = keel.rewrite_tag("@tag proves: :does_a\n", "does-a",
                                  "ffffff", "elixir")
        found = keel.ElixirAdapter.tag_re.search(out)
        self.assertEqual(found.group(2), "ffffff")


class TestPythonRunnerRunsWhatTheCollectorCounts(unittest.TestCase):
    """discover's default test*.py never ran *_test.py — green over a failure."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-runner-")
        self.addCleanup(shutil.rmtree, self.root, True)
        for folder in ("keel/waves", "keel/contracts", "tests"):
            os.makedirs(os.path.join(self.root, folder))
        with open(os.path.join(self.root, "pyproject.toml"), "w") as handle:
            handle.write("[project]\nname='d'\n")
        with open(os.path.join(self.root, "tests/__init__.py"), "w") as handle:
            handle.write("")

    def scenario(self):
        with open(os.path.join(self.root, "keel/waves/0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\nscenarios:\n  does-a: {}\n---\n\n"
                         "## scenario: does-a\n\n**Given** щось.\n")
        return keel.Project(self.root).waves["0001-a"].scenario_body("does-a")

    def test_the_runner_is_handed_the_collectors_own_list(self):
        """Паритет по суті: не два правила, звірені руками, а один список."""
        with open(os.path.join(self.root, "tests/greet_test.py"), "w") as handle:
            handle.write("import unittest\n")
        os.makedirs(os.path.join(self.root, "tests/unit"))
        with open(os.path.join(self.root, "tests/unit/test_deep.py"), "w") as handle:
            handle.write("import unittest\n")
        with open(os.path.join(self.root, "tests/conftest.py"), "w") as handle:
            handle.write("import unittest\n")
        adapter = keel.PythonAdapter()
        collected = {os.path.relpath(x, self.root)
                     for x in adapter.test_files(self.root)}
        script = adapter.test_command(self.root)[-1]
        for name in collected:
            self.assertIn(name, script)
        self.assertNotIn("conftest.py", script)
        self.assertEqual(collected,
                         {"tests/greet_test.py", "tests/unit/test_deep.py"})

    def test_a_failing_test_in_a_nested_dir_without_init_is_red(self):
        """discover пропускав не-пакетну теку — зелене над червоним тестом."""
        body = self.scenario()
        os.makedirs(os.path.join(self.root, "tests/unit"))
        with open(os.path.join(self.root, "tests/unit/foo_test.py"), "w",
                  encoding="utf-8") as handle:
            handle.write("import unittest\n"
                         f"# proves: does-a, rev: \"{keel.revision(body)}\"\n"
                         "class T(unittest.TestCase):\n"
                         "    def test_x(self):\n"
                         "        self.fail('червоний')\n")
        problems = keel.check_scenarios(keel.Project(self.root))
        self.assertTrue(any("the tests are red" in x.message for x in problems),
                        [x.message for x in problems])

    def test_a_failing_suffix_test_turns_check_5_red(self):
        body = self.scenario()
        with open(os.path.join(self.root, "tests/greet_test.py"), "w",
                  encoding="utf-8") as handle:
            handle.write("import unittest\n"
                         f"# proves: does-a, rev: \"{keel.revision(body)}\"\n"
                         "class T(unittest.TestCase):\n"
                         "    def test_x(self):\n"
                         "        self.fail('червоний')\n")
        problems = keel.check_scenarios(keel.Project(self.root))
        self.assertTrue(any("the tests are red" in x.message for x in problems),
                        [x.message for x in problems])

    def test_a_conftest_that_cannot_import_does_not_break_check_5(self):
        body = self.scenario()
        with open(os.path.join(self.root, "tests/conftest.py"), "w") as handle:
            handle.write("import немає_такого_модуля\n")
        with open(os.path.join(self.root, "tests/greet_test.py"), "w",
                  encoding="utf-8") as handle:
            handle.write("import unittest\n"
                         f"# proves: does-a, rev: \"{keel.revision(body)}\"\n"
                         "class T(unittest.TestCase):\n"
                         "    def test_x(self):\n"
                         "        self.assertTrue(True)\n")
        problems = keel.check_scenarios(keel.Project(self.root))
        self.assertEqual(problems, [], [x.message for x in problems])

    def test_the_start_directory_follows_test_dirs(self):
        os.rename(os.path.join(self.root, "tests"),
                  os.path.join(self.root, "test"))
        with open(os.path.join(self.root, "test/greet_test.py"), "w") as handle:
            handle.write("import unittest\n")
        script = keel.PythonAdapter().test_command(self.root)[-1]
        self.assertIn("test/greet_test.py", script)


class TestTheDictatedTagFollowsTheAdapter(unittest.TestCase):
    """next dictated the Elixir form to Python operators — invisible tags."""

    def test_each_dialect_dictates_its_own(self):
        self.assertIn(":does_a", keel.ElixirAdapter.tag_example("does-a", "ffffff"))
        python = keel.PythonAdapter.tag_example("does-a", "ffffff")
        self.assertNotIn(":does", python)
        self.assertTrue(python.startswith("# proves: does-a"))

    def test_what_is_dictated_is_what_the_collector_reads(self):
        """Написане під диктовку мусить читатися своїм же збирачем."""
        for adapter in (keel.ElixirAdapter, keel.PythonAdapter):
            example = adapter.tag_example("does-a", "ffffff")
            found = adapter.tag_re.search(example)
            self.assertIsNotNone(found, adapter.name)
            self.assertEqual(found.group(2), "ffffff", adapter.name)


class TestGitEdges(unittest.TestCase):
    """Two git surfaces the scope check leans on: the baseline and the diff."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-git-")
        self.addCleanup(shutil.rmtree, self.root, True)

    def git(self, *args, cwd=None):
        return subprocess.run(["git", "-C", cwd or self.root, *args],
                              capture_output=True, text=True, check=True)

    def repo(self, branch="main"):
        subprocess.run(["git", "init", "-q", "-b", branch, self.root], check=True)
        for name, value in (("user.email", "t@e"), ("user.name", "t")):
            self.git("config", name, value)

    def test_a_default_branch_with_a_slash_resolves(self):
        """release/2024 не має ставати неіснуючим origin/2024."""
        self.repo("release/2024")
        with open(os.path.join(self.root, "f.txt"), "w") as handle:
            handle.write("вміст\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        self.git("update-ref", "refs/remotes/origin/release/2024", "HEAD")
        self.git("symbolic-ref", "refs/remotes/origin/HEAD",
                 "refs/remotes/origin/release/2024")
        self.git("checkout", "-q", "-b", "feature")
        g = keel.Git(self.root)
        # origin/release/2024 is as right as release/2024 and better as a
        # baseline; what must never happen is the truncated origin/2024.
        self.assertIn("release/2024", g.main_branch)
        self.assertNotIn("origin/2024", g.main_branch)
        self.assertEqual(g.main_short, "release/2024")
        self.assertTrue(g.merge_base(g.main_branch))

    def test_an_origin_prefix_comes_off_for_main_short(self):
        self.repo()
        with open(os.path.join(self.root, "f.txt"), "w") as handle:
            handle.write("x\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        self.assertEqual(keel.Git(self.root).main_short, "main")

    def test_a_cyrillic_path_arrives_unmangled(self):
        """quotePath калічив diff/log — файл.txt ніколи не збігався з оголошеним."""
        self.repo()
        with open(os.path.join(self.root, "файл.txt"), "w", encoding="utf-8") as handle:
            handle.write("вміст\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        with open(os.path.join(self.root, "файл.txt"), "a", encoding="utf-8") as handle:
            handle.write("ще\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "зміна")
        base = subprocess.run(["git", "-C", self.root, "rev-parse", "HEAD~1"],
                              capture_output=True, text=True).stdout.strip()
        changed = keel.Git(self.root).changed_files(base)
        self.assertEqual(changed, {"файл.txt"})

    def test_a_committed_rename_reports_both_names(self):
        """Інакше вердикт четвірки перевертався в момент комміту."""
        self.repo()
        with open(os.path.join(self.root, "old.txt"), "w") as handle:
            handle.write("вміст досить довгий для rename-детекції\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        base = subprocess.run(["git", "-C", self.root, "rev-parse", "HEAD"],
                              capture_output=True, text=True).stdout.strip()
        self.git("mv", "old.txt", "new.txt")
        self.git("commit", "-q", "-m", "rename")
        changed = keel.Git(self.root).changed_files(base)
        self.assertEqual(changed, {"old.txt", "new.txt"})

    def test_standing_on_a_default_branch_named_trunk(self):
        """Одногілковий захист не має фарбувати сам default у червоне."""
        self.repo("trunk")
        with open(os.path.join(self.root, "f.txt"), "w") as handle:
            handle.write("x\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        # Повний клон пізнається за wildcard-refspec, а не за числом гілок:
        # CI регулярно доfetch-ує базу в одногілковий клон.
        self.git("config", "remote.origin.url", "/dev/null")
        self.git("config", "remote.origin.fetch",
                 "+refs/heads/*:refs/remotes/origin/*")
        self.git("update-ref", "refs/remotes/origin/trunk", "HEAD")
        self.git("symbolic-ref", "refs/remotes/origin/HEAD",
                 "refs/remotes/origin/trunk")
        g = keel.Git(self.root)
        self.assertEqual(g.main_branch, "trunk")
        self.assertEqual(g.main_short, "trunk")

    def test_the_refspec_tells_the_clone_shapes_apart(self):
        """Не імена й не лічильник гілок: --single-branch звужує саме refspec."""
        self.repo()
        with open(os.path.join(self.root, "f.txt"), "w") as handle:
            handle.write("x\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        self.git("config", "remote.origin.url", "/dev/null")
        g = keel.Git(self.root)
        self.git("config", "remote.origin.fetch",
                 "+refs/heads/*:refs/remotes/origin/*")
        self.assertTrue(keel.Git(self.root).tracks_whole_remote)
        self.git("config", "remote.origin.fetch",
                 "+refs/heads/work:refs/remotes/origin/work")
        self.assertFalse(keel.Git(self.root).tracks_whole_remote)

    def test_no_refspec_at_all_errs_towards_distrust(self):
        """Невизначеність має схилятись до гучної відмови, не до тихого зеленого."""
        self.repo()
        with open(os.path.join(self.root, "f.txt"), "w") as handle:
            handle.write("x\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        self.assertFalse(keel.Git(self.root).tracks_whole_remote)

    def test_a_stale_main_does_not_hijack_a_trunk_default(self):
        """Повний клон на trunk із залишковим main тримає trunk."""
        self.repo("trunk")
        with open(os.path.join(self.root, "f.txt"), "w") as handle:
            handle.write("x\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        self.git("branch", "main", "HEAD")
        self.git("config", "remote.origin.url", "/dev/null")
        self.git("config", "remote.origin.fetch",
                 "+refs/heads/*:refs/remotes/origin/*")
        self.git("update-ref", "refs/remotes/origin/trunk", "HEAD")
        self.git("symbolic-ref", "refs/remotes/origin/HEAD",
                 "refs/remotes/origin/trunk")
        self.assertEqual(keel.Git(self.root).main_branch, "trunk")

    def test_a_ci_base_fetch_does_not_defeat_the_single_branch_guard(self):
        """clone --single-branch + fetch origin main — стандартний CI: дві
        гілки в origin, і лічильник сам по собі повірив би origin/HEAD."""
        self.repo()
        with open(os.path.join(self.root, "f.txt"), "w") as handle:
            handle.write("x\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        self.git("checkout", "-q", "-b", "0001-work")
        with open(os.path.join(self.root, "f.txt"), "a") as handle:
            handle.write("y\n")
        self.git("commit", "-qam", "робота")
        # так виглядає одногілковий клон із домальованою базою
        self.git("update-ref", "refs/remotes/origin/0001-work", "HEAD")
        self.git("update-ref", "refs/remotes/origin/main", "HEAD~1")
        self.git("symbolic-ref", "refs/remotes/origin/HEAD",
                 "refs/remotes/origin/0001-work")
        self.git("branch", "-q", "-D", "main")
        g = keel.Git(self.root)
        self.assertNotEqual(g.main_branch, "0001-work",
                            "гілка стала власною базою")
        self.assertTrue(g.merge_base(g.main_branch))

    def test_a_single_branch_clone_is_still_distrusted(self):
        """Гілка під тестом не має ставати власною базою — як і раніше."""
        self.repo()
        with open(os.path.join(self.root, "f.txt"), "w") as handle:
            handle.write("x\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        self.git("checkout", "-q", "-b", "0001-work")
        self.git("update-ref", "refs/remotes/origin/0001-work", "HEAD")
        self.git("symbolic-ref", "refs/remotes/origin/HEAD",
                 "refs/remotes/origin/0001-work")
        self.assertNotEqual(keel.Git(self.root).main_branch, "0001-work")

    def test_a_worktree_rename_does_not_inject_a_phantom_path(self):
        self.repo()
        with open(os.path.join(self.root, "old.txt"), "w") as handle:
            handle.write("достатньо вмісту щоб rename було видно\n")
        self.git("add", "-A"); self.git("commit", "-q", "-m", "base")
        os.rename(os.path.join(self.root, "old.txt"),
                  os.path.join(self.root, "renamed.txt"))
        self.git("add", "-N", "renamed.txt")
        changed = keel.Git(self.root).changed_files("")
        self.assertEqual(changed, {"old.txt", "renamed.txt"})
        self.assertNotIn(".txt", changed)


# ─────────────────────────────────────────────────────────────────────────────
# keel next
# ─────────────────────────────────────────────────────────────────────────────


if __name__ == "__main__":
    unittest.main()


class TestASkippedTestProvesNothing(unittest.TestCase):
    """Пропущений тест лишає набір успішним, тож `wasSuccessful()` — усе, чим
    була «зелень», — правдиве над тестом, чиє тіло є голим провалом."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-skip-")
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        for folder in ("keel/waves", "keel/contracts", "src", "tests"):
            os.makedirs(os.path.join(self.root, folder))
        self.write("pyproject.toml", "[project]\nname = 'demo'\nversion = '0.1'\n")
        self.write("src/__init__.py", "")
        self.write("src/thing.py", "def run(a):\n    return a\n")
        self.write("keel/contracts/thing.md",
                   "---\nmodule: src.thing\nexports: [run/1]\n---\n\nОбіцянка.\n")
        self.write("keel/waves/0001-demo.md", """---
depends_on: []

scenarios:
  runs-ok: {proves: thing}

transforms:
  build-it:
    implements: [runs-ok]
    contracts:  [thing]
    files:      [src/thing.py]
---

## Навіщо

Демо.

## scenario: runs-ok

**Given** одне, **When** друге, **Then** третє.

## transform: build-it

Робить.

Межі: нічого.
""")

    def write(self, name, text):
        path = os.path.join(self.root, name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)

    def write_test(self, skip):
        project = keel.Project(self.root)
        rev = project.waves["0001-demo"].scenario_revision("runs-ok")
        self.write("tests/test_demo.py",
                   "import unittest\n"
                   "from src.thing import run\n\n"
                   "class T(unittest.TestCase):\n"
                   f'    # proves: runs-ok, rev: "{rev}"\n'
                   + ('    @unittest.skip("ще не написаний")\n' if skip else "")
                   + "    def test_runs_ok(self):\n"
                   + ("        self.fail('не доведено нічим')\n" if skip
                      else "        self.assertEqual(run(1), 1)\n"))

    def test_a_test_that_runs_is_still_green(self):
        self.write_test(skip=False)
        self.assertEqual(keel.check_scenarios(keel.Project(self.root)), [])

    def test_a_skipped_test_is_not_a_proof(self):
        self.write_test(skip=True)
        problems = keel.check_scenarios(keel.Project(self.root))
        self.assertTrue(any("did not run" in p.message for p in problems),
                        [p.message for p in problems])

    def test_the_run_leaves_no_bytecode_behind(self):
        """Проба й прогін писали __pycache__, і перевірка 4 звинувачувала
        гілку в тому, що інструмент насмітив секундою раніше."""
        self.write_test(skip=False)
        keel.check_scenarios(keel.Project(self.root))
        keel.check_exports(keel.Project(self.root))
        left = [name for _, dirs, _ in os.walk(self.root) for name in dirs
                if name == "__pycache__"]
        self.assertEqual(left, [])


class TestWhatDidNotRunIsNamedPreciselyOrCounted(unittest.TestCase):
    """Пропущений тест лишає набір успішним. Хто вміє назвати — називає; хто
    вміє лише порахувати — рахує; мовчати не вміє ніхто."""

    def test_python_names_them(self):
        names, count = keel.PythonAdapter().not_run(
            f"{keel.SKIP_MARK}tests.test_demo.T.test_runs_ok\nінший рядок\n")
        self.assertEqual(names, ["tests.test_demo.T.test_runs_ok"])
        self.assertEqual(count, 0)

    def test_elixir_counts_them(self):
        names, count = keel.ElixirAdapter().not_run(
            "Finished in 0.2 seconds\n5 tests, 0 failures, 1 excluded, 2 skipped\n")
        self.assertEqual(names, [])
        self.assertEqual(count, 3)

    def test_elixir_says_nothing_when_everything_ran(self):
        self.assertEqual(
            keel.ElixirAdapter().not_run("5 tests, 0 failures\n"), ([], 0))

    def test_the_base_adapter_promises_the_same_shape(self):
        self.assertEqual(keel.Adapter().not_run("будь-що"), ([], 0))


class TestTheSkippedMatchIsAnchored(unittest.TestCase):
    """Пошук підрядком робив сценарій `ok` власником чужого
    `test_runs_ok_on_windows` — червоне, з якого не вийти."""

    def test_an_unrelated_skip_does_not_claim_a_short_slug(self):
        self.assertNotEqual(
            keel.normalise_slug("test_runs_ok_on_windows".rsplit(".", 1)[-1]),
            "test-ok")

    def test_the_conventional_name_still_matches(self):
        self.assertEqual(
            keel.normalise_slug("tests.test_demo.T.test_runs_ok".rsplit(".", 1)[-1]),
            "test-runs-ok")

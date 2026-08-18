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
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        self.fixture.write("keel/steps/0001-session-loop.md",
                           text.replace("розмова завершується", "розмова завершується сама"))
        problems = keel.check_scenarios(self.project, run_tests=False)
        self.assertEqual(len(problems), 1)
        self.assertIn("the test holds", problems[0].message)

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
        os.makedirs(os.path.join(self.root, "keel/steps"))
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

    def test_a_contract_without_verify_at_all_is_left_alone(self):
        self.write("keel/contracts/prose.md", "---\nmodule: demo\n---\n\nСама проза.\n")
        self.assertEqual(keel.check_exports(keel.Project(self.root)), [])

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
        for folder in ("keel/steps", "keel/contracts"):
            os.makedirs(os.path.join(self.root, folder))
        with open(os.path.join(self.root, "keel/steps/0001-a.md"), "w",
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

    def test_no_marker_at_all_is_still_no_adapter(self):
        self.assertIsNone(keel.Project(self.root).adapter)


# ─────────────────────────────────────────────────────────────────────────────
# Nothing that runs the project's own code may run without a bound
# ─────────────────────────────────────────────────────────────────────────────

class SleepAdapter(keel.Adapter):
    """An adapter whose test command never finishes."""

    name = "sleepy"

    def test_command(self):
        return ["sleep", "30"]

    def test_files(self, root):
        return []

    def exports(self, root, modules):
        return {}


class TestNothingHangsForever(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-hang-")
        self.addCleanup(shutil.rmtree, self.root, True)
        os.makedirs(os.path.join(self.root, "keel/steps"))
        os.makedirs(os.path.join(self.root, "keel/contracts"))
        with open(os.path.join(self.root, "keel/steps/0001-a.md"), "w",
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
        self.assertIn("was not found", probe.stderr)

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

    def test_commas_inside_a_type_do_not_add_arguments(self):
        """{:ok, term()} — один аргумент, а не два."""
        self.assertEqual(
            keel.promised_signature("halt({:ok, term()}, [a: b]) :: :ok"),
            ("halt", 2))

    def test_a_question_mark_belongs_to_the_name(self):
        self.assertEqual(keel.promised_signature("valid?(t()) :: boolean()"),
                         ("valid?", 1))

    def test_nonsense_is_nonsense(self):
        for entry in ("сміття", "run/x", "run/", ":: t()", "run :: t()",
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
        os.makedirs(os.path.join(self.root, "keel/steps"))

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
        os.makedirs(os.path.join(self.root, "keel/steps"))
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
        os.makedirs(os.path.join(cls.root, "keel", "steps"))

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


# ─────────────────────────────────────────────────────────────────────────────
# keel next
# ─────────────────────────────────────────────────────────────────────────────


if __name__ == "__main__":
    unittest.main()

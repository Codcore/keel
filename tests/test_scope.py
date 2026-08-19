#!/usr/bin/env python3
"""Check 4: what the branch touched against what it declared."""

import os
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args, ProjectCase  # noqa: E402




# ─────────────────────────────────────────────────────────────────────────────
# Check 4: scope
# ─────────────────────────────────────────────────────────────────────────────

class TestDriftFromWhatWasApproved(ProjectCase):
    """Схвалення виводиться з того, що крок дійшов до головної гілки.

    Далі ніщо не заважало гілці його переписати: `keel/` поза скоупом навмисно,
    щоб `update` міг оновити наші файли посеред роботи. Прогулянка дала крок,
    правлений тричі після схвалення — і названо це було лише тому, що агент сам
    захотів назвати. Дрейф лишається дозволеним; кінчається мовчання.
    """

    def touch_the_step(self):
        step = "keel/steps/0001-session-loop.md"
        self.fixture.write(step, self.fixture.read(step) + "\nЩе абзац.\n")

    def test_a_work_branch_names_the_difference(self):
        self.fixture.branch("0001-session-loop")
        self.touch_the_step()
        drifted = keel.drifted_from_main(self.project)
        self.assertEqual([name for name, _, _ in drifted],
                         ["keel/steps/0001-session-loop.md"])

    def test_an_untouched_step_says_nothing(self):
        self.fixture.branch("0001-session-loop")
        self.assertEqual(keel.drifted_from_main(self.project), [])

    def test_a_plan_branch_is_where_a_step_is_written(self):
        self.fixture.branch("plan/0001-session-loop")
        self.touch_the_step()
        self.assertEqual(keel.drifted_from_main(self.project), [])

    def test_the_main_branch_has_nothing_to_compare_with(self):
        self.touch_the_step()
        self.assertEqual(keel.drifted_from_main(self.project), [])

    def test_a_document_this_branch_created_is_not_drift(self):
        """Новий контракт — це робота, а не дрейф уже схваленого."""
        self.fixture.branch("0001-session-loop")
        self.fixture.write("keel/contracts/brand-new.md",
                           "---\nmodule: Demo.New\nexports: [run/1]\n---\n\nНове.\n")
        self.assertEqual(keel.drifted_from_main(self.project), [])


class TestScopeAsksOnlyAboutClosedTransforms(ProjectCase):
    """Крок робиться по трансформі за раз, і заслон має це знати.

    Інакше pre-commit не пускає жодного комміту, крім останнього, а агент,
    який зустрів заслон, крізь який чесно не пройти, вчиться казати
    `--no-verify`. Перевірено наживо: вистачило однієї відмови.
    """

    TWO = """---
depends_on: []

scenarios:
  first-holds:  {proves: session-run@%(rev)s}
  second-holds: {proves: session-run@%(rev)s}

transforms:
  do-the-first:
    implements: [first-holds]
    contracts:  [session-run@%(rev)s]
    files:      [lib/first.ex]

  do-the-second:
    implements: [second-holds]
    contracts:  [session-run@%(rev)s]
    files:      [lib/second.ex]
---

## Навіщо

Дві трансформи, щоб було видно, що буває після першої.

## scenario: first-holds

**Given** одне, **When** друге, **Then** третє.

## scenario: second-holds

**Given** одне, **When** друге, **Then** третє.

## transform: do-the-first

Робить перше.

Межі: не робить другого.

## transform: do-the-second

Робить друге.

Межі: не робить першого.
"""

    def setUp(self):
        super().setUp()
        self.fixture.write("keel/steps/0002-two-moves.md",
                           self.TWO % {"rev": self.fixture.contract_rev})
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "план на дві трансформи")
        self.fixture.branch("0002-two-moves")

    def commit_the_first(self):
        self.fixture.write("lib/first.ex", "defmodule First do\nend\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "do-the-first: перше")

    def test_the_second_transform_is_not_owed_before_its_commit(self):
        self.commit_the_first()
        problems = keel.check_scope(self.project)
        self.assertEqual([p.message for p in problems], [], [p.render() for p in problems])

    def test_reaching_outside_the_step_still_shows_at_once(self):
        self.fixture.write("lib/first.ex", "defmodule First do\nend\n")
        self.fixture.write("lib/nobody_declared.ex", "defmodule Nope do\nend\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "do-the-first: перше і зайве")
        problems = keel.check_scope(self.project)
        self.assertTrue(any("lib/nobody_declared.ex" in p.message for p in problems),
                        [p.render() for p in problems])

    def test_a_closed_transform_that_touched_nothing_is_still_caught(self):
        """Комміт зі слагом є, а файла він не приніс — це та сама тиша."""
        self.fixture.write("lib/second.ex", "defmodule Second do\nend\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "do-the-first: слаг першої, а файл другої")
        problems = keel.check_scope(self.project)
        self.assertTrue(any("lib/first.ex" in p.message for p in problems),
                        [p.render() for p in problems])

    def test_when_every_transform_is_closed_the_whole_list_is_owed(self):
        self.commit_the_first()
        self.fixture.git("commit", "--allow-empty", "-m", "do-the-second: порожній")
        problems = keel.check_scope(self.project)
        self.assertTrue(any("lib/second.ex" in p.message for p in problems),
                        [p.render() for p in problems])


class TestScope(ProjectCase):
    def test_main_branch_is_not_checked(self):
        self.assertEqual(keel.check_scope(self.project), [])

    def test_declared_and_touched_is_clean(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "defmodule Demo.Session do\n  def run(_,_,_), do: :ok\nend\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_touched_beyond_scope(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("lib/extra.ex", "не оголошено\n")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("lib/extra.ex", problems[0].message)
        self.assertIn("not declared", problems[0].message)

    def test_declared_but_untouched_once_the_transform_is_closed(self):
        """Поки комміту зі слагом немає, трансформа ще нічого не винна."""
        self.fixture.branch("0001-session-loop")
        self.assertEqual(keel.check_scope(self.project), [])

        self.fixture.git("commit", "--allow-empty", "-m", "drive-turns: нічого не приніс")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("declared but not changed", problems[0].message)

    def test_committed_change_counts(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns: перший хід")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_keel_documents_are_outside_scope(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("keel/contracts/no-retry.md", "---\nverify: \"true\"\n---\n\nх\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_a_plan_branch_may_carry_keels_own_files(self):
        """Інакше перший же комміт плану впирається в те, що поклав init."""
        self.fixture.branch("plan/0001-session-loop")
        for name in ("AGENTS.md", ".claude/skills/keel-plan/SKILL.md",
                     ".cursor/hooks.json", ".github/workflows/keel.yml",
                     ".claude/settings.json"):
            self.fixture.write(name, "породжене\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_a_work_branch_may_carry_keels_own_files_too(self):
        """`update` серед роботи не має вимагати оголосити наш власний скіл."""
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        for name in ("AGENTS.md", ".claude/skills/keel-plan/SKILL.md",
                     ".cursor/hooks.json", ".github/workflows/keel.yml",
                     ".claude/settings.json"):
            self.fixture.write(name, "породжене\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_a_declared_keel_file_earns_no_false_report(self):
        """Оголошений AGENTS.md давав «declared but not changed» над diff-ом,
        який його явно змінив."""
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        text = self.fixture.read("keel/steps/0001-session-loop.md")
        self.fixture.write("keel/steps/0001-session-loop.md",
                           text.replace("files:      [lib/session.ex]",
                                        "files:      [lib/session.ex, AGENTS.md]"))
        self.fixture.write("AGENTS.md", "змінений блок\n")
        self.assertEqual(keel.check_scope(self.project), [])

    def test_a_work_branch_still_catches_an_undeclared_project_file(self):
        """Звільнення стосується нашої обстановки, не будь-чого поруч."""
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("lib/stray.ex", "не оголошено\n")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("lib/stray.ex", problems[0].message)

    def test_plan_branch_must_not_touch_code(self):
        self.fixture.branch("plan/0001-session-loop")
        self.fixture.write("lib/session.ex", "код у гілці плану\n")
        problems = keel.check_scope(self.project)
        self.assertEqual(len(problems), 1)
        self.assertIn("a plan branch is touching code", problems[0].message)

    def test_a_missing_merge_base_is_red_not_silently_green(self):
        """Без бази diff бачить лише незакомічене — і все закомічене проходить."""
        self.fixture.git("branch", "-m", "main", "trunk")
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("lib/undeclared.ex", "не оголошено\n")
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "drive-turns: хід")
        problems = keel.check_scope(self.project)
        self.assertTrue(problems)
        self.assertIn("cannot tell where this branch left from", problems[0].message)

    def test_origin_head_pointing_at_the_work_branch_is_not_trusted(self):
        """Одногілковий клон робив гілку власною базою — і все проходило."""
        self.fixture.branch("0001-session-loop")
        self.fixture.git("update-ref", "refs/remotes/origin/HEAD",
                         "refs/heads/0001-session-loop")
        self.fixture.git("symbolic-ref", "refs/remotes/origin/HEAD",
                         "refs/remotes/origin/0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("lib/stray.ex", "не оголошено\n")
        problems = keel.check_scope(self.project)
        self.assertTrue(problems, "перевірка мовчала на неоголошеному файлі")
        self.assertTrue(any("lib/stray.ex" in p.message
                            or "не знайшов, від чого" in p.message for p in problems))

    def test_branch_that_is_not_a_step(self):
        self.fixture.branch("random-branch")
        problems = keel.check_scope(self.project)
        self.assertIn("is not named after a step", problems[0].message)


# ─────────────────────────────────────────────────────────────────────────────
# Check 5: scenarios and tags
# ─────────────────────────────────────────────────────────────────────────────




class TestBranchOverride(ProjectCase):
    def test_detached_head_with_branch_flag(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("commit", "-am", "drive-turns: хід")
        self.fixture.git("checkout", "-q", "--detach")
        project = self.project
        self.assertIn("the head is detached", keel.check_scope(project)[0].message)
        project.branch_override = "0001-session-loop"
        self.assertEqual(keel.check_scope(project), [])




class TestAFreshRepository(unittest.TestCase):
    """The first `keel check` a new project ever runs."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-fresh-")
        self.addCleanup(shutil.rmtree, self.root, True)
        for folder in ("keel/steps", "keel/contracts"):
            os.makedirs(os.path.join(self.root, folder))
        subprocess.run(["git", "init", "-q", "-b", "main", self.root], check=True)

    def test_no_commits_is_not_a_detached_head(self):
        """HEAD вказує на гілку, якої ще немає — rev-parse падав, і порожня
        відповідь читалась як відчеплена голова."""
        git = keel.Git(self.root)
        self.assertEqual(git.branch, "main")
        self.assertFalse(git.has_commits)
        self.assertEqual(keel.check_scope(keel.Project(self.root)), [])

    def test_once_there_is_a_commit_the_check_works_as_before(self):
        for key, value in (("user.email", "t@e"), ("user.name", "t")):
            subprocess.run(["git", "-C", self.root, "config", key, value], check=True)
        subprocess.run(["git", "-C", self.root, "commit", "-q", "--allow-empty",
                        "-m", "base"], check=True)
        git = keel.Git(self.root)
        self.assertTrue(git.has_commits)
        self.assertEqual(git.branch, "main")


class TestNestedKeelRoot(unittest.TestCase):
    """A keel root inside a bigger repository — a layout find_root supports."""

    def setUp(self):
        self.top = tempfile.mkdtemp(prefix="keel-nested-")
        self.addCleanup(shutil.rmtree, self.top, True)
        self.root = os.path.join(self.top, "sub")
        for folder in ("keel/steps", "keel/contracts", "lib"):
            os.makedirs(os.path.join(self.root, folder))
        os.makedirs(os.path.join(self.top, "other"))
        with open(os.path.join(self.root, "keel/steps/0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\ntransforms:\n  do:\n    files: [lib/foo.txt]\n"
                         "---\n\n## Why\n\nх.\n\n## transform: do\n\nЩось.\n")
        with open(os.path.join(self.top, "other/x.txt"), "w") as handle:
            handle.write("чуже\n")
        self.git("init", "-q", "-b", "main", ".")
        self.git("config", "user.email", "t@e")
        self.git("config", "user.name", "t")
        self.git("add", "-A")
        self.git("commit", "-q", "-m", "base")
        self.git("checkout", "-q", "-b", "0001-a")

    def git(self, *args):
        subprocess.run(["git", "-C", self.top, *args], check=False,
                       capture_output=True)

    def test_a_declared_file_matches_despite_the_prefix(self):
        """git каже sub/lib/foo.txt, крок оголошує lib/foo.txt."""
        with open(os.path.join(self.root, "lib/foo.txt"), "w") as handle:
            handle.write("змінено\n")
        self.assertEqual(keel.check_scope(keel.Project(self.root)), [])

    def test_a_sibling_directory_is_not_this_steps_business(self):
        with open(os.path.join(self.root, "lib/foo.txt"), "w") as handle:
            handle.write("змінено\n")
        with open(os.path.join(self.top, "other/x.txt"), "a") as handle:
            handle.write("чужа зміна\n")
        self.assertEqual(keel.check_scope(keel.Project(self.root)), [])

    def test_an_undeclared_file_inside_the_keel_root_is_still_caught(self):
        with open(os.path.join(self.root, "lib/foo.txt"), "w") as handle:
            handle.write("змінено\n")
        with open(os.path.join(self.root, "lib/stray.txt"), "w") as handle:
            handle.write("не оголошено\n")
        problems = keel.check_scope(keel.Project(self.root))
        self.assertEqual(len(problems), 1)
        self.assertIn("lib/stray.txt", problems[0].message)


if __name__ == "__main__":
    unittest.main()


class TestDriftIsMeasuredFromTheBranchPoint(ProjectCase):
    """Головна гілка йде далі, поки гілка відкрита.

    Проти вершини `main` чужа злита правка читалась як дрейф цієї гілки — тобто
    повідомлення спрацьовувало там, де не сталось нічого, а саме так воно й
    перестає читатись.
    """

    def test_somebody_elses_merge_is_not_this_branch_drift(self):
        self.fixture.branch("0001-session-loop")
        self.fixture.git("checkout", "main")
        step = "keel/steps/0001-session-loop.md"
        self.fixture.write(step, self.fixture.read(step) + "\nПравка на main.\n")
        self.fixture.git("commit", "-am", "хтось інший правив свій крок")
        self.fixture.git("checkout", "0001-session-loop")
        self.assertEqual(keel.drifted_from_main(self.project), [])

    def test_this_branch_own_change_is_still_named(self):
        self.fixture.branch("0001-session-loop")
        step = "keel/steps/0001-session-loop.md"
        self.fixture.write(step, self.fixture.read(step) + "\nПравка тут.\n")
        drifted = keel.drifted_from_main(self.project)
        self.assertEqual([name for name, _, _ in drifted], [step])


class TestATransformSlugBelongsToOneStep(ProjectCase):
    """Слаг у повідомленні комміта — єдиний звʼязок роботи з планом.

    Спільний слаг не ідентифікує нічого: комміт закривав трансформу в обох
    кроках, і `next` оголошував завершеним проєкт, у якому крок ніхто не
    починав.
    """

    def second_step_with(self, transform):
        rev = self.fixture.contract_rev
        self.fixture.write("keel/steps/0002-other.md", f"""---
depends_on: []

scenarios:
  other-holds: {{proves: session-run@{rev}}}

transforms:
  {transform}:
    implements: [other-holds]
    contracts:  [session-run@{rev}]
    files:      [lib/other.ex]
---

## Навіщо

Другий крок.

## scenario: other-holds

**Given** одне, **When** друге, **Then** третє.

## transform: {transform}

Робить.

Межі: нічого.
""")

    def test_a_slug_two_steps_share_is_refused(self):
        self.second_step_with("drive-turns")
        problems = keel.shared_transform_slugs(self.project)
        self.assertTrue(any("drive-turns" in p.message for p in problems),
                        [p.message for p in problems])

    def test_distinct_slugs_pass(self):
        self.second_step_with("other-turns")
        self.assertEqual(keel.shared_transform_slugs(self.project), [])

    def test_a_disagreement_is_not_a_parse_error(self):
        """Документи читаються чудово — вони суперечать одне одному."""
        self.second_step_with("drive-turns")
        self.assertEqual(keel.check_structure(self.project), [])

    def test_next_refuses_to_answer_from_ambiguous_documents(self):
        self.second_step_with("drive-turns")
        answer = keel.main_branch_answer(self.project)
        self.assertIn("do not agree with themselves", answer)
        self.assertNotIn("every step is finished", answer)


class TestAKeelRootNestedInABiggerRepository(unittest.TestCase):
    """`<ref>:<шлях>` рахує шлях від кореня репозиторію й ігнорує `git -C`.

    Keel говорить шляхами від кореня keel, і поки вони збігались, ніхто цього
    не помічав. У монорепо `next` відмовляв усю роботу словами «план не
    схвалений», хоч план лежав на main.
    """

    def setUp(self):
        self.top = tempfile.mkdtemp(prefix="keel-nested-")
        self.addCleanup(shutil.rmtree, self.top, ignore_errors=True)
        self.root = os.path.join(self.top, "sub")
        for folder in ("keel/steps", "keel/contracts", "lib"):
            os.makedirs(os.path.join(self.root, folder))
        self.write("../README.md", "корінь монорепо\n")
        self.write("keel/contracts/thing.md",
                   "---\nmodule: Demo.Thing\nexports: [run/1]\n---\n\nОбіцянка.\n")
        self.write("keel/steps/0001-demo.md", """---
depends_on: []

scenarios:
  it-works: {proves: thing}

transforms:
  do-it:
    implements: [it-works]
    contracts:  [thing]
    files:      [lib/one.ex]
---

## Навіщо

Демо.

## scenario: it-works

**Given** одне, **When** друге, **Then** третє.

## transform: do-it

Робить.

Межі: нічого.
""")
        self.write("lib/one.ex", "one\n")
        self.git("init", "-b", "main")
        self.git("config", "user.email", "t@e.co")
        self.git("config", "user.name", "t")
        self.git("add", "-A")
        self.git("commit", "-m", "монорепо з keel у sub/")

    def write(self, name, text):
        path = os.path.join(self.root, name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)

    def git(self, *args):
        return subprocess.run(["git", "-C", self.top, *args], capture_output=True,
                              text=True, stdin=subprocess.DEVNULL, timeout=30)

    @property
    def project(self):
        return keel.Project(self.root)

    def test_the_step_is_seen_on_the_main_branch(self):
        project = self.project
        self.assertTrue(project.git.file_in_branch(
            project.git.main_branch, "keel/steps/0001-demo.md"))

    def test_the_approved_file_list_is_readable(self):
        project = self.project
        self.assertEqual(
            keel.approved_files(project, project.steps["0001-demo"]),
            {"lib/one.ex"})

    def test_a_contract_already_on_main_does_not_read_as_arriving(self):
        self.assertEqual(self.project.arriving_contracts, set())

    def test_next_hands_out_the_work(self):
        from io import StringIO
        self.git("checkout", "-b", "0001-demo")
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_next(self.project, Args(json=False))
        finally:
            sys.stdout = saved
        self.assertEqual(code, 0, stream.getvalue())
        self.assertIn("do-it", stream.getvalue())


class TestDeletingSomebodyElsesDocument(ProjectCase):
    """Найтихіший шлях повз заслон: те, чого немає, не назве ніщо.

    `keel/` виведено з-під скоупу, щоб `update` міг оновити наші файли посеред
    роботи, а нота про дрейф читає лише зміни. Тож `git rm` чужого кроку
    проходив усі гейти зеленим.
    """

    def setUp(self):
        super().setUp()
        self.fixture.write("keel/steps/0002-other.md",
                           self.fixture.read("keel/steps/0001-session-loop.md")
                           .replace("drive-turns", "other-turns")
                           .replace("finishes-when-no-tool-called", "other-holds"))
        self.fixture.git("add", "-A")
        self.fixture.git("commit", "-m", "другий крок")
        self.fixture.branch("0001-session-loop")

    def test_removing_another_step_is_named(self):
        self.fixture.git("rm", "-q", "keel/steps/0002-other.md")
        self.fixture.git("commit", "-m", "drive-turns: прибрав те, що заважало")
        problems = keel.check_scope(self.project)
        self.assertTrue(any("0002-other" in p.message for p in problems),
                        [p.message for p in problems])

    def test_removing_a_contract_is_named_too(self):
        self.fixture.git("rm", "-q", "keel/contracts/session-run.md")
        self.fixture.git("commit", "-m", "drive-turns: і контракт теж")
        problems = keel.check_scope(self.project)
        self.assertTrue(any("session-run" in p.message for p in problems),
                        [p.message for p in problems])

    def test_touching_nothing_says_nothing(self):
        self.assertEqual(
            [p for p in keel.check_scope(self.project) if "deleted" in p.message], [])

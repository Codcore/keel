#!/usr/bin/env python3
"""The git hooks, including real commits going through them."""

import os
import shutil
import subprocess
import tempfile
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args, ProjectCase  # noqa: E402




# ─────────────────────────────────────────────────────────────────────────────
# keel hooks
# ─────────────────────────────────────────────────────────────────────────────

class TestHooks(ProjectCase):
    def capture(self, **kwargs):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_hooks(self.project, Args(
                **{"install": False, "force": False, **kwargs}))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def hook(self, name):
        return os.path.join(self.fixture.root, ".git", "hooks", name)

    def test_status_before_install(self):
        code, out = self.capture()
        self.assertEqual(code, 0)
        self.assertIn("pre-commit: missing", out)
        self.assertIn("pre-push: missing", out)

    def test_install_writes_both_hooks(self):
        code, _ = self.capture(install=True)
        self.assertEqual(code, 0)
        for name in ("pre-commit", "pre-push"):
            self.assertTrue(os.access(self.hook(name), os.X_OK), name)
            self.assertIn(keel.HOOK_MARK, self.fixture.read(f".git/hooks/{name}"))
        self.assertIn("check --fast", self.fixture.read(".git/hooks/pre-commit"))
        self.assertNotIn("--fast", self.fixture.read(".git/hooks/pre-push"))

    def test_status_after_install(self):
        self.capture(install=True)
        code, out = self.capture()
        self.assertEqual(code, 0)
        self.assertIn("both are in place", out)

    def test_install_is_idempotent(self):
        self.capture(install=True)
        first = self.fixture.read(".git/hooks/pre-commit")
        self.capture(install=True)
        self.assertEqual(first, self.fixture.read(".git/hooks/pre-commit"))

    def test_foreign_hook_is_left_alone(self):
        self.fixture.write(".git/hooks/pre-commit", "#!/bin/sh\necho чуже\n")
        code, out = self.capture(install=True)
        self.assertEqual(code, 1)
        self.assertIn("another tool owns this hook", out)
        self.assertIn("чуже", self.fixture.read(".git/hooks/pre-commit"))

    def test_force_overwrites_foreign_hook(self):
        self.fixture.write(".git/hooks/pre-commit", "#!/bin/sh\necho чуже\n")
        code, _ = self.capture(install=True, force=True)
        self.assertEqual(code, 0)
        self.assertIn(keel.HOOK_MARK, self.fixture.read(".git/hooks/pre-commit"))

    def test_hook_blocks_a_commit_beyond_scope(self):
        self.capture(install=True)
        # keel.__file__ is already the tool; going up two directories pointed
        # the variable at a path that does not exist, so `[ -f "${KEEL}" ]`
        # failed and the commit was blocked through the baked fallback instead
        # — the branch this test is named for was never reached.
        env = dict(os.environ, KEEL=os.path.abspath(keel.__file__))
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.write("lib/extra.ex", "не оголошено\n")
        self.fixture.git("add", "-A")
        done = subprocess.run(
            ["git", "-C", self.fixture.root, "-c", "user.email=t@e.com",
             "-c", "user.name=t", "commit", "-m", "drive-turns: спроба"],
            capture_output=True, text=True, env=env)
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("lib/extra.ex", done.stdout + done.stderr)

    def test_hook_lets_a_clean_commit_through(self):
        self.capture(install=True)
        env = dict(os.environ, KEEL=os.path.abspath(keel.__file__))
        self.fixture.branch("0001-session-loop")
        self.fixture.write("lib/session.ex", "змінено\n")
        self.fixture.git("add", "-A")
        done = subprocess.run(
            ["git", "-C", self.fixture.root, "-c", "user.email=t@e.com",
             "-c", "user.name=t", "commit", "-m", "drive-turns: перший хід"],
            capture_output=True, text=True, env=env)
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)


# ─────────────────────────────────────────────────────────────────────────────
# keel init
# ─────────────────────────────────────────────────────────────────────────────




class TestHooksLandWhereGitReadsThem(unittest.TestCase):
    """A hook written where git never looks is the purest silent green."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-hookpath-")
        self.addCleanup(shutil.rmtree, self.root, True)
        os.makedirs(os.path.join(self.root, "keel", "waves"))
        os.makedirs(os.path.join(self.root, "keel", "contracts"))
        subprocess.run(["git", "init", "-q", "-b", "main", self.root], check=True)
        for key, value in (("user.email", "t@e"), ("user.name", "t")):
            subprocess.run(["git", "-C", self.root, "config", key, value], check=True)

    def install(self, root):
        from io import StringIO
        from tests.support import Args
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_hooks(keel.Project(root), Args(install=True, force=False))
        finally:
            sys.stdout = saved
        return stream.getvalue()

    def where_git_reads(self, root):
        return subprocess.run(["git", "-C", root, "rev-parse", "--git-path", "hooks"],
                              capture_output=True, text=True).stdout.strip()

    def test_core_hookspath_is_honoured(self):
        """husky-подібний репозиторій: хук у .git/hooks git просто ігнорує."""
        os.makedirs(os.path.join(self.root, ".husky"))
        subprocess.run(["git", "-C", self.root, "config", "core.hooksPath", ".husky"],
                       check=True)
        self.install(self.root)
        self.assertTrue(os.path.exists(os.path.join(self.root, ".husky", "pre-commit")))

    def test_a_linked_worktree_gets_its_hooks_where_git_looks(self):
        subprocess.run(["git", "-C", self.root, "commit", "-q", "--allow-empty",
                        "-m", "base"], check=True)
        side = os.path.join(self.root, "..", "side-" + os.path.basename(self.root))
        subprocess.run(["git", "-C", self.root, "worktree", "add", "-q", side,
                        "-b", "side"], check=True)
        self.addCleanup(shutil.rmtree, side, True)
        os.makedirs(os.path.join(side, "keel", "waves"), exist_ok=True)
        self.install(side)
        target = os.path.join(self.where_git_reads(side), "pre-commit")
        self.assertTrue(os.path.exists(target), target)


if __name__ == "__main__":
    unittest.main()


class TestTheHookFindsACompiledTool(ProjectCase):
    """Засіб їде у випробування зібраним двійником, і хук мусить його знайти.

    ЗНАЙДЕНО 25 серпня 2026. Дві біди в одному місці: у хуку шукали лише
    `keel/keel.py`, а зібраний засіб зветься `keel/keel` і не має розширення;
    і `__file__` під Nuitka onefile вказує в тимчасову теку розпакування, яка
    зникає з кінцем процесу — тобто в файл, що йде під гіт, запікався мертвий
    шлях із номером процесу всередині.
    """

    def install(self):
        keel.cmd_hooks(self.project, Args(install=True, force=False))
        return self.fixture.read(".git/hooks/pre-commit")

    def test_the_project_copy_may_be_a_binary(self):
        text = self.install()
        self.assertIn('if [ -x "$root/keel/keel" ]', text)
        self.assertIn('if [ -f "$root/keel/keel.py" ]', text)

    def test_without_a_baked_path_the_hook_still_works(self):
        script = keel.hook_script("pre-commit", None)
        self.assertNotIn("if [ -f \"\" ]", script)
        # Три пошуки поперед нього лишаються, і остання відповідь теж.
        self.assertIn("command -v keel", script)
        self.assertIn("keel: no tool found", script)

    def test_a_real_path_is_still_baked_in(self):
        """Межа: звичайний запуск із дерева нічого не втрачає."""
        script = keel.hook_script("pre-commit", "/opt/keel/keel.py")
        self.assertIn('if [ -f "/opt/keel/keel.py" ]', script)

    def test_a_temporary_extraction_is_refused(self):
        rozpakovano = os.path.join(tempfile.gettempdir(), "onefile_1234", "keel.py")
        spravzhnij = keel.__file__
        keel.__file__ = rozpakovano
        try:
            self.assertIsNone(keel.baked_path())
        finally:
            keel.__file__ = spravzhnij
        self.assertTrue(keel.baked_path().endswith("keel.py"))

    def test_a_compiled_tool_bakes_in_the_binary_not_the_interpreter(self):
        """Під onefile `sys.executable` — розпакований python, а не засіб."""
        dvijnyk = os.path.join(self.fixture.root, "keel", "keel")
        keel.__dict__["__compiled__"] = object()
        argv, executable = sys.argv, sys.executable
        sys.argv = [dvijnyk]
        sys.executable = os.path.join(tempfile.gettempdir(), "onefile_9", "python")
        try:
            self.assertEqual(keel.baked_path(), dvijnyk)
        finally:
            sys.argv, sys.executable = argv, executable
            del keel.__dict__["__compiled__"]

class TestTheProjectCopyWinsOverPATH(ProjectCase):
    """Копія в проєкті йде поперед `PATH` — інакше судить чужа версія.

    ЗНАЙДЕНО ПРОГОНОМ 28 серпня 2026, і це була тиха зелена.

    Mellum закомітила `PLAN.md` на гілці `plan/clean_readings`, коли
    `keel/waves/` була порожня. Хук спрацював і сказав `clean`. Заслон
    зробленості одразу по тому покликав ТУ САМУ команду — `check --fast` — і
    дістав «branch plan/clean_readings names no wave, and keel/waves/ is
    empty».

    Дві однакові команди на однаковому стані дали протилежне, бо кликали різні
    засоби: хук брав перший `keel` із `PATH`, а там лежав глобально
    встановлений 0.8.7, тоді як проєкт возив 0.8.29. Перевірка про гілку без
    хвилі зʼявилась у 0.8.15 і 0.8.19 — старий засіб про неї не знав.

    Проєкт возить свою копію саме для того, щоб перевірки не залежали від
    того, що встановлено на машині.
    """

    def install(self):
        keel.cmd_hooks(self.project, Args(install=True, force=False))
        return self.fixture.read(".git/hooks/pre-commit")

    def test_the_project_copy_is_tried_before_PATH(self):
        text = self.install()
        proekt = text.index('if [ -f "$root/keel/keel.py" ]')
        shlyah = text.index("command -v keel")
        self.assertLess(proekt, shlyah,
                        "копія в проєкті мусить іти поперед PATH")

    def test_an_explicit_KEEL_still_wins(self):
        """Названий рукою шлях лишається першим: це вибір, а не оточення."""
        text = self.install()
        nazvanyj = text.index("${KEEL:-}")
        proekt = text.index('if [ -f "$root/keel/keel.py" ]')
        self.assertLess(nazvanyj, proekt)

    def test_the_baked_path_stays_last(self):
        """Запечений шлях — остання надія, і після правки він нею лишився."""
        script = keel.hook_script("pre-commit", "/opt/keel/keel.py")
        zapechenyj = script.index('if [ -f "/opt/keel/keel.py" ]')
        shlyah = script.index("command -v keel")
        self.assertLess(shlyah, zapechenyj)

    def test_every_way_of_finding_the_tool_survives(self):
        """Порядок змінено, та жодного способу не загублено."""
        text = self.install()
        for sposib in ("${KEEL:-}", '"$root/keel/keel.py"', '"$root/keel/keel"',
                       "command -v keel", "keel: no tool found"):
            self.assertIn(sposib, text)

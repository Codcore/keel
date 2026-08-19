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

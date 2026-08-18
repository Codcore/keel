#!/usr/bin/env python3
"""The installer, run for real against a clone of this repository."""

import os
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402


class TestInstaller(unittest.TestCase):
    """`sh install.sh` against the working copy, into a throwaway home."""

    @classmethod
    def setUpClass(cls):
        cls.source = keel.home()
        cls.script = os.path.join(cls.source, "install.sh")
        if not os.path.exists(cls.script):
            raise unittest.SkipTest("install.sh відсутній")

    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="keel-install-")
        self.addCleanup(shutil.rmtree, self.tmp, True)
        self.keel_home = os.path.join(self.tmp, ".keel")
        self.bin = os.path.join(self.tmp, "bin")

    def install(self, **env):
        return subprocess.run(
            ["/bin/sh", self.script], capture_output=True, text=True,
            env={**os.environ, "KEEL_REPO": self.source,
                 "KEEL_HOME": self.keel_home, "KEEL_BIN": self.bin,
                 "HOME": self.tmp, **env})

    def test_it_clones_and_leaves_a_working_command(self):
        done = self.install()
        self.assertEqual(done.returncode, 0, done.stderr)
        shim = os.path.join(self.bin, "keel")
        self.assertTrue(os.access(shim, os.X_OK), "шим не виконуваний")
        version = subprocess.run([shim, "--version"], capture_output=True, text=True)
        self.assertEqual(version.returncode, 0, version.stderr)
        self.assertEqual(version.stdout.strip(), keel.VERSION)

    def test_the_references_arrive_too(self):
        """Одного файла не досить: init копіює довідники поруч із інструментом."""
        self.install()
        for name in keel.REFERENCES + ("PRINCIPLES.md",):
            self.assertTrue(os.path.exists(os.path.join(self.keel_home, name)), name)
            self.assertTrue(os.path.exists(
                os.path.join(self.keel_home, "docs", "uk", name)), f"uk/{name}")
        self.assertTrue(os.path.exists(os.path.join(self.keel_home, keel.REVISIONS)))

    def test_the_installed_copy_can_set_up_a_project(self):
        """Наскрізь: встановили — і воно ставить Keel у справжній проєкт."""
        self.install()
        project = os.path.join(self.tmp, "project")
        os.makedirs(project)
        subprocess.run(["git", "init", "-b", "main", "-q", project], check=True)
        done = subprocess.run([os.path.join(self.bin, "keel"), "-C", project, "init"],
                              capture_output=True, text=True,
                              env={**os.environ, "HOME": self.tmp})
        self.assertEqual(done.returncode, 0, done.stderr + done.stdout)
        self.assertTrue(os.path.exists(os.path.join(project, "keel", "keel.py")))
        self.assertTrue(os.path.exists(os.path.join(project, "AGENTS.md")))

    def test_a_second_run_updates_instead_of_failing(self):
        self.install()
        done = self.install()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn("updating", done.stdout)

    def test_it_says_when_the_bin_directory_is_off_the_path(self):
        done = self.install(PATH="/usr/bin:/bin")
        self.assertIn("not on your PATH", done.stdout)

    def test_it_stays_quiet_about_the_path_when_the_directory_is_on_it(self):
        done = self.install(PATH=f"{self.bin}:{os.environ['PATH']}")
        self.assertNotIn("not on your PATH", done.stdout)

    def test_it_refuses_without_the_tools_it_needs(self):
        """Порожній PATH — ні git, ні python3; має сказати, а не впасти невиразно."""
        empty = os.path.join(self.tmp, "empty")
        os.makedirs(empty, exist_ok=True)
        done = self.install(PATH=empty)
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("is required", done.stderr)

    def test_the_shim_points_at_the_clone_not_at_this_checkout(self):
        self.install()
        with open(os.path.join(self.bin, "keel"), encoding="utf-8") as handle:
            shim = handle.read()
        self.assertIn(self.keel_home, shim)
        self.assertNotIn(self.source, shim)


if __name__ == "__main__":
    unittest.main()

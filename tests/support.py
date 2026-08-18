#!/usr/bin/env python3
"""Shared fixture: a real project in a temporary directory."""

import os
import subprocess
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402

try:                                 # only to check what we generate, never to run
    import yaml                      # noqa: F401
    HAS_PYYAML = True
except ImportError:
    HAS_PYYAML = False


STEP = """---
depends_on: []

scenarios:
  finishes-when-no-tool-called: {{proves: session-run@{rev}}}

transforms:
  drive-turns:
    implements: [finishes-when-no-tool-called]
    contracts:  [session-run@{rev}]
    files:      [lib/session.ex]
---

## Навіщо

Одна розмова з моделлю проти набору інструментів, який дали ззовні.

## scenario: finishes-when-no-tool-called

**Given** порожній набір інструментів,
**When** модель відповідає текстом,
**Then** розмова завершується.

## transform: drive-turns

Крутити ходи, доки модель кличе інструменти.

Межі: лічильника спроб немає.
"""

CONTRACT = """---
module: Demo.Session
exports: [run/3]
---

Одна розмова з однією моделлю.
"""

class Fixture:
    """A project with git, a step, a contract and the Elixir adapter."""

    def __init__(self):
        self.root = tempfile.mkdtemp(prefix="keel-test-")
        for folder in ("keel/steps", "keel/contracts", "keel/decisions", "lib", "test"):
            os.makedirs(os.path.join(self.root, folder), exist_ok=True)
        self.write("mix.exs", "defmodule Demo.MixProject do\nend\n")
        self.write("keel/contracts/session-run.md", CONTRACT)
        self.contract_rev = keel.revision(CONTRACT)
        self.write("keel/steps/0001-session-loop.md", STEP.format(rev=self.contract_rev))
        self.write("lib/session.ex", "defmodule Demo.Session do\nend\n")
        self.git("init", "-b", "main")
        self.git("config", "user.email", "test@example.com")
        self.git("config", "user.name", "test")
        self.git("add", "-A")
        self.git("commit", "-m", "план")

    # — files —

    def path(self, name):
        return os.path.join(self.root, name)

    def write(self, name, text):
        target = self.path(name)
        os.makedirs(os.path.dirname(target), exist_ok=True)
        with open(target, "w", encoding="utf-8") as handle:
            handle.write(text)

    def read(self, name):
        with open(self.path(name), encoding="utf-8") as handle:
            return handle.read()

    # — git —

    def git(self, *args):
        return subprocess.run(["git", "-C", self.root, *args],
                              capture_output=True, text=True, check=False)

    def branch(self, name):
        self.git("checkout", "-b", name)

    # — the tool —

    def project(self):
        return keel.Project(self.root)

    def scenario_rev(self):
        return self.project().steps["0001-session-loop"].scenario_revision(
            "finishes-when-no-tool-called")

    def close(self):
        shutil.rmtree(self.root, ignore_errors=True)


class ProjectCase(unittest.TestCase):
    def setUp(self):
        self.fixture = Fixture()
        self.addCleanup(self.fixture.close)

    @property
    def project(self):
        return self.fixture.project()


# ─────────────────────────────────────────────────────────────────────────────
# Documents
# ─────────────────────────────────────────────────────────────────────────────


class Args:
    def __init__(self, **kwargs):
        self.__dict__.update(kwargs)
